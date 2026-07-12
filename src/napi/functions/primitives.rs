// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

use std::ffi::c_void;

use boa_engine::{js_string, Context, JsObject, JsValue};

use super::super::env::{NapiEnv, NapiValue};
use super::super::status::NapiStatus;

pub unsafe fn napi_create_number(
    _env: *mut NapiEnv,
    value: f64,
    result: *mut NapiValue,
) -> NapiStatus {
    let boxed = Box::new(value);
    *result = Box::into_raw(boxed) as NapiValue;
    NapiStatus::Ok
}

pub unsafe fn napi_create_int32(
    _env: *mut NapiEnv,
    value: i32,
    result: *mut NapiValue,
) -> NapiStatus {
    napi_create_number(_env, value as f64, result)
}

pub unsafe fn napi_create_uint32(
    _env: *mut NapiEnv,
    value: u32,
    result: *mut NapiValue,
) -> NapiStatus {
    napi_create_number(_env, value as f64, result)
}

pub unsafe fn napi_create_int64(
    _env: *mut NapiEnv,
    value: i64,
    result: *mut NapiValue,
) -> NapiStatus {
    napi_create_number(_env, value as f64, result)
}

pub unsafe fn napi_create_double(
    _env: *mut NapiEnv,
    value: f64,
    result: *mut NapiValue,
) -> NapiStatus {
    napi_create_number(_env, value, result)
}

pub unsafe fn napi_create_bool(
    _env: *mut NapiEnv,
    value: bool,
    result: *mut NapiValue,
) -> NapiStatus {
    *result = if value { 2usize as NapiValue } else { 3usize as NapiValue };
    NapiStatus::Ok
}

pub unsafe fn napi_create_null(
    _env: *mut NapiEnv,
    result: *mut NapiValue,
) -> NapiStatus {
    *result = 1usize as NapiValue;
    NapiStatus::Ok
}

pub unsafe fn napi_create_undefined(
    _env: *mut NapiEnv,
    result: *mut NapiValue,
) -> NapiStatus {
    *result = std::ptr::null_mut::<c_void>();
    NapiStatus::Ok
}

pub unsafe fn napi_get_value_int32(
    _env: *mut NapiEnv,
    value: NapiValue,
    result: *mut i32,
) -> NapiStatus {
    if value.is_null() || (value as usize) < 4096 {
        return NapiStatus::NumberExpected;
    }
    let n: f64 = unsafe { *(value as *const f64) };
    *result = n as i32;
    NapiStatus::Ok
}

pub unsafe fn napi_get_value_int64(
    _env: *mut NapiEnv,
    value: NapiValue,
    result: *mut i64,
) -> NapiStatus {
    if value.is_null() || (value as usize) < 4096 {
        return NapiStatus::NumberExpected;
    }
    let n: f64 = unsafe { *(value as *const f64) };
    *result = n as i64;
    NapiStatus::Ok
}

pub unsafe fn napi_get_value_double(
    _env: *mut NapiEnv,
    value: NapiValue,
    result: *mut f64,
) -> NapiStatus {
    if value.is_null() || (value as usize) < 4096 {
        return NapiStatus::NumberExpected;
    }
    *result = unsafe { *(value as *const f64) };
    NapiStatus::Ok
}

pub unsafe fn napi_get_value_bool(
    _env: *mut NapiEnv,
    value: NapiValue,
    result: *mut bool,
) -> NapiStatus {
    let addr = value as usize;
    if addr == 2 {
        *result = true;
        NapiStatus::Ok
    } else if addr == 3 {
        *result = false;
        NapiStatus::Ok
    } else {
        NapiStatus::BooleanExpected
    }
}

pub unsafe fn napi_typeof(
    _env: *mut NapiEnv,
    value: NapiValue,
    result: *mut i32,
) -> NapiStatus {
    let addr = value as usize;
    *result = if value.is_null() {
        0 // undefined
    } else if addr == 1 {
        1 // null
    } else if addr == 2 || addr == 3 {
        4 // boolean
    } else if addr < 4096 {
        5 // number
    } else {
        6 // object (catch-all for string/function/external)
    };
    NapiStatus::Ok
}

pub unsafe fn napi_strict_equals(
    _env: *mut NapiEnv,
    lhs: NapiValue,
    rhs: NapiValue,
    result: *mut bool,
) -> NapiStatus {
    *result = lhs == rhs;
    NapiStatus::Ok
}

pub unsafe fn napi_get_boolean(
    _env: *mut NapiEnv,
    value: bool,
    result: *mut NapiValue,
) -> NapiStatus {
    napi_create_bool(_env, value, result)
}

pub unsafe fn napi_get_null(
    _env: *mut NapiEnv,
    result: *mut NapiValue,
) -> NapiStatus {
    napi_create_null(_env, result)
}

pub unsafe fn napi_get_undefined(
    _env: *mut NapiEnv,
    result: *mut NapiValue,
) -> NapiStatus {
    napi_create_undefined(_env, result)
}
