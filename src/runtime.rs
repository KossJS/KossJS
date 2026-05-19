use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::os::raw::c_void;
use std::rc::Rc;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use boa_engine::object::builtins::{JsFunction, JsPromise};
use boa_engine::{Context, JsError, JsNativeError, JsValue, Module, Source, NativeFunction};
use boa_runtime::Console;
use tokio::runtime::Runtime;

use crate::bindings;
use crate::module_loader::KossModuleLoader;
use crate::worker::{WorkerEvent, WorkerPool};

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
                        const key = line.substring(0, idx).trim();
                        const value = line.substring(idx + 1).trim();
                        this.set(key, value);
                    }
                }
            }
        }
    }
    
    get(name) {
        return this._headers[name.toLowerCase()] || null;
    }
    
    set(name, value) {
        this._headers[name.toLowerCase()] = value;
    }
    
    has(name) {
        return name.toLowerCase() in this._headers;
    }
    
    delete(name) {
        delete this._headers[name.toLowerCase()];
    }
    
    forEach(callback, thisArg) {
        for (const [key, value] of Object.entries(this._headers)) {
            callback.call(thisArg, value, key, this);
        }
    }
}

class Response {
    constructor(body, options = {}) {
        this._body = typeof body === 'string' ? body : (body || '');
        this.status = options.status || 200;
        this.statusText = options.statusText || 'OK';
        this.headers = options.headers instanceof Headers ? options.headers : new Headers(options.headers || {});
        this.url = options.url || '';
        this.ok = this.status >= 200 && this.status < 300;
        this.redirected = false;
        this.type = options.type || 'basic';
        this._used = false;
    }
    
    get body() {
        return this._body;
    }
    
    get bodyUsed() {
        return this._used;
    }
    
    clone() {
        if (this._used) {
            throw new Error('Body already used');
        }
        return new Response(this._body, {
            status: this.status,
            statusText: this.statusText,
            headers: new Headers(this.headers),
            url: this.url,
        });
    }
    
    text() {
        if (this._used) {
            throw new Error('Body already used');
        }
        this._used = true;
        return this._body;
    }
    
    json() {
        if (this._used) {
            throw new Error('Body already used');
        }
        this._used = true;
        return JSON.parse(this._body);
    }
}

function buildRequest(url, options) {
    options = options || {};
    return {
        url: url,
        method: options.method || 'GET',
        headers: options.headers || {},
        body: options.body,
    };
}

async function fetchSync(url, options) {
    const req = buildRequest(url, options);
    
    let responseJson;
    try {
        const promise = __koss_fetch(req.url, JSON.stringify({
            method: req.method,
            headers: req.headers,
            body: req.body,
        }));
        responseJson = await promise;
    } catch (e) {
        throw new FetchError('network error: ' + (e.message || e), 'system', e);
    }
    
    let response;
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
        url: req.url,
    });
}

globalThis.Headers = Headers;
globalThis.Response = Response;
globalThis.AbortError = AbortError;
globalThis.FetchError = FetchError;
globalThis.fetch = fetchSync;
globalThis.fetchSync = fetchSync;
"#;

// ---------------------------------------------------------------------------
// Async I/O event loop infrastructure
// ---------------------------------------------------------------------------

/// Result from an async I/O operation (sent across threads)
pub(crate) struct AsyncIoResult {
    promise_id: u64,
    result: Result<String, String>,
}

/// Resolver functions for a pending Promise (main thread only)
pub struct PendingResolver {
    pub resolve: JsFunction,
    pub reject: JsFunction,
}

/// Per-instance event loop driving async I/O and microtasks
pub struct KossEventLoop {
    pub runtime: Runtime,
    pub(crate) io_tx: mpsc::Sender<AsyncIoResult>,
    pub(crate) io_rx: mpsc::Receiver<AsyncIoResult>,
    pub next_promise_id: u64,
    pub pending: HashMap<u64, PendingResolver>,
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
        Some(KossEventLoop {
            runtime,
            io_tx,
            io_rx,
            next_promise_id: 1,
            pending: HashMap::new(),
        })
    }

    /// Process all completed async I/O operations and resolve their promises.
    /// Must be called from the main thread (where the Boa Context lives).
    pub fn process_io_results(&mut self, ctx: &mut Context) {
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

    /// Allocate a new promise ID and store the resolvers
    pub fn register_promise(&mut self, resolve: JsFunction, reject: JsFunction) -> u64 {
        let id = self.next_promise_id;
        self.next_promise_id += 1;
        self.pending.insert(id, PendingResolver { resolve, reject });
        id
    }

    /// Spawn an async task on the tokio runtime
    pub fn spawn<F>(&self, future: F)
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        self.runtime.spawn(future);
    }
}

