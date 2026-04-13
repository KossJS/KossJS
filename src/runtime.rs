use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::rc::Rc;

use boa_engine::{Context, JsError, JsValue, Module, Source};
use boa_runtime::Console;

use crate::bindings;
use crate::module_loader::KossModuleLoader;

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

function fetchSync(url, options) {
    const req = buildRequest(url, options);
    
    let responseJson;
    try {
        responseJson = __koss_fetch(req.url, JSON.stringify({
            method: req.method,
            headers: req.headers,
            body: req.body,
        }));
    } catch (e) {
        throw new FetchError('network error', 'system', e);
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
// Opaque handle — each KossInstance is an isolated JS VM
// ---------------------------------------------------------------------------
pub struct KossInstance {
    context: Context,
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

fn register_native_fetch(ctx: &mut Context) {
    use boa_engine::NativeFunction;

    let native = NativeFunction::from_copy_closure(move |_this, args, ctx| {
        if args.len() < 2 {
            return Ok(JsValue::undefined());
        }

        let url = js_value_to_string(&args[0], ctx);
        let request_json = js_value_to_string(&args[1], ctx);

        let json_str = match CString::new(request_json) {
            Ok(c) => c,
            Err(_) => return Ok(JsValue::undefined()),
        };

        let result = bindings::fetch::fetch_with_url(&url, json_str.to_str().unwrap_or(""));

        match result {
            Ok(response) => {
                let json = serde_json::to_string(&response).unwrap_or_default();
                Ok(JsValue::from(boa_engine::JsString::from(json)))
            }
            Err(_) => Ok(JsValue::undefined()),
        }
    });

    let js_func = native.to_js_function(ctx.realm());

    ctx.register_global_property(
        boa_engine::js_string!("__koss_fetch"),
        js_func,
        boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
    )
    .ok();
}

fn register_nodejs_globals(ctx: &mut Context) {
    // Register primordials
    let primordials_code = r#"
    const primordials = {
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
    };
    
    // internalBinding - calls Rust implementations via __koss_bindings
    const internalBinding = function(name) {
        return __koss_bindings(name);
    };
    globalThis.internalBinding = internalBinding;
    "#;

    let source = boa_parser::Source::from_bytes(primordials_code.as_bytes());
    match ctx.eval(source) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Warning: Failed to register primordials: {:?}", e);
        }
    }

    // Register process as a minimal stub
    let process_code = r#"
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
    
    let currentModule = { id: '<root>', filename: '<root>', loaded: false };
    
    function require(path) {
        const builtinModules = [
            'assert', 'buffer', 'child_process', 'cluster', 'console', 'constants',
            'crypto', 'dgram', 'dns', 'domain', 'events', 'fs', 'http', 'https',
            'module', 'net', 'os', 'path', 'process', 'punycode', 'querystring',
            'readline', 'repl', 'stream', 'string_decoder', 'sys', 'timers',
            'tls', 'trace_events', 'tty', 'url', 'util', 'v8', 'vm', 'wasi',
            'worker_threads', 'zlib', 'async_hooks', 'diagnostics_channel',
            'perf_hooks', 'http2', 'http3', 'sqlite', 'test', 'wasi', 'sea',
            'events', 'url', 'string_decoder', 'stream', 'http', 'https',
            'net', 'tls', 'http2', 'zlib', 'assert', 'buffer', 'console'
        ];
        
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
            if (typeof __koss_load_module === 'function') {
                const result = __koss_load_module(normalizedPath);
                module.exports = result;
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
    let mut context = Context::default();
    register_console(&mut context);
    register_nodejs_globals(&mut context);
    register_fetch_polyfill(&mut context);
    register_native_fetch(&mut context);
    let instance = Box::new(KossInstance { context });
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
        let mut context = boa_engine::context::ContextBuilder::default()
            .module_loader(loader)
            .build()
            .unwrap_or_default();
        register_console(&mut context);
        register_nodejs_globals(&mut context);
        register_fetch_polyfill(&mut context);
        register_native_fetch(&mut context);
        let instance = Box::new(KossInstance { context });
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
    static VERSION: &[u8] = b"0.1.0-dev.2\0";
    VERSION.as_ptr() as *const c_char
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
