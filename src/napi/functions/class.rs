use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_void;

use super::super::env::{NapiCallback, NapiEnv, NapiPropertyDescriptor, NapiValue};
use super::super::status::NapiStatus;
use super::super::value::get_value_as;
use super::function::napi_create_function;

thread_local! {
    static CLASS_REGISTRY: RefCell<HashMap<usize, Vec<(String, NapiCallback, u32)>>> = RefCell::new(HashMap::new());
}

pub unsafe fn napi_define_class(
    env: *mut NapiEnv,
    utf8name: *const u8,
    _length: isize,
    constructor: NapiCallback,
    data: *mut c_void,
    property_count: usize,
    properties: *const NapiPropertyDescriptor,
    result: *mut NapiValue,
) -> NapiStatus {
    let _ctx = unsafe { &mut *(*env).ctx };
    let class_id: usize = unsafe { std::mem::transmute(constructor) };

    let mut methods: Vec<(String, NapiCallback, u32)> = Vec::new();
    for i in 0..property_count {
        let prop = unsafe { &*properties.add(i) };
        let name = if !prop.utf8name.is_null() {
            let cstr = unsafe { std::ffi::CStr::from_ptr(prop.utf8name as *const i8) };
            cstr.to_string_lossy().to_string()
        } else {
            continue;
        };
        let cb = if let Some(m) = prop.method { m }
                 else if let Some(g) = prop.getter { g }
                 else { continue };
        methods.push((name, cb, prop.attributes));
    }
    CLASS_REGISTRY.with(|reg| {
        reg.borrow_mut().insert(class_id, methods);
    });

    let obj = boa_engine::JsObject::with_object_proto(_ctx.intrinsics());
    let boxed = Box::new(obj);
    *result = Box::into_raw(boxed) as NapiValue;
    NapiStatus::Ok
}

pub unsafe fn napi_wrap(
    _env: *mut NapiEnv,
    js_object: NapiValue,
    native_object: *mut c_void,
    _finalize_cb: Option<unsafe extern "C" fn(napi_env: *mut NapiEnv, data: *mut c_void, hint: *mut c_void)>,
    _finalize_hint: *mut c_void,
    _result: *mut NapiValue,
) -> NapiStatus {
    let obj = match get_value_as(js_object) {
        Some(o) => o,
        None => return NapiStatus::ObjectExpected,
    };
    let addr = native_object as usize;
    obj.insert_property(
        boa_engine::js_string!("__napi_wrapped__"),
        boa_engine::property::PropertyDescriptor::builder()
            .value(boa_engine::JsValue::from(addr as f64))
            .writable(false)
            .enumerable(false)
            .configurable(false),
    );
    NapiStatus::Ok
}

pub unsafe fn napi_unwrap(
    _env: *mut NapiEnv,
    js_object: NapiValue,
    result: *mut *mut c_void,
) -> NapiStatus {
    let obj = match get_value_as(js_object) {
        Some(o) => o,
        None => return NapiStatus::ObjectExpected,
    };
    let props = obj.borrow();
    let pk: boa_engine::property::PropertyKey = boa_engine::js_string!("__napi_wrapped__").into();
    let data = if let Some(desc) = props.properties().get(&pk) {
        desc.value().and_then(|v| v.as_number()).map(|n| n as usize as *mut c_void).unwrap_or(std::ptr::null_mut())
    } else {
        std::ptr::null_mut()
    };
    drop(props);
    *result = data;
    NapiStatus::Ok
}

pub unsafe fn napi_instanceof(
    _env: *mut NapiEnv,
    _object: NapiValue,
    _constructor: NapiValue,
    result: *mut bool,
) -> NapiStatus {
    *result = false;
    NapiStatus::Ok
}
