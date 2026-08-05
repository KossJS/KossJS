// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::os::raw::c_void;
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(any(target_os = "windows", all(target_os = "linux", not(target_env = "ohos")), target_os = "macos"))]
use std::sync::Arc;
#[cfg(any(target_os = "windows", all(target_os = "linux", not(target_env = "ohos")), target_os = "macos"))]
use std::sync::atomic::AtomicUsize;
use std::sync::Mutex;
use std::net::{TcpListener, TcpStream, ToSocketAddrs, IpAddr};
use std::time::{Duration, Instant};

use boa_engine::object::builtins::{JsFunction, JsPromise};
use boa_engine::{Context, JsError, JsNativeError, JsObject, JsValue, Module, Source, NativeFunction};
use boa_engine::js_string;
use boa_runtime::Console;
use tokio::runtime::Runtime;

use crate::bindings;
use crate::buffer;
use crate::license_output::output_license_once;
use crate::version::get_version;
use crate::module_loader::KossModuleLoader;


// One-time warning flags for unstable mode (stable=false)
static FFI_STABLE_WARNED: AtomicBool = AtomicBool::new(false);

struct ApiInstanceState {
    owner: std::thread::ThreadId,
    busy: bool,
}

static API_INSTANCES: std::sync::LazyLock<Mutex<HashMap<usize, ApiInstanceState>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

struct ApiGuard {
    key: usize,
}

impl Drop for ApiGuard {
    fn drop(&mut self) {
        if let Ok(mut instances) = API_INSTANCES.lock() {
            if let Some(state) = instances.get_mut(&self.key) {
                state.busy = false;
            }
        }
    }
}

fn register_api_instance(key: usize) {
    if let Ok(mut instances) = API_INSTANCES.lock() {
        instances.insert(
            key,
            ApiInstanceState {
                owner: std::thread::current().id(),
                busy: false,
            },
        );
    }
}

fn unregister_api_instance(key: usize) {
    if let Ok(mut instances) = API_INSTANCES.lock() {
        instances.remove(&key);
    }
}

fn enter_api(key: usize) -> Result<ApiGuard, &'static str> {
    let mut instances = API_INSTANCES.lock().map_err(|_| "API registry unavailable")?;
    let state = instances.get_mut(&key).ok_or("invalid or destroyed KossInstance")?;
    if state.owner != std::thread::current().id() {
        return Err("KossInstance used from a non-owner thread");
    }
    if state.busy {
        return Err("concurrent or reentrant KossInstance access");
    }
    state.busy = true;
    Ok(ApiGuard { key })
}

macro_rules! api_guard_result {
    ($ptr:expr) => {
        match enter_api($ptr as usize) {
            Ok(guard) => guard,
            Err(message) => return KossResult::err(3, message),
        }
    };
}

const MAX_TCP_CONNECTIONS: usize = 256;
const MAX_TCP_SERVERS: usize = 64;

#[derive(Default)]
struct SocketState {
    next_connection_fd: u32,
    next_server_fd: u32,
    connections: HashMap<u32, TcpStream>,
    servers: HashMap<u32, TcpListener>,
}

impl SocketState {
    fn new() -> Self {
        Self {
            next_connection_fd: 1,
            next_server_fd: 1001,
            connections: HashMap::new(),
            servers: HashMap::new(),
        }
    }
}

const FETCH_POLYFILL_CODE: &str = r#"
'use strict';

class AbortError extends Error {
    constructor(message) {
        super(message);
        this.name = 'AbortError';
    }
}

class FetchError extends Error {
    constructor(message, type, cause) {
        super(message);
        this.name = 'FetchError';
        this.type = type;
        this.cause = cause;
    }
}

class Headers {
    constructor(init) {
        this._headers = {};
        if (init instanceof Headers) {
            for (const [key, value] of init.entries()) {
                this.set(key, value);
            }
        } else if (init) {
            if (typeof init === 'object') {
                for (const [key, value] of Object.entries(init)) {
                    this.set(key, value);
                }
            } else if (typeof init === 'string') {
                const lines = init.split('\r\n');
                for (const line of lines) {
                    const idx = line.indexOf(':');
                    if (idx > 0) {
                        this.set(line.substring(0, idx).trim(), line.substring(idx + 1).trim());
                    }
                }
            }
        }
    }
    get(name) { return this._headers[name.toLowerCase()] || null; }
    set(name, value) { this._headers[name.toLowerCase()] = value; }
    has(name) { return name.toLowerCase() in this._headers; }
    delete(name) { delete this._headers[name.toLowerCase()]; }
    forEach(callback, thisArg) {
        for (const [key, value] of Object.entries(this._headers)) {
            callback.call(thisArg, value, key, this);
        }
    }
    keys() { return Object.keys(this._headers)[Symbol.iterator](); }
    values() { return Object.values(this._headers)[Symbol.iterator](); }
    entries() { return Object.entries(this._headers)[Symbol.iterator](); }
    [Symbol.iterator]() { return this.entries(); }
}

class Response {
    constructor(body, options = {}) {
        this._body = body !== null && body !== undefined ? String(body) : '';
        this.status = options.status || 200;
        this.statusText = options.statusText || '';
        this.headers = options.headers instanceof Headers ? options.headers : new Headers(options.headers || {});
        this.url = options.url || '';
        this.ok = this.status >= 200 && this.status < 300;
        this.redirected = false;
        this.type = options.type || 'basic';
        this._used = false;
    }
    get body() { return this._body; }
    get bodyUsed() { return this._used; }
    clone() {
        if (this._used) throw new TypeError('Body already used');
        return new Response(this._body, {
            status: this.status, statusText: this.statusText,
            headers: new Headers(this.headers), url: this.url,
        });
    }
    text() {
        if (this._used) throw new TypeError('Body already used');
        this._used = true;
        return Promise.resolve(String(this._body));
    }
    json() {
        if (this._used) throw new TypeError('Body already used');
        this._used = true;
        return Promise.resolve(JSON.parse(this._body));
    }
    arrayBuffer() {
        if (this._used) throw new TypeError('Body already used');
        this._used = true;
        var buf = new ArrayBuffer(this._body.length);
        var view = new Uint8Array(buf);
        for (var i = 0; i < this._body.length; i++) view[i] = this._body.charCodeAt(i) & 0xff;
        return Promise.resolve(buf);
    }
    blob() {
        return this.text().then(function(t) { return new Blob([t]); });
    }
    static error() { return new Response(null, { status: 0, statusText: '', type: 'error' }); }
    static redirect(url, status) { return new Response(null, { status: status || 302, headers: { Location: url }, type: 'redirect' }); }
}

async function fetch(input, init) {
    var url = typeof input === 'string' ? input : (input.url || String(input));
    var options = init || {};
    if (typeof input === 'object' && input !== null) {
        if (init === undefined) {
            options = {};
            for (var k in input) { if (k !== 'url') options[k] = input[k]; }
        }
    }
    var method = (options.method || 'GET').toUpperCase();
    var headers = options.headers || {};
    var body = options.body !== undefined ? String(options.body) : undefined;

    var nativeUrl = url;
    var nativeBody = undefined;
    if (method === 'GET' || method === 'HEAD') {
        nativeBody = undefined;
    } else {
        nativeBody = body;
    }

    var nativeHeaders = {};
    if (headers instanceof Headers) {
        for (const [k, v] of headers.entries()) { nativeHeaders[k] = v; }
    } else if (typeof headers === 'object') {
        for (var k in headers) { if (headers.hasOwnProperty(k)) nativeHeaders[k] = headers[k]; }
    }

    var requestJson;
    try {
        requestJson = JSON.stringify({ method: method, headers: nativeHeaders, body: nativeBody });
    } catch (e) {
        throw new TypeError('Failed to serialize request: ' + e.message);
    }

    var responseJson;
    try {
        var promise = __koss_fetch(nativeUrl, requestJson);
        responseJson = await promise;
    } catch (e) {
        throw new FetchError('network error: ' + (e.message || e), 'system', e);
    }

    var response;
    try {
        response = JSON.parse(responseJson);
    } catch (e) {
        throw new FetchError('invalid response JSON', 'invalid-json', e);
    }

    if (!response || typeof response.status === 'undefined') {
        throw new FetchError('invalid response from server', 'invalid-response');
    }

    return new Response(response.body || '', {
        status: response.status,
        statusText: response.statusText || '',
        headers: response.headers || {},
        url: nativeUrl,
    });
}

globalThis.Headers = Headers;
globalThis.Response = Response;
globalThis.AbortError = AbortError;
globalThis.FetchError = FetchError;
globalThis.fetch = fetch;
"#;

// ---------------------------------------------------------------------------
// Async I/O event loop infrastructure
// ---------------------------------------------------------------------------

/// Result from an async I/O operation (sent across threads)
pub(crate) struct AsyncIoResult {
    pub(crate) promise_id: u64,
    pub(crate) result: Result<String, String>,
}

/// Resolver functions for a pending Promise (main thread only)
pub struct PendingResolver {
    pub resolve: JsFunction,
    pub reject: JsFunction,
}

/// Callback request from async FFI (blocking thread → main thread)
#[cfg(any(target_os = "windows", all(target_os = "linux", not(target_env = "ohos")), target_os = "macos"))]
pub(crate) struct CallbackRequest {
    pub instance_ptr: usize,
    pub task_id: u64,
    pub cb_index: usize,
    pub args: Vec<Vec<u8>>,
    pub arg_types: Vec<crate::_senri_ffi::types::OwnedFfiType>,
    pub ret_type: crate::_senri_ffi::types::OwnedFfiType,
    pub resp_tx: tokio::sync::oneshot::Sender<Result<Vec<u8>, String>>,
}

/// Active async FFI task metadata (main thread)
#[cfg(any(target_os = "windows", all(target_os = "linux", not(target_env = "ohos")), target_os = "macos"))]
pub(crate) struct AsyncFfiTask {
    pub canceled: Arc<AtomicBool>,
    pub allow_force_kill: bool,
    #[allow(dead_code)]
    pub callback_timeout_ms: u64,
    pub thread_handle: Option<std::thread::JoinHandle<()>>,
}

/// Per-instance event loop driving async I/O and microtasks
pub struct KossEventLoop {
    pub runtime: Runtime,
    pub(crate) io_tx: mpsc::Sender<AsyncIoResult>,
    pub(crate) io_rx: mpsc::Receiver<AsyncIoResult>,
    pub next_promise_id: u64,
    pub pending: HashMap<u64, PendingResolver>,
    #[cfg(any(target_os = "windows", all(target_os = "linux", not(target_env = "ohos")), target_os = "macos"))]
    pub(crate) callback_tx: mpsc::Sender<CallbackRequest>,
    #[cfg(any(target_os = "windows", all(target_os = "linux", not(target_env = "ohos")), target_os = "macos"))]
    pub(crate) callback_rx: mpsc::Receiver<CallbackRequest>,
    #[cfg(any(target_os = "windows", all(target_os = "linux", not(target_env = "ohos")), target_os = "macos"))]
    pub(crate) async_tasks: HashMap<u64, AsyncFfiTask>,
    #[cfg(any(target_os = "windows", all(target_os = "linux", not(target_env = "ohos")), target_os = "macos"))]
    pub(crate) ffi_callback_fns: HashMap<(u64, usize), JsFunction>,
    #[cfg(any(target_os = "windows", all(target_os = "linux", not(target_env = "ohos")), target_os = "macos"))]
    pub(crate) ffi_next_task_id: u64,
    #[cfg(any(target_os = "windows", all(target_os = "linux", not(target_env = "ohos")), target_os = "macos"))]
    pub(crate) ffi_max_concurrency: Arc<AtomicUsize>,
}

impl KossEventLoop {
    pub fn new() -> Option<Self> {
        let (io_tx, io_rx) = mpsc::channel();
        let runtime = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                eprintln!("Warning: Failed to create tokio runtime: {e}");
                return None;
            }
        };
        #[cfg(any(target_os = "windows", all(target_os = "linux", not(target_env = "ohos")), target_os = "macos"))]
        {
            let (callback_tx, callback_rx) = mpsc::channel();
            Some(KossEventLoop {
                runtime,
                io_tx,
                io_rx,
                callback_tx,
                callback_rx,
                next_promise_id: 1,
                pending: HashMap::new(),
                async_tasks: HashMap::new(),
                ffi_callback_fns: HashMap::new(),
                ffi_next_task_id: 1,
                ffi_max_concurrency: Arc::new(AtomicUsize::new(64)),
            })
        }
        #[cfg(not(any(target_os = "windows", all(target_os = "linux", not(target_env = "ohos")), target_os = "macos")))]
        {
            Some(KossEventLoop {
                runtime,
                io_tx,
                io_rx,
                next_promise_id: 1,
                pending: HashMap::new(),
            })
        }
    }

    /// Process all completed async I/O operations and resolve their promises.
    /// Must be called from the main thread (where the Boa Context lives).
    pub fn process_io_results(&mut self, ctx: &mut Context) {
        // Drive the current-thread runtime to give spawned async tasks
        // (e.g., fetch) CPU time and I/O polling opportunities. Without
        // this, tasks spawned via self.runtime.spawn() are enqueued but
        // never executed because new_current_thread() has no background
        // driver thread.
        if !self.pending.is_empty() {
            self.runtime.block_on(tokio::task::yield_now());
        }

        // Process callback requests from async FFI tasks (C → JS callbacks)
        #[cfg(any(target_os = "windows", all(target_os = "linux", not(target_env = "ohos")), target_os = "macos"))]
        while let Ok(req) = self.callback_rx.try_recv() {
            let canceled = self.async_tasks
                .get(&req.task_id)
                .map(|t| t.canceled.load(Ordering::Acquire))
                .unwrap_or(true);

            let response = if canceled {
                Err("task canceled".to_string())
            } else if let Some(js_fn) = self.ffi_callback_fns.get_mut(&(req.task_id, req.cb_index)) {
                let mut js_args: Vec<JsValue> = Vec::with_capacity(req.args.len());
                for (i, raw_bytes) in req.args.iter().enumerate() {
                    let type_info = &req.arg_types[i];
                    let val = ffi_bytes_to_js_value(raw_bytes, type_info);
                    js_args.push(val);
                }
                let instance = unsafe { &*(req.instance_ptr as *mut KossInstance) };
                if authorize_operation(
                    instance,
                    crate::sandbox::FFI_CALLBACK,
                    "ffi.callbackInvoke",
                    &js_args,
                    ctx,
                )
                .is_err()
                {
                    Ok(vec![0u8; req.ret_type.sizeof()])
                } else {
                    match js_fn.call(&JsValue::undefined(), &js_args, ctx) {
                    Ok(js_val) => {
                        let ret_bytes = ffi_js_value_to_bytes(&js_val, &req.ret_type);
                        Ok(ret_bytes)
                    }
                    Err(_) => Ok(vec![0u8; req.ret_type.sizeof()]),
                    }
                }
            } else {
                Ok(vec![0u8; req.ret_type.sizeof()])
            };
            let _ = req.resp_tx.send(response);
        }

        // Process async I/O results (fetch, etc.)
        while let Ok(AsyncIoResult { promise_id, result }) = self.io_rx.try_recv() {
            if let Some(resolver) = self.pending.remove(&promise_id) {
                match result {
                    Ok(json) => {
                        let js_val = JsValue::from(boa_engine::js_string!(json));
                        let _ = resolver.resolve.call(
                            &JsValue::undefined(),
                            &[js_val],
                            ctx,
                        );
                    }
                    Err(err) => {
                        let js_err = JsValue::from(boa_engine::js_string!(err));
                        let _ = resolver.reject.call(
                            &JsValue::undefined(),
                            &[js_err],
                            ctx,
                        );
                    }
                }
            }
        }
    }

    /// Allocate a new promise ID and store the resolvers.
    /// Returns None on overflow (after 2^64-1 registrations).
    pub fn register_promise(&mut self, resolve: JsFunction, reject: JsFunction) -> Option<u64> {
        let id = self.next_promise_id;
        self.next_promise_id = self.next_promise_id.checked_add(1)?;
        self.pending.insert(id, PendingResolver { resolve, reject });
        Some(id)
    }

    /// Spawn an async task on the tokio runtime
    pub fn spawn<F>(&self, future: F)
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        self.runtime.spawn(future);
    }

    /// Register a new async FFI task and return its task_id.
    #[cfg(any(target_os = "windows", all(target_os = "linux", not(target_env = "ohos")), target_os = "macos"))]
    pub fn register_ffi_task(
        &mut self,
        canceled: Arc<AtomicBool>,
        allow_force_kill: bool,
        callback_timeout_ms: u64,
    ) -> u64 {
        let id = self.ffi_next_task_id;
        self.ffi_next_task_id = self.ffi_next_task_id.wrapping_add(1);
        self.async_tasks.insert(id, AsyncFfiTask {
            canceled,
            allow_force_kill,
            callback_timeout_ms,
            thread_handle: None,
        });
        id
    }

    /// Store the thread handle for an async FFI task.
    #[cfg(any(target_os = "windows", all(target_os = "linux", not(target_env = "ohos")), target_os = "macos"))]
    pub fn set_ffi_task_thread(&mut self, task_id: u64, handle: std::thread::JoinHandle<()>) {
        if let Some(task) = self.async_tasks.get_mut(&task_id) {
            task.thread_handle = Some(handle);
        }
    }

    /// Register a JS callback function for a task/cb_index slot.
    #[cfg(any(target_os = "windows", all(target_os = "linux", not(target_env = "ohos")), target_os = "macos"))]
    pub fn register_ffi_callback_fn(&mut self, task_id: u64, cb_index: usize, func: JsFunction) {
        self.ffi_callback_fns.insert((task_id, cb_index), func);
    }

    /// Get a clone of the callback channel sender.
    #[cfg(any(target_os = "windows", all(target_os = "linux", not(target_env = "ohos")), target_os = "macos"))]
    pub(crate) fn callback_tx_clone(&self) -> mpsc::Sender<CallbackRequest> {
        self.callback_tx.clone()
    }

    /// Get the max concurrency AtomicUsize for FFI tasks.
    #[cfg(any(target_os = "windows", all(target_os = "linux", not(target_env = "ohos")), target_os = "macos"))]
    pub fn ffi_max_concurrency(&self) -> Arc<AtomicUsize> {
        self.ffi_max_concurrency.clone()
    }

    /// Force kill an async FFI task (kill OS thread).
    #[cfg(any(target_os = "windows", all(target_os = "linux", not(target_env = "ohos")), target_os = "macos"))]
    pub fn force_kill_ffi_task(&mut self, task_id: u64) {
        if let Some(task) = self.async_tasks.get_mut(&task_id) {
            task.canceled.store(true, Ordering::Release);
            if task.allow_force_kill {
                if let Some(handle) = task.thread_handle.take() {
                    drop(handle);
                }
            }
        }
    }

    /// Remove a completed FFI task (cleanup after async call finishes).
    #[cfg(any(target_os = "windows", all(target_os = "linux", not(target_env = "ohos")), target_os = "macos"))]
    pub fn remove_ffi_task(&mut self, task_id: u64) {
        self.async_tasks.remove(&task_id);
        let keys: Vec<(u64, usize)> = self.ffi_callback_fns.keys()
            .filter(|(tid, _)| *tid == task_id)
            .cloned()
            .collect();
        for k in keys {
            self.ffi_callback_fns.remove(&k);
        }
    }
}

// ---------------------------------------------------------------------------
// FFI callback value conversion helpers
// ---------------------------------------------------------------------------
#[cfg(any(target_os = "windows", all(target_os = "linux", not(target_env = "ohos")), target_os = "macos"))]
fn ffi_bytes_to_js_value(bytes: &[u8], type_info: &crate::_senri_ffi::types::OwnedFfiType) -> JsValue {
    use crate::_senri_ffi::types::OwnedFfiType;
    match type_info {
        OwnedFfiType::Void => JsValue::undefined(),
        OwnedFfiType::Int8 => JsValue::from(i8::from_le_bytes(bytes.try_into().unwrap()) as f64),
        OwnedFfiType::Uint8 => JsValue::from(u8::from_le_bytes(bytes.try_into().unwrap()) as f64),
        OwnedFfiType::Int16 => JsValue::from(i16::from_le_bytes(bytes.try_into().unwrap()) as f64),
        OwnedFfiType::Uint16 => JsValue::from(u16::from_le_bytes(bytes.try_into().unwrap()) as f64),
        OwnedFfiType::Int32 => JsValue::from(i32::from_le_bytes(bytes.try_into().unwrap()) as f64),
        OwnedFfiType::Uint32 => JsValue::from(u32::from_le_bytes(bytes.try_into().unwrap()) as f64),
        OwnedFfiType::Int64 => JsValue::from(i64::from_le_bytes(bytes.try_into().unwrap()) as f64),
        OwnedFfiType::Uint64 => JsValue::from(u64::from_le_bytes(bytes.try_into().unwrap()) as f64),
        OwnedFfiType::Float32 => JsValue::from(f32::from_le_bytes(bytes.try_into().unwrap()) as f64),
        OwnedFfiType::Float64 => JsValue::from(f64::from_le_bytes(bytes.try_into().unwrap())),
        OwnedFfiType::Pointer | OwnedFfiType::Callback { .. } => {
            let addr = usize::from_le_bytes(bytes.try_into().unwrap());
            JsValue::from(addr as f64)
        }
        OwnedFfiType::CString => {
            let addr = usize::from_le_bytes(bytes.try_into().unwrap());
            if addr == 0 {
                JsValue::null()
            } else {
                let cstr = unsafe { std::ffi::CStr::from_ptr(addr as *const std::ffi::c_char) };
                let s = cstr.to_string_lossy().to_string();
                JsValue::from(js_string!(s))
            }
        }
        OwnedFfiType::Struct { .. } | OwnedFfiType::Array { .. } => {
            JsValue::from(js_string!("[binary data]"))
        }
        OwnedFfiType::VarArg => {
            let addr = usize::from_le_bytes(bytes.try_into().unwrap());
            JsValue::from(addr as f64)
        }
    }
}

