use std::ffi::{c_void, CString};
use std::os::raw::c_char;

use boa_engine::{js_string, Context, JsError, JsNativeError, JsObject, JsValue};

use super::env::{NapiEnv, NapiValue};
use super::status::NapiStatus;

pub unsafe fn js_to_napi(js: &JsValue, ctx: &mut Context) -> NapiValue {
    if js.is_undefined() {
        return std::ptr::null_mut::<c_void>();
    }
    if js.is_null() {
        return 1usize as NapiValue;
    }
    if let Some(b) = js.as_boolean() {
        return if b { 2usize as NapiValue } else { 3usize as NapiValue };
    }
    if let Some(n) = js.as_number() {
        let boxed = Box::new(n);
        return Box::into_raw(boxed) as NapiValue;
    }
    if let Some(s) = js.as_string() {
        let s = s.to_std_string_escaped();
        let cstr = CString::new(s).unwrap_or_default();
        return cstr.into_raw() as NapiValue;
    }
    js.as_object().map(|obj| {
        Box::into_raw(Box::new(obj.clone())) as NapiValue
    }).unwrap_or(std::ptr::null_mut::<c_void>())
}

pub fn get_napi_value_type(val: NapiValue) -> i32 {
    if val.is_null() {
        0 // napi_undefined
    } else {
        let addr = val as usize;
        if addr == 1 {
            1 // napi_null
        } else if addr == 2 || addr == 3 {
            4 // napi_boolean
        } else if addr < 4096 {
            5 // napi_number (boxed)
        } else {
            6 // napi_object / napi_function / napi_string / napi_external
        }
    }
}

pub unsafe fn napi_to_js(val: NapiValue, env: *mut NapiEnv, _ctx: &mut Context) -> Result<JsValue, JsError> {
    if val.is_null() {
        return Ok(JsValue::undefined());
    }
    let addr = val as usize;
    if addr == 1 {
        return Ok(JsValue::null());
    }
    if addr == 2 {
        return Ok(JsValue::from(true));
    }
    if addr == 3 {
        return Ok(JsValue::from(false));
    }
    if addr < 4096 {
        let n: f64 = unsafe { *(val as *const f64) };
        return Ok(JsValue::from(n));
    }
    if addr > 0x10000 {
        let cstr = unsafe { std::ffi::CStr::from_ptr(val as *const c_char) };
        if let Ok(s) = cstr.to_str() {
            return Ok(JsValue::from(js_string!(s)));
        }
    }
    Ok(JsValue::undefined())
}

pub unsafe fn get_value_as(obj: NapiValue) -> Option<boa_engine::JsObject> {
    if obj.is_null() || (obj as usize) < 4096 {
        return None;
    }
    unsafe {
        let ptr = obj as *const boa_engine::JsObject;
        Some((*ptr).clone())
    }
}
