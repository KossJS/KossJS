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
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
#[cfg(any(target_os = "windows", all(target_os = "linux", not(target_env = "ohos")), target_os = "macos"))]
use std::sync::Arc;
#[cfg(any(target_os = "windows", all(target_os = "linux", not(target_env = "ohos")), target_os = "macos"))]
use std::sync::atomic::AtomicUsize;
use std::sync::Mutex;
use std::net::{TcpListener, TcpStream, ToSocketAddrs, IpAddr};
use std::time::{Duration, Instant};

use boa_engine::object::builtins::{JsFunction, JsPromise};
use boa_engine::{Context, JsError, JsNativeError, JsValue, Module, Source, NativeFunction};
use boa_engine::js_string;
use boa_runtime::Console;
use tokio::runtime::Runtime;

use crate::bindings;
use crate::buffer;
use crate::license_output::output_license_once;
use crate::version::get_version;
use crate::module_loader::KossModuleLoader;
use crate::worker::{WorkerEvent, WorkerPool};

// One-time warning flags for unstable mode (stable=false)
static FFI_STABLE_WARNED: AtomicBool = AtomicBool::new(false);
static WORKER_STABLE_WARNED: AtomicBool = AtomicBool::new(false);

// TCP socket storage for persistent connections
static NEXT_TCP_FD: AtomicU32 = AtomicU32::new(1);
static TCP_CONNECTIONS: std::sync::LazyLock<Mutex<HashMap<u32, TcpStream>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));
static NEXT_SERVER_FD: AtomicU32 = AtomicU32::new(1001);
static TCP_SERVERS: std::sync::LazyLock<Mutex<HashMap<u32, TcpListener>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

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
                match js_fn.call(&JsValue::undefined(), &js_args, ctx) {
                    Ok(js_val) => {
                        let ret_bytes = ffi_js_value_to_bytes(&js_val, &req.ret_type);
                        Ok(ret_bytes)
                    }
                    Err(_) => Ok(vec![0u8; req.ret_type.sizeof()]),
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
/// Maximum permitted worker pool size (CWE-400: prevent resource exhaustion).
const MAX_WORKER_POOL_SIZE: usize = 64;

/// Maximum permitted externally-loaded module code size (CWE-94: prevent
/// code injection via oversized external module payloads).
const MAX_EXTERNAL_MODULE_CODE_SIZE: usize = 10 * 1024 * 1024; // 10 MiB

use crate::sandbox::{
    AuditDecision, KOSS_CAP_ALL_CRYPTO, KOSS_CAP_ALL_FS, KOSS_CAP_ALL_NET,
    KOSS_CAP_EXTERNAL_LOADER, KOSS_CAP_WORKER, SandboxState,
};

// ---------------------------------------------------------------------------
// Opaque handle — each KossInstance is an isolated JS VM
// ---------------------------------------------------------------------------
// SAFETY: Boa Context is not Sync. All mutable access to the context and its
// associated fields (event_loop, worker_pool, external_module_loader) MUST
// occur on the same thread that created the instance. The NativeFunction
// closures below capture raw pointers (or Rc handles) to these fields, and
// are guaranteed by Boa's single-threaded execution model to only be invoked
// from the owning thread.
//
// THREAD-SAFETY WARNING FOR HOSTS (CWE-362):
// The C ABI functions below (koss_eval, koss_tick, koss_worker_execute, etc.)
// directly dereference `*mut KossInstance` without any mutex or lock. All
// calls to C API functions for a given KossInstance MUST be made from a
// single thread. Concurrent access from multiple threads will cause
// undefined behavior (data races, memory corruption, crashes).
// For multi-threaded hosts, serialise all KossInstance access through an
// external mutex or ensure exclusive thread ownership.
pub struct KossInstance {
    pub event_loop: Option<KossEventLoop>,
    pub worker_pool: Option<WorkerPool>,
    /// Optional external module loader callback (e.g. from Python).
    /// Called as a fallback when the embedded stdlib doesn't contain the module.
    pub external_module_loader: Option<NativeCallback>,
    /// Bitmask of enabled capabilities (see KOSS_CAP_* constants).
    pub capabilities: u32,
    /// Sandbox state: audit mask and future extension fields.
    pub sandbox: SandboxState,
    /// When true, FFI and Worker capabilities are disabled (stable mode).
    pub stable: bool,
    /// Builtin module flags — controls which koss:* modules are visible.
    /// See KOSS_BUILTIN_* constants in builtins.rs.
    pub builtins: u32,
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
            worker_pool: None,
            external_module_loader: None,
            capabilities: caps,
            sandbox: SandboxState::default(),
            stable,
            builtins,
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

        // Special case: worker/worker_threads require stable=false
        // (KOSS_CAP_WORKER bit overlaps with FS_MKDIR, requiring stable-aware logic)
        if module_name == "worker" || module_name == "worker_threads" {
            let inst = unsafe { &*instance_ptr };
            if inst.stable {
                return Err(JsError::from(JsNativeError::error()
                    .with_message("KossCapabilityError: Module 'worker_threads' is disabled in stable mode. Set stable=false when creating the instance to enable Worker features.")));
            }
        }

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
        // Check NET_FETCH capability
        let inst = unsafe { &mut *instance_ptr };
        if !crate::sandbox::has_cap(inst.capabilities, crate::sandbox::NET_FETCH) {
            return Err(JsError::from(JsNativeError::typ()
                .with_message("KossCapabilityError: capability denied for fetch")));
        }

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
    macro_rules! reg_fs {
        ($name:expr, $closure:expr) => {{
            let js_fn = NativeFunction::from_copy_closure($closure)
                .to_js_function(instance.context.realm());
            instance.context.register_global_property(
                boa_engine::js_string!($name),
                js_fn,
                boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
            ).ok();
        }};
    }

    // __koss_fs_exists(path) -> bool
    reg_fs!("__koss_fs_exists", |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
        let path = args.first().ok_or_else(|| JsNativeError::error().with_message("exists: path required"))?
            .to_string(ctx).map_err(|_| JsNativeError::error().with_message("exists: path must be string"))?;
        Ok(JsValue::from(crate::bindings::fs::exists_sync(&path.to_std_string_escaped())))
    });

    // __koss_fs_read(path) -> { code: 0, value: base64_string }
    reg_fs!("__koss_fs_read", |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
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

    // __koss_fs_write(path, data) -> { code: 0 }
    reg_fs!("__koss_fs_write", |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
        if args.len() < 2 {
            return Err(JsNativeError::error().with_message("write: path and data required").into());
        }
        let path = args[0].to_string(ctx).map_err(|_| JsNativeError::error().with_message("write: path must be string"))?;
        let data_val = args[1].to_string(ctx).map_err(|_| JsNativeError::error().with_message("write: data must be string"))?;
        let path_str = path.to_std_string_escaped();
        let data_str = data_val.to_std_string_escaped();
        let bytes = data_str.as_bytes();
        match std::fs::write(&path_str, bytes) {
            Ok(()) => Ok(JsValue::from(boa_engine::js_string!("{\"code\":0}"))),
            Err(e) => Ok(JsValue::from(boa_engine::js_string!(format!("{{\"code\":1,\"value\":\"{}\"}}", e)))),
        }
    });

    // __koss_fs_stat(path) -> { code: 0, value: json_string }
    reg_fs!("__koss_fs_stat", |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
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
    reg_fs!("__koss_fs_mkdir", |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
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
    reg_fs!("__koss_fs_readdir", |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
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
    reg_fs!("__koss_fs_unlink", |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
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
    reg_fs!("__koss_fs_rename", |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
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
    reg_fs!("__koss_fs_copy", |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
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
    reg_fs!("__koss_fs_realpath", |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
        let path = args.first().ok_or_else(|| JsNativeError::error().with_message("realpath: path required"))?
            .to_string(ctx).map_err(|_| JsNativeError::error().with_message("realpath: path must be string"))?;
        let path_str = path.to_std_string_escaped();
        match std::fs::canonicalize(&path_str) {
            Ok(p) => Ok(JsValue::from(boa_engine::js_string!(p.to_string_lossy().to_string()))),
            Err(e) => Err(JsNativeError::error().with_message(format!("realpath failed: {e}")).into()),
        }
    });
}

fn register_net_functions(instance: &mut KossInstance) {
    // __koss_tcp_connect(host, port) -> fd
    let connect_fn = NativeFunction::from_copy_closure(
        move |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
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
                    let fd = NEXT_TCP_FD.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    if let Ok(mut socks) = TCP_CONNECTIONS.lock() {
                        socks.insert(fd, stream);
                    }
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
            if args.len() < 2 {
                return Err(JsNativeError::error().with_message("tcp_write: fd and data required").into());
            }
            let fd = args[0].as_number().ok_or_else(|| JsNativeError::error().with_message("tcp_write: fd must be number"))? as u32;
            let data = args[1].to_string(ctx).map_err(|_| JsNativeError::error().with_message("tcp_write: data must be string"))?;
            let data_str = data.to_std_string_escaped();

            if let Ok(mut socks) = TCP_CONNECTIONS.lock() {
                if let Some(stream) = socks.get_mut(&fd) {
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
        move |_this: &JsValue, args: &[JsValue], _ctx: &mut Context| -> Result<JsValue, JsError> {
            if args.is_empty() {
                return Err(JsNativeError::error().with_message("tcp_read: fd required").into());
            }
            let fd = args[0].as_number().ok_or_else(|| JsNativeError::error().with_message("tcp_read: fd must be number"))? as u32;

            if let Ok(mut socks) = TCP_CONNECTIONS.lock() {
                if let Some(stream) = socks.get_mut(&fd) {
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
        move |_this: &JsValue, args: &[JsValue], _ctx: &mut Context| -> Result<JsValue, JsError> {
            if args.is_empty() {
                return Err(JsNativeError::error().with_message("tcp_close: fd required").into());
            }
            let fd = args[0].as_number().ok_or_else(|| JsNativeError::error().with_message("tcp_close: fd must be number"))? as u32;

            if let Ok(mut socks) = TCP_CONNECTIONS.lock() {
                socks.remove(&fd);
            }
            if let Ok(mut servers) = TCP_SERVERS.lock() {
                servers.remove(&fd);
            }
            Ok(JsValue::undefined())
        },
    );
    instance.context.register_global_property(
        js_string!("__koss_tcp_close"),
        close_fn.to_js_function(instance.context.realm()),
        boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
    ).ok();

    // __koss_tcp_listen(host, port, backlog) -> server_fd
    let listen_fn = NativeFunction::from_copy_closure(
        move |_this: &JsValue, args: &[JsValue], _ctx: &mut Context| -> Result<JsValue, JsError> {
            if args.len() < 2 {
                return Err(JsNativeError::error().with_message("tcp_listen: host and port required").into());
            }
            let host = args[0].to_string(_ctx).map_err(|_| JsNativeError::error().with_message("tcp_listen: host must be string"))?;
            let host_str = host.to_std_string_escaped();
            let port = args[1].as_number().ok_or_else(|| JsNativeError::error().with_message("tcp_listen: port must be number"))? as u16;

            if is_ssrf_blocked(&host_str) {
                return Err(JsNativeError::error().with_message(format!("SSRF blocked: {host_str}")).into());
            }

            let addr = format!("{host_str}:{port}");
            match TcpListener::bind(&addr) {
                Ok(listener) => {
                    let _ = listener.set_nonblocking(true);
                    let fd = NEXT_SERVER_FD.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    if let Ok(mut servers) = TCP_SERVERS.lock() {
                        servers.insert(fd, listener);
                    }
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
        move |_this: &JsValue, args: &[JsValue], _ctx: &mut Context| -> Result<JsValue, JsError> {
            if args.is_empty() {
                return Err(JsNativeError::error().with_message("tcp_accept: server_fd required").into());
            }
            let sfd = args[0].as_number().ok_or_else(|| JsNativeError::error().with_message("tcp_accept: server_fd must be number"))? as u32;

            if let Ok(mut servers) = TCP_SERVERS.lock() {
                if let Some(listener) = servers.get_mut(&sfd) {
                    match listener.accept() {
                        Ok((stream, _peer_addr)) => {
                            let _ = stream.set_nonblocking(true);
                            let fd = NEXT_TCP_FD.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            if let Ok(mut socks) = TCP_CONNECTIONS.lock() {
                                socks.insert(fd, stream);
                            }
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

    // __koss_dns_lookup(hostname) -> json_string of IP addresses
    let dns_fn = NativeFunction::from_copy_closure(
        move |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
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

fn register_crypto_functions(instance: &mut KossInstance) {
    let ctx = &mut instance.context;

    let rand_fn = NativeFunction::from_copy_closure(
        move |_this: &JsValue, args: &[JsValue], _ctx: &mut Context| -> Result<JsValue, JsError> {
            let size = args.first().and_then(|v| v.as_number()).unwrap_or(32.0) as usize;
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

fn bytes_to_json_arr(data: &[u8]) -> String {
    let parts: Vec<String> = data.iter().map(|b| b.to_string()).collect();
    format!("[{}]", parts.join(","))
}

fn json_arr_to_bytes(json: &str) -> Result<Vec<u8>, String> {
    let v: Vec<u8> = serde_json::from_str(json).map_err(|e| format!("parse error: {e}"))?;
    Ok(v)
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

fn register_worker_api(instance: &mut KossInstance) {
    if instance.stable {
        // Register stub functions that throw explicit errors
        let fns = [
            "__koss_create_worker_pool",
            "__koss_worker_post_message",
            "__koss_worker_execute",
            "__koss_worker_try_recv",
            "__koss_worker_terminate",
            "__koss_worker_shutdown",
        ];
        for name in &fns {
            let js_fn = unsafe {
                NativeFunction::from_closure(
                    move |_t: &JsValue, _a: &[JsValue], _c: &mut Context| -> Result<JsValue, JsError> {
                        Err(JsNativeError::typ()
                            .with_message("Worker is disabled in stable mode. Set stable=false when creating the instance to enable Worker features.")
                            .into())
                    },
                )
            }.to_js_function(instance.context.realm());
            instance.context.register_global_property(
                boa_engine::js_string!(*name),
                js_fn,
                boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
            ).ok();
        }
        return;
    }

    let instance_ptr = instance as *mut KossInstance;

    // __koss_create_worker_pool(size) → creates worker pool
    let create_pool = NativeFunction::from_copy_closure(move |_this, args, _ctx| {
        let inst = unsafe { &mut *instance_ptr };
        let size = args.first().and_then(|v| v.as_number()).unwrap_or(1.0) as i32;
        if size <= 0 {
            return Ok(JsValue::undefined());
        }
        inst.worker_pool = Some(WorkerPool::new(size as usize));
        Ok(JsValue::from(boa_engine::js_string!(format!("{{\"created\":{size}}}"))))
    });

    let js_create_pool = create_pool.to_js_function(instance.context.realm());
    instance.context.register_global_property(
        boa_engine::js_string!("__koss_create_worker_pool"),
        js_create_pool,
        boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
    ).ok();

    // __koss_worker_post_message(workerId, data)
    let instance_ptr2 = instance as *mut KossInstance;
    let post_msg = NativeFunction::from_copy_closure(move |_this, args, ctx| {
        let inst = unsafe { &mut *instance_ptr2 };
        let pool = match inst.worker_pool.as_ref() {
            Some(p) => p,
            None => return Ok(JsValue::undefined()),
        };
        let worker_id = args.get(0).and_then(|v| v.as_number()).unwrap_or(0.0) as usize;
        let data = args.get(1).map(|v| js_value_to_string(v, ctx)).unwrap_or_default();
        match pool.post_message(worker_id, &data) {
            Ok(()) => Ok(JsValue::from(boa_engine::js_string!("ok"))),
            Err(e) => Ok(JsValue::from(boa_engine::js_string!(e))),
        }
    });

    let js_post_msg = post_msg.to_js_function(instance.context.realm());
    instance.context.register_global_property(
        boa_engine::js_string!("__koss_worker_post_message"),
        js_post_msg,
        boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
    ).ok();

    // __koss_worker_execute(workerId, code)
    let instance_ptr3 = instance as *mut KossInstance;
    let exec = NativeFunction::from_copy_closure(move |_this, args, ctx| {
        let inst = unsafe { &mut *instance_ptr3 };
        let pool = match inst.worker_pool.as_ref() {
            Some(p) => p,
            None => return Ok(JsValue::undefined()),
        };
        let worker_id = args.get(0).and_then(|v| v.as_number()).unwrap_or(0.0) as usize;
        let code = args.get(1).map(|v| js_value_to_string(v, ctx)).unwrap_or_default();
        match pool.execute(worker_id, &code) {
            Ok(cmd_id) => Ok(JsValue::from(boa_engine::js_string!(format!("{{\"commandId\":{cmd_id}}}")))),
            Err(e) => Ok(JsValue::from(boa_engine::js_string!(e))),
        }
    });

    let js_exec = exec.to_js_function(instance.context.realm());
    instance.context.register_global_property(
        boa_engine::js_string!("__koss_worker_execute"),
        js_exec,
        boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
    ).ok();

    // __koss_worker_try_recv() → gets next message from any worker
    let instance_ptr4 = instance as *mut KossInstance;
    let recv = NativeFunction::from_copy_closure(move |_this, _args, _ctx| {
        let inst = unsafe { &mut *instance_ptr4 };
        let pool = match inst.worker_pool.as_ref() {
            Some(p) => p,
            None => return Ok(JsValue::null()),
        };
        match pool.try_recv() {
            Some(event) => {
                let json = match event {
                    WorkerEvent::Result { worker_id, id, success, value } => {
                        serde_json::json!({
                            "type": "result",
                            "workerId": worker_id,
                            "id": id,
                            "success": success,
                            "value": value,
                        })
                    }
                    WorkerEvent::Message { worker_id, data } => {
                        serde_json::json!({
                            "type": "message",
                            "workerId": worker_id,
                            "data": data,
                        })
                    }
                    WorkerEvent::Error { worker_id, message } => {
                        serde_json::json!({
                            "type": "error",
                            "workerId": worker_id,
                            "message": message,
                        })
                    }
                };
                let s = serde_json::to_string(&json).unwrap_or_default();
                Ok(JsValue::from(boa_engine::js_string!(s)))
            }
            None => Ok(JsValue::null()),
        }
    });

    let js_recv = recv.to_js_function(instance.context.realm());
    instance.context.register_global_property(
        boa_engine::js_string!("__koss_worker_try_recv"),
        js_recv,
        boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
    ).ok();

    // __koss_worker_terminate(workerId)
    let instance_ptr5 = instance as *mut KossInstance;
    let term = NativeFunction::from_copy_closure(move |_this, args, _ctx| {
        let inst = unsafe { &mut *instance_ptr5 };
        let pool = match inst.worker_pool.as_mut() {
            Some(p) => p,
            None => return Ok(JsValue::undefined()),
        };
        let worker_id = args.get(0).and_then(|v| v.as_number()).unwrap_or(0.0) as usize;
        match pool.terminate(worker_id) {
            Ok(()) => Ok(JsValue::from(boa_engine::js_string!("ok"))),
            Err(e) => Ok(JsValue::from(boa_engine::js_string!(e))),
        }
    });

    let js_term = term.to_js_function(instance.context.realm());
    instance.context.register_global_property(
        boa_engine::js_string!("__koss_worker_terminate"),
        js_term,
        boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
    ).ok();

    // __koss_worker_shutdown()
    let instance_ptr6 = instance as *mut KossInstance;
    let shutdown = NativeFunction::from_copy_closure(move |_this, _args, _ctx| {
        let inst = unsafe { &mut *instance_ptr6 };
        if let Some(ref mut pool) = inst.worker_pool {
            pool.shutdown();
        }
        inst.worker_pool = None;
        Ok(JsValue::from(boa_engine::js_string!("ok")))
    });

    let js_shutdown = shutdown.to_js_function(instance.context.realm());
    instance.context.register_global_property(
        boa_engine::js_string!("__koss_worker_shutdown"),
        js_shutdown,
        boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
    ).ok();
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
                        eval(parsed.code);
                        globalThis.module = originalModule;
                        globalThis.exports = originalExports;
                        globalThis.Buffer = originalBuffer;
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
        caps & !(crate::sandbox::KOSS_CAP_ALL_FFI | crate::sandbox::KOSS_CAP_WORKER)
    } else {
        if caps & crate::sandbox::KOSS_CAP_ALL_FFI != 0 {
            if !FFI_STABLE_WARNED.swap(true, Ordering::Relaxed) {
                eprintln!("[KossJS WARNING] Unstable mode: FFI features are enabled.");
                eprintln!("[KossJS WARNING] FFI may have security implications in production.");
            }
        }
        if caps & crate::sandbox::KOSS_CAP_WORKER != 0 {
            if !WORKER_STABLE_WARNED.swap(true, Ordering::Relaxed) {
                eprintln!("[KossJS WARNING] Unstable mode: Worker threads are enabled.");
                eprintln!("[KossJS WARNING] Workers may have security implications in production.");
            }
        }
        caps
    };

    let mut instance = Box::new(KossInstance::new(context, effective_caps, stable, builtins));
    register_console(&mut instance.context);
    register_koss_global(&mut instance.context, stable);
    buffer::register_buffer_globals(&mut instance.context);
    register_dlopen_binding(&mut instance.context);
    register_native_bindings(&mut instance);
    register_fs_functions(&mut instance);
    register_nodejs_globals(&mut instance.context);
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
    if effective_caps & KOSS_CAP_ALL_NET != 0 {
        register_fetch_polyfill(&mut instance.context);
        register_native_fetch(&mut instance);
        register_net_functions(&mut instance);
        register_crypto_functions(&mut instance);
        register_zlib_functions_all(&mut instance);
    }
    let has_worker = effective_caps & KOSS_CAP_WORKER != 0;
    if has_worker {
        register_worker_api(&mut instance);
    } else if stable && (caps & KOSS_CAP_WORKER != 0) {
        let worker_stub_code = r#"globalThis.__koss_create_worker_pool = function() { throw new Error('Worker is disabled in stable mode'); };"#;
        if let Err(e) = instance.context.eval(boa_parser::Source::from_bytes(worker_stub_code.as_bytes())) {
            eprintln!("Warning: Failed to register Worker stub: {:?}", e);
        }
    }
    Box::into_raw(instance)
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
/// When `stable` is true, FFI and Worker capabilities are stripped from `caps`.
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

        let instance = &mut *ptr;

        let native = boa_engine::NativeFunction::from_copy_closure(move |_this, args, ctx| {
            if args.len() < 2 {
                return Ok(JsValue::undefined());
            }

            let _url = js_value_to_string(&args[0], ctx);
            let request_json = js_value_to_string(&args[1], ctx);

            let json_str = match CString::new(request_json.clone()) {
                Ok(c) => c,
                Err(_) => return Ok(JsValue::undefined()),
            };

            let result_ptr = koss_fetch(ptr as *mut KossInstance, json_str.as_ptr());

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
        let instance = &*ptr;
        instance.capabilities
    }
}

/// Returns true if the instance was created in stable mode (FFI/Worker disabled).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_is_stable(ptr: *mut KossInstance) -> bool {
    output_license_once();
    unsafe {
        if ptr.is_null() {
            return true;
        }
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
// C ABI — Worker pool management
// ===========================================================================

/// Create a worker pool with the given number of worker threads.
/// Each worker runs in its own OS thread with an isolated JS Context.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_create_worker_pool(
    ptr: *mut KossInstance,
    size: i32,
) -> KossResult {
    output_license_once();
    unsafe {
        if ptr.is_null() {
            return KossResult::err(2, "null pointer");
        }
        if size <= 0 {
            return KossResult::err(2, "worker pool size must be positive");
        }

        let instance = &mut *ptr;
        if instance.capabilities & KOSS_CAP_WORKER == 0 {
            let msg = if instance.stable {
                "Worker is disabled in stable mode. Set stable=false when creating the instance to enable Worker features."
            } else {
                "worker capability disabled"
            };
            return KossResult::err(1, msg);
        }
        instance.worker_pool = Some(WorkerPool::new((size as usize).min(MAX_WORKER_POOL_SIZE)));
        let capped = (size as usize).min(MAX_WORKER_POOL_SIZE);
        KossResult::ok(&format!("{{\"created\":{capped}}}"))
    }
}

/// Post a message to a worker thread. The message is a JSON string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_worker_post_message(
    ptr: *mut KossInstance,
    worker_id: i32,
    data: *const c_char,
) -> KossResult {
    output_license_once();
    unsafe {
        if ptr.is_null() || data.is_null() {
            return KossResult::err(2, "null pointer");
        }

        let instance = &mut *ptr;
        if instance.capabilities & KOSS_CAP_WORKER == 0 {
            let msg = if instance.stable {
                "Worker is disabled in stable mode. Set stable=false when creating the instance to enable Worker features."
            } else {
                "worker capability disabled"
            };
            return KossResult::err(1, msg);
        }
        let pool = match instance.worker_pool.as_ref() {
            Some(p) => p,
            None => return KossResult::err(1, "no worker pool created"),
        };

        let data_str = match CStr::from_ptr(data).to_str() {
            Ok(s) => s,
            Err(e) => return KossResult::err(2, &format!("invalid UTF-8: {e}")),
        };

        match pool.post_message(worker_id as usize, data_str) {
            Ok(()) => KossResult::ok("ok"),
            Err(e) => KossResult::err(1, &e),
        }
    }
}

/// Execute JavaScript code on a worker thread. Returns a command ID.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_worker_execute(
    ptr: *mut KossInstance,
    worker_id: i32,
    code: *const c_char,
) -> KossResult {
    output_license_once();
    unsafe {
        if ptr.is_null() || code.is_null() {
            return KossResult::err(2, "null pointer");
        }

        let instance = &mut *ptr;
        if instance.capabilities & KOSS_CAP_WORKER == 0 {
            let msg = if instance.stable {
                "Worker is disabled in stable mode. Set stable=false when creating the instance to enable Worker features."
            } else {
                "worker capability disabled"
            };
            return KossResult::err(1, msg);
        }
        let pool = match instance.worker_pool.as_ref() {
            Some(p) => p,
            None => return KossResult::err(1, "no worker pool created"),
        };

        let code_str = match CStr::from_ptr(code).to_str() {
            Ok(s) => s,
            Err(e) => return KossResult::err(2, &format!("invalid UTF-8: {e}")),
        };

        match pool.execute(worker_id as usize, code_str) {
            Ok(cmd_id) => KossResult::ok(&format!("{{\"commandId\":{cmd_id}}}")),
            Err(e) => KossResult::err(1, &e),
        }
    }
}

/// Try to receive a message from any worker (non-blocking).
/// Returns JSON or "null" if no message available.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_worker_try_recv(
    ptr: *mut KossInstance,
) -> KossResult {
    output_license_once();
    unsafe {
        if ptr.is_null() {
            return KossResult::err(2, "null pointer");
        }

        let instance = &mut *ptr;
        if instance.capabilities & KOSS_CAP_WORKER == 0 {
            let msg = if instance.stable {
                "Worker is disabled in stable mode. Set stable=false when creating the instance to enable Worker features."
            } else {
                "worker capability disabled"
            };
            return KossResult::err(1, msg);
        }
        let pool = match instance.worker_pool.as_ref() {
            Some(p) => p,
            None => return KossResult::err(1, "no worker pool created"),
        };

        match pool.try_recv() {
            Some(event) => {
                let json = match event {
                    WorkerEvent::Result { worker_id, id, success, value } => {
                        serde_json::json!({
                            "type": "result",
                            "workerId": worker_id,
                            "id": id,
                            "success": success,
                            "value": value,
                        })
                    }
                    WorkerEvent::Message { worker_id, data } => {
                        serde_json::json!({
                            "type": "message",
                            "workerId": worker_id,
                            "data": data,
                        })
                    }
                    WorkerEvent::Error { worker_id, message } => {
                        serde_json::json!({
                            "type": "error",
                            "workerId": worker_id,
                            "message": message,
                        })
                    }
                };
                KossResult::ok(&json.to_string())
            }
            None => KossResult::ok("null"),
        }
    }
}

/// Terminate a specific worker thread.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_worker_terminate(
    ptr: *mut KossInstance,
    worker_id: i32,
) -> KossResult {
    output_license_once();
    unsafe {
        if ptr.is_null() {
            return KossResult::err(2, "null pointer");
        }

        let instance = &mut *ptr;
        if instance.capabilities & KOSS_CAP_WORKER == 0 {
            let msg = if instance.stable {
                "Worker is disabled in stable mode. Set stable=false when creating the instance to enable Worker features."
            } else {
                "worker capability disabled"
            };
            return KossResult::err(1, msg);
        }
        let pool = match instance.worker_pool.as_mut() {
            Some(p) => p,
            None => return KossResult::err(1, "no worker pool created"),
        };

        match pool.terminate(worker_id as usize) {
            Ok(()) => KossResult::ok("ok"),
            Err(e) => KossResult::err(1, &e),
        }
    }
}

/// Shut down all worker threads and clean up the pool.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_worker_shutdown(
    ptr: *mut KossInstance,
) -> KossResult {
    output_license_once();
    unsafe {
        if ptr.is_null() {
            return KossResult::err(2, "null pointer");
        }

        let instance = &mut *ptr;
        if instance.capabilities & KOSS_CAP_WORKER == 0 {
            let msg = if instance.stable {
                "Worker is disabled in stable mode. Set stable=false when creating the instance to enable Worker features."
            } else {
                "worker capability disabled"
            };
            return KossResult::err(1, msg);
        }
        if let Some(ref mut pool) = instance.worker_pool {
            pool.shutdown();
        }
        instance.worker_pool = None;
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
        // Worker
        "worker" | "worker_threads" => KOSS_CAP_WORKER,
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
        "worker_threads" => Ok(serde_json::json!({
            "Worker": true,
            "isMainThread": true,
            "parentPort": null,
            "workerData": null,
            "getEnvironmentData": true,
            "setEnvironmentData": true,
            "SHARE_ENV": true,
            "threadId": 0,
        })
        .to_string()),
        "worker" => Ok(serde_json::json!({
            "createWorker": true,
            "postMessage": true,
            "onMessage": true,
            "terminate": true,
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

        let json_str = match CStr::from_ptr(url_json).to_str() {
            Ok(s) => s,
            Err(e) => return KossResult::err(2, &format!("invalid UTF-8: {e}")),
        };

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
fn register_dlopen_binding(ctx: &mut Context) {
    let dlopen_fn = unsafe {
        NativeFunction::from_closure(
            move |_this: &JsValue, args: &[JsValue], _ctx: &mut Context| -> Result<JsValue, JsError> {
                let module = args.first()
                    .and_then(|v| v.as_object())
                    .ok_or_else(|| JsNativeError::error().with_message("process.dlopen: module required"))?;
                let filename = args.get(1)
                    .and_then(|v| v.as_string())
                    .map(|s| s.to_std_string_escaped())
                    .ok_or_else(|| JsNativeError::error().with_message("process.dlopen: filename required"))?;

                crate::bindings::process_dlopen::dlopen_impl(&module, &filename, _ctx)?;
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
fn register_dlopen_binding(ctx: &mut Context) {
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
        let msg = cancel_error_message("worker", true);
        assert!(msg.contains("KossCancelError"));
        assert!(msg.contains("worker"));
        assert!(msg.contains("sandbox audit cancelled"));
    }

    #[test]
    fn test_cancel_error_message_debug_disabled() {
        let msg = cancel_error_message("worker", false);
        assert!(msg.contains("KossCancelError"));
        assert!(msg.contains("Access denied"));
        assert!(!msg.contains("worker"));
    }
}