#[cfg(any(target_os = "windows", all(target_os = "linux", not(target_env = "ohos")), target_os = "macos"))]
fn ffi_js_value_to_bytes(val: &JsValue, type_info: &crate::_senri_ffi::types::OwnedFfiType) -> Vec<u8> {
    use crate::_senri_ffi::types::OwnedFfiType;
    match type_info {
        OwnedFfiType::Void => Vec::new(),
        OwnedFfiType::Int8 => {
            let v = val.as_number().map(|n| n as i8).unwrap_or(0);
            v.to_le_bytes().to_vec()
        }
        OwnedFfiType::Uint8 => {
            let v = val.as_number().map(|n| n as u8).unwrap_or(0);
            v.to_le_bytes().to_vec()
        }
        OwnedFfiType::Int16 => {
            let v = val.as_number().map(|n| n as i16).unwrap_or(0);
            v.to_le_bytes().to_vec()
        }
        OwnedFfiType::Uint16 => {
            let v = val.as_number().map(|n| n as u16).unwrap_or(0);
            v.to_le_bytes().to_vec()
        }
        OwnedFfiType::Int32 => {
            let v = val.as_number().map(|n| n as i32).unwrap_or(0);
            v.to_le_bytes().to_vec()
        }
        OwnedFfiType::Uint32 => {
            let v = val.as_number().map(|n| n as u32).unwrap_or(0);
            v.to_le_bytes().to_vec()
        }
        OwnedFfiType::Int64 => {
            let v = val.as_number().map(|n| n as i64).unwrap_or(0);
            v.to_le_bytes().to_vec()
        }
        OwnedFfiType::Uint64 => {
            let v = val.as_number().map(|n| n as u64).unwrap_or(0);
            v.to_le_bytes().to_vec()
        }
        OwnedFfiType::Float32 => {
            let v = val.as_number().map(|n| n as f32).unwrap_or(0.0);
            v.to_le_bytes().to_vec()
        }
        OwnedFfiType::Float64 => {
            let v = val.as_number().unwrap_or(0.0);
            v.to_le_bytes().to_vec()
        }
        OwnedFfiType::Pointer | OwnedFfiType::Callback { .. } => {
            let addr = val.as_number().map(|n| n as usize).unwrap_or(0);
            addr.to_le_bytes().to_vec()
        }
        OwnedFfiType::CString => {
            let ptr_val: usize = if val.is_null() || val.is_undefined() {
                0
            } else if let Some(s) = val.as_string() {
                let cstr = std::ffi::CString::new(s.to_std_string_escaped().as_bytes())
                    .unwrap_or_default();
                cstr.into_raw() as usize
            } else {
                0
            };
            ptr_val.to_le_bytes().to_vec()
        }
        _ => {
            vec![0u8; type_info.sizeof()]
        }
    }
}

// ── Constants ──────────────────────────────────────────────────────────────

/// Maximum permitted externally-loaded module code size (CWE-94: prevent
/// code injection via oversized external module payloads).
const MAX_EXTERNAL_MODULE_CODE_SIZE: usize = 10 * 1024 * 1024; // 10 MiB

use crate::sandbox::{
    AuditDecision, KOSS_CAP_ALL_CRYPTO, KOSS_CAP_ALL_FS, KOSS_CAP_ALL_NET,
    KOSS_CAP_EXTERNAL_LOADER, SandboxState,
};

// ---------------------------------------------------------------------------
// Opaque handle — each KossInstance is an isolated JS VM
// ---------------------------------------------------------------------------
// SAFETY: Boa Context is not Sync. All mutable access to the context and its
// associated fields (event_loop, external_module_loader) MUST
// occur on the same thread that created the instance. The NativeFunction
// closures below capture raw pointers (or Rc handles) to these fields, and
// are guaranteed by Boa's single-threaded execution model to only be invoked
// from the owning thread.
//
// THREAD-SAFETY WARNING FOR HOSTS (CWE-362):
// The C ABI functions below (koss_eval, koss_tick, etc.)
// directly dereference `*mut KossInstance` without any mutex or lock. All
// calls to C API functions for a given KossInstance MUST be made from a
// single thread. Concurrent access from multiple threads will cause
// undefined behavior (data races, memory corruption, crashes).
// For multi-threaded hosts, serialise all KossInstance access through an
// external mutex or ensure exclusive thread ownership.
pub struct KossInstance {
    pub event_loop: Option<KossEventLoop>,
    /// Optional external module loader callback (e.g. from Python).
    /// Called as a fallback when the embedded stdlib doesn't contain the module.
    pub external_module_loader: Option<NativeCallback>,
    /// Bitmask of enabled capabilities (see KOSS_CAP_* constants).
    pub capabilities: u32,
    /// Sandbox state: audit mask and future extension fields.
    pub sandbox: SandboxState,
    /// When true, FFI capabilities are disabled (stable mode).
    pub stable: bool,
    /// Builtin module flags — controls which koss:* modules are visible.
    /// See KOSS_BUILTIN_* constants in builtins.rs.
    pub builtins: u32,
    socket_state: Mutex<SocketState>,
    /// Context MUST be the last field: Rust drops struct fields in declaration
    /// order, and other fields (event_loop.ffi_callback_fns) hold JsFunction
    /// handles that reference the Context. Dropping Context first causes
    /// use-after-free segfaults.
    pub context: Context,
}

impl KossInstance {
    pub fn new(context: Context, caps: u32, stable: bool, builtins: u32) -> Self {
        KossInstance {
            event_loop: KossEventLoop::new(),
            external_module_loader: None,
            capabilities: caps,
            sandbox: SandboxState::default(),
            stable,
            builtins,
            socket_state: Mutex::new(SocketState::new()),
            context,
        }
    }

    /// Drive the event loop: process I/O results, run microtasks, return false when idle
    pub fn tick(&mut self) -> bool {
        if let Some(ref mut el) = self.event_loop {
            el.process_io_results(&mut self.context);
            let _ = self.context.run_jobs();
            // Return true if there are still pending promises
            !el.pending.is_empty()
        } else {
            false
        }
    }

    /// Run the event loop until the main promise resolves or timeout
    pub fn run_until_complete(&mut self, promise: &JsPromise, timeout_ms: u64) -> bool {
        let deadline = Instant::now() + Duration::from_millis(timeout_ms);
        let max_iterations = 100_000u64;
        let mut iteration = 0u64;
        let mut consecutive_idle: u32 = 0;
        loop {
            let had_work = self.tick();

            match promise.state() {
                boa_engine::builtins::promise::PromiseState::Fulfilled(_) => return true,
                boa_engine::builtins::promise::PromiseState::Rejected(_) => return true,
                boa_engine::builtins::promise::PromiseState::Pending => {}
            }

            iteration += 1;
            if iteration >= max_iterations {
                return false;
            }

            if Instant::now() >= deadline {
                return false;
            }

            // Adaptive sleep: reduce polling frequency when idle to avoid
            // busy-wait DoS (CWE-400). Max back-off: 100ms.
            if had_work {
                consecutive_idle = 0;
                std::thread::sleep(Duration::from_micros(100));
            } else {
                consecutive_idle = consecutive_idle.saturating_add(1);
                let backoff_ms = 1u64.saturating_mul((consecutive_idle as u64).min(100));
                std::thread::sleep(Duration::from_millis(backoff_ms));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Result passed back across FFI
// ---------------------------------------------------------------------------
#[repr(C)]
pub struct KossResult {
    /// 0 = success, 1 = JS error, 2 = invalid argument
    pub code: i32,
    /// Heap-allocated C string — caller must free with koss_free_string
    pub value: *mut c_char,
}

impl KossResult {
    fn ok(val: &str) -> Self {
        let c = match CString::new(val) {
            Ok(c) => c,
            Err(_) => {
                let safe = val.replace('\0', "\u{FFFD}");
                CString::new(safe).unwrap_or_else(|_| CString::new("(null byte stripped)").unwrap())
            }
        };
        KossResult {
            code: 0,
            value: c.into_raw(),
        }
    }

    fn err(code: i32, msg: &str) -> Self {
        let c = match CString::new(msg) {
            Ok(c) => c,
            Err(_) => {
                let safe = msg.replace('\0', "\u{FFFD}");
                CString::new(safe).unwrap_or_else(|_| CString::new("(null byte stripped)").unwrap())
            }
        };
        KossResult {
            code,
            value: c.into_raw(),
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------
fn js_value_to_string(val: &JsValue, ctx: &mut Context) -> String {
    match val.to_string(ctx) {
        Ok(s) => s.to_std_string_escaped(),
        Err(_) => format!("{:?}", val),
    }
}

fn js_error_to_string(err: &JsError, ctx: &mut Context) -> String {
    // Try native error first
    if let Ok(native) = err.try_native(ctx) {
        let msg = native.message().to_string();
        if !msg.is_empty() {
            return msg;
        }
    }
    // Try to convert the JsError value to string via JS toString()
    let val = err.to_opaque(ctx);
    if let Ok(s) = val.to_string(ctx) {
        let str = s.to_std_string_escaped();
        if !str.is_empty() && str != "[object Object]" {
            return str;
        }
    }
    // Fallback: debug format
    format!("{:?}", err)
}

fn register_console(ctx: &mut Context) {
    let console = Console::init(ctx);
    let _ = ctx.register_global_property(
        boa_engine::js_string!("console"),
        console,
        boa_engine::property::Attribute::all(),
    );
}

fn register_koss_global(ctx: &mut Context, stable: bool) {
    let version = match std::str::from_utf8(get_version()) {
        Ok(s) => s.trim_end_matches('\0').to_string(),
        Err(_) => "unknown".to_string(),
    };

    // Rust 层创建对象
    let mut obj = boa_engine::object::ObjectInitializer::new(ctx);
    obj.property(
        boa_engine::js_string!("version"),
        boa_engine::JsValue::from(boa_engine::js_string!(version)),
        boa_engine::property::Attribute::READONLY
            | boa_engine::property::Attribute::ENUMERABLE
            | boa_engine::property::Attribute::PERMANENT,
    );
    obj.property(
        boa_engine::js_string!("runtime"),
        boa_engine::JsValue::from(boa_engine::js_string!("KossJS")),
        boa_engine::property::Attribute::READONLY
            | boa_engine::property::Attribute::ENUMERABLE
            | boa_engine::property::Attribute::PERMANENT,
    );
    obj.property(
        boa_engine::js_string!("isStable"),
        boa_engine::JsValue::from(stable),
        boa_engine::property::Attribute::READONLY
            | boa_engine::property::Attribute::ENUMERABLE
            | boa_engine::property::Attribute::PERMANENT,
    );
    let koss_obj = obj.build();

    // Rust 层注册到 globalThis（不设 PERMANENT，让 JS 层能替换并做最终保护）
    let _ = ctx.register_global_property(
        boa_engine::js_string!("KossJS"),
        koss_obj,
        boa_engine::property::Attribute::READONLY
            | boa_engine::property::Attribute::CONFIGURABLE,
    );

    // JS 层加固：无原型 + 冻结
    let harden_code = r#"
    (function() {
        var safe = Object.create(null);
        safe.version = globalThis.KossJS.version;
        safe.runtime = globalThis.KossJS.runtime;
        safe.isStable = globalThis.KossJS.isStable;
        Object.freeze(safe);
        Object.defineProperty(globalThis, 'KossJS', {
            value: safe,
            writable: false,
            enumerable: false,
            configurable: false
        });
    })();
    "#;
    let source = boa_parser::Source::from_bytes(harden_code.as_bytes());
    if let Err(e) = ctx.eval(source) {
        eprintln!("Warning: Failed to harden KossJS global: {:?}", e);
    }
}

fn register_fetch_polyfill(ctx: &mut Context) {
    let source = Source::from_bytes(FETCH_POLYFILL_CODE.as_bytes());
    if let Err(e) = ctx.eval(source) {
        eprintln!("Warning: Failed to register fetch polyfill: {:?}", e);
    }
}

fn register_native_bindings(instance: &mut KossInstance) {
    let instance_ptr = instance as *mut KossInstance;

    let native = NativeFunction::from_copy_closure(move |_this, args, ctx| {
        if args.is_empty() {
            return Ok(JsValue::undefined());
        }
        let name = args[0].to_string(ctx).unwrap_or_default();
        let name_str = name.to_std_string_escaped();
        let inst = unsafe { &*instance_ptr };
        let debug = inst.sandbox.audit_debug;
        let decision = is_capability_enabled(inst.capabilities, inst.sandbox.audit_mask, &name_str);
        match decision {
            AuditDecision::DenyCapability => {
                let msg = capability_error_message(&name_str, debug);
                return Err(JsError::from(JsNativeError::error().with_message(msg)));
            }
            AuditDecision::Allow => {}
            AuditDecision::NeedAudit => {
                if let Some(audit_fn) = inst.sandbox.sync_audit {
                    let target = match CString::new(name_str.clone()) {
                        Ok(c) => c,
                        Err(_) => return Ok(JsValue::from(boa_engine::js_string!("{}"))),
                    };
                    let allowed = unsafe {
                        audit_fn(
                            target.as_ptr(),
                            std::ptr::null(),
                            0,
                            std::ptr::null(),
                            inst.sandbox.sync_userdata,
                        )
                    };
                    if !allowed {
                        let msg = security_error_message(&name_str, debug);
                        return Err(JsError::from(JsNativeError::error().with_message(msg)));
                    }
                }
            }
        }
        match handle_binding(&name_str) {
            Ok(json) => Ok(JsValue::from(boa_engine::js_string!(json))),
            Err(_) => Ok(JsValue::undefined()),
        }
    });

    let js_func = native.to_js_function(instance.context.realm());

    instance
        .context
        .register_global_property(
            boa_engine::js_string!("__koss_bindings"),
            js_func,
            boa_engine::property::Attribute::WRITABLE
                | boa_engine::property::Attribute::CONFIGURABLE,
        )
        .ok();
}

/// Register `__koss_load_module` for CommonJS `require()`.
/// First tries embedded stdlib; if not found, delegates to an
/// externally-registered module loader (set via `koss_register_module_loader`).
fn register_internal_module_loader(instance: &mut KossInstance) {
    let instance_ptr = instance as *mut KossInstance;
    let caps = instance.capabilities;

    let native = NativeFunction::from_copy_closure(move |_this, args, context| {
        // Check MODULE_LOAD capability
        if !crate::sandbox::has_cap(caps, crate::sandbox::MODULE_LOAD) {
            return Err(JsError::from(JsNativeError::typ()
                .with_message("KossCapabilityError: capability denied for require")));
        }

        if args.is_empty() {
            return Ok(JsValue::null());
        }
        let name = match args[0].to_string(context) {
            Ok(s) => s.to_std_string_escaped(),
            Err(_) => return Ok(JsValue::null()),
        };

        let module_name = if name.starts_with("node:") {
            &name[5..]
        } else {
            &name
        };

        // 0. Try koss: protocol builtins (koss:node/*, koss:bun, koss:deno, koss:io, etc.)
        if crate::builtins::is_koss_specifier(&name) {
            let inst = unsafe { &*instance_ptr };
            match crate::builtins::resolve_builtin_specifier(&name, inst.builtins) {
                Ok((source, _is_internal)) => {
                    let json = serde_json::json!({"type": "module", "code": source});
                    return Ok(JsValue::from(boa_engine::js_string!(json.to_string())));
                }
                Err(e) => {
                    return Err(JsError::from(JsNativeError::error().with_message(e)));
                }
            }
        }

        // 1. Try embedded stdlib
        let direct_rel = format!("{}.js", module_name);
        if let Some(content) = crate::embedded_stdlib::get(&direct_rel) {
            let json = serde_json::json!({"type": "module", "code": content});
            return Ok(JsValue::from(boa_engine::js_string!(json.to_string())));
        }

        let index_rel = format!("{}/index.js", module_name);
        if let Some(content) = crate::embedded_stdlib::get(&index_rel) {
            let json = serde_json::json!({"type": "module", "code": content});
            return Ok(JsValue::from(boa_engine::js_string!(json.to_string())));
        }

        // 1b. Try koss:node/{name} as fallback for bare module names
        if !module_name.starts_with("koss:") {
            let koss_node_name = format!("koss:node/{}", module_name);
            let inst = unsafe { &*instance_ptr };
            match crate::builtins::resolve_builtin_specifier(&koss_node_name, inst.builtins) {
                Ok((source, _is_internal)) => {
                    let json = serde_json::json!({"type": "module", "code": source});
                    return Ok(JsValue::from(boa_engine::js_string!(json.to_string())));
                }
                Err(_) => {}
            }
        }

        // 2. Fallback: try externally-registered module loader (if enabled)
        if caps & KOSS_CAP_EXTERNAL_LOADER != 0 {
            let inst = unsafe { &*instance_ptr };
            if let Some(external) = inst.external_module_loader {
                let argc = 1i32;
                let c_name = CString::new(name.as_str()).unwrap_or(CString::new("").unwrap());
                let mut ptrs = [c_name.as_ptr()];
                let result = unsafe { external(argc, ptrs.as_mut_ptr() as *mut c_void) };

                if !result.is_null() {
                    let result_str = unsafe {
                        CStr::from_ptr(result as *const c_char)
                            .to_str()
                            .unwrap_or("")
                            .to_string()
                    };
                    // Validate external module against code size limit (CWE-94)
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&result_str) {
                        if let Some(code) = parsed.get("code").and_then(|c| c.as_str()) {
                            if code.len() > MAX_EXTERNAL_MODULE_CODE_SIZE {
                                return Ok(JsValue::null());
                            }
                        }
                    }
                    return Ok(JsValue::from(boa_engine::js_string!(result_str)));
                }
            }
        }

        Ok(JsValue::null())
    });

    let js_func = native.to_js_function(instance.context.realm());
    let _ = instance.context.register_global_property(
        boa_engine::js_string!("__koss_load_module"),
        js_func,
        boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
    );
}

fn register_native_fetch(instance: &mut KossInstance) {
    let instance_ptr = instance as *mut KossInstance;

    let native = NativeFunction::from_copy_closure(move |_this, args, ctx| {
        let inst = unsafe { &mut *instance_ptr };
        authorize_operation(inst, crate::sandbox::NET_FETCH, "net.fetch", args, ctx)?;

        if args.len() < 2 {
            return Ok(JsValue::undefined());
        }

        let event_loop = match inst.event_loop.as_mut() {
            Some(el) => el,
            None => {
                // No event loop, fall back to synchronous
                let url = js_value_to_string(&args[0], ctx);
                let request_json = js_value_to_string(&args[1], ctx);
                let json_str = match CString::new(request_json) {
                    Ok(c) => c,
                    Err(_) => return Err(JsError::from(JsNativeError::typ().with_message("fetch: invalid request body encoding"))),
                };
                return match bindings::fetch::fetch_with_url(&url, json_str.to_str().unwrap_or("")) {
                    Ok(response) => {
                        let json = serde_json::to_string(&response).unwrap_or_default();
                        Ok(JsValue::from(boa_engine::js_string!(json)))
                    }
                    Err(e) => Err(JsError::from(JsNativeError::typ().with_message(format!("fetch: {e}")))),
                };
            }
        };

        let url = js_value_to_string(&args[0], ctx);
        let request_json = js_value_to_string(&args[1], ctx);

        // Create a pending Promise with resolving functions
        let (promise, resolvers) = JsPromise::new_pending(ctx);

        let promise_id = match event_loop.register_promise(resolvers.resolve.clone(), resolvers.reject.clone()) {
            Some(id) => id,
            None => return Err(JsError::from(JsNativeError::typ().with_message("fetch: too many pending promises (overflow)"))),
        };
        let io_tx_clone = event_loop.io_tx.clone();

        // Clone the strings for the async task
        let url_clone = url.clone();
        let json_clone = request_json.clone();

        // Spawn the async HTTP request on tokio
        event_loop.runtime.spawn(async move {
            let result = bindings::fetch::fetch_async(&url_clone, &json_clone).await;
            let _ = io_tx_clone.send(AsyncIoResult {
                promise_id,
                result,
            });
        });

        Ok(promise.into())
    });

    let js_func = native.to_js_function(instance.context.realm());

    instance
        .context
        .register_global_property(
            boa_engine::js_string!("__koss_fetch"),
            js_func,
            boa_engine::property::Attribute::WRITABLE
                | boa_engine::property::Attribute::CONFIGURABLE,
        )
        .ok();
}

fn is_ssrf_blocked(host: &str) -> bool {
    if let Ok(ip) = host.parse::<IpAddr>() {
        return match ip {
            IpAddr::V4(v4) => {
                let o = v4.octets();
                o[0] == 127 || o[0] == 10 || o[0] == 0
                    || (o[0] == 172 && o[1] >= 16 && o[1] <= 31)
                    || (o[0] == 192 && o[1] == 168)
                    || (o[0] == 169 && o[1] == 254)
                    || (o[0] >= 224)
            }
            IpAddr::V6(v6) => v6.is_loopback(),
        };
    }
    if let Ok(addrs) = format!("{}:0", host).to_socket_addrs() {
        for addr in addrs {
            if is_ssrf_blocked(&addr.ip().to_string()) {
                return true;
            }
        }
    }
    false
}

fn register_fs_functions(instance: &mut KossInstance) {
    // Capability snapshot: the low-level __koss_fs_* globals are gated by the
    // instance capability bitmask so sandboxed instances cannot bypass the
    // sandbox via these globals (mirrors the NET_FETCH gate on __koss_fetch).
    let instance_ptr = instance as *mut KossInstance;
    macro_rules! reg_fs {
        ($name:expr, $cap:expr, $closure:expr) => {{
            let inner = $closure;
            let required_cap: u32 = $cap;
            let js_fn = NativeFunction::from_copy_closure(
                move |this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
                    let inst = unsafe { &*instance_ptr };
                    authorize_operation(inst, required_cap, $name, args, ctx)?;
                    inner(this, args, ctx)
                },
            )
            .to_js_function(instance.context.realm());
            instance.context.register_global_property(
                boa_engine::js_string!($name),
                js_fn,
                boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
            ).ok();
        }};
    }

    // __koss_fs_exists(path) -> bool
    reg_fs!("__koss_fs_exists", crate::sandbox::FS_READ, |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
        let path = args.first().ok_or_else(|| JsNativeError::error().with_message("exists: path required"))?
            .to_string(ctx).map_err(|_| JsNativeError::error().with_message("exists: path must be string"))?;
        Ok(JsValue::from(crate::bindings::fs::exists_sync(&path.to_std_string_escaped())))
    });

    // __koss_fs_read(path) -> { code: 0, value: base64_string }
    reg_fs!("__koss_fs_read", crate::sandbox::FS_READ, |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
        let path = args.first().ok_or_else(|| JsNativeError::error().with_message("read: path required"))?
            .to_string(ctx).map_err(|_| JsNativeError::error().with_message("read: path must be string"))?;
        let path_str = path.to_std_string_escaped();
        match std::fs::read(&path_str) {
            Ok(data) => {
                use base64::Engine;
                let encoded = base64::engine::general_purpose::STANDARD.encode(&data);
                Ok(JsValue::from(boa_engine::js_string!(format!("{{\"code\":0,\"value\":\"{}\"}}", encoded))))
            }
            Err(e) => Ok(JsValue::from(boa_engine::js_string!(format!("{{\"code\":1,\"value\":\"{}\"}}", e)))),
        }
    });

    // __koss_fs_write(path, data, is_base64) -> { code: 0 }
    // When is_base64 is true, `data` is a base64 string that is decoded to raw
    // bytes before writing. This keeps binary writes lossless (a latin1/utf8
    // string round-trip would corrupt any byte >= 0x80).
    reg_fs!("__koss_fs_write", crate::sandbox::FS_WRITE, |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
        if args.len() < 2 {
            return Err(JsNativeError::error().with_message("write: path and data required").into());
        }
        let path = args[0].to_string(ctx).map_err(|_| JsNativeError::error().with_message("write: path must be string"))?;
        let data_val = args[1].to_string(ctx).map_err(|_| JsNativeError::error().with_message("write: data must be string"))?;
        let is_base64 = args.get(2).map(|v| v.to_boolean()).unwrap_or(false);
        let path_str = path.to_std_string_escaped();
        let data_str = data_val.to_std_string_escaped();
        let bytes: Vec<u8> = if is_base64 {
            use base64::Engine;
            match base64::engine::general_purpose::STANDARD.decode(data_str.as_bytes()) {
                Ok(b) => b,
                Err(_) => return Ok(JsValue::from(boa_engine::js_string!("{\"code\":1,\"value\":\"invalid base64 data\"}"))),
            }
        } else {
            data_str.into_bytes()
        };
        match std::fs::write(&path_str, &bytes) {
            Ok(()) => Ok(JsValue::from(boa_engine::js_string!("{\"code\":0}"))),
            Err(e) => Ok(JsValue::from(boa_engine::js_string!(format!("{{\"code\":1,\"value\":\"{}\"}}", e)))),
        }
    });

    // __koss_fs_stat(path) -> { code: 0, value: json_string }
    reg_fs!("__koss_fs_stat", crate::sandbox::FS_READ, |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
        let path = args.first().ok_or_else(|| JsNativeError::error().with_message("stat: path required"))?
            .to_string(ctx).map_err(|_| JsNativeError::error().with_message("stat: path must be string"))?;
        let path_str = path.to_std_string_escaped();
        match std::fs::metadata(&path_str) {
            Ok(meta) => {
                let mtime = meta.modified().ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_millis() as i64).unwrap_or(0);
                let ctime = meta.created().ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_millis() as i64).unwrap_or(0);
                let json = format!(
                    "{{\"size\":{},\"mtime\":{},\"ctime\":{},\"isFile\":{},\"isDirectory\":{},\"isSymlink\":{}}}",
                    meta.len(), mtime, ctime, meta.is_file(), meta.is_dir(), meta.file_type().is_symlink()
                );
                Ok(JsValue::from(boa_engine::js_string!(format!("{{\"code\":0,\"value\":{}}}", json))))
            }
            Err(e) => Ok(JsValue::from(boa_engine::js_string!(format!("{{\"code\":1,\"value\":\"{}\"}}", e)))),
        }
    });

    // __koss_fs_mkdir(path, recursive) -> { code: 0 }
    reg_fs!("__koss_fs_mkdir", crate::sandbox::FS_MKDIR, |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
        let path = args.first().ok_or_else(|| JsNativeError::error().with_message("mkdir: path required"))?
            .to_string(ctx).map_err(|_| JsNativeError::error().with_message("mkdir: path must be string"))?;
        let recursive = args.get(1).and_then(|v| v.as_number()).map(|n| n != 0.0).unwrap_or(false);
        let path_str = path.to_std_string_escaped();
        let result = if recursive {
            std::fs::create_dir_all(&path_str)
        } else {
            std::fs::create_dir(&path_str)
        };
        match result {
            Ok(()) => Ok(JsValue::from(boa_engine::js_string!("{\"code\":0}"))),
            Err(e) => Ok(JsValue::from(boa_engine::js_string!(format!("{{\"code\":1,\"value\":\"{}\"}}", e)))),
        }
    });

    // __koss_fs_readdir(path) -> { code: 0, value: json_array }
    reg_fs!("__koss_fs_readdir", crate::sandbox::FS_READ, |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
        let path = args.first().ok_or_else(|| JsNativeError::error().with_message("readdir: path required"))?
            .to_string(ctx).map_err(|_| JsNativeError::error().with_message("readdir: path must be string"))?;
        let path_str = path.to_std_string_escaped();
        match std::fs::read_dir(&path_str) {
            Ok(entries) => {
                let names: Vec<String> = entries.filter_map(|e| e.ok())
                    .filter_map(|e| e.file_name().to_str().map(|s| format!("\"{}\"", s.replace('\\', "\\\\"))))
                    .collect();
                let json = format!("[{}]", names.join(","));
                Ok(JsValue::from(boa_engine::js_string!(format!("{{\"code\":0,\"value\":{}}}", json))))
            }
            Err(e) => Ok(JsValue::from(boa_engine::js_string!(format!("{{\"code\":1,\"value\":\"{}\"}}", e)))),
        }
    });

    // __koss_fs_unlink(path) -> { code: 0 }
    reg_fs!("__koss_fs_unlink", crate::sandbox::FS_DELETE, |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
        let path = args.first().ok_or_else(|| JsNativeError::error().with_message("unlink: path required"))?
            .to_string(ctx).map_err(|_| JsNativeError::error().with_message("unlink: path must be string"))?;
        let path_str = path.to_std_string_escaped();
        let result = std::fs::remove_file(&path_str).or_else(|_| std::fs::remove_dir(&path_str));
        match result {
            Ok(()) => Ok(JsValue::from(boa_engine::js_string!("{\"code\":0}"))),
            Err(e) => Ok(JsValue::from(boa_engine::js_string!(format!("{{\"code\":1,\"value\":\"{}\"}}", e)))),
        }
    });

    // __koss_fs_rename(old, new) -> { code: 0 }
    reg_fs!("__koss_fs_rename", crate::sandbox::FS_RENAME, |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
        if args.len() < 2 {
            return Err(JsNativeError::error().with_message("rename: old and new path required").into());
        }
        let old = args[0].to_string(ctx).map_err(|_| JsNativeError::error().with_message("rename: old path must be string"))?;
        let new = args[1].to_string(ctx).map_err(|_| JsNativeError::error().with_message("rename: new path must be string"))?;
        match std::fs::rename(&old.to_std_string_escaped(), &new.to_std_string_escaped()) {
            Ok(()) => Ok(JsValue::from(boa_engine::js_string!("{\"code\":0}"))),
            Err(e) => Ok(JsValue::from(boa_engine::js_string!(format!("{{\"code\":1,\"value\":\"{}\"}}", e)))),
        }
    });

    // __koss_fs_copy(src, dst) -> { code: 0 }
    reg_fs!("__koss_fs_copy", crate::sandbox::FS_WRITE, |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
        if args.len() < 2 {
            return Err(JsNativeError::error().with_message("copy: src and dst required").into());
        }
        let src = args[0].to_string(ctx).map_err(|_| JsNativeError::error().with_message("copy: src must be string"))?;
        let dst = args[1].to_string(ctx).map_err(|_| JsNativeError::error().with_message("copy: dst must be string"))?;
        match std::fs::copy(&src.to_std_string_escaped(), &dst.to_std_string_escaped()) {
            Ok(_) => Ok(JsValue::from(boa_engine::js_string!("{\"code\":0}"))),
            Err(e) => Ok(JsValue::from(boa_engine::js_string!(format!("{{\"code\":1,\"value\":\"{}\"}}", e)))),
        }
    });

    // __koss_fs_realpath(path) -> path string
    reg_fs!("__koss_fs_realpath", crate::sandbox::FS_READ, |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
        let path = args.first().ok_or_else(|| JsNativeError::error().with_message("realpath: path required"))?
            .to_string(ctx).map_err(|_| JsNativeError::error().with_message("realpath: path must be string"))?;
        let path_str = path.to_std_string_escaped();
        match std::fs::canonicalize(&path_str) {
            Ok(p) => Ok(JsValue::from(boa_engine::js_string!(p.to_string_lossy().to_string()))),
            Err(e) => Err(JsNativeError::error().with_message(format!("realpath failed: {e}")).into()),
        }
    });
}

