use std::ffi::CString;

use super::super::env::{NapiEnv, NapiValue};
use super::super::status::NapiStatus;

pub unsafe fn napi_throw(
    env: *mut NapiEnv,
    _error: NapiValue,
) -> NapiStatus {
    unsafe {
        (*env).set_error(NapiStatus::GenericFailure, "napi_throw called");
    }
    NapiStatus::PendingException
}

pub unsafe fn napi_throw_error(
    env: *mut NapiEnv,
    _code: *const u8,
    msg: *const u8,
) -> NapiStatus {
    let msg_str = if msg.is_null() {
        "Unknown N-API error".to_string()
    } else {
        let cstr = unsafe { std::ffi::CStr::from_ptr(msg as *const std::ffi::c_char) };
        cstr.to_string_lossy().to_string()
    };
    unsafe {
        (*env).set_error(NapiStatus::GenericFailure, &msg_str);
    }
    NapiStatus::PendingException
}

pub unsafe fn napi_throw_type_error(
    env: *mut NapiEnv,
    _code: *const u8,
    msg: *const u8,
) -> NapiStatus {
    napi_throw_error(env, std::ptr::null(), msg)
}

pub unsafe fn napi_create_error(
    _env: *mut NapiEnv,
    _code: NapiValue,
    msg: NapiValue,
    result: *mut NapiValue,
) -> NapiStatus {
    *result = msg;
    NapiStatus::Ok
}

pub unsafe fn napi_create_type_error(
    _env: *mut NapiEnv,
    _code: NapiValue,
    msg: NapiValue,
    result: *mut NapiValue,
) -> NapiStatus {
    *result = msg;
    NapiStatus::Ok
}

pub unsafe fn napi_is_exception_pending(
    env: *mut NapiEnv,
    result: *mut bool,
) -> NapiStatus {
    let has_error = unsafe { (*env).last_error.borrow().is_some() };
    *result = has_error;
    NapiStatus::Ok
}

pub unsafe fn napi_get_and_clear_last_exception(
    env: *mut NapiEnv,
    result: *mut NapiValue,
) -> NapiStatus {
    let err = unsafe { (*env).take_error() };
    match err {
        Some((_, msg)) => {
            let cstr = CString::new(msg).unwrap_or_default();
            *result = cstr.into_raw() as NapiValue;
            NapiStatus::Ok
        }
        None => {
            *result = std::ptr::null_mut();
            NapiStatus::Ok
        }
    }
}
