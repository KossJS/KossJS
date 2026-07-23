// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

use super::super::env::{NapiEnv, NapiValue};
use super::super::status::NapiStatus;
use super::super::value::{
    alloc_slot, as_slot, get_napi_value_type, napi_bool, napi_null, napi_undefined, NapiSlot,
    NAPI_FALSE, NAPI_TRUE,
};

pub unsafe fn napi_create_number(
    _env: *mut NapiEnv,
    value: f64,
    result: *mut NapiValue,
) -> NapiStatus {
    *result = alloc_slot(NapiSlot::Number(value));
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
    *result = napi_bool(value);
    NapiStatus::Ok
}

pub unsafe fn napi_create_null(
    _env: *mut NapiEnv,
    result: *mut NapiValue,
) -> NapiStatus {
    *result = napi_null();
    NapiStatus::Ok
}

pub unsafe fn napi_create_undefined(
    _env: *mut NapiEnv,
    result: *mut NapiValue,
) -> NapiStatus {
    *result = napi_undefined();
    NapiStatus::Ok
}

pub unsafe fn napi_get_value_int32(
    _env: *mut NapiEnv,
    value: NapiValue,
    result: *mut i32,
) -> NapiStatus {
    match unsafe { as_slot(value) } {
        Some(NapiSlot::Number(n)) => {
            *result = *n as i32;
            NapiStatus::Ok
        }
        _ => NapiStatus::NumberExpected,
    }
}

pub unsafe fn napi_get_value_int64(
    _env: *mut NapiEnv,
    value: NapiValue,
    result: *mut i64,
) -> NapiStatus {
    match unsafe { as_slot(value) } {
        Some(NapiSlot::Number(n)) => {
            *result = *n as i64;
            NapiStatus::Ok
        }
        _ => NapiStatus::NumberExpected,
    }
}

pub unsafe fn napi_get_value_double(
    _env: *mut NapiEnv,
    value: NapiValue,
    result: *mut f64,
) -> NapiStatus {
    match unsafe { as_slot(value) } {
        Some(NapiSlot::Number(n)) => {
            *result = *n;
            NapiStatus::Ok
        }
        _ => NapiStatus::NumberExpected,
    }
}

pub unsafe fn napi_get_value_bool(
    _env: *mut NapiEnv,
    value: NapiValue,
    result: *mut bool,
) -> NapiStatus {
    let addr = value as usize;
    if addr == NAPI_TRUE {
        *result = true;
        NapiStatus::Ok
    } else if addr == NAPI_FALSE {
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
    *result = get_napi_value_type(value);
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