pub(crate) fn authorize_operation(
    instance: &KossInstance,
    required_cap: u32,
    target: &str,
    args: &[JsValue],
    ctx: &mut Context,
) -> Result<(), JsError> {
    match crate::sandbox::check_audit_decision(
        instance.capabilities,
        instance.sandbox.audit_mask,
        required_cap,
    ) {
        AuditDecision::Allow => Ok(()),
        AuditDecision::DenyCapability => Err(JsNativeError::error()
            .with_message(capability_error_message(target, instance.sandbox.audit_debug))
            .into()),
        AuditDecision::NeedAudit => {
            let Some(audit_fn) = instance.sandbox.sync_audit else {
                return Ok(());
            };
            let c_target = CString::new(target)
                .map_err(|_| JsNativeError::error().with_message("invalid audit target"))?;
            let c_args: Vec<CString> = args
                .iter()
                .map(|arg| CString::new(js_value_to_string(arg, ctx)).unwrap_or_default())
                .collect();
            let arg_ptrs: Vec<*const c_char> = c_args.iter().map(|arg| arg.as_ptr()).collect();
            let pwd = std::env::current_dir()
                .ok()
                .and_then(|path| CString::new(path.to_string_lossy().as_bytes()).ok());
            let allowed = unsafe {
                audit_fn(
                    c_target.as_ptr(),
                    arg_ptrs.as_ptr(),
                    arg_ptrs.len() as i32,
                    pwd.as_ref().map_or(std::ptr::null(), |path| path.as_ptr()),
                    instance.sandbox.sync_userdata,
                )
            };
            if allowed {
                Ok(())
            } else {
                Err(JsNativeError::error()
                    .with_message(security_error_message(target, instance.sandbox.audit_debug))
                    .into())
            }
        }
    }
}

fn register_net_functions(instance: &mut KossInstance) {
    let caps = instance.capabilities;
    let instance_ptr = instance as *mut KossInstance;
    if crate::sandbox::has_cap(caps, crate::sandbox::NET_TCP_CLIENT) {
    // __koss_tcp_connect(host, port) -> fd
    let connect_fn = NativeFunction::from_copy_closure(
        move |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
            let inst = unsafe { &*instance_ptr };
            authorize_operation(inst, crate::sandbox::NET_TCP_CLIENT, "net.tcpConnect", args, ctx)?;
            if args.len() < 2 {
                return Err(JsNativeError::error().with_message("tcp_connect: host and port required").into());
            }
            let host = args[0].to_string(ctx).map_err(|_| JsNativeError::error().with_message("tcp_connect: host must be string"))?;
            let host_str = host.to_std_string_escaped();
            let port = args[1].as_number().ok_or_else(|| JsNativeError::error().with_message("tcp_connect: port must be number"))? as u16;

            if is_ssrf_blocked(&host_str) {
                return Err(JsNativeError::error().with_message(format!("SSRF blocked: {host_str}")).into());
            }

            let addr = format!("{host_str}:{port}");
            match TcpStream::connect(&addr) {
                Ok(stream) => {
                    let _ = stream.set_nonblocking(true);
                    let mut sockets = inst.socket_state.lock().map_err(|_| JsNativeError::error().with_message("socket state unavailable"))?;
                    if sockets.connections.len() >= MAX_TCP_CONNECTIONS {
                        return Err(JsNativeError::error().with_message("TCP connection limit reached").into());
                    }
                    let fd = sockets.next_connection_fd;
                    sockets.next_connection_fd = sockets.next_connection_fd.checked_add(1).ok_or_else(|| JsNativeError::error().with_message("TCP descriptor space exhausted"))?;
                    sockets.connections.insert(fd, stream);
                    Ok(JsValue::from(fd as i32))
                }
                Err(e) => Err(JsNativeError::error().with_message(format!("connect failed: {e}")).into()),
            }
        },
    );
    instance.context.register_global_property(
        js_string!("__koss_tcp_connect"),
        connect_fn.to_js_function(instance.context.realm()),
        boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
    ).ok();

    // __koss_tcp_write(fd, data) -> bytes_written
    let write_fn = NativeFunction::from_copy_closure(
        move |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
            let inst = unsafe { &*instance_ptr };
            authorize_operation(inst, crate::sandbox::NET_TCP_CLIENT, "net.tcpWrite", args, ctx)?;
            if args.len() < 2 {
                return Err(JsNativeError::error().with_message("tcp_write: fd and data required").into());
            }
            let fd = args[0].as_number().ok_or_else(|| JsNativeError::error().with_message("tcp_write: fd must be number"))? as u32;
            let data = args[1].to_string(ctx).map_err(|_| JsNativeError::error().with_message("tcp_write: data must be string"))?;
            let data_str = data.to_std_string_escaped();

            if let Ok(mut sockets) = inst.socket_state.lock() {
                if let Some(stream) = sockets.connections.get_mut(&fd) {
                    use std::io::Write;
                    match stream.write(data_str.as_bytes()) {
                        Ok(n) => return Ok(JsValue::from(n as i32)),
                        Err(e) => return Err(JsNativeError::error().with_message(format!("write failed: {e}")).into()),
                    }
                }
            }
            Err(JsNativeError::error().with_message("tcp_write: invalid fd").into())
        },
    );
    instance.context.register_global_property(
        js_string!("__koss_tcp_write"),
        write_fn.to_js_function(instance.context.realm()),
        boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
    ).ok();

    // __koss_tcp_read(fd) -> string | undefined
    let read_fn = NativeFunction::from_copy_closure(
        move |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
            let inst = unsafe { &*instance_ptr };
            authorize_operation(inst, crate::sandbox::NET_TCP_CLIENT, "net.tcpRead", args, ctx)?;
            if args.is_empty() {
                return Err(JsNativeError::error().with_message("tcp_read: fd required").into());
            }
            let fd = args[0].as_number().ok_or_else(|| JsNativeError::error().with_message("tcp_read: fd must be number"))? as u32;

            if let Ok(mut sockets) = inst.socket_state.lock() {
                if let Some(stream) = sockets.connections.get_mut(&fd) {
                    let mut buf = [0u8; 65536];
                    use std::io::Read;
                    match stream.read(&mut buf) {
                        Ok(0) => return Ok(JsValue::null()),
                        Ok(n) => {
                            let s = String::from_utf8_lossy(&buf[..n]).to_string();
                            return Ok(JsValue::from(js_string!(s)));
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            return Ok(JsValue::null());
                        }
                        Err(e) => return Err(JsNativeError::error().with_message(format!("read failed: {e}")).into()),
                    }
                }
            }
            Err(JsNativeError::error().with_message("tcp_read: invalid fd").into())
        },
    );
    instance.context.register_global_property(
        js_string!("__koss_tcp_read"),
        read_fn.to_js_function(instance.context.realm()),
        boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
    ).ok();

    // __koss_tcp_close(fd)
    let close_fn = NativeFunction::from_copy_closure(
        move |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
            let inst = unsafe { &*instance_ptr };
            authorize_operation(inst, crate::sandbox::NET_TCP_CLIENT, "net.tcpClose", args, ctx)?;
            if args.is_empty() {
                return Err(JsNativeError::error().with_message("tcp_close: fd required").into());
            }
            let fd = args[0].as_number().ok_or_else(|| JsNativeError::error().with_message("tcp_close: fd must be number"))? as u32;

            if let Ok(mut sockets) = inst.socket_state.lock() {
                sockets.connections.remove(&fd);
                sockets.servers.remove(&fd);
            }
            Ok(JsValue::undefined())
        },
    );
    instance.context.register_global_property(
        js_string!("__koss_tcp_close"),
        close_fn.to_js_function(instance.context.realm()),
        boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
    ).ok();
    }

    // __koss_tcp_listen(host, port, backlog) -> server_fd
    if crate::sandbox::has_cap(caps, crate::sandbox::NET_TCP_SERVER) {
    let listen_fn = NativeFunction::from_copy_closure(
        move |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
            let inst = unsafe { &*instance_ptr };
            authorize_operation(inst, crate::sandbox::NET_TCP_SERVER, "net.tcpListen", args, ctx)?;
            if args.len() < 2 {
                return Err(JsNativeError::error().with_message("tcp_listen: host and port required").into());
            }
            let host = args[0].to_string(ctx).map_err(|_| JsNativeError::error().with_message("tcp_listen: host must be string"))?;
            let host_str = host.to_std_string_escaped();
            let port = args[1].as_number().ok_or_else(|| JsNativeError::error().with_message("tcp_listen: port must be number"))? as u16;

            if is_ssrf_blocked(&host_str) {
                return Err(JsNativeError::error().with_message(format!("SSRF blocked: {host_str}")).into());
            }

            let addr = format!("{host_str}:{port}");
            match TcpListener::bind(&addr) {
                Ok(listener) => {
                    let _ = listener.set_nonblocking(true);
                    let mut sockets = inst.socket_state.lock().map_err(|_| JsNativeError::error().with_message("socket state unavailable"))?;
                    if sockets.servers.len() >= MAX_TCP_SERVERS {
                        return Err(JsNativeError::error().with_message("TCP server limit reached").into());
                    }
                    let fd = sockets.next_server_fd;
                    sockets.next_server_fd = sockets.next_server_fd.checked_add(1).ok_or_else(|| JsNativeError::error().with_message("TCP server descriptor space exhausted"))?;
                    sockets.servers.insert(fd, listener);
                    Ok(JsValue::from(fd as i32))
                }
                Err(e) => Err(JsNativeError::error().with_message(format!("listen failed: {e}")).into()),
            }
        },
    );
    instance.context.register_global_property(
        js_string!("__koss_tcp_listen"),
        listen_fn.to_js_function(instance.context.realm()),
        boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
    ).ok();

    // __koss_tcp_accept(server_fd) -> client_fd | undefined
    let accept_fn = NativeFunction::from_copy_closure(
        move |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
            let inst = unsafe { &*instance_ptr };
            authorize_operation(inst, crate::sandbox::NET_TCP_SERVER, "net.tcpAccept", args, ctx)?;
            if args.is_empty() {
                return Err(JsNativeError::error().with_message("tcp_accept: server_fd required").into());
            }
            let sfd = args[0].as_number().ok_or_else(|| JsNativeError::error().with_message("tcp_accept: server_fd must be number"))? as u32;

            if let Ok(mut sockets) = inst.socket_state.lock() {
                let accepted = sockets.servers.get_mut(&sfd).map(|listener| listener.accept());
                if let Some(result) = accepted {
                    match result {
                        Ok((stream, _peer_addr)) => {
                            let _ = stream.set_nonblocking(true);
                            if sockets.connections.len() >= MAX_TCP_CONNECTIONS {
                                return Err(JsNativeError::error().with_message("TCP connection limit reached").into());
                            }
                            let fd = sockets.next_connection_fd;
                            sockets.next_connection_fd = sockets.next_connection_fd.checked_add(1).ok_or_else(|| JsNativeError::error().with_message("TCP descriptor space exhausted"))?;
                            sockets.connections.insert(fd, stream);
                            return Ok(JsValue::from(fd as i32));
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            return Ok(JsValue::null());
                        }
                        Err(e) => return Err(JsNativeError::error().with_message(format!("accept failed: {e}")).into()),
                    }
                }
            }
            Err(JsNativeError::error().with_message("tcp_accept: invalid server_fd").into())
        },
    );
    instance.context.register_global_property(
        js_string!("__koss_tcp_accept"),
        accept_fn.to_js_function(instance.context.realm()),
        boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
    ).ok();
    }

    // __koss_dns_lookup(hostname) -> json_string of IP addresses
    if crate::sandbox::has_cap(caps, crate::sandbox::NET_DNS) {
    let dns_fn = NativeFunction::from_copy_closure(
        move |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
            let inst = unsafe { &*instance_ptr };
            authorize_operation(inst, crate::sandbox::NET_DNS, "net.dnsLookup", args, ctx)?;
            if args.is_empty() {
                return Err(JsNativeError::error().with_message("dns_lookup: hostname required").into());
            }
            let hostname = args[0].to_string(ctx).map_err(|_| JsNativeError::error().with_message("dns_lookup: hostname must be string"))?;
            let host_str = hostname.to_std_string_escaped();

            let addr = format!("{host_str}:80");
            match addr.to_socket_addrs() {
                Ok(addrs) => {
                    let ips: Vec<String> = addrs.map(|a| a.ip().to_string()).collect();
                    let json = serde_json::json!(ips).to_string();
                    Ok(JsValue::from(js_string!(json)))
                }
                Err(e) => Err(JsNativeError::error().with_message(format!("dns lookup failed: {e}")).into()),
            }
        },
    );
    instance.context.register_global_property(
        js_string!("__koss_dns_lookup"),
        dns_fn.to_js_function(instance.context.realm()),
        boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
    ).ok();
    }
}

