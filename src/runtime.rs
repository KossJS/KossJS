use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use boa_engine::{Context, JsError, JsValue, Source};
use boa_runtime::Console;

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
    ctx.register_global_property(
        boa_engine::js_string!("console"),
        console,
        boa_engine::property::Attribute::all(),
    );
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
    let instance = Box::new(KossInstance { context });
    Box::into_raw(instance)
}

/// Destroy a JS instance and free all associated memory.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_destroy(ptr: *mut KossInstance) {
    if !ptr.is_null() {
        drop(Box::from_raw(ptr));
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

/// Execute a JavaScript file. Returns the result of the last expression.
///
/// # Safety
/// - `ptr` must be a valid pointer from `koss_create`
/// - `path` must be a valid null-terminated UTF-8 file path
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_run_file(ptr: *mut KossInstance, path: *const c_char) -> KossResult {
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

    instance.context.register_global_property(
        js_key,
        js_val,
        boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
    );

    KossResult::ok("ok")
}

/// Set a global number (f64) variable in the JS context.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_set_global_number(
    ptr: *mut KossInstance,
    name: *const c_char,
    value: f64,
) -> KossResult {
    if ptr.is_null() || name.is_null() {
        return KossResult::err(2, "null pointer");
    }

    let instance = &mut *ptr;
    let name_str = match CStr::from_ptr(name).to_str() {
        Ok(s) => s,
        Err(e) => return KossResult::err(2, &format!("invalid UTF-8: {e}")),
    };

    let js_key = boa_engine::js_string!(name_str);
    let js_val = boa_engine::JsValue::from(value);

    instance.context.register_global_property(
        js_key,
        js_val,
        boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
    );

    KossResult::ok("ok")
}

/// Set a global boolean variable in the JS context.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_set_global_bool(
    ptr: *mut KossInstance,
    name: *const c_char,
    value: bool,
) -> KossResult {
    if ptr.is_null() || name.is_null() {
        return KossResult::err(2, "null pointer");
    }

    let instance = &mut *ptr;
    let name_str = match CStr::from_ptr(name).to_str() {
        Ok(s) => s,
        Err(e) => return KossResult::err(2, &format!("invalid UTF-8: {e}")),
    };

    let js_key = boa_engine::js_string!(name_str);
    let js_val = boa_engine::JsValue::from(value);

    instance.context.register_global_property(
        js_key,
        js_val,
        boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
    );

    KossResult::ok("ok")
}

// ===========================================================================
// C ABI — Native function registration (host callable from JS)
// ===========================================================================

/// Callback type: receives (argc, argv) where argv is an array of C strings.
/// Must return a heap-allocated C string (will be freed by KossJS).
/// Return null for undefined.
pub type KossNativeFn = unsafe extern "C" fn(argc: i32, argv: *const *const c_char) -> *mut c_char;

/// Register a native (host) function that can be called from JavaScript.
///
/// ```js
/// // After registering "myFunc" from C/Java/Python:
/// const result = myFunc("hello", 42);
/// ```
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_register_function(
    ptr: *mut KossInstance,
    name: *const c_char,
    func: KossNativeFn,
) -> KossResult {
    if ptr.is_null() || name.is_null() {
        return KossResult::err(2, "null pointer");
    }

    let instance = &mut *ptr;
    let name_str = match CStr::from_ptr(name).to_str() {
        Ok(s) => s.to_string(),
        Err(e) => return KossResult::err(2, &format!("invalid UTF-8: {e}")),
    };

    let func_name = name_str.clone();

    // Wrap the C function pointer into a Boa NativeFunction
    let native = boa_engine::NativeFunction::from_copy_closure(move |_this, args, ctx| {
        // Convert JS args → C strings
        let mut c_strings: Vec<CString> = Vec::with_capacity(args.len());
        let mut c_ptrs: Vec<*const c_char> = Vec::with_capacity(args.len());

        for arg in args {
            let s = js_value_to_string(arg, ctx);
            let cs = CString::new(s).unwrap_or_default();
            c_ptrs.push(cs.as_ptr());
            c_strings.push(cs);
        }

        let result_ptr = func(c_ptrs.len() as i32, c_ptrs.as_ptr());

        if result_ptr.is_null() {
            Ok(JsValue::undefined())
        } else {
            let result_str = CStr::from_ptr(result_ptr)
                .to_str()
                .unwrap_or("")
                .to_string();
            let js_str = boa_engine::JsString::from(result_str.as_str());
            Ok(JsValue::from(js_str))
        }
    });

    let js_func = native.to_js_function(instance.context.realm());
    let js_key = boa_engine::js_string!(&*name_str);

    instance.context.register_global_property(
        js_key,
        js_func,
        boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE,
    );

    KossResult::ok("ok")
}

// ===========================================================================
// C ABI — Memory management
// ===========================================================================

/// Free a string returned by KossJS (from KossResult.value).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_free_string(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}

/// Free a KossResult (frees the inner string).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_free_result(result: KossResult) {
    if !result.value.is_null() {
        drop(CString::from_raw(result.value));
    }
}

// ===========================================================================
// C ABI — Builtins
// ===========================================================================

#[unsafe(no_mangle)]
pub extern "C" fn koss_register_builtin(
    ptr: *mut KossInstance,
    builtin_name: *const c_char,
    func: KossNativeFn,
) -> KossResult {
    if ptr.is_null() || builtin_name.is_null() {
        return KossResult::err(2, "null pointer");
    }

    let instance = unsafe { &mut *ptr };

    let name_str = unsafe {
        match CStr::from_ptr(builtin_name).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return KossResult::err(2, "invalid UTF-8"),
        }
    };

    let func_name = name_str.clone();

    let native = boa_engine::NativeFunction::from_copy_closure(move |_this, args, ctx| {
        let mut c_strings: Vec<CString> = Vec::with_capacity(args.len());
        let mut c_ptrs: Vec<*const c_char> = Vec::with_capacity(args.len());

        for arg in args {
            let s = js_value_to_string(arg, ctx);
            let cs = CString::new(s).unwrap_or_default();
            c_ptrs.push(cs.as_ptr());
            c_strings.push(cs);
        }

        let result_ptr = unsafe { func(c_ptrs.len() as i32, c_ptrs.as_ptr()) };

        if result_ptr.is_null() {
            Ok(JsValue::undefined())
        } else {
            let result_str = unsafe {
                CStr::from_ptr(result_ptr)
                    .to_str()
                    .unwrap_or("")
                    .to_string()
            };
            let js_str = boa_engine::JsString::from(result_str.as_str());
            Ok(JsValue::from(js_str))
        }
    });

    let js_func = native.to_js_function(instance.context.realm());

    instance
        .context
        .register_global_property(
            boa_engine::js_string!(&*name_str),
            js_func,
            boa_engine::property::Attribute::WRITABLE
                | boa_engine::property::Attribute::CONFIGURABLE,
        )
        .ok();

    KossResult::ok("ok")
}

// ===========================================================================
// C ABI — Version info
// ===========================================================================

/// Returns the KossJS version string.
#[unsafe(no_mangle)]
pub extern "C" fn koss_version() -> *const c_char {
    static VERSION: &[u8] = b"0.1.0\0";
    VERSION.as_ptr() as *const c_char
}