// ---------------------------------------------------------------------------
// Opaque handle — each KossInstance is an isolated JS VM
// ---------------------------------------------------------------------------
pub struct KossInstance {
    pub context: Context,
    pub event_loop: Option<KossEventLoop>,
    pub worker_pool: Option<WorkerPool>,
    /// Optional external module loader callback (e.g. from Python).
    /// Called as a fallback when the embedded stdlib doesn't contain the module.
    pub external_module_loader: Option<NativeCallback>,
}

impl KossInstance {
    pub fn new(context: Context) -> Self {
        KossInstance {
            context,
            event_loop: KossEventLoop::new(),
            worker_pool: None,
            external_module_loader: None,
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
        loop {
            self.tick();

            match promise.state() {
                boa_engine::builtins::promise::PromiseState::Fulfilled(_) => return true,
                boa_engine::builtins::promise::PromiseState::Rejected(_) => return true,
                boa_engine::builtins::promise::PromiseState::Pending => {}
            }

            if Instant::now() >= deadline {
                return false;
            }

            std::thread::sleep(Duration::from_millis(1));
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
        let c = CString::new(val).unwrap_or_default();
        KossResult {
            code: 0,
            value: c.into_raw(),
        }
    }

    fn err(code: i32, msg: &str) -> Self {
        let c = CString::new(msg).unwrap_or_default();
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
    match err.try_native(ctx) {
        Ok(native) => native.message().to_string(),
        Err(_) => format!("{:?}", err),
    }
}

fn register_console(ctx: &mut Context) {
    let console = Console::init(ctx);
    let _ = ctx.register_global_property(
        boa_engine::js_string!("console"),
        console,
        boa_engine::property::Attribute::all(),
    );
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
        let _inst = unsafe { &*instance_ptr };
        let name_str = name.to_std_string_escaped();
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

    let native = NativeFunction::from_copy_closure(move |_this, args, context| {
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

        // 2. Fallback: try externally-registered module loader
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
                return Ok(JsValue::from(boa_engine::js_string!(result_str)));
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
        if args.len() < 2 {
            return Ok(JsValue::undefined());
        }

        let inst = unsafe { &mut *instance_ptr };
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

        let promise_id = event_loop.register_promise(resolvers.resolve.clone(), resolvers.reject.clone());
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

fn register_worker_api(instance: &mut KossInstance) {
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
    "#;

    let source = boa_parser::Source::from_bytes(primordials_code.as_bytes());
    match ctx.eval(source) {
        Ok(_) => {}
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
                boa_engine::property::Attribute::WRITABLE
                    | boa_engine::property::Attribute::CONFIGURABLE,
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
        
        const module = {
            id: normalizedPath,
            filename: normalizedPath,
            loaded: false,
            exports: {}
        };
        
        Module._cache[normalizedPath] = module;
        
        try {
            let loadedViaLoader = false;
            if (typeof __koss_load_module === 'function') {
                const result = __koss_load_module(normalizedPath);
                if (result !== null && result !== undefined) {
                    loadedViaLoader = true;
                    const parsed = JSON.parse(result);
                    if (parsed.type === 'module' && typeof parsed.code === 'string') {
                        const wrappedCode = '(function(exports, require, module, __filename, __dirname) {\n' + parsed.code + '\n})';
                        const wrapper = eval(wrappedCode);
                        wrapper(module.exports, require, module, normalizedPath, normalizedPath);
                    } else if (parsed.type === 'object') {
                        module.exports = parsed.value;
                    }
                }
            }
            if (!loadedViaLoader && _exportsCustomized) {
                module.exports = _customExports;
                _exportsCustomized = false;
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

/// Create a new isolated JS instance. Returns an opaque pointer.
/// The caller owns this pointer and must free it with `koss_destroy`.
#[unsafe(no_mangle)]
pub extern "C" fn koss_create() -> *mut KossInstance {
    let context = match boa_engine::context::ContextBuilder::default().build() {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Warning: Failed to create Boa context: {e}");
            return std::ptr::null_mut();
        }
    };
    let mut instance = Box::new(KossInstance::new(context));
    register_console(&mut instance.context);
    register_native_bindings(&mut instance);
    register_internal_module_loader(&mut instance);
    register_nodejs_globals(&mut instance.context);
    register_fetch_polyfill(&mut instance.context);
    register_native_fetch(&mut instance);
    register_worker_api(&mut instance);
    Box::into_raw(instance)
}

/// Create a new isolated JS instance with module resolution enabled.
/// `root_dir` specifies the base directory for resolving bare module specifiers.
///
/// # Safety
/// - `root_dir` must be a valid null-terminated UTF-8 string
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_create_with_modules(root_dir: *const c_char) -> *mut KossInstance {
    unsafe {
        if root_dir.is_null() {
            return koss_create();
        }

        let root_str = match CStr::from_ptr(root_dir).to_str() {
            Ok(s) => s,
            Err(_) => return koss_create(),
        };

        let loader = Rc::new(KossModuleLoader::new(root_str));
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
        let mut instance = Box::new(KossInstance::new(context));
        register_console(&mut instance.context);
        register_native_bindings(&mut instance);
        register_internal_module_loader(&mut instance);
        register_nodejs_globals(&mut instance.context);
        register_fetch_polyfill(&mut instance.context);
        register_native_fetch(&mut instance);
        register_worker_api(&mut instance);
        Box::into_raw(instance)
    }
}

/// Destroy a JS instance and free all associated memory.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_destroy(ptr: *mut KossInstance) {
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
///
/// # Safety
/// - `ptr` must be a valid pointer from `koss_create`
/// - `path` must be a valid null-terminated UTF-8 file path
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_run_file(ptr: *mut KossInstance, path: *const c_char) -> KossResult {
    unsafe {
        if ptr.is_null() || path.is_null() {
            return KossResult::err(2, "null pointer");
        }

        let instance = &mut *ptr;
        let path_str = match CStr::from_ptr(path).to_str() {
            Ok(s) => s,
            Err(e) => return KossResult::err(2, &format!("invalid UTF-8: {e}")),
        };

        let source = match Source::from_filepath(std::path::Path::new(path_str)) {
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
            loop {
                el.process_io_results(&mut instance.context);
                let _ = instance.context.run_jobs();

                let idle = el.pending.is_empty();
                if idle {
                    let _ = instance.context.run_jobs();
                    break;
                }

                if Instant::now() >= deadline {
                    return KossResult::err(1, "async execution timed out");
                }

                std::thread::sleep(Duration::from_millis(1));
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
    if !ptr.is_null() {
        unsafe {
            drop(CString::from_raw(ptr));
        }
    }
}

/// Free a KossResult struct and its associated value string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_free_result(result: KossResult) {
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
    static VERSION: &[u8] = b"0.1.0-dev.5.fix\0";
    VERSION.as_ptr() as *const c_char
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
    unsafe {
        if ptr.is_null() {
            return KossResult::err(2, "null pointer");
        }
        if size <= 0 {
            return KossResult::err(2, "worker pool size must be positive");
        }

        let instance = &mut *ptr;
        instance.worker_pool = Some(WorkerPool::new(size as usize));
        KossResult::ok(&format!("{{\"created\":{size}}}"))
    }
}

/// Post a message to a worker thread. The message is a JSON string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_worker_post_message(
    ptr: *mut KossInstance,
    worker_id: i32,
    data: *const c_char,
) -> KossResult {
    unsafe {
        if ptr.is_null() || data.is_null() {
            return KossResult::err(2, "null pointer");
        }

        let instance = &mut *ptr;
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
    unsafe {
        if ptr.is_null() || code.is_null() {
            return KossResult::err(2, "null pointer");
        }

        let instance = &mut *ptr;
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
    unsafe {
        if ptr.is_null() {
            return KossResult::err(2, "null pointer");
        }

        let instance = &mut *ptr;
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
    unsafe {
        if ptr.is_null() {
            return KossResult::err(2, "null pointer");
        }

        let instance = &mut *ptr;
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
    unsafe {
        if ptr.is_null() {
            return KossResult::err(2, "null pointer");
        }

        let instance = &mut *ptr;
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
    unsafe {
        if ptr.is_null() || binding_name.is_null() {
            return KossResult::err(2, "null pointer");
        }

        let _instance = &mut *ptr;
        let name_str = match CStr::from_ptr(binding_name).to_str() {
            Ok(s) => s,
            Err(e) => return KossResult::err(2, &format!("invalid UTF-8: {e}")),
        };

        let result = handle_binding(name_str);
        match result {
            Ok(json) => KossResult::ok(&json),
            Err(e) => KossResult::err(1, &e),
        }
    }
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
/// Uses JS JSON.parse internally for safe parsing.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_set_global_json(
    ptr: *mut KossInstance,
    name: *const c_char,
    json_str: *const c_char,
) -> KossResult {
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

        // Validate JSON with serde_json
        if let Err(e) = serde_json::from_str::<serde_json::Value>(json) {
            return KossResult::err(1, &format!("invalid JSON: {e}"));
        }

        // Wrap in parentheses to handle objects correctly (avoid block parsing)
        let wrapped = format!("({})", json);
        let source = Source::from_bytes(wrapped.as_bytes());
        match instance.context.eval(source) {
            Ok(val) => {
                let js_key = boa_engine::js_string!(name_str);
                let _ = instance.context.register_global_property(
                    js_key,
                    val,
                    boa_engine::property::Attribute::WRITABLE
                        | boa_engine::property::Attribute::CONFIGURABLE,
                );
                KossResult::ok("ok")
            }
            Err(e) => KossResult::err(1, &format!("JSON eval error: {e}")),
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

/// Set a nested property path using JS eval.
/// Creates intermediate objects as needed (e.g., "Math.max" -> globalThis.Math.max = value)
fn set_nested_property(ctx: &mut Context, path: &str, value: boa_engine::JsValue) {
    // Register with a temp name first
    let temp_key = format!("__koss_tmp_{}", path.replace('.', "_"));
    let _ = ctx.register_global_property(
        boa_engine::js_string!(temp_key.as_str()),
        value,
        boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
    );

    // Use JS eval to nest it into the correct location and clean up temp
    let eval_code = format!(
        r#"
(function() {{
    var parts = '{}'.split('.');
    var obj = globalThis;
    for (var i = 0; i < parts.length - 1; i++) {{
        if (typeof obj[parts[i]] !== 'object' || obj[parts[i]] === null) {{
            obj[parts[i]] = {{}};
        }}
        obj = obj[parts[i]];
    }}
    obj[parts[parts.length - 1]] = globalThis.{};  // Copy from temp
    delete globalThis.{};                           // Clean up temp
}})();
"#,
        path, temp_key, temp_key
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
    callback: *mut c_void,
) -> KossResult {
    unsafe {
        if ptr.is_null() || name.is_null() || callback.is_null() {
            return KossResult::err(2, "null pointer");
        }

        let instance = &mut *ptr;
        let name_str = match CStr::from_ptr(name).to_str() {
            Ok(s) => s,
            Err(e) => return KossResult::err(2, &format!("invalid UTF-8: {e}")),
        };

        let cb: NativeCallback = std::mem::transmute(callback);

        if name_str.contains('.') {
            let js_func = register_native_function(&mut instance.context, cb);
            set_nested_property(&mut instance.context, name_str, js_func);
        } else {
            let js_func = register_native_function(&mut instance.context, cb);
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
    callback: *mut c_void,
) -> KossResult {
    unsafe {
        if ptr.is_null() {
            return KossResult::err(2, "null pointer");
        }

        let instance = &mut *ptr;
        if callback.is_null() {
            instance.external_module_loader = None;
            return KossResult::ok("external loader cleared");
        }

        let cb: NativeCallback = std::mem::transmute(callback);
        instance.external_module_loader = Some(cb);

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
    callback: *mut c_void,
) -> KossResult {
    unsafe {
        if ptr.is_null() || class_name.is_null() || methods_json.is_null() || callback.is_null() {
            return KossResult::err(2, "null pointer");
        }

        let instance = &mut *ptr;
        let name_str = match CStr::from_ptr(class_name).to_str() {
            Ok(s) => s,
            Err(e) => return KossResult::err(2, &format!("invalid UTF-8: {e}")),
        };
        let methods_str = match CStr::from_ptr(methods_json).to_str() {
            Ok(s) => s,
            Err(e) => return KossResult::err(2, &format!("invalid UTF-8: {e}")),
        };

        // Validate methods JSON
        let method_names: Vec<String> = match serde_json::from_str(methods_str) {
            Ok(v) => v,
            Err(e) => return KossResult::err(2, &format!("invalid methods JSON: {e}")),
        };

        let cb: NativeCallback = std::mem::transmute(callback);

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
            let result = cb(argc + 1, ptrs.as_ptr() as *mut c_void);

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