fn register_runtime_capabilities(instance: &mut KossInstance) {
    let caps = instance.capabilities;
    if crate::sandbox::has_cap(caps, crate::sandbox::NET_FETCH) {
        register_fetch_polyfill(&mut instance.context);
        register_native_fetch(instance);
    }
    if caps & (crate::sandbox::NET_TCP_CLIENT
        | crate::sandbox::NET_TCP_SERVER
        | crate::sandbox::NET_DNS) != 0
    {
        register_net_functions(instance);
    }
    if caps & KOSS_CAP_ALL_CRYPTO != 0 {
        register_crypto_functions(instance);
    }
    register_zlib_functions_all(instance);
}

fn register_crypto_functions(instance: &mut KossInstance) {
    let caps = instance.capabilities;
    let ctx = &mut instance.context;

    if crate::sandbox::has_cap(caps, crate::sandbox::CRYPTO_RANDOM) {
    let rand_fn = NativeFunction::from_copy_closure(
        move |_this: &JsValue, args: &[JsValue], _ctx: &mut Context| -> Result<JsValue, JsError> {
            let size = args.first()
                .map(|v| v.to_number(_ctx).unwrap_or(32.0) as usize)
                .unwrap_or(32);
            let data = crate::bindings::crypto::get_random_values(size);
            let json = serde_json::json!(data).to_string();
            Ok(JsValue::from(js_string!(json)))
        },
    );
    ctx.register_global_property(
        js_string!("__koss_random_bytes"),
        rand_fn.to_js_function(ctx.realm()),
        boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
    ).ok();
    }

    if crate::sandbox::has_cap(caps, crate::sandbox::CRYPTO_HASH) {
    let hash_fn = NativeFunction::from_copy_closure(
        move |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
            if args.len() < 2 {
                return Err(JsNativeError::error().with_message("hash: algorithm and data required").into());
            }
            let algo = args[0].to_string(ctx).map_err(|_| JsNativeError::error().with_message("hash: algorithm must be string"))?;
            let data = args[1].to_string(ctx).map_err(|_| JsNativeError::error().with_message("hash: data must be string"))?;
            let algo_str = algo.to_std_string_escaped();
            let data_str = data.to_std_string_escaped();
            match crate::bindings::crypto::create_hash(&algo_str, &data_str) {
                Ok(hex) => Ok(JsValue::from(js_string!(hex))),
                Err(e) => Err(JsNativeError::error().with_message(format!("hash failed: {e}")).into()),
            }
        },
    );
    ctx.register_global_property(
        js_string!("__koss_hash"),
        hash_fn.to_js_function(ctx.realm()),
        boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
    ).ok();
    }

    if crate::sandbox::has_cap(caps, crate::sandbox::CRYPTO_RANDOM) {
    let random_uuid_fn = NativeFunction::from_copy_closure(
        move |_this: &JsValue, _args: &[JsValue], _ctx: &mut Context| -> Result<JsValue, JsError> {
            let uuid = crate::bindings::crypto::random_uuid();
            Ok(JsValue::from(js_string!(uuid)))
        },
    );
    ctx.register_global_property(
        js_string!("__koss_random_uuid"),
        random_uuid_fn.to_js_function(ctx.realm()),
        boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
    ).ok();
    }

    // ===== Bytes-based crypto functions =====

    if crate::sandbox::has_cap(caps, crate::sandbox::CRYPTO_HASH) {
    let hash_bytes_fn = NativeFunction::from_copy_closure(
        move |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
            if args.len() < 2 {
                return Err(JsNativeError::error().with_message("hashBytes: algorithm and data required").into());
            }
            let algo = args[0].to_string(ctx).map_err(|_| JsNativeError::error().with_message("hashBytes: algorithm must be string"))?;
            let data_val = args.get(1).ok_or_else(|| JsNativeError::error().with_message("hashBytes: data required"))?;
            let data_bytes = js_value_to_bytes(data_val, ctx)?;
            let algo_str = algo.to_std_string_escaped();
            match crate::bindings::crypto::hash_bytes(&algo_str, &data_bytes) {
                Ok(hash) => Ok(bytes_to_js_value(&hash)),
                Err(e) => Err(JsNativeError::error().with_message(format!("hashBytes failed: {e}")).into()),
            }
        },
    );
    ctx.register_global_property(
        js_string!("__koss_hash_bytes"),
        hash_bytes_fn.to_js_function(ctx.realm()),
        boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
    ).ok();
    }

    if crate::sandbox::has_cap(caps, crate::sandbox::CRYPTO_HMAC) {
    let hmac_bytes_fn = NativeFunction::from_copy_closure(
        move |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
            if args.len() < 3 {
                return Err(JsNativeError::error().with_message("hmacBytes: algorithm, key, and data required").into());
            }
            let algo = args[0].to_string(ctx).map_err(|_| JsNativeError::error().with_message("hmacBytes: algorithm must be string"))?;
            let key_bytes = js_value_to_bytes(args.get(1).unwrap(), ctx)?;
            let data_bytes = js_value_to_bytes(args.get(2).unwrap(), ctx)?;
            let algo_str = algo.to_std_string_escaped();
            match crate::bindings::crypto::hmac_bytes(&algo_str, &key_bytes, &data_bytes) {
                Ok(mac) => Ok(bytes_to_js_value(&mac)),
                Err(e) => Err(JsNativeError::error().with_message(format!("hmacBytes failed: {e}")).into()),
            }
        },
    );
    ctx.register_global_property(
        js_string!("__koss_hmac_bytes"),
        hmac_bytes_fn.to_js_function(ctx.realm()),
        boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
    ).ok();
    }

    if crate::sandbox::has_cap(caps, crate::sandbox::CRYPTO_PBKDF2) {
    let pbkdf2_bytes_fn = NativeFunction::from_copy_closure(
        move |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
            if args.len() < 4 {
                return Err(JsNativeError::error().with_message("pbkdf2Bytes: password, salt, iterations, keyLen required").into());
            }
            let password = js_value_to_bytes(args.get(0).unwrap(), ctx)?;
            let salt = js_value_to_bytes(args.get(1).unwrap(), ctx)?;
            let iterations = args.get(2).and_then(|v| v.as_number()).unwrap_or(100_000.0) as u32;
            let key_len = args.get(3).and_then(|v| v.as_number()).unwrap_or(32.0) as u32;
            match crate::bindings::crypto::pbkdf2_bytes(&password, &salt, iterations, key_len) {
                Ok(key) => Ok(bytes_to_js_value(&key)),
                Err(e) => Err(JsNativeError::error().with_message(format!("pbkdf2Bytes failed: {e}")).into()),
            }
        },
    );
    ctx.register_global_property(
        js_string!("__koss_pbkdf2_bytes"),
        pbkdf2_bytes_fn.to_js_function(ctx.realm()),
        boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
    ).ok();
    }

    if crate::sandbox::has_cap(caps, crate::sandbox::CRYPTO_HMAC) {
    let aes_gcm_encrypt_fn = NativeFunction::from_copy_closure(
        move |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
            if args.len() < 4 {
                return Err(JsNativeError::error().with_message("aesGcmEncrypt: key, nonce, aad, plaintext required").into());
            }
            let key = js_value_to_bytes(args.get(0).unwrap(), ctx)?;
            let nonce = js_value_to_bytes(args.get(1).unwrap(), ctx)?;
            let aad = js_value_to_bytes(args.get(2).unwrap(), ctx)?;
            let plaintext = js_value_to_bytes(args.get(3).unwrap(), ctx)?;
            match crate::bindings::crypto::aes_gcm_encrypt(&key, &nonce, &aad, &plaintext) {
                Ok(ct) => Ok(bytes_to_js_value(&ct)),
                Err(e) => Err(JsNativeError::error().with_message(format!("aesGcmEncrypt failed: {e}")).into()),
            }
        },
    );
    ctx.register_global_property(
        js_string!("__koss_aes_gcm_encrypt"),
        aes_gcm_encrypt_fn.to_js_function(ctx.realm()),
        boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
    ).ok();

    let aes_gcm_decrypt_fn = NativeFunction::from_copy_closure(
        move |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
            if args.len() < 4 {
                return Err(JsNativeError::error().with_message("aesGcmDecrypt: key, nonce, aad, ciphertext required").into());
            }
            let key = js_value_to_bytes(args.get(0).unwrap(), ctx)?;
            let nonce = js_value_to_bytes(args.get(1).unwrap(), ctx)?;
            let aad = js_value_to_bytes(args.get(2).unwrap(), ctx)?;
            let ciphertext = js_value_to_bytes(args.get(3).unwrap(), ctx)?;
            match crate::bindings::crypto::aes_gcm_decrypt(&key, &nonce, &aad, &ciphertext) {
                Ok(pt) => Ok(bytes_to_js_value(&pt)),
                Err(e) => Err(JsNativeError::error().with_message(format!("aesGcmDecrypt failed: {e}")).into()),
            }
        },
    );
    ctx.register_global_property(
        js_string!("__koss_aes_gcm_decrypt"),
        aes_gcm_decrypt_fn.to_js_function(ctx.realm()),
        boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
    ).ok();
    }

    if crate::sandbox::has_cap(caps, crate::sandbox::CRYPTO_RANDOM) {
    let ed25519_keypair_fn = NativeFunction::from_copy_closure(
        move |_this: &JsValue, _args: &[JsValue], _ctx: &mut Context| -> Result<JsValue, JsError> {
            match crate::bindings::crypto::ed25519_keypair() {
                Ok((pubkey, privkey)) => {
                    let obj = JsObject::with_null_proto();
                    let _ = obj.set(js_string!("publicKey"), bytes_to_js_value(&pubkey), false, _ctx);
                    let _ = obj.set(js_string!("privateKey"), bytes_to_js_value(&privkey), false, _ctx);
                    Ok(JsValue::from(obj))
                }
                Err(e) => Err(JsNativeError::error().with_message(format!("ed25519KeyPair failed: {e}")).into()),
            }
        },
    );
    ctx.register_global_property(
        js_string!("__koss_ed25519_keypair"),
        ed25519_keypair_fn.to_js_function(ctx.realm()),
        boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
    ).ok();
    }

    if crate::sandbox::has_cap(caps, crate::sandbox::CRYPTO_HASH) {
    let ed25519_sign_fn = NativeFunction::from_copy_closure(
        move |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
            if args.len() < 2 {
                return Err(JsNativeError::error().with_message("ed25519Sign: privateKey and message required").into());
            }
            let privkey = js_value_to_bytes(args.get(0).unwrap(), ctx)?;
            let message = js_value_to_bytes(args.get(1).unwrap(), ctx)?;
            match crate::bindings::crypto::ed25519_sign(&privkey, &message) {
                Ok(sig) => Ok(bytes_to_js_value(&sig)),
                Err(e) => Err(JsNativeError::error().with_message(format!("ed25519Sign failed: {e}")).into()),
            }
        },
    );
    ctx.register_global_property(
        js_string!("__koss_ed25519_sign"),
        ed25519_sign_fn.to_js_function(ctx.realm()),
        boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
    ).ok();

    let ed25519_verify_fn = NativeFunction::from_copy_closure(
        move |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
            if args.len() < 3 {
                return Err(JsNativeError::error().with_message("ed25519Verify: publicKey, message, and signature required").into());
            }
            let pubkey = js_value_to_bytes(args.get(0).unwrap(), ctx)?;
            let message = js_value_to_bytes(args.get(1).unwrap(), ctx)?;
            let signature = js_value_to_bytes(args.get(2).unwrap(), ctx)?;
            match crate::bindings::crypto::ed25519_verify(&pubkey, &message, &signature) {
                Ok(valid) => Ok(JsValue::from(valid)),
                Err(e) => Err(JsNativeError::error().with_message(format!("ed25519Verify failed: {e}")).into()),
            }
        },
    );
    ctx.register_global_property(
        js_string!("__koss_ed25519_verify"),
        ed25519_verify_fn.to_js_function(ctx.realm()),
        boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
    ).ok();
    }

    if crate::sandbox::has_cap(caps, crate::sandbox::CRYPTO_HASH) {
    let timing_safe_equal_fn = NativeFunction::from_copy_closure(
        move |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
            if args.len() < 2 {
                return Err(JsNativeError::error().with_message("timingSafeEqual: two arguments required").into());
            }
            let a = js_value_to_bytes(args.get(0).unwrap(), ctx)?;
            let b = js_value_to_bytes(args.get(1).unwrap(), ctx)?;
            Ok(JsValue::from(crate::bindings::crypto::timing_safe_equal(&a, &b)))
        },
    );
    ctx.register_global_property(
        js_string!("__koss_timing_safe_equal"),
        timing_safe_equal_fn.to_js_function(ctx.realm()),
        boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
    ).ok();
    }
}

fn bytes_to_json_arr(data: &[u8]) -> String {
    let parts: Vec<String> = data.iter().map(|b| b.to_string()).collect();
    format!("[{}]", parts.join(","))
}

fn json_arr_to_bytes(json: &str) -> Result<Vec<u8>, String> {
    let v: Vec<u8> = serde_json::from_str(json).map_err(|e| format!("parse error: {e}"))?;
    Ok(v)
}

fn js_value_to_bytes(val: &JsValue, ctx: &mut Context) -> Result<Vec<u8>, JsError> {
    if let Some(s) = val.as_string() {
        let s = s.to_std_string_escaped();
        if s.starts_with('[') {
            return json_arr_to_bytes(&s)
                .map_err(|e| JsNativeError::error().with_message(format!("byte parse error: {e}")).into());
        }
        return Ok(s.into_bytes());
    }
    if let Some(n) = val.as_number() {
        return Ok(vec![n as u8]);
    }
    if let Some(obj) = val.as_object() {
        let len_val = obj.get(js_string!("length"), ctx)?;
        let len = len_val.to_number(ctx)? as usize;
        let mut bytes = Vec::with_capacity(len);
        for i in 0..len {
            let v = obj.get(i, ctx)?;
            bytes.push(v.to_number(ctx)? as u8);
        }
        return Ok(bytes);
    }
    Err(JsNativeError::error().with_message("expected Uint8Array, Buffer, or byte array").into())
}

fn bytes_to_js_value(data: &[u8]) -> JsValue {
    let json = serde_json::json!(data).to_string();
    JsValue::from(js_string!(json))
}

fn register_zlib_functions(ctx: &mut Context) {
    let gzip_fn = NativeFunction::from_copy_closure(
        move |_this: &JsValue, args: &[JsValue], _ctx: &mut Context| -> Result<JsValue, JsError> {
            use std::io::Read;
            use flate2::Compression;
            use flate2::read::GzEncoder;
            if args.is_empty() {
                return Err(JsNativeError::error().with_message("gzip: data required").into());
            }
            let json_str = args[0].to_string(_ctx).map_err(|_| JsNativeError::error().with_message("gzip: data must be string"))?;
            let input = json_arr_to_bytes(&json_str.to_std_string_escaped())
                .map_err(|e| JsNativeError::error().with_message(format!("gzip: {e}")))?;
            let mut encoder = GzEncoder::new(&input[..], Compression::default());
            let mut out = Vec::new();
            encoder.read_to_end(&mut out).map_err(|e| JsNativeError::error().with_message(format!("gzip: {e}")))?;
            Ok(JsValue::from(js_string!(bytes_to_json_arr(&out))))
        },
    );
    ctx.register_global_property(
        js_string!("__koss_gzip"),
        gzip_fn.to_js_function(ctx.realm()),
        boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
    ).ok();

    let gunzip_fn = NativeFunction::from_copy_closure(
        move |_this: &JsValue, args: &[JsValue], _ctx: &mut Context| -> Result<JsValue, JsError> {
            use std::io::Read;
            use flate2::read::GzDecoder;
            if args.is_empty() {
                return Err(JsNativeError::error().with_message("gunzip: data required").into());
            }
            let json_str = args[0].to_string(_ctx).map_err(|_| JsNativeError::error().with_message("gunzip: data must be string"))?;
            let input = json_arr_to_bytes(&json_str.to_std_string_escaped())
                .map_err(|e| JsNativeError::error().with_message(format!("gunzip: {e}")))?;
            let mut decoder = GzDecoder::new(&input[..]);
            let mut out = Vec::new();
            decoder.read_to_end(&mut out).map_err(|e| JsNativeError::error().with_message(format!("gunzip: {e}")))?;
            Ok(JsValue::from(js_string!(bytes_to_json_arr(&out))))
        },
    );
    ctx.register_global_property(
        js_string!("__koss_gunzip"),
        gunzip_fn.to_js_function(ctx.realm()),
        boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
    ).ok();

    let deflate_fn = NativeFunction::from_copy_closure(
        move |_this: &JsValue, args: &[JsValue], _ctx: &mut Context| -> Result<JsValue, JsError> {
            // use std::io::{Read, Write};
            use std::io::{Read};
            use flate2::Compression;
            use flate2::read::DeflateEncoder;
            if args.is_empty() {
                return Err(JsNativeError::error().with_message("deflate: data required").into());
            }
            let json_str = args[0].to_string(_ctx).map_err(|_| JsNativeError::error().with_message("deflate: data must be string"))?;
            let input = json_arr_to_bytes(&json_str.to_std_string_escaped())
                .map_err(|e| JsNativeError::error().with_message(format!("deflate: {e}")))?;
            let mut encoder = DeflateEncoder::new(&input[..], Compression::default());
            let mut out = Vec::new();
            encoder.read_to_end(&mut out).map_err(|e| JsNativeError::error().with_message(format!("deflate: {e}")))?;
            Ok(JsValue::from(js_string!(bytes_to_json_arr(&out))))
        },
    );
    ctx.register_global_property(
        js_string!("__koss_deflate"),
        deflate_fn.to_js_function(ctx.realm()),
        boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
    ).ok();

    let inflate_fn = NativeFunction::from_copy_closure(
        move |_this: &JsValue, args: &[JsValue], _ctx: &mut Context| -> Result<JsValue, JsError> {
            use std::io::Read;
            use flate2::read::DeflateDecoder;
            if args.is_empty() {
                return Err(JsNativeError::error().with_message("inflate: data required").into());
            }
            let json_str = args[0].to_string(_ctx).map_err(|_| JsNativeError::error().with_message("inflate: data must be string"))?;
            let input = json_arr_to_bytes(&json_str.to_std_string_escaped())
                .map_err(|e| JsNativeError::error().with_message(format!("inflate: {e}")))?;
            let mut decoder = DeflateDecoder::new(&input[..]);
            let mut out = Vec::new();
            decoder.read_to_end(&mut out).map_err(|e| JsNativeError::error().with_message(format!("inflate: {e}")))?;
            Ok(JsValue::from(js_string!(bytes_to_json_arr(&out))))
        },
    );
    ctx.register_global_property(
        js_string!("__koss_inflate"),
        inflate_fn.to_js_function(ctx.realm()),
        boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
    ).ok();
}

fn register_zlib_functions_all(instance: &mut KossInstance) {
    register_zlib_functions(&mut instance.context);
}

