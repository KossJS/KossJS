use std::ffi::c_void;

use boa_engine::{js_string, JsValue};

use super::super::env::{NapiEnv, NapiPropertyDescriptor, NapiValue, NAPI_CONFIGURABLE, NAPI_ENUMERABLE, NAPI_WRITABLE};
use super::super::status::NapiStatus;
use super::super::value::get_value_as;

pub unsafe fn napi_create_object(
    _env: *mut NapiEnv,
    result: *mut NapiValue,
) -> NapiStatus {
    let ctx = unsafe { &mut *(*_env).ctx };
    let obj = boa_engine::JsObject::with_object_proto(ctx.intrinsics());
    let boxed = Box::new(obj);
    *result = Box::into_raw(boxed) as NapiValue;
    NapiStatus::Ok
}

pub unsafe fn napi_set_property(
    env: *mut NapiEnv,
    object: NapiValue,
    key: NapiValue,
    value: NapiValue,
) -> NapiStatus {
    let obj = match get_value_as(object) {
        Some(o) => o,
        None => return NapiStatus::ObjectExpected,
    };
    let ctx = unsafe { &mut *(*env).ctx };
    let key_str = key_value_to_string(key);
    obj.insert_property(
        js_string!(key_str),
        boa_engine::property::PropertyDescriptor::builder()
            .value(value_to_js(value, ctx))
            .writable(true)
            .enumerable(true)
            .configurable(true),
    );
    NapiStatus::Ok
}

pub unsafe fn napi_get_property(
    env: *mut NapiEnv,
    object: NapiValue,
    key: NapiValue,
    result: *mut NapiValue,
) -> NapiStatus {
    let obj = match get_value_as(object) {
        Some(o) => o,
        None => return NapiStatus::ObjectExpected,
    };
    let ctx = unsafe { &mut *(*env).ctx };
    let key_str = key_value_to_string(key);
    let props = obj.borrow();
    let pk: boa_engine::property::PropertyKey = boa_engine::JsString::from(key_str.as_str()).into();
    if let Some(desc) = props.properties().get(&pk) {
        if let Some(val) = desc.value() {
            *result = js_value_to_napi_value(val, ctx);
            return NapiStatus::Ok;
        }
    }
    *result = std::ptr::null_mut();
    NapiStatus::Ok
}

pub unsafe fn napi_has_property(
    env: *mut NapiEnv,
    object: NapiValue,
    key: NapiValue,
    result: *mut bool,
) -> NapiStatus {
    let obj = match get_value_as(object) {
        Some(o) => o,
        None => return NapiStatus::ObjectExpected,
    };
    let key_str = key_value_to_string(key);
    let props = obj.borrow();
    let pk: boa_engine::property::PropertyKey = js_string!(key_str).into();
    *result = props.properties().get(&pk).is_some();
    NapiStatus::Ok
}

pub unsafe fn napi_set_named_property(
    env: *mut NapiEnv,
    object: NapiValue,
    utf8name: *const u8,
    value: NapiValue,
) -> NapiStatus {
    if utf8name.is_null() {
        return NapiStatus::InvalidArg;
    }
    let napi_key = napi_value_from_cstr(utf8name);
    napi_set_property(env, object, napi_key, value)
}

pub unsafe fn napi_get_named_property(
    env: *mut NapiEnv,
    object: NapiValue,
    utf8name: *const u8,
    result: *mut NapiValue,
) -> NapiStatus {
    if utf8name.is_null() {
        return NapiStatus::InvalidArg;
    }
    let napi_key = napi_value_from_cstr(utf8name);
    napi_get_property(env, object, napi_key, result)
}

pub unsafe fn napi_define_properties(
    env: *mut NapiEnv,
    object: NapiValue,
    property_count: usize,
    properties: *const NapiPropertyDescriptor,
) -> NapiStatus {
    let obj = match get_value_as(object) {
        Some(o) => o,
        None => return NapiStatus::ObjectExpected,
    };
    let ctx = unsafe { &mut *(*env).ctx };

    for i in 0..property_count {
        let prop = unsafe { &*properties.add(i) };
        let name = if !prop.utf8name.is_null() {
            let cstr = unsafe { std::ffi::CStr::from_ptr(prop.utf8name as *const i8) };
            cstr.to_string_lossy().to_string()
        } else {
            "".to_string()
        };

        let writable = (prop.attributes & NAPI_WRITABLE) != 0;
        let enumerable = (prop.attributes & NAPI_ENUMERABLE) != 0;
        let configurable = (prop.attributes & NAPI_CONFIGURABLE) != 0;

        let value = value_to_js(prop.value, ctx);

        obj.insert_property(
            js_string!(name),
            boa_engine::property::PropertyDescriptor::builder()
                .value(value)
                .writable(writable)
                .enumerable(enumerable)
                .configurable(configurable),
        );
    }
    NapiStatus::Ok
}

fn key_value_to_string(key: NapiValue) -> String {
    if key.is_null() {
        return String::new();
    }
    let addr = key as usize;
    if addr > 0x10000 {
        if let Ok(cstr) = unsafe { std::ffi::CStr::from_ptr(key as *const i8) }.to_str() {
            return cstr.to_string();
        }
    }
    if addr < 4096 && addr > 0 {
        let n: f64 = unsafe { *(key as *const f64) };
        return format!("{}", n as i64);
    }
    String::new()
}

fn napi_value_from_cstr(ptr: *const u8) -> NapiValue {
    let cstr = unsafe { std::ffi::CStr::from_ptr(ptr as *const i8) };
    let s = cstr.to_string_lossy().to_string();
    let cstring = std::ffi::CString::new(s).unwrap_or_default();
    cstring.into_raw() as NapiValue
}

pub fn value_to_js(val: NapiValue, _ctx: &mut boa_engine::Context) -> JsValue {
    if val.is_null() {
        return JsValue::undefined();
    }
    let addr = val as usize;
    if addr == 1 {
        return JsValue::null();
    }
    if addr == 2 {
        return JsValue::from(true);
    }
    if addr == 3 {
        return JsValue::from(false);
    }
    if addr < 4096 {
        let n: f64 = unsafe { *(val as *const f64) };
        return JsValue::from(n);
    }
    if addr > 0x10000 {
        if let Ok(cstr) = unsafe { std::ffi::CStr::from_ptr(val as *const i8) }.to_str() {
            return JsValue::from(js_string!(cstr));
        }
    }
    JsValue::undefined()
}

pub fn js_value_to_napi_value(js: &JsValue, _ctx: &mut boa_engine::Context) -> NapiValue {
    unsafe { super::super::value::js_to_napi(js, _ctx) }
}