fn register_nodejs_globals(ctx: &mut Context) {
    // Register primordials
    let primordials_code = r#"
    const primordials = globalThis.primordials = {
        Array: Array,
        ArrayBuffer: ArrayBuffer,
        ArrayBufferIsView: ArrayBuffer.isView,
        ArrayIsArray: Array.isArray,
        ArrayPrototype: Array.prototype,
        ArrayPrototypeEntries: Array.prototype.entries,
        ArrayPrototypeEvery: Array.prototype.every,
        ArrayPrototypeFill: Array.prototype.fill,
        ArrayPrototypeFilter: Array.prototype.filter,
        ArrayPrototypeFind: Array.prototype.find,
        ArrayPrototypeFindIndex: Array.prototype.findIndex,
        ArrayPrototypeForEach: Array.prototype.forEach,
        ArrayPrototypeIncludes: Array.prototype.includes,
        ArrayPrototypeIndexOf: Array.prototype.indexOf,
        ArrayPrototypeJoin: Array.prototype.join,
        ArrayPrototypeKeys: Array.prototype.keys,
        ArrayPrototypeLastIndexOf: Array.prototype.lastIndexOf,
        ArrayPrototypeMap: Array.prototype.map,
        ArrayPrototypePop: Array.prototype.pop,
        ArrayPrototypePush: Array.prototype.push,
        ArrayPrototypeReduce: Array.prototype.reduce,
        ArrayPrototypeReduceRight: Array.prototype.reduceRight,
        ArrayPrototypeReverse: Array.prototype.reverse,
        ArrayPrototypeShift: Array.prototype.shift,
        ArrayPrototypeSlice: Array.prototype.slice,
        ArrayPrototypeSome: Array.prototype.some,
        ArrayPrototypeSort: Array.prototype.sort,
        ArrayPrototypeSplice: Array.prototype.splice,
        ArrayPrototypeUnshift: Array.prototype.unshift,
        ArrayPrototypeValues: Array.prototype.values,
        BigInt: BigInt,
        BigInt64Array: BigInt64Array,
        Boolean: Boolean,
        DataView: DataView,
        Date: Date,
        Error: Error,
        EvalError: EvalError,
        Float32Array: Float32Array,
        Float64Array: Float64Array,
        Function: Function,
        Int8Array: Int8Array,
        Int16Array: Int16Array,
        Int32Array: Int32Array,
        Map: Map,
        MapPrototype: Map.prototype,
        MapPrototypeEntries: Map.prototype.entries,
        MapPrototypeForEach: Map.prototype.forEach,
        MapPrototypeGet: Map.prototype.get,
        MapPrototypeHas: Map.prototype.has,
        MapPrototypeKeys: Map.prototype.keys,
        MapPrototypeSet: Map.prototype.set,
        MapPrototypeValues: Map.prototype.values,
        Math: Math,
        Number: Number,
        NumberIsFinite: Number.isFinite,
        NumberIsInteger: Number.isInteger,
        NumberIsNaN: Number.isNaN,
        NumberMAX_SAFE_INTEGER: Number.MAX_SAFE_INTEGER,
        NumberMIN_SAFE_INTEGER: Number.MIN_SAFE_INTEGER,
        Object: Object,
        ObjectAssign: Object.assign,
        ObjectCreate: Object.create,
        ObjectDefineProperty: Object.defineProperty,
        ObjectDefineProperties: Object.defineProperties,
        ObjectFreeze: Object.freeze,
        ObjectGetOwnPropertyDescriptor: Object.getOwnPropertyDescriptor,
        ObjectGetOwnPropertyNames: Object.getOwnPropertyNames,
        ObjectGetPrototypeOf: Object.getPrototypeOf,
        ObjectHasOwn: Object.hasOwn,
        ObjectIs: Object.is,
        ObjectIsExtensible: Object.isExtensible,
        ObjectIsFrozen: Object.isFrozen,
        ObjectKeys: Object.keys,
        ObjectPrototype: Object.prototype,
        ObjectPrototypeHasOwnProperty: Object.prototype.hasOwnProperty,
        ObjectPrototypeToString: Object.prototype.toString,
        ObjectSeal: Object.seal,
        ObjectSetPrototypeOf: Object.setPrototypeOf,
        Promise: Promise,
        PromiseAll: Promise.all,
        PromiseAllSettled: Promise.allSettled,
        PromiseAny: Promise.any,
        PromiseRace: Promise.race,
        PromisePrototypeThen: Promise.prototype.then,
        PromisePrototypeCatch: Promise.prototype.catch,
        PromisePrototypeFinally: Promise.prototype.finally,
        Proxy: Proxy,
        RangeError: RangeError,
        ReferenceError: ReferenceError,
        Reflect: Reflect,
        ReflectApply: Reflect.apply,
        ReflectConstruct: Reflect.construct,
        ReflectDefineProperty: Reflect.defineProperty,
        ReflectDeleteProperty: Reflect.deleteProperty,
        ReflectGet: Reflect.get,
        ReflectGetOwnPropertyDescriptor: Reflect.getOwnPropertyDescriptor,
        ReflectGetPrototypeOf: Reflect.getPrototypeOf,
        ReflectHas: Reflect.has,
        ReflectIsExtensible: Reflect.isExtensible,
        ReflectOwnKeys: Reflect.ownKeys,
        ReflectPreventExtensions: Reflect.preventExtensions,
        ReflectSet: Reflect.set,
        ReflectSetPrototypeOf: Reflect.setPrototypeOf,
        RegExp: RegExp,
        RegExpPrototype: RegExp.prototype,
        RegExpPrototypeExec: RegExp.prototype.exec,
        RegExpPrototypeTest: RegExp.prototype.test,
        Set: Set,
        SetPrototype: Set.prototype,
        SetPrototypeEntries: Set.prototype.entries,
        SetPrototypeForEach: Set.prototype.forEach,
        SetPrototypeHas: Set.prototype.has,
        SetPrototypeValues: Set.prototype.values,
        SharedArrayBuffer: SharedArrayBuffer,
        String: String,
        StringPrototype: String.prototype,
        StringPrototypeCharAt: String.prototype.charAt,
        StringPrototypeCharCodeAt: String.prototype.charCodeAt,
        StringPrototypeCodePointAt: String.prototype.codePointAt,
        StringPrototypeConcat: String.prototype.concat,
        StringPrototypeEndsWith: String.prototype.endsWith,
        StringPrototypeIncludes: String.prototype.includes,
        StringPrototypeIndexOf: String.prototype.indexOf,
        StringPrototypeLastIndexOf: String.prototype.lastIndexOf,
        StringPrototypeMatch: String.prototype.match,
        StringPrototypeMatchAll: String.prototype.matchAll,
        StringPrototypePadEnd: String.prototype.padEnd,
        StringPrototypePadStart: String.prototype.padStart,
        StringPrototypeRepeat: String.prototype.repeat,
        StringPrototypeReplace: String.prototype.replace,
        StringPrototypeReplaceAll: String.prototype.replaceAll,
        StringPrototypeSearch: String.prototype.search,
        StringPrototypeSlice: String.prototype.slice,
        StringPrototypeSplit: String.prototype.split,
        StringPrototypeStartsWith: String.prototype.startsWith,
        StringPrototypeSubstring: String.prototype.substring,
        StringPrototypeToLowerCase: String.prototype.toLowerCase,
        StringPrototypeToString: String.prototype.toString,
        StringPrototypeToUpperCase: String.prototype.toUpperCase,
        StringPrototypeTrim: String.prototype.trim,
        StringPrototypeTrimEnd: String.prototype.trimEnd,
        StringPrototypeTrimStart: String.prototype.trimStart,
        Symbol: Symbol,
        SymbolFor: Symbol.for,
        SymbolPrototype: Symbol.prototype,
        SymbolPrototypeToString: Symbol.prototype.toString,
        SyntaxError: SyntaxError,
        TypeError: TypeError,
        Uint8Array: Uint8Array,
        Uint8ClampedArray: Uint8ClampedArray,
        Uint16Array: Uint16Array,
        Uint32Array: Uint32Array,
        URIError: URIError,
        WeakMap: WeakMap,
        WeakMapPrototype: WeakMap.prototype,
        WeakMapPrototypeDelete: WeakMap.prototype.delete,
        WeakMapPrototypeGet: WeakMap.prototype.get,
        WeakMapPrototypeHas: WeakMap.prototype.has,
        WeakMapPrototypeSet: WeakMap.prototype.set,
        WeakSet: WeakSet,
        WeakSetPrototype: WeakSet.prototype,
        WeakSetPrototypeDelete: WeakSet.prototype.delete,
        WeakSetPrototypeHas: WeakSet.prototype.has,
        WeakSetPrototypeSet: WeakSet.prototype.set,
        JSON: JSON,
        JSONParse: JSON.parse,
        JSONStringify: JSON.stringify,
        MathAbs: Math.abs,
        MathAcos: Math.acos,
        MathAcosh: Math.acosh,
        MathAsin: Math.asin,
        MathAsinh: Math.asinh,
        MathAtan: Math.atan,
        MathAtanh: Math.atanh,
        MathAtan2: Math.atan2,
        MathCeil: Math.ceil,
        MathClz32: Math.clz32,
        MathCos: Math.cos,
        MathCosh: Math.cosh,
        MathExp: Math.exp,
        MathExpm1: Math.expm1,
        MathFloor: Math.floor,
        MathFround: Math.fround,
        MathImul: Math.imul,
        MathLog: Math.log,
        MathLog10: Math.log10,
        MathLog1p: Math.log1p,
        MathLog2: Math.log2,
        MathMax: Math.max,
        MathMin: Math.min,
        MathPow: Math.pow,
        MathRandom: Math.random,
        MathRound: Math.round,
        MathSign: Math.sign,
        MathSin: Math.sin,
        MathSinh: Math.sinh,
        MathSqrt: Math.sqrt,
        MathTan: Math.tan,
        MathTanh: Math.tanh,
        MathTrunc: Math.trunc,
        parseInt: parseInt,
        parseFloat: parseFloat,
        isFinite: isFinite,
        isNaN: isNaN,
        decodeURI: decodeURI,
        decodeURIComponent: decodeURIComponent,
        encodeURI: encodeURI,
        encodeURIComponent: encodeURIComponent,
        eval: eval,
        undefined: undefined,
        Infinity: Infinity,
        NaN: NaN,
        DateNow: Date.now,
        DateParse: Date.parse,
        DateUTC: Date.UTC,
        SafeMap: Map,
        SafeSet: Set,
        SafeWeakMap: WeakMap,
        SafeWeakSet: WeakSet,
    };
    
    // internalBinding - calls Rust implementations via __koss_bindings
    const internalBinding = function(name) {
        const result = __koss_bindings(name);
        if (typeof result === 'string') {
            return JSON.parse(result);
        }
        return result || {};
    };
    globalThis.internalBinding = internalBinding;

    // Stubs for Node.js internal functions
    const getInternalBinding = function(name) {
        return {};
    };
    globalThis.getInternalBinding = getInternalBinding;

    const getLinkedBinding = function(name) {
        return {};
    };
    globalThis.getLinkedBinding = getLinkedBinding;

    // Simple setTimeout/clearTimeout using nextTick
    var _timerCounter = 0;
    var _timers = {};
    globalThis.setTimeout = function(fn, ms) {
        if (typeof fn !== 'function') return 0;
        ms = ms || 0;
        var id = ++_timerCounter;
        var start = Date.now();
        var ticker = function() {
            if (!_timers[id]) return;
            if (Date.now() - start >= ms) {
                delete _timers[id];
                try { fn(); } catch(e) {}
            } else {
                process.nextTick(ticker);
            }
        };
        _timers[id] = true;
        if (ms === 0) {
            process.nextTick(function() {
                if (_timers[id]) { delete _timers[id]; try { fn(); } catch(e) {} }
            });
        } else {
            process.nextTick(ticker);
        }
        return id;
    };
    globalThis.clearTimeout = function(id) {
        delete _timers[id];
    };
    globalThis.setInterval = function(fn, ms) {
        if (typeof fn !== 'function') return 0;
        ms = ms || 0;
        var id = ++_timerCounter;
        var next_run = Date.now() + ms;
        var ticker = function() {
            if (!_timers[id]) return;
            if (Date.now() >= next_run) {
                try { fn(); } catch(e) {}
                next_run = Date.now() + ms;
            }
            process.nextTick(ticker);
        };
        _timers[id] = true;
        process.nextTick(ticker);
        return id;
    };
    globalThis.clearInterval = function(id) {
        delete _timers[id];
    };
    "#;

    let source = boa_parser::Source::from_bytes(primordials_code.as_bytes());
    match ctx.eval(source) {
        Ok(_) => {
            // Freeze all primordials prototypes to prevent prototype pollution (CWE-1321)
            let _ = ctx.eval(boa_parser::Source::from_bytes(
                b"(function(){var p=globalThis.primordials;for(var k in p){var v=p[k];if(v&&typeof v==='object'&&v!==null)try{Object.freeze(v)}catch(e){}}})()",
            ));
        }
        Err(e) => {
            eprintln!("Warning: Failed to register primordials: {:?}", e);
        }
    }

    // Register process as a minimal stub
    let platform_str = match std::env::consts::OS {
        "windows" => "win32",
        "macos" => "darwin",
        other => other,
    };
    let arch_str = match std::env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        "arm" => "arm",
        other => other,
    };

    let process_code_template = r#"
const process = {
    version: '20.0.0',
    versions: {
        node: '20.0.0',
        v8: '11.0.0',
    },
    platform: 'win32',
    arch: 'x64',
    env: {},
    argv: [],
    execArgv: [],
    pid: 1,
    ppid: 0,
    uptime: () => 0,
    memoryUsage: () => ({ rss: 0, heapTotal: 0, heapUsed: 0, external: 0 }),
    cpuUsage: () => ({ user: 0, system: 0 }),
    nextTick: (fn) => setTimeout(fn, 0),
    release: {
        name: 'node',
    },
    featureFlags: {},
    emitWarning: (warning) => console.warn(warning),
    chdir: () => {},
    cwd: () => '/',
    exit: (code) => {},
    getuid: () => 0,
    getgid: () => 0,
    setuid: () => {},
    setgid: () => {},
    kill: () => {},
    hrtime: () => [0, 0],
    resourceUsage: () => ({}),
    kill: (pid, sig) => {},
    abort: () => {},
    umask: () => 0o022,
    title: 'kossjs',
    argv0: 'kossjs',
    mainModule: undefined,
};
"#;

    let process_code = process_code_template
        .replace("'win32'", &format!("'{platform_str}'"))
        .replace("'x64'", &format!("'{arch_str}'"));

    let source = boa_parser::Source::from_bytes(process_code.as_bytes());
    match ctx.eval(source) {
        Ok(val) => {
            let _ = ctx.register_global_property(
                boa_engine::js_string!("process"),
                val,
                boa_engine::property::Attribute::READONLY
                    | boa_engine::property::Attribute::NON_ENUMERABLE,
            );
        }
        Err(e) => {
            eprintln!("Warning: Failed to register process: {:?}", e);
        }
    }

    // Register CommonJS module system
    let module_system_code = r#"
(function(globalThis) {
    'use strict';
    
    const Module = {
        _cache: {},
        _extensions: { '.js': function(module, filename) {} }
    };
    
    let _exportsCustomized = false;
    let _customExports = {};
    
    let currentModule = {
        id: '<root>',
        filename: '<root>',
        loaded: false
    };
    
    Object.defineProperty(currentModule, 'exports', {
        get: function() { return _customExports; },
        set: function(val) {
            _customExports = val;
            _exportsCustomized = true;
            Module._cache = {};
        },
        configurable: true,
        enumerable: true
    });
    
    function require(path) {
        const normalizedPath = path;
        
        if (Module._cache[normalizedPath]) {
            return Module._cache[normalizedPath].exports;
        }
        
        const module = Object.create(null);
        module.id = normalizedPath;
        module.filename = normalizedPath;
        module.loaded = false;
        module.exports = {};
        
        Module._cache[normalizedPath] = module;
        
        try {
            if (typeof __koss_load_module === 'function') {
                const result = __koss_load_module(normalizedPath);
                if (result !== null && result !== undefined) {
                    const parsed = JSON.parse(result);
                    if (parsed.type === 'module' && typeof parsed.code === 'string') {
                        const originalModule = globalThis.module;
                        const originalExports = globalThis.exports;
                        const originalBuffer = globalThis.Buffer;
                        delete globalThis.Buffer;
                        globalThis.module = module;
                        globalThis.exports = module.exports;
                        try {
                            eval(parsed.code);
                        } finally {
                            globalThis.module = originalModule;
                            globalThis.exports = originalExports;
                            globalThis.Buffer = originalBuffer;
                        }
                    } else if (parsed.type === 'object') {
                        module.exports = parsed.value;
                    }
                }
            }
            module.loaded = true;
        } catch (e) {
            delete Module._cache[normalizedPath];
            throw e;
        }
        
        return module.exports;
    }
    
    require.cache = Module._cache;
    require.resolve = function(path) {
        return path;
    };
    
    globalThis.require = require;
    globalThis.module = currentModule;
    globalThis.exports = currentModule.exports;
    globalThis.Module = Module;
})(globalThis);
"#;
    let source = boa_parser::Source::from_bytes(module_system_code.as_bytes());
    if let Err(e) = ctx.eval(source) {
        eprintln!("Warning: Failed to register module system: {:?}", e);
    }
}

// ===========================================================================
// C ABI — Instance lifecycle
// ===========================================================================

/// Internal helper: create a fully initialized KossInstance from parts.
fn create_instance_internal(
    context: boa_engine::Context,
    caps: u32,
    builtins: u32,
    stable: bool,
    _root_dir: Option<&str>,
) -> *mut KossInstance {
    let effective_caps = if stable {
        caps & !crate::sandbox::KOSS_CAP_ALL_FFI
    } else {
        if caps & crate::sandbox::KOSS_CAP_ALL_FFI != 0 {
            if !FFI_STABLE_WARNED.swap(true, Ordering::Relaxed) {
                eprintln!("[KossJS WARNING] Unstable mode: FFI features are enabled.");
                eprintln!("[KossJS WARNING] FFI may have security implications in production.");
            }
        }
        caps
    };

    let mut instance = Box::new(KossInstance::new(context, effective_caps, stable, builtins));
    register_console(&mut instance.context);
    register_koss_global(&mut instance.context, stable);
    buffer::register_buffer_globals(&mut instance.context);
    register_native_bindings(&mut instance);
    register_fs_functions(&mut instance);
    register_nodejs_globals(&mut instance.context);
    if !stable && crate::sandbox::has_cap(effective_caps, crate::sandbox::NATIVE_ADDON) {
        register_dlopen_binding(&mut instance);
    }
    // Register TextEncoder/TextDecoder as globals if not already present
    let te_code = r#"
    (function() {
        if (typeof TextEncoder === 'undefined') {
            function TextEncoder() {
                this.encode = function(str) {
                    var bytes = [];
                    for (var i = 0; i < str.length; i++) {
                        var c = str.charCodeAt(i);
                        if (c < 0x80) bytes.push(c);
                        else if (c < 0x800) { bytes.push(0xc0|(c>>6)); bytes.push(0x80|(c&0x3f)); }
                        else { bytes.push(0xe0|(c>>12)); bytes.push(0x80|((c>>6)&0x3f)); bytes.push(0x80|(c&0x3f)); }
                    }
                    return new Uint8Array(bytes);
                };
            }
            globalThis.TextEncoder = TextEncoder;
        }
        if (typeof TextDecoder === 'undefined') {
            function TextDecoder() {
                this.decode = function(bytes) {
                    if (!bytes || bytes.length === 0) return '';
                    var chars = [], i = 0;
                    while (i < bytes.length) {
                        var b = bytes[i++];
                        if (b < 0x80) chars.push(b);
                        else if (b < 0xe0) { var b2=bytes[i++]&0x3f; chars.push(((b&0x1f)<<6)|b2); }
                        else if (b < 0xf0) { var b2=bytes[i++]&0x3f; var b3=bytes[i++]&0x3f; chars.push(((b&0x0f)<<12)|(b2<<6)|b3); }
                    }
                    return String.fromCharCode.apply(null, chars);
                };
            }
            globalThis.TextDecoder = TextDecoder;
        }
    })();
    "#;
    if let Err(e) = instance.context.eval(boa_parser::Source::from_bytes(te_code.as_bytes())) {
        eprintln!("Warning: Failed to register TextEncoder/TextDecoder: {:?}", e);
    }
    if effective_caps & crate::sandbox::MODULE_LOAD != 0 {
        register_internal_module_loader(&mut instance);
    }
    let has_ffi = effective_caps & crate::sandbox::KOSS_CAP_ALL_FFI != 0;
    if has_ffi {
        register_senri_ffi_impl(&mut instance);
    } else if stable && (caps & crate::sandbox::KOSS_CAP_ALL_FFI != 0) {
        let ffi_stub_code = r#"globalThis._senri_ffi = { func: function() { throw new Error('FFI is disabled in stable mode'); } };"#;
        if let Err(e) = instance.context.eval(boa_parser::Source::from_bytes(ffi_stub_code.as_bytes())) {
            eprintln!("Warning: Failed to register FFI stub: {:?}", e);
        }
    }
    register_runtime_capabilities(&mut instance);
    let ptr = Box::into_raw(instance);
    register_api_instance(ptr as usize);
    ptr
}

/// Create a new isolated JS instance with specific capabilities, builtin flags, and stable mode.
/// Builtin flags control which koss:* modules are visible to user code.
#[unsafe(no_mangle)]
pub extern "C" fn koss_create_with_builtins(
    caps: u32,
    builtins: u32,
    stable: bool,
) -> *mut KossInstance {
    output_license_once();
    let root = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string());
    let loader = Rc::new(KossModuleLoader::new_with_builtins(&root, builtins));
    let context = match boa_engine::context::ContextBuilder::default()
        .module_loader(loader)
        .build()
    {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Warning: Failed to create Boa context: {e}");
            return std::ptr::null_mut();
        }
    };
    create_instance_internal(context, caps, builtins, stable, None)
}

/// Create a new isolated JS instance with specific capabilities and stable mode.
/// Uses KOSS_BUILTIN_ALL for backward compatibility.
/// When `stable` is true, FFI capabilities are stripped from `caps`.
#[unsafe(no_mangle)]
pub extern "C" fn koss_create_with_caps(caps: u32, stable: bool) -> *mut KossInstance {
    koss_create_with_builtins(caps, crate::builtins::KOSS_BUILTIN_ALL, stable)
}

/// Create a new isolated JS instance with module resolution, specific capabilities,
/// builtin flags, and stable mode.
///
/// # Safety
/// - `root_dir` must be a valid null-terminated UTF-8 string
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_create_with_modules_and_builtins(
    root_dir: *const c_char,
    caps: u32,
    builtins: u32,
    stable: bool,
) -> *mut KossInstance {
    output_license_once();
    unsafe {
        if root_dir.is_null() {
            return koss_create_with_builtins(caps, builtins, stable);
        }
        let root_str = match CStr::from_ptr(root_dir).to_str() {
            Ok(s) => s,
            Err(_) => return koss_create_with_builtins(caps, builtins, stable),
        };
        let loader = Rc::new(KossModuleLoader::new_with_builtins(root_str, builtins));
        let context = match boa_engine::context::ContextBuilder::default()
            .module_loader(loader)
            .build()
        {
            Ok(ctx) => ctx,
            Err(e) => {
                eprintln!("Warning: Failed to create Boa context: {e}");
                return std::ptr::null_mut();
            }
        };
        create_instance_internal(context, caps, builtins, stable, Some(root_str))
    }
}

/// Create a new isolated JS instance with module resolution, specific capabilities,
/// and stable mode. Uses KOSS_BUILTIN_ALL for backward compatibility.
///
/// # Safety
/// - `root_dir` must be a valid null-terminated UTF-8 string
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_create_with_modules_and_caps(
    root_dir: *const c_char,
    caps: u32,
    stable: bool,
) -> *mut KossInstance {
    unsafe {
        koss_create_with_modules_and_builtins(root_dir, caps, crate::builtins::KOSS_BUILTIN_ALL, stable)
    }
}

/// Destroy a JS instance and free all associated memory.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_destroy(ptr: *mut KossInstance) {
    output_license_once();
    unsafe {
        if !ptr.is_null() {
            let guard = match enter_api(ptr as usize) {
                Ok(guard) => guard,
                Err(_) => return,
            };
            unregister_api_instance(ptr as usize);
            std::mem::forget(guard);
            drop(Box::from_raw(ptr));
        }
    }
}

// ===========================================================================
// C ABI — Code execution
// ===========================================================================

/// Evaluate a JavaScript string. Returns the result as a string.
///
/// # Safety
/// - `ptr` must be a valid pointer from `koss_create`
/// - `code` must be a valid null-terminated UTF-8 string
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_eval(ptr: *mut KossInstance, code: *const c_char) -> KossResult {
    output_license_once();
    unsafe {
        if ptr.is_null() || code.is_null() {
            return KossResult::err(2, "null pointer");
        }

        let _api_guard = api_guard_result!(ptr);
        let instance = &mut *ptr;
        let code_str = match CStr::from_ptr(code).to_str() {
            Ok(s) => s,
            Err(e) => return KossResult::err(2, &format!("invalid UTF-8: {e}")),
        };

        let source = Source::from_bytes(code_str.as_bytes());
        match instance.context.eval(source) {
            Ok(val) => {
                let s = if val.is_object() {
                    safe_js_value_to_json(&val, &mut instance.context)
                        .unwrap_or_else(|| js_value_to_string(&val, &mut instance.context))
                } else {
                    js_value_to_string(&val, &mut instance.context)
                };
                KossResult::ok(&s)
            }
            Err(err) => {
                let s = js_error_to_string(&err, &mut instance.context);
                KossResult::err(1, &s)
            }
        }
    }
}

/// Safely convert a JsValue (object) to a JSON string, handling cycles and functions.

/// Escape a string for safe inclusion in a JS single-quoted string literal.
pub(crate) fn escape_js_string(s: &str) -> String {
    let mut escaped = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '\'' => escaped.push_str("\\'"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\u{2028}' => escaped.push_str("\\u2028"),
            '\u{2029}' => escaped.push_str("\\u2029"),
            other => escaped.push(other),
        }
    }
    escaped
}
fn safe_js_value_to_json(val: &JsValue, ctx: &mut Context) -> Option<String> {
    let temp_key = "__koss_safe_json_val__";
    let _ = ctx.register_global_property(
        boa_engine::js_string!(temp_key),
        val.clone(),
        boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
    );
    let json_code = r#"(function() {
        var seen = new WeakSet();
        return JSON.stringify(globalThis.__koss_safe_json_val__, function(k, v) {
            if (typeof v === 'object' && v !== null) {
                if (seen.has(v)) return undefined;
                seen.add(v);
            }
            if (typeof v === 'function') return undefined;
            return v;
        });
    })()"#;
    let result = ctx.eval(Source::from_bytes(json_code.as_bytes()));
    let _ = ctx.eval(Source::from_bytes(
        format!("delete globalThis.{}", temp_key).as_bytes(),
    ));
    match result {
        Ok(js_val) => match js_val.to_string(ctx) {
            Ok(s) => {
                let std_str = s.to_std_string_escaped();
                if std_str == "null" || std_str.is_empty() {
                    None
                } else {
                    Some(std_str)
                }
            }
            Err(_) => None,
        },
        Err(_) => None,
    }
}

/// Execute a JavaScript file. Returns the result of the last expression.
/// The file path is canonicalized for safety.
///
/// # Safety
/// - `ptr` must be a valid pointer from `koss_create`
/// - `path` must be a valid null-terminated UTF-8 file path
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_run_file(ptr: *mut KossInstance, path: *const c_char) -> KossResult {
    output_license_once();
    unsafe {
        if ptr.is_null() || path.is_null() {
            return KossResult::err(2, "null pointer");
        }

        let _api_guard = api_guard_result!(ptr);
        let instance = &mut *ptr;
        let path_str = match CStr::from_ptr(path).to_str() {
            Ok(s) => s,
            Err(e) => return KossResult::err(2, &format!("invalid UTF-8: {e}")),
        };

        let file_path = std::path::Path::new(path_str);
        // Canonicalize to resolve symlinks and normalize path (CWE-22)
        let canonical = match file_path.canonicalize() {
            Ok(p) => p,
            Err(e) => return KossResult::err(2, &format!("cannot resolve path: {e}")),
        };

        if !canonical.is_file() {
            return KossResult::err(2, "path is not a file");
        }

        let source = match Source::from_filepath(&canonical) {
            Ok(s) => s,
            Err(e) => return KossResult::err(2, &format!("cannot read file: {e}")),
        };

        match instance.context.eval(source) {
            Ok(val) => {
                let s = js_value_to_string(&val, &mut instance.context);
                KossResult::ok(&s)
            }
            Err(err) => {
                let s = js_error_to_string(&err, &mut instance.context);
                KossResult::err(1, &s)
            }
        }
    }
}

/// Execute a JavaScript file as an ES Module (supports import/export syntax).
/// The instance should be created with `koss_create_with_modules` for full
/// module resolution support.
///
/// # Safety
/// - `ptr` must be a valid pointer from `koss_create` or `koss_create_with_modules`
/// - `path` must be a valid null-terminated UTF-8 file path
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_run_module(
    ptr: *mut KossInstance,
    path: *const c_char,
) -> KossResult {
    output_license_once();
    if ptr.is_null() || path.is_null() {
        return KossResult::err(2, "null pointer");
    }

    let _api_guard = api_guard_result!(ptr);
    let instance = unsafe { &mut *ptr };
    let path_str = match unsafe { CStr::from_ptr(path) }.to_str() {
        Ok(s) => s,
        Err(e) => return KossResult::err(2, &format!("invalid UTF-8: {e}")),
    };

    let file_path = std::path::Path::new(path_str);
    let source = match Source::from_filepath(file_path) {
        Ok(s) => s,
        Err(e) => return KossResult::err(2, &format!("cannot read file: {e}")),
    };

    // Parse as ES Module
    let module = match Module::parse(source, None, &mut instance.context) {
        Ok(m) => m,
        Err(err) => {
            let s = js_error_to_string(&err, &mut instance.context);
            return KossResult::err(1, &format!("module parse error: {s}"));
        }
    };

    // Load, link, and evaluate the module
    let promise = module.load_link_evaluate(&mut instance.context);

    // Drive the job queue to completion so async module loading finishes
    let _ = instance.context.run_jobs();

    // Check the promise result
    match promise.state() {
        boa_engine::builtins::promise::PromiseState::Fulfilled(val) => {
            let s = js_value_to_string(&val, &mut instance.context);
            KossResult::ok(&s)
        }
        boa_engine::builtins::promise::PromiseState::Rejected(err) => {
            let s = js_value_to_string(&err, &mut instance.context);
            KossResult::err(1, &format!("module error: {s}"))
        }
        boa_engine::builtins::promise::PromiseState::Pending => {
            KossResult::err(1, "module evaluation timed out (still pending)")
        }
    }
}

/// Execute a JavaScript string as an ES Module (supports import/export syntax).
/// The instance should be created with `koss_create_with_modules` for full
/// module resolution support.
///
/// # Safety
/// - `ptr` must be a valid pointer from `koss_create` or `koss_create_with_modules`
/// - `code` must be a valid null-terminated UTF-8 string
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_run_module_string(
    ptr: *mut KossInstance,
    code: *const c_char,
) -> KossResult {
    output_license_once();
    if ptr.is_null() || code.is_null() {
        return KossResult::err(2, "null pointer");
    }

    let _api_guard = api_guard_result!(ptr);
    let instance = unsafe { &mut *ptr };
    let code_str = match unsafe { CStr::from_ptr(code) }.to_str() {
        Ok(s) => s,
        Err(e) => return KossResult::err(2, &format!("invalid UTF-8: {e}")),
    };

    let source = Source::from_bytes(code_str.as_bytes());

    // Parse as ES Module
    let module = match Module::parse(source, None, &mut instance.context) {
        Ok(m) => m,
        Err(err) => {
            let s = js_error_to_string(&err, &mut instance.context);
            return KossResult::err(1, &format!("module parse error: {s}"));
        }
    };

    // Load, link, and evaluate the module
    let promise = module.load_link_evaluate(&mut instance.context);
    let _ = instance.context.run_jobs();

    match promise.state() {
        boa_engine::builtins::promise::PromiseState::Fulfilled(val) => {
            let s = js_value_to_string(&val, &mut instance.context);
            KossResult::ok(&s)
        }
        boa_engine::builtins::promise::PromiseState::Rejected(err) => {
            let s = js_value_to_string(&err, &mut instance.context);
            KossResult::err(1, &format!("module error: {s}"))
        }
        boa_engine::builtins::promise::PromiseState::Pending => {
            KossResult::err(1, "module evaluation timed out (still pending)")
        }
    }
}

/// Execute a JavaScript text string. Returns the result of the last expression.
///
/// # Safety
/// - `ptr` must be a valid pointer from `koss_create`
/// - `code` must be a valid null-terminated UTF-8 string
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_run_string(
    ptr: *mut KossInstance,
    code: *const c_char,
) -> KossResult {
    output_license_once();
    unsafe {
        if ptr.is_null() || code.is_null() {
            return KossResult::err(2, "null pointer");
        }

        let _api_guard = api_guard_result!(ptr);
        let instance = &mut *ptr;
        let code_str = match CStr::from_ptr(code).to_str() {
            Ok(s) => s,
            Err(e) => return KossResult::err(2, &format!("invalid UTF-8: {e}")),
        };

        let source = Source::from_bytes(code_str.as_bytes());

        match instance.context.eval(source) {
            Ok(val) => {
                let s = js_value_to_string(&val, &mut instance.context);
                KossResult::ok(&s)
            }
            Err(err) => {
                let s = js_error_to_string(&err, &mut instance.context);
                KossResult::err(1, &s)
            }
        }
    }
}

/// Evaluate JavaScript code and drive the async event loop to completion.
/// The event loop processes async I/O (fetch, timers) and drains microtasks
/// until either all pending operations complete or the timeout is reached.
///
/// # Safety
/// - `ptr` must be a valid pointer from `koss_create`
/// - `code` must be a valid null-terminated UTF-8 string
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_run_async(
    ptr: *mut KossInstance,
    code: *const c_char,
    timeout_ms: u64,
) -> KossResult {
    output_license_once();
    unsafe {
        if ptr.is_null() || code.is_null() {
            return KossResult::err(2, "null pointer");
        }

        let _api_guard = api_guard_result!(ptr);
        let instance = &mut *ptr;
        let code_str = match CStr::from_ptr(code).to_str() {
            Ok(s) => s,
            Err(e) => return KossResult::err(2, &format!("invalid UTF-8: {e}")),
        };

        let source = Source::from_bytes(code_str.as_bytes());
        let val = match instance.context.eval(source) {
            Ok(v) => v,
            Err(err) => {
                let s = js_error_to_string(&err, &mut instance.context);
                return KossResult::err(1, &s);
            }
        };

        if let Some(ref mut el) = instance.event_loop {
            let deadline = Instant::now() + Duration::from_millis(timeout_ms);
            let max_iterations = 100_000u64;
            let mut iteration = 0u64;
            let mut consecutive_idle: u32 = 0;
            loop {
                el.process_io_results(&mut instance.context);
                let _ = instance.context.run_jobs();

                let idle = el.pending.is_empty();
                if idle {
                    let _ = instance.context.run_jobs();
                    break;
                }

                iteration += 1;
                if iteration >= max_iterations {
                    return KossResult::err(1, "async execution exceeded max iterations");
                }

                if Instant::now() >= deadline {
                    return KossResult::err(1, "async execution timed out");
                }

                // Adaptive sleep to avoid busy-wait DoS (CWE-400)
                consecutive_idle = consecutive_idle.saturating_add(1);
                let backoff_ms = 1u64.saturating_mul((consecutive_idle as u64).min(50));
                std::thread::sleep(Duration::from_millis(backoff_ms));
            }
        }

        // Try to extract the resolved value if the result is a Promise
        let is_promise_obj = val.is_object()
            && JsPromise::from_object(
                match val.as_object() {
                    Some(o) => o.clone(),
                    None => {
                        let s = js_value_to_string(&val, &mut instance.context);
                        return KossResult::ok(&s);
                    }
                },
            )
            .is_ok();

        if is_promise_obj {
            let obj = val.as_object().unwrap().clone();
            if let Ok(promise) = JsPromise::from_object(obj) {
                match promise.state() {
                    boa_engine::builtins::promise::PromiseState::Fulfilled(resolved) => {
                        let s = js_value_to_string(&resolved, &mut instance.context);
                        KossResult::ok(&s)
                    }
                    boa_engine::builtins::promise::PromiseState::Rejected(err) => {
                        let s = js_value_to_string(&err, &mut instance.context);
                        KossResult::err(1, &s)
                    }
                    boa_engine::builtins::promise::PromiseState::Pending => {
                        let s = js_value_to_string(&val, &mut instance.context);
                        KossResult::ok(&s)
                    }
                }
            } else {
                let s = js_value_to_string(&val, &mut instance.context);
                KossResult::ok(&s)
            }
        } else {
            let s = js_value_to_string(&val, &mut instance.context);
            KossResult::ok(&s)
        }
    }
}

/// Run a single tick of the event loop: process completed async I/O
/// and drain the microtask queue. Returns "1" if there are still pending
/// async operations, "0" if idle.
///
/// # Safety
/// - `ptr` must be a valid pointer from `koss_create`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_tick(ptr: *mut KossInstance) -> KossResult {
    output_license_once();
    unsafe {
        if ptr.is_null() {
            return KossResult::err(2, "null pointer");
        }

        let _api_guard = api_guard_result!(ptr);
        let instance = &mut *ptr;
        let has_pending = instance.tick();
        let _ = instance.context.run_jobs();

        KossResult::ok(if has_pending { "1" } else { "0" })
    }
}

// ===========================================================================
// C ABI — Memory management
// ===========================================================================

/// Free a C string that was allocated by the Rust side (e.g., from KossResult).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_free_string(ptr: *mut c_char) {
    output_license_once();
    if !ptr.is_null() {
        unsafe {
            drop(CString::from_raw(ptr));
        }
    }
}

/// Free a KossResult struct and its associated value string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_free_result(result: KossResult) {
    output_license_once();
    if !result.value.is_null() {
        unsafe {
            drop(CString::from_raw(result.value));
        }
    }
}

// ===========================================================================
// C ABI — Global variable injection (host → JS)
// ===========================================================================

/// Set a global string variable in the JS context.
/// Useful for injecting config, paths, etc. from the host.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_set_global_string(
    ptr: *mut KossInstance,
    name: *const c_char,
    value: *const c_char,
) -> KossResult {
    output_license_once();
    unsafe {
        if ptr.is_null() || name.is_null() || value.is_null() {
            return KossResult::err(2, "null pointer");
        }

        let _api_guard = api_guard_result!(ptr);
        let instance = &mut *ptr;
        let name_str = match CStr::from_ptr(name).to_str() {
            Ok(s) => s,
            Err(e) => return KossResult::err(2, &format!("invalid UTF-8: {e}")),
        };
        let value_str = match CStr::from_ptr(value).to_str() {
            Ok(s) => s,
            Err(e) => return KossResult::err(2, &format!("invalid UTF-8: {e}")),
        };

        let js_key = boa_engine::js_string!(name_str);
        let js_val = boa_engine::JsValue::from(boa_engine::js_string!(value_str));

        let _ = instance.context.register_global_property(
            js_key,
            js_val,
            boa_engine::property::Attribute::WRITABLE
                | boa_engine::property::Attribute::CONFIGURABLE,
        );

        KossResult::ok("ok")
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_register_fetch(ptr: *mut KossInstance) -> KossResult {
    output_license_once();
    unsafe {
        if ptr.is_null() {
            return KossResult::err(2, "null pointer");
        }

        let _api_guard = api_guard_result!(ptr);
        let instance = &mut *ptr;

        let native = boa_engine::NativeFunction::from_copy_closure(move |_this, args, ctx| {
            if args.len() < 2 {
                return Ok(JsValue::undefined());
            }

            let _url = js_value_to_string(&args[0], ctx);
            let request_json = js_value_to_string(&args[1], ctx);

            let result_ptr = fetch_json(&request_json);

            if result_ptr.code == 0 && !result_ptr.value.is_null() {
                let response_str = match CStr::from_ptr(result_ptr.value).to_str() {
                    Ok(s) => s.to_string(),
                    Err(_) => String::new(),
                };
                let _ = CString::from_raw(result_ptr.value);
                let js_str = boa_engine::JsString::from(response_str.as_str());
                Ok(JsValue::from(js_str))
            } else {
                if !result_ptr.value.is_null() {
                    let _ = CString::from_raw(result_ptr.value);
                }
                Ok(JsValue::undefined())
            }
        });

        let js_func = native.to_js_function(instance.context.realm());

        instance
            .context
            .register_global_property(
                boa_engine::js_string!("__koss_fetch"),
                js_func,
                boa_engine::property::Attribute::WRITABLE
                    | boa_engine::property::Attribute::CONFIGURABLE,
            )
            .ok();

        KossResult::ok("ok")
    }
}

// ===========================================================================
// C ABI — Version info
// ===========================================================================

/// Returns the KossJS version string.
#[unsafe(no_mangle)]
pub extern "C" fn koss_version() -> *const c_char {
    output_license_once();
    get_version().as_ptr() as *const c_char
}

/// Query the capability mask for a KossJS instance.
/// Returns the bitmask set at creation time (read-only).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_get_capabilities(ptr: *mut KossInstance) -> u32 {
    output_license_once();
    unsafe {
        if ptr.is_null() {
            return 0;
        }
        let _api_guard = match enter_api(ptr as usize) {
            Ok(guard) => guard,
            Err(_) => return 0,
        };
        let instance = &*ptr;
        instance.capabilities
    }
}

/// Returns true if the instance was created in stable mode (FFI disabled).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_is_stable(ptr: *mut KossInstance) -> bool {
    output_license_once();
    unsafe {
        if ptr.is_null() {
            return false;
        }
        let _api_guard = match enter_api(ptr as usize) {
            Ok(guard) => guard,
            Err(_) => return false,
        };
        (*ptr).stable
    }
}

/// Query the builtin module flags for a KossJS instance.
/// Returns the bitmask of enabled builtin flags.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_get_builtins(ptr: *mut KossInstance) -> u32 {
    output_license_once();
    unsafe {
        if ptr.is_null() {
            return 0;
        }
        let _api_guard = match enter_api(ptr as usize) {
            Ok(guard) => guard,
            Err(_) => return 0,
        };
        (*ptr).builtins
    }
}

/// Check if a specific builtin flag is enabled.
/// `flag` is a single KOSS_BUILTIN_* value.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_is_builtin_enabled(ptr: *mut KossInstance, flag: u32) -> bool {
    output_license_once();
    unsafe {
        if ptr.is_null() {
            return false;
        }
        let _api_guard = match enter_api(ptr as usize) {
            Ok(guard) => guard,
            Err(_) => return false,
        };
        (*ptr).builtins & flag != 0
    }
}

/// Set the audit mask for a KossJS instance.
/// The audit mask controls which capability operations trigger audit callbacks.
/// Only bits corresponding to already-granted capabilities are applied;
/// bits for ungranted capabilities are silently ignored.
///
/// # Safety
/// - `ptr` must be a valid pointer from `koss_create`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_set_audit_mask(ptr: *mut KossInstance, mask: u32) -> KossResult {
    output_license_once();
    unsafe {
        if ptr.is_null() {
            return KossResult::err(2, "null pointer");
        }
        let _api_guard = api_guard_result!(ptr);
        let instance = &mut *ptr;
        instance.sandbox.audit_mask = mask & instance.capabilities;
        KossResult::ok("ok")
    }
}

/// Get the current audit mask for a KossJS instance.
/// Returns 0 if the pointer is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_get_audit_mask(ptr: *mut KossInstance) -> u32 {
    output_license_once();
    unsafe {
        if ptr.is_null() {
            return 0;
        }
        let _api_guard = match enter_api(ptr as usize) {
            Ok(guard) => guard,
            Err(_) => return 0,
        };
        (*ptr).sandbox.audit_mask
    }
}

/// Register a synchronous audit callback for a KossJS instance.
///
/// The callback is invoked when a capability operation is about to be performed
/// and the corresponding bit in the audit mask is set. The callback receives
/// the target (e.g. "fs.readFileSync"), an array of string arguments, the
/// current working directory, and the userdata pointer. Return true to allow
/// the operation, false to block it (which throws a KossSecurityError).
///
/// Pass a NULL callback pointer (or callback with address 0) to clear the
/// audit callback.
///
/// # Safety
/// - `ptr` must be a valid pointer from `koss_create`
/// - `callback`, if non-null, must be a valid function pointer for the
///   lifetime of the KossInstance
/// - The caller must ensure the userdata pointer remains valid for the
///   lifetime of the callback registration
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_check_sandbox(
    ptr: *mut KossInstance,
    callback: crate::sandbox::AuditCallback,
    userdata: *mut c_void,
) -> KossResult {
    output_license_once();
    unsafe {
        if ptr.is_null() {
            return KossResult::err(2, "null pointer");
        }
        let _api_guard = api_guard_result!(ptr);
        let instance = &mut *ptr;
        if callback as usize == 0 {
            instance.sandbox.sync_audit = None;
            instance.sandbox.sync_userdata = std::ptr::null_mut();
            return KossResult::ok("audit callback cleared");
        }
        instance.sandbox.sync_audit = Some(callback);
        instance.sandbox.sync_userdata = userdata;
        KossResult::ok("ok")
    }
}

// ===========================================================================
// C ABI — Internal Bindings
// ===========================================================================

/// Handle internalBinding calls from JS - returns JSON with binding results
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_get_binding(
    ptr: *mut KossInstance,
    binding_name: *const c_char,
) -> KossResult {
    output_license_once();
    unsafe {
        if ptr.is_null() || binding_name.is_null() {
            return KossResult::err(2, "null pointer");
        }

        let _api_guard = api_guard_result!(ptr);
        let instance = &mut *ptr;
        let name_str = match CStr::from_ptr(binding_name).to_str() {
            Ok(s) => s,
            Err(e) => return KossResult::err(2, &format!("invalid UTF-8: {e}")),
        };

        let debug = instance.sandbox.audit_debug;
        let decision = is_capability_enabled(instance.capabilities, instance.sandbox.audit_mask, name_str);
        match decision {
            AuditDecision::DenyCapability => {
                let msg = capability_error_message(name_str, debug);
                return KossResult::err(3, &msg);
            }
            AuditDecision::Allow => {}
            AuditDecision::NeedAudit => {
                if let Some(audit_fn) = instance.sandbox.sync_audit {
                    let target = match CString::new(name_str) {
                        Ok(c) => c,
                        Err(_) => return KossResult::ok("{}"),
                    };
                    let allowed = audit_fn(
                        target.as_ptr(),
                        std::ptr::null(),
                        0,
                        std::ptr::null(),
                        instance.sandbox.sync_userdata,
                    );
                    if !allowed {
                        let msg = security_error_message(name_str, debug);
                        return KossResult::err(4, &msg);
                    }
                }
            }
        }

        let result = handle_binding(name_str);
        match result {
            Ok(json) => KossResult::ok(&json),
            Err(e) => KossResult::err(1, &e),
        }
    }
}

/// Check if a binding is enabled under the given capabilities mask.
/// Returns an AuditDecision indicating whether to allow, deny, or audit.
fn is_capability_enabled(caps: u32, audit_mask: u32, name: &str) -> AuditDecision {
    let required = match name {
        // 文件系统模块
        "fs" | "fs/promises" => KOSS_CAP_ALL_FS,
        // 网络模块
        "net" | "url" | "http_parser" | "dns" | "dgram" => KOSS_CAP_ALL_NET,
        // 加密模块
        "crypto" => KOSS_CAP_ALL_CRYPTO,
        _ => return AuditDecision::Allow, // always-available modules
    };
    
    // 第一道闸门：能力位检查
    if caps & required == 0 {
        return AuditDecision::DenyCapability;
    }
    
    // 第二道闸门：审核掩码检查
    if audit_mask & required != 0 {
        return AuditDecision::NeedAudit;
    }
    
    AuditDecision::Allow
}

fn handle_binding(name: &str) -> Result<String, String> {
    use crate::bindings;

    match name {
        "fs" => Ok(serde_json::json!({
            "access": true,
            "existsSync": true,
            "readFileUtf8": true,
            "open": true,
            "close": true,
            "read": true,
            "writeBuffer": true,
            "writeString": true,
            "rename": true,
            "renameSync": true,
            "unlink": true,
            "unlinkSync": true,
            "mkdir": true,
            "mkdirSync": true,
            "rmdir": true,
            "rmdirSync": true,
            "readdir": true,
            "stat": true,
            "lstat": true,
            "fstat": true,
            "readlink": true,
            "symlink": true,
            "link": true,
            "truncate": true,
            "ftruncate": true,
            "chmod": true,
            "fchmod": true,
            "chown": true,
            "fchown": true,
            "copyFile": true,
            "rmSync": true,
            "statfs": true,
        })
        .to_string()),
        "os" => Ok(serde_json::json!({
            "getCPUs": true,
            "getFreeMem": true,
            "getTotalMem": true,
            "getHomeDirectory": true,
            "getHostname": true,
            "getInterfaceAddresses": true,
            "getLoadAvg": true,
            "getUptime": true,
            "getOSInformation": true,
            "isBigEndian": false,
            "getTempDir": true,
            "getUserInfo": true,
            "getAvailableParallelism": true,
            "getPID": true,
            "getGID": true,
            "getUID": true,
        })
        .to_string()),
        "timers" => Ok(serde_json::json!({
            "scheduleTimer": true,
            "toggleTimerRef": true,
            "getLibuvNow": true,
            "getTimerStart": true,
            "clearTimer": true,
            "activeTimerCount": true,
            "immediateInfo": [0, 0, 0],
            "timeoutInfo": [0],
        })
        .to_string()),
        "crypto" => Ok(serde_json::json!({
            "getRandomValues": true,
            "randomInt": true,
            "randomUUID": true,
            "createHash": true,
            "createHmac": true,
            "pbkdf2": true,
            "generatePrime": true,
            "getConstants": true,
        })
        .to_string()),
        "net" => Ok(serde_json::json!({
            "isIP": true,
            "isIPv4": true,
            "isIPv6": true,
            "parseIP": true,
            "getProtocolFamily": true,
            "getSocketType": true,
            "newTCPSocket": true,
            "newUDPSocket": true,
            "tcpBind": true,
            "tcpConnect": true,
            "udpBind": true,
            "getLocalAddress": true,
            "getLocalPort": true,
            "dnsLookup": true,
            "getSocketError": true,
            "setNoDelay": true,
            "setKeepAlive": true,
            "setReuseAddr": true,
        })
        .to_string()),
        "constants" => Ok(serde_json::json!({
            "fs": bindings::constants::fs_flags(),
            "os": bindings::constants::os_constants(),
            "signals": bindings::constants::signals(),
            "crypto": bindings::crypto::get_crypto_constants(),
        })
        .to_string()),
        "buffer" => Ok(serde_json::json!({
            "byteLengthUtf8": true,
            "compare": true,
            "copy": true,
            "fill": true,
            "isAscii": true,
            "isUtf8": true,
            "asciiSlice": true,
            "utf8Slice": true,
            "latin1Slice": true,
            "hexSlice": true,
            "base64Slice": true,
            "indexOfBuffer": true,
            "indexOfNumber": true,
            "indexOfString": true,
            "swap16": true,
            "swap32": true,
            "swap64": true,
        })
        .to_string()),
        "http_parser" => Ok(serde_json::json!({
            "parseRequest": true,
            "parseResponse": true,
            "methodStringToInt": true,
            "methodIntToString": true,
            "statusText": true,
        })
        .to_string()),
        "url" => Ok(serde_json::json!({
            "parseURL": true,
            "formatURL": true,
            "parseQueryString": true,
            "encodeURIComponent": true,
        })
        .to_string()),
        "util" => Ok(serde_json::json!({
            "getSystemErrorName": true,
            "getSystemErrorCode": true,
            "inspect": true,
            "constants": {
                "ALL_PROPERTIES": 0,
                "ONLY_ENUMERABLE": 1,
                "kPending": 0,
                "kRejected": 1
            },
            "getOwnNonIndexProperties": null,
            "getPromiseDetails": null,
            "getProxyDetails": null,
            "previewEntries": null,
            "getConstructorName": null,
            "getExternalValue": null,
            "arrayBufferViewType": null,
            "getCrypto": null,
        })
        .to_string()),
        "config" => Ok(serde_json::json!({
            "hasInspector": false,
            "hasTracing": true,
            "hasIntl": false,
            "hasOpenSSL": true,
            "noBrowserGlobals": false,
            "hasNodeOptions": false,
            "hasSmallICU": false,
        })
        .to_string()),
        "errors" => Ok(serde_json::json!({
            "triggerUncaughtException": null,
            "exitCodes": {
                "kNoFailure": 0,
                "kGenericUserError": 1,
                "kInvalidCommandLineArgument": 9,
            },
            "setGetSourceMapErrorSource": null,
        })
        .to_string()),
        "performance" => Ok(serde_json::json!({
            "constants": {
                "NODE_PERFORMANCE_GC_MAJOR": 1,
                "NODE_PERFORMANCE_GC_MINOR": 2,
                "NODE_PERFORMANCE_GC_INCREMENTAL": 3,
                "NODE_PERFORMANCE_GC_WEAKCB": 4,
                "NODE_PERFORMANCE_GC_FLAGS_NO": 0,
                "NODE_PERFORMANCE_GC_FLAGS_CONSTRUCT_RETAINED": 1,
                "NODE_PERFORMANCE_GC_FLAGS_FORCED": 2,
                "NODE_PERFORMANCE_GC_FLAGS_SYNCHRONOUS_PHANTOM_PROCESSING": 4,
                "NODE_PERFORMANCE_GC_FLAGS_ALL_AVAILABLE_GARBAGE": 8,
                "NODE_PERFORMANCE_GC_FLAGS_ALL_EXTERNAL_MEMORY": 16,
                "NODE_PERFORMANCE_GC_FLAGS_SCHEDULE_IDLE": 32,
            },
        })
        .to_string()),
        "diagnostics_channel" => Ok(serde_json::json!({
            "subscribers": {},
            "hasSubscribers": true,
            "channel": true,
            "subscribe": true,
            "unsubscribe": true,
        })
        .to_string()),
        "trace_events" => Ok(serde_json::json!({
            "createTraceEvent": true,
            "getTraceCategories": true,
            "enableTrace": true,
            "disableTrace": true,
        })
        .to_string()),
        "fetch" => Ok(serde_json::json!({
            "fetch": true,
        })
        .to_string()),
        // "util" => Ok(serde_json::json!({
        //     "constants": {
        //         "ALL_PROPERTIES": 0,
        //         "ONLY_ENUMERABLE": 1,
        //         "kPending": 0,
        //         "kRejected": 1
        //     },
        //     "getOwnNonIndexProperties": null,
        //     "getPromiseDetails": null,
        //     "getProxyDetails": null,
        //     "previewEntries": null,
        //     "getConstructorName": null,
        //     "getExternalValue": null,
        //     "arrayBufferViewType": null,
        //     "getCrypto": null,
        // })
        // .to_string()),
        _ => Ok("{}".to_string()),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_fetch(ptr: *mut KossInstance, url_json: *const c_char) -> KossResult {
    output_license_once();
    unsafe {
        if ptr.is_null() || url_json.is_null() {
            return KossResult::err(2, "null pointer");
        }

        let _api_guard = api_guard_result!(ptr);
        let json_str = match CStr::from_ptr(url_json).to_str() {
            Ok(s) => s,
            Err(e) => return KossResult::err(2, &format!("invalid UTF-8: {e}")),
        };

        fetch_json(json_str)
    }
}

fn fetch_json(json_str: &str) -> KossResult {
    #[derive(serde::Deserialize)]
    #[allow(dead_code)]
    struct FetchInput {
        url: String,
        #[serde(flatten)]
        request: bindings::fetch::FetchRequest,
    }

    let input: FetchInput = match serde_json::from_str(json_str) {
        Ok(i) => i,
        Err(e) => return KossResult::err(1, &format!("parse error: {}", e)),
    };

    match bindings::fetch::fetch_with_url(&input.url, json_str) {
        Ok(response) => {
            let json = serde_json::to_string(&response).unwrap_or_default();
            KossResult::ok(&json)
        }
        Err(e) => KossResult::err(1, &format!("fetch error: {}", e)),
    }
}

// ===========================================================================
// Error message generation helpers
// ===========================================================================

pub fn capability_error_message(target: &str, debug: bool) -> String {
    if debug {
        format!("KossCapabilityError: capability denied for {target}")
    } else {
        "KossCapabilityError: Access denied".to_string()
    }
}

pub fn security_error_message(target: &str, debug: bool) -> String {
    if debug {
        format!("KossSecurityError: sandbox audit denied for {target}")
    } else {
        "KossSecurityError: Access denied".to_string()
    }
}

pub fn timeout_error_message(target: &str, debug: bool) -> String {
    if debug {
        format!("KossTimeoutError: sandbox audit timed out for {target}")
    } else {
        "KossTimeoutError: Access denied".to_string()
    }
}

pub fn cancel_error_message(target: &str, debug: bool) -> String {
    if debug {
        format!("KossCancelError: sandbox audit cancelled for {target}")
    } else {
        "KossCancelError: Access denied".to_string()
    }
}

// ===========================================================================
// C ABI — Audit debug mode
// ===========================================================================

/// Enable or disable audit debug mode for a KossJS instance.
/// When debug mode is enabled:
/// - Sync/async callback exceptions are output to stderr
/// - Audit denial reasons include additional error information
/// - Async audit timeouts or hangs log warnings
/// - Rejection reentry logs current depth and configured max depth
/// Production environments should disable debug mode to avoid information leakage.
///
/// # Safety
/// - `ptr` must be a valid pointer from `koss_create` (or NULL, which is a no-op)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_enable_audit_debug(ptr: *mut KossInstance, enable: bool) {
    output_license_once();
    unsafe {
        if ptr.is_null() {
            return;
        }
        let _api_guard = match enter_api(ptr as usize) {
            Ok(guard) => guard,
            Err(_) => return,
        };
        (*ptr).sandbox.audit_debug = enable;
    }
}

// ===========================================================================
// Type aliases for native callbacks
// ===========================================================================

/// Native callback type: receives (argc, argv) and returns a C string or null.
/// The returned string must be freed by the caller (Python side manages this).
type NativeCallback = unsafe extern "C" fn(argc: i32, argv: *mut c_void) -> *mut c_void;

// ===========================================================================
// C ABI — Global variable injection (extended)
// ===========================================================================

/// Set a global number variable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_set_global_number(
    ptr: *mut KossInstance,
    name: *const c_char,
    value: f64,
) -> KossResult {
    output_license_once();
    unsafe {
        if ptr.is_null() || name.is_null() {
            return KossResult::err(2, "null pointer");
        }
        let _api_guard = api_guard_result!(ptr);
        let instance = &mut *ptr;
        let name_str = match CStr::from_ptr(name).to_str() {
            Ok(s) => s,
            Err(e) => return KossResult::err(2, &format!("invalid UTF-8: {e}")),
        };
        let js_key = boa_engine::js_string!(name_str);
        let js_val = JsValue::from(value);
        let _ = instance.context.register_global_property(
            js_key,
            js_val,
            boa_engine::property::Attribute::WRITABLE
                | boa_engine::property::Attribute::CONFIGURABLE,
        );
        KossResult::ok("ok")
    }
}

/// Set a global boolean variable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_set_global_bool(
    ptr: *mut KossInstance,
    name: *const c_char,
    value: bool,
) -> KossResult {
    output_license_once();
    unsafe {
        if ptr.is_null() || name.is_null() {
            return KossResult::err(2, "null pointer");
        }
        let _api_guard = api_guard_result!(ptr);
        let instance = &mut *ptr;
        let name_str = match CStr::from_ptr(name).to_str() {
            Ok(s) => s,
            Err(e) => return KossResult::err(2, &format!("invalid UTF-8: {e}")),
        };
        let js_key = boa_engine::js_string!(name_str);
        let js_val = JsValue::from(value);
        let _ = instance.context.register_global_property(
            js_key,
            js_val,
            boa_engine::property::Attribute::WRITABLE
                | boa_engine::property::Attribute::CONFIGURABLE,
        );
        KossResult::ok("ok")
    }
}

/// Set a global null variable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_set_global_null(
    ptr: *mut KossInstance,
    name: *const c_char,
) -> KossResult {
    output_license_once();
    unsafe {
        if ptr.is_null() || name.is_null() {
            return KossResult::err(2, "null pointer");
        }
        let _api_guard = api_guard_result!(ptr);
        let instance = &mut *ptr;
        let name_str = match CStr::from_ptr(name).to_str() {
            Ok(s) => s,
            Err(e) => return KossResult::err(2, &format!("invalid UTF-8: {e}")),
        };
        let js_key = boa_engine::js_string!(name_str);
        let _ = instance.context.register_global_property(
            js_key,
            JsValue::null(),
            boa_engine::property::Attribute::WRITABLE
                | boa_engine::property::Attribute::CONFIGURABLE,
        );
        KossResult::ok("ok")
    }
}

/// Set a global undefined variable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_set_global_undefined(
    ptr: *mut KossInstance,
    name: *const c_char,
) -> KossResult {
    output_license_once();
    unsafe {
        if ptr.is_null() || name.is_null() {
            return KossResult::err(2, "null pointer");
        }
        let _api_guard = api_guard_result!(ptr);
        let instance = &mut *ptr;
        let name_str = match CStr::from_ptr(name).to_str() {
            Ok(s) => s,
            Err(e) => return KossResult::err(2, &format!("invalid UTF-8: {e}")),
        };
        let js_key = boa_engine::js_string!(name_str);
        let _ = instance.context.register_global_property(
            js_key,
            JsValue::undefined(),
            boa_engine::property::Attribute::WRITABLE
                | boa_engine::property::Attribute::CONFIGURABLE,
        );
        KossResult::ok("ok")
    }
}

/// Set a global variable from a JSON string (supports objects, arrays, strings, numbers).
/// Uses serde_json validation + Boa native JSON.parse via global property (no eval of user data).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_set_global_json(
    ptr: *mut KossInstance,
    name: *const c_char,
    json_str: *const c_char,
) -> KossResult {
    output_license_once();
    unsafe {
        if ptr.is_null() || name.is_null() || json_str.is_null() {
            return KossResult::err(2, "null pointer");
        }
        let _api_guard = api_guard_result!(ptr);
        let instance = &mut *ptr;
        let name_str = match CStr::from_ptr(name).to_str() {
            Ok(s) => s,
            Err(e) => return KossResult::err(2, &format!("invalid UTF-8: {e}")),
        };
        let json = match CStr::from_ptr(json_str).to_str() {
            Ok(s) => s,
            Err(e) => return KossResult::err(2, &format!("invalid UTF-8: {e}")),
        };

        // Validate JSON with serde_json (Rust-side pre-validation)
        if let Err(e) = serde_json::from_str::<serde_json::Value>(json) {
            return KossResult::err(1, &format!("invalid JSON: {e}"));
        }

        // Register raw JSON string as a temp global (no eval, native API)
        let temp_key = "__koss_json_tmp__";
        let _ = instance.context.register_global_property(
            boa_engine::js_string!(temp_key),
            JsValue::from(boa_engine::js_string!(json)),
            boa_engine::property::Attribute::WRITABLE
                | boa_engine::property::Attribute::CONFIGURABLE,
        );

        // Parse via hardcoded JS — no user data concatenated into code
        let parse_code = "JSON.parse(globalThis.__koss_json_tmp__)";
        let source = Source::from_bytes(parse_code.as_bytes());
        match instance.context.eval(source) {
            Ok(val) => {
                let js_key = boa_engine::js_string!(name_str);
                let _ = instance.context.register_global_property(
                    js_key,
                    val,
                    boa_engine::property::Attribute::WRITABLE
                        | boa_engine::property::Attribute::CONFIGURABLE,
                );
                // Cleanup temp global
                let _ = instance
                    .context
                    .eval(Source::from_bytes(b"delete globalThis.__koss_json_tmp__"));
                KossResult::ok("ok")
            }
            Err(e) => KossResult::err(1, &format!("JSON parse error: {e}")),
        }
    }
}

// ===========================================================================
// C ABI — Function registration (host → JS)
// ===========================================================================

/// Internal helper to create a JS NativeFunction from a C callback and register it
/// as a global. Returns the JsValue so callers can use it for further operations.
fn register_native_function(
    ctx: &mut Context,
    callback: NativeCallback,
) -> boa_engine::JsValue {
    let native = NativeFunction::from_copy_closure(move |_this, args, ctx| {
        let argc = args.len() as i32;
        let mut c_strings: Vec<CString> = Vec::with_capacity(args.len());
        let mut ptrs: Vec<*const c_char> = Vec::with_capacity(args.len());

        for arg in args {
            let s = js_value_to_string(arg, ctx);
            let c_str = CString::new(s).unwrap_or(CString::new("").unwrap());
            ptrs.push(c_str.as_ptr());
            c_strings.push(c_str);
        }

        let result = unsafe { callback(argc, ptrs.as_ptr() as *mut c_void) };

        if result.is_null() {
            return Ok(JsValue::undefined());
        }

        let result_str = unsafe {
            CStr::from_ptr(result as *const c_char)
                .to_str()
                .unwrap_or("")
                .to_string()
        };
        // Note: result memory is managed by Python (callback_allocations)
        // We do NOT free it here since Python allocated it with msvcrt malloc
        Ok(JsValue::from(boa_engine::js_string!(result_str)))
    });

    let js_func = native.to_js_function(ctx.realm());
    js_func.into()
}

/// Set a nested property path using bracket notation via JS eval.
/// Path components are escaped via escape_js_string for safe inclusion in
/// single-quoted string literals. Intermediate objects are created as needed.
fn set_nested_property(ctx: &mut Context, path: &str, value: boa_engine::JsValue) {
    let temp_key = format!(
        "__koss_tmp_{}",
        path.replace('.', "_")
            .replace(|c: char| !c.is_alphanumeric() && c != '_', "_")
    );
    let _ = ctx.register_global_property(
        boa_engine::js_string!(temp_key.as_str()),
        value,
        boa_engine::property::Attribute::WRITABLE
            | boa_engine::property::Attribute::CONFIGURABLE,
    );

    let parts: Vec<&str> = path.split('.').collect();
    let last_escaped = escape_js_string(parts.last().copied().unwrap_or(""));

    let mut create_chain = String::from("var o = globalThis;");
    for part in parts.iter().take(parts.len().saturating_sub(1)) {
        let esc = escape_js_string(part);
        create_chain.push_str(&format!(
            "if (typeof o['{}'] !== 'object' || o['{}'] === null) {{ o['{}'] = {{}}; }} o = o['{}'];",
            esc, esc, esc, esc
        ));
    }

    let eval_code = format!(
        "{{ {} o['{}'] = globalThis.{}; delete globalThis.{}; }}",
        create_chain, last_escaped, temp_key, temp_key
    );

    let source = Source::from_bytes(eval_code.as_bytes());
    let _ = ctx.eval(source);
}

/// Register a global function from a C callback.
/// Supports dotted paths (e.g., "Math.max") for mounting to nested objects.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_register_function(
    ptr: *mut KossInstance,
    name: *const c_char,
    callback: NativeCallback,
) -> KossResult {
    output_license_once();
    unsafe {
        if ptr.is_null() || name.is_null() || callback as usize == 0 {
            return KossResult::err(2, "null pointer");
        }

        let _api_guard = api_guard_result!(ptr);
        let instance = &mut *ptr;
        let name_str = match CStr::from_ptr(name).to_str() {
            Ok(s) => s,
            Err(e) => return KossResult::err(2, &format!("invalid UTF-8: {e}")),
        };

        if name_str.contains('.') {
            let js_func = register_native_function(&mut instance.context, callback);
            set_nested_property(&mut instance.context, name_str, js_func);
        } else {
            let js_func = register_native_function(&mut instance.context, callback);
            let _ = instance.context.register_global_property(
                boa_engine::js_string!(name_str),
                js_func,
                boa_engine::property::Attribute::WRITABLE
                    | boa_engine::property::Attribute::CONFIGURABLE,
            );
        }

        KossResult::ok("ok")
    }
}

// ===========================================================================
// C ABI — Module loader registration
// ===========================================================================

/// Register the CommonJS module loader callback.
/// The callback receives (module_name_string) and returns JSON string or null.
/// The returned JSON should be `{"type": "module", "code": "..."}`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_register_module_loader(
    ptr: *mut KossInstance,
    callback: NativeCallback,
) -> KossResult {
    output_license_once();
    unsafe {
        if ptr.is_null() {
            return KossResult::err(2, "null pointer");
        }

        let _api_guard = api_guard_result!(ptr);
        let instance = &mut *ptr;
        if callback as usize == 0 {
            instance.external_module_loader = None;
            return KossResult::ok("external loader cleared");
        }

        instance.external_module_loader = Some(callback);

        KossResult::ok("ok")
    }
}

// ===========================================================================
// C ABI — Class registration
// ===========================================================================

/// Register a JavaScript class backed by a native callback.
///
/// `class_name` - the JS class name
/// `methods_json` - JSON array of method names (e.g., `["method1", "method2"]`)
/// `callback` - receives (method_name, argc, argv) and returns JSON string or null
///
/// The callback will be invoked as `callback(method_name, argc, argv)` where:
/// - `method_name` is a C string naming the method to call
/// - `argc` is the argument count
/// - `argv` is an array of C strings
/// - Returns a C string (JSON) or null
///
/// The class constructor creates instances with methods that call back to the native
/// callback. Each method passes the method name as the first argument.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_register_class(
    ptr: *mut KossInstance,
    class_name: *const c_char,
    methods_json: *const c_char,
    callback: NativeCallback,
) -> KossResult {
    output_license_once();
    unsafe {
        if ptr.is_null() || class_name.is_null() || methods_json.is_null() || callback as usize == 0 {
            return KossResult::err(2, "null pointer");
        }

        let _api_guard = api_guard_result!(ptr);
        let instance = &mut *ptr;
        let name_str = match CStr::from_ptr(class_name).to_str() {
            Ok(s) => s,
            Err(e) => return KossResult::err(2, &format!("invalid UTF-8: {e}")),
        };
        // Sanitize: only allow valid JavaScript identifier characters
        if name_str.is_empty() || !name_str.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '$') {
            return KossResult::err(2, "class name must be a valid JavaScript identifier");
        }
        let methods_str = match CStr::from_ptr(methods_json).to_str() {
            Ok(s) => s,
            Err(e) => return KossResult::err(2, &format!("invalid UTF-8: {e}")),
        };

        // Validate methods JSON
        let method_names: Vec<String> = match serde_json::from_str(methods_str) {
            Ok(v) => v,
            Err(e) => return KossResult::err(2, &format!("invalid methods JSON: {e}")),
        };

        // Register the method dispatcher function
        let dispatcher = NativeFunction::from_copy_closure(move |_this, args, ctx| {
            // First arg is the method name
            if args.is_empty() {
                return Ok(JsValue::undefined());
            }
            let method_name = js_value_to_string(&args[0], ctx);

            // Remaining args are the method arguments
            let argc = (args.len() - 1) as i32;
            let mut c_strings: Vec<CString> = Vec::with_capacity(args.len());
            let mut ptrs: Vec<*const c_char> = Vec::with_capacity(args.len());

            // First pointer is the method name
            let name_cstr = CString::new(method_name.clone()).unwrap_or(CString::new("").unwrap());
            ptrs.push(name_cstr.as_ptr());
            c_strings.push(name_cstr);

            // Remaining pointers are the args
            for arg in &args[1..] {
                let s = js_value_to_string(arg, ctx);
                let c_str = CString::new(s).unwrap_or(CString::new("").unwrap());
                ptrs.push(c_str.as_ptr());
                c_strings.push(c_str);
            }

            // Callback receives (method_name, argc, argv)
            let result = callback(argc + 1, ptrs.as_ptr() as *mut c_void);

            if result.is_null() {
                return Ok(JsValue::undefined());
            }

            let result_str = CStr::from_ptr(result as *const c_char)
                .to_str()
                .unwrap_or("")
                .to_string();
            Ok(JsValue::from(boa_engine::js_string!(result_str)))
        });

        let dispatcher_func = dispatcher.to_js_function(instance.context.realm());

        // Register the dispatcher with a unique name
        let dispatcher_key = format!("__koss_class_{}", name_str);
        let _ = instance.context.register_global_property(
            boa_engine::js_string!(dispatcher_key.as_str()),
            dispatcher_func,
            boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
        );

        // Create the JS class constructor via eval
        let methods_array_json = serde_json::to_string(&method_names).unwrap_or_default();
        let class_code = format!(
            r#"
(function() {{
    var methods = {};
    var dispatcher = globalThis.{} || function() {{}};
    function {}() {{
        var self = {{}};
        methods.forEach(function(m) {{
            self[m] = function() {{
                var args = Array.prototype.slice.call(arguments);
                var allArgs = [m].concat(args);
                return dispatcher.apply(null, allArgs);
            }};
        }});
        return self;
    }}
    globalThis.{} = {};
}})();
"#,
            methods_array_json, dispatcher_key, name_str, name_str, name_str
        );

        let source = Source::from_bytes(class_code.as_bytes());
        match instance.context.eval(source) {
            Ok(_) => KossResult::ok("ok"),
            Err(e) => KossResult::err(1, &format!("class registration error: {e}")),
        }
    }
}

#[cfg(any(target_os = "windows", all(target_os = "linux", not(target_env = "ohos")), target_os = "macos"))]
fn register_senri_ffi_impl(instance: &mut KossInstance) {
    if instance.stable {
        // Register stub that throws explicit error
        let ctx = &mut instance.context;
        use boa_engine::object::ObjectInitializer;
        use boa_engine::property::Attribute;

        let mut ob = ObjectInitializer::new(ctx);
        let api_names = [("func", 0usize), ("open", 1usize)];
        for (name, len) in &api_names {
            let err_fn = unsafe {
                NativeFunction::from_closure(
                    move |_t: &JsValue, _a: &[JsValue], _c: &mut Context| -> Result<JsValue, JsError> {
                        Err(JsNativeError::typ()
                            .with_message("FFI is disabled in stable mode. Set stable=false when creating the instance to enable FFI features.")
                            .into())
                    },
                )
            };
            ob.function(err_fn, js_string!(*name), *len);
        }
        let senri_obj = ob.build();
        ctx.register_global_property(
            boa_engine::js_string!("_senri_ffi"),
            senri_obj,
            Attribute::READONLY | Attribute::NON_ENUMERABLE,
        ).ok();
        return;
    }
    let ptr = instance as *mut KossInstance as *mut c_void;
    crate::_senri_ffi::register_senri_ffi(
        &mut instance.context,
        ptr,
    );
}

#[cfg(not(any(target_os = "windows", all(target_os = "linux", not(target_env = "ohos")), target_os = "macos")))]
fn register_senri_ffi_impl(instance: &mut KossInstance) {
    let ctx = &mut instance.context;
    use boa_engine::object::ObjectInitializer;
    use boa_engine::property::Attribute;

    if instance.stable {
        // Register stub that throws explicit error
        let mut ob = ObjectInitializer::new(ctx);
        let api_names = [("func", 0usize), ("open", 1usize)];
        for (name, len) in &api_names {
            let err_fn = unsafe {
                NativeFunction::from_closure(
                    move |_t: &JsValue, _a: &[JsValue], _c: &mut Context| -> Result<JsValue, JsError> {
                        Err(JsNativeError::typ()
                            .with_message("FFI is disabled in stable mode. Set stable=false when creating the instance to enable FFI features.")
                            .into())
                    },
                )
            };
            ob.function(err_fn, js_string!(*name), *len);
        }
        let senri_obj = ob.build();
        ctx.register_global_property(
            boa_engine::js_string!("_senri_ffi"),
            senri_obj,
            Attribute::READONLY | Attribute::NON_ENUMERABLE,
        ).ok();
        return;
    }

    let types_obj = {
        let mut tb = ObjectInitializer::new(ctx);
        let type_names: [&str; 14] = [
            "void", "int8", "uint8", "int16", "uint16", "int32", "uint32",
            "int64", "uint64", "float32", "float64", "pointer", "cstring", "...",
        ];
        for name in &type_names {
            tb.property(
                js_string!(*name),
                js_string!(*name),
                Attribute::READONLY | Attribute::NON_ENUMERABLE,
            );
        }
        tb.build()
    };

    let mut builder = ObjectInitializer::new(ctx);
    builder.property(
        js_string!("types"),
        types_obj,
        Attribute::READONLY | Attribute::NON_ENUMERABLE,
    );

    let api_names: [(&str, usize); 12] = [
        ("open", 1),
        ("struct", 2),
        ("pointer", 1),
        ("array", 2),
        ("callback", 2),
        ("createCallback", 3),
        ("alloc", 1),
        ("allocType", 2),
        ("free", 1),
        ("addressOf", 1),
        ("errno", 0),
        ("strerror", 1),
    ];

    for (name, len) in &api_names {
        let err_clone = unsafe {
            NativeFunction::from_closure(
                move |_t: &JsValue, _a: &[JsValue], _c: &mut Context| -> Result<JsValue, JsError> {
                    Err(JsNativeError::error()
                        .with_message(format!(
                            "_senri_ffi is not supported on {}. Dynamic library loading is restricted on mobile platforms (Android/iOS/HarmonyOS). Use Windows/Linux/macOS instead.",
                            std::env::consts::OS
                        ))
                        .into())
                },
            )
        };
        builder.function(err_clone, js_string!(*name), *len as usize);
    }

    let senri_obj = builder.build();
    let _ = ctx.register_global_property(
        js_string!("_senri_ffi"),
        senri_obj,
        Attribute::all(),
    );
}

#[cfg(any(target_os = "windows", all(target_os = "linux", not(target_env = "ohos")), target_os = "macos"))]
fn register_dlopen_binding(instance: &mut KossInstance) {
    let instance_ptr = instance as *mut KossInstance;
    let ctx = &mut instance.context;
    let dlopen_fn = unsafe {
        NativeFunction::from_closure(
            move |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
                let instance = &*instance_ptr;
                authorize_operation(instance, crate::sandbox::NATIVE_ADDON, "nativeAddon.dlopen", args, ctx)?;
                let module = args.first()
                    .and_then(|v| v.as_object())
                    .ok_or_else(|| JsNativeError::error().with_message("process.dlopen: module required"))?;
                let filename = args.get(1)
                    .and_then(|v| v.as_string())
                    .map(|s| s.to_std_string_escaped())
                    .ok_or_else(|| JsNativeError::error().with_message("process.dlopen: filename required"))?;

                crate::bindings::process_dlopen::dlopen_impl(&module, &filename, ctx)?;
                Ok(JsValue::undefined())
            },
        )
    };
    let js_func = dlopen_fn.to_js_function(ctx.realm());
    let _ = ctx.register_global_property(
        js_string!("__koss_dlopen"),
        js_func,
        boa_engine::property::Attribute::all(),
    );

    let bootstrap = r#"
    (function() {
        if (typeof process === 'undefined') { globalThis.process = {}; }
        process.dlopen = function(mod, filename) {
            return __koss_dlopen(mod, filename);
        };
    })();
    "#;
    let source = Source::from_bytes(bootstrap.as_bytes());
    let _ = ctx.eval(source);
}

#[cfg(not(any(target_os = "windows", all(target_os = "linux", not(target_env = "ohos")), target_os = "macos")))]
fn register_dlopen_binding(instance: &mut KossInstance) {
    let ctx = &mut instance.context;
    let dlopen_fn = unsafe {
        NativeFunction::from_closure(
            move |_this: &JsValue, _args: &[JsValue], _ctx: &mut Context| -> Result<JsValue, JsError> {
                Err(JsNativeError::error()
                    .with_message(format!(
                        "process.dlopen is not supported on {}. Native .node addons require _senri_ffi which is restricted on mobile platforms (Android/iOS/HarmonyOS). Use Windows/Linux/macOS instead.",
                        std::env::consts::OS
                    ))
                    .into())
            },
        )
    };
    let js_func = dlopen_fn.to_js_function(ctx.realm());
    let _ = ctx.register_global_property(
        js_string!("__koss_dlopen"),
        js_func,
        boa_engine::property::Attribute::all(),
    );

    let bootstrap = r#"
    (function() {
        if (typeof process === 'undefined') { globalThis.process = {}; }
        process.dlopen = function(mod, filename) {
            return __koss_dlopen(mod, filename);
        };
    })();
    "#;
    let source = Source::from_bytes(bootstrap.as_bytes());
    let _ = ctx.eval(source);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static AUDIT_CALLS: AtomicUsize = AtomicUsize::new(0);

    unsafe extern "C" fn deny_audit(
        _target: *const c_char,
        _args: *const *const c_char,
        _argc: i32,
        _pwd: *const c_char,
        _userdata: *mut c_void,
    ) -> bool {
        AUDIT_CALLS.fetch_add(1, Ordering::SeqCst);
        false
    }

    #[test]
    fn test_capability_error_message_debug_enabled() {
        let msg = capability_error_message("fs", true);
        assert!(msg.contains("KossCapabilityError"));
        assert!(msg.contains("fs"));
        assert!(msg.contains("capability denied"));
    }

    #[test]
    fn test_capability_error_message_debug_disabled() {
        let msg = capability_error_message("fs", false);
        assert!(msg.contains("KossCapabilityError"));
        assert!(msg.contains("Access denied"));
        assert!(!msg.contains("fs"));
    }

    #[test]
    fn test_security_error_message_debug_enabled() {
        let msg = security_error_message("net", true);
        assert!(msg.contains("KossSecurityError"));
        assert!(msg.contains("net"));
        assert!(msg.contains("sandbox audit denied"));
    }

    #[test]
    fn test_security_error_message_debug_disabled() {
        let msg = security_error_message("net", false);
        assert!(msg.contains("KossSecurityError"));
        assert!(msg.contains("Access denied"));
        assert!(!msg.contains("net"));
    }

    #[test]
    fn test_timeout_error_message_debug_enabled() {
        let msg = timeout_error_message("crypto", true);
        assert!(msg.contains("KossTimeoutError"));
        assert!(msg.contains("crypto"));
        assert!(msg.contains("sandbox audit timed out"));
    }

    #[test]
    fn test_timeout_error_message_debug_disabled() {
        let msg = timeout_error_message("crypto", false);
        assert!(msg.contains("KossTimeoutError"));
        assert!(msg.contains("Access denied"));
        assert!(!msg.contains("crypto"));
    }

    #[test]
    fn test_cancel_error_message_debug_enabled() {
        let msg = cancel_error_message("fs", true);
        assert!(msg.contains("KossCancelError"));
        assert!(msg.contains("fs"));
        assert!(msg.contains("sandbox audit cancelled"));
    }

    #[test]
    fn test_cancel_error_message_debug_disabled() {
        let msg = cancel_error_message("fs", false);
        assert!(msg.contains("KossCancelError"));
        assert!(msg.contains("Access denied"));
        assert!(!msg.contains("fs"));
    }

    #[test]
    fn test_crypto_registration_requires_crypto_capability() {
        let context = Context::default();
        let mut instance = KossInstance::new(
            context,
            crate::sandbox::NET_FETCH,
            true,
            crate::builtins::KOSS_BUILTIN_ALL,
        );

        register_runtime_capabilities(&mut instance);

        let value = instance
            .context
            .eval(Source::from_bytes(b"typeof __koss_random_bytes"))
            .expect("capability check script should run");
        assert_eq!(value.to_string(&mut instance.context).unwrap().to_std_string_escaped(), "undefined");
    }

    #[test]
    fn test_tcp_server_is_not_registered_for_client_only_capability() {
        let context = Context::default();
        let mut instance = KossInstance::new(
            context,
            crate::sandbox::NET_TCP_CLIENT,
            true,
            crate::builtins::KOSS_BUILTIN_ALL,
        );

        register_runtime_capabilities(&mut instance);

        let value = instance
            .context
            .eval(Source::from_bytes(b"typeof __koss_tcp_listen"))
            .expect("capability check script should run");
        assert_eq!(value.to_string(&mut instance.context).unwrap().to_std_string_escaped(), "undefined");
    }

    #[test]
    fn test_network_bindings_are_registered_per_capability() {
        for (caps, bindings) in [
            (crate::sandbox::NET_TCP_CLIENT, &["__koss_tcp_connect"][..]),
            (crate::sandbox::NET_TCP_SERVER, &["__koss_tcp_listen", "__koss_tcp_accept"][..]),
            (crate::sandbox::NET_DNS, &["__koss_dns_lookup"][..]),
        ] {
            let context = Context::default();
            let mut instance = KossInstance::new(
                context,
                caps,
                true,
                crate::builtins::KOSS_BUILTIN_ALL,
            );

            register_runtime_capabilities(&mut instance);

            for binding in bindings {
                let value = instance
                    .context
                    .eval(Source::from_bytes(format!("typeof {binding}").as_bytes()))
                    .expect("capability check script should run");
                assert_eq!(
                    value.to_string(&mut instance.context).unwrap().to_std_string_escaped(),
                    "function",
                    "{binding} should be registered for caps={caps:#x}"
                );
            }
        }
    }

    #[test]
    fn test_crypto_bindings_are_registered_per_capability() {
        for (caps, bindings) in [
            (crate::sandbox::CRYPTO_HASH, &["__koss_hash", "__koss_hash_bytes"][..]),
            (crate::sandbox::CRYPTO_RANDOM, &["__koss_random_bytes", "__koss_random_uuid"][..]),
            (crate::sandbox::CRYPTO_HMAC, &["__koss_hmac_bytes", "__koss_aes_gcm_encrypt"][..]),
            (crate::sandbox::CRYPTO_PBKDF2, &["__koss_pbkdf2_bytes"][..]),
        ] {
            let context = Context::default();
            let mut instance = KossInstance::new(
                context,
                caps,
                true,
                crate::builtins::KOSS_BUILTIN_ALL,
            );

            register_runtime_capabilities(&mut instance);

            for binding in bindings {
                let value = instance
                    .context
                    .eval(Source::from_bytes(format!("typeof {binding}").as_bytes()))
                    .expect("capability check script should run");
                assert_eq!(
                    value.to_string(&mut instance.context).unwrap().to_std_string_escaped(),
                    "function",
                    "{binding} should be registered for caps={caps:#x}"
                );
            }
        }

        let context = Context::default();
        let mut instance = KossInstance::new(
            context,
            crate::sandbox::CRYPTO_HASH,
            true,
            crate::builtins::KOSS_BUILTIN_ALL,
        );

        register_runtime_capabilities(&mut instance);

        for (binding, expected) in [
            ("__koss_hash", "function"),
            ("__koss_hash_bytes", "function"),
            ("__koss_random_bytes", "undefined"),
            ("__koss_hmac_bytes", "undefined"),
            ("__koss_pbkdf2_bytes", "undefined"),
        ] {
            let value = instance
                .context
                .eval(Source::from_bytes(format!("typeof {binding}").as_bytes()))
                .expect("capability check script should run");
            assert_eq!(
                value.to_string(&mut instance.context).unwrap().to_std_string_escaped(),
                expected,
                "{binding} registration did not match its capability"
            );
        }
    }

    #[test]
    fn test_dlopen_requires_unstable_native_addon_capability() {
        for (caps, stable, expected) in [
            (crate::sandbox::NATIVE_ADDON, false, "function"),
            (crate::sandbox::NATIVE_ADDON, true, "undefined"),
            (crate::sandbox::KOSS_CAP_SANDBOX, false, "undefined"),
        ] {
            let instance = unsafe { &mut *koss_create_with_caps(caps, stable) };
            let value = instance
                .context
                .eval(Source::from_bytes(b"typeof process.dlopen"))
                .expect("capability check script should run");
            assert_eq!(
                value.to_string(&mut instance.context).unwrap().to_std_string_escaped(),
                expected,
                "dlopen registration did not match stable={stable}, caps={caps:#x}"
            );
            unsafe { koss_destroy(instance) };
        }
    }

    #[test]
    fn test_low_level_fs_operation_honors_audit_denial() {
        AUDIT_CALLS.store(0, Ordering::SeqCst);
        let context = Context::default();
        let mut instance = KossInstance::new(
            context,
            crate::sandbox::FS_READ,
            true,
            crate::builtins::KOSS_BUILTIN_ALL,
        );
        instance.sandbox.audit_mask = crate::sandbox::FS_READ;
        instance.sandbox.sync_audit = Some(deny_audit);
        register_fs_functions(&mut instance);

        let result = instance
            .context
            .eval(Source::from_bytes(b"__koss_fs_exists('.')"));

        assert!(result.is_err());
        assert_eq!(AUDIT_CALLS.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_api_guard_rejects_reentry_and_cross_thread_access() {
        let key = usize::MAX - 17;
        register_api_instance(key);

        let guard = enter_api(key).expect("owner thread should enter instance");
        assert!(enter_api(key).is_err());
        let cross_thread = std::thread::spawn(move || enter_api(key).is_err())
            .join()
            .unwrap();
        assert!(cross_thread);

        drop(guard);
        assert!(enter_api(key).is_ok());
        unregister_api_instance(key);
    }

    #[test]
    fn test_public_api_rejects_cross_thread_instance_access() {
        let ptr = koss_create_with_caps(crate::sandbox::KOSS_CAP_ALL, false);
        assert!(!ptr.is_null());
        let key = ptr as usize;

        let (result_code, capabilities, stable, builtins, builtin_enabled, audit_mask) =
            std::thread::spawn(move || unsafe {
                let result = koss_set_audit_mask(key as *mut KossInstance, u32::MAX);
                let result_code = result.code;
                koss_free_result(result);
                koss_enable_audit_debug(key as *mut KossInstance, true);
                (
                    result_code,
                    koss_get_capabilities(key as *mut KossInstance),
                    koss_is_stable(key as *mut KossInstance),
                    koss_get_builtins(key as *mut KossInstance),
                    koss_is_builtin_enabled(key as *mut KossInstance, 1),
                    koss_get_audit_mask(key as *mut KossInstance),
                )
            })
            .join()
            .unwrap();

        assert_eq!(result_code, 3);
        assert_eq!(capabilities, 0);
        assert!(!stable);
        assert_eq!(builtins, 0);
        assert!(!builtin_enabled);
        assert_eq!(audit_mask, 0);
        assert!(!unsafe { (*ptr).sandbox.audit_debug });

        unsafe { koss_destroy(ptr) };
    }
}
