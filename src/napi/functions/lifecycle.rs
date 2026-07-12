// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

use std::ffi::c_void;

use super::super::env::{NapiEnv, NapiValue};
use super::super::status::NapiStatus;

pub unsafe fn napi_add_finalizer(
    _env: *mut NapiEnv,
    _js_object: NapiValue,
    _native_object: *mut c_void,
    _finalize_cb: Option<unsafe extern "C" fn(env: *mut NapiEnv, data: *mut c_void, hint: *mut c_void)>,
    _finalize_hint: *mut c_void,
    _result: *mut NapiValue,
) -> NapiStatus {
    NapiStatus::Ok
}

pub unsafe fn napi_open_handle_scope(
    env: *mut NapiEnv,
    _result: *mut NapiValue,
) -> NapiStatus {
    unsafe {
        let mut count = (*env).open_handle_scopes.borrow_mut();
        *count += 1;
    }
    NapiStatus::Ok
}

pub unsafe fn napi_close_handle_scope(
    env: *mut NapiEnv,
    _scope: NapiValue,
) -> NapiStatus {
    unsafe {
        let mut count = (*env).open_handle_scopes.borrow_mut();
        if *count > 0 {
            *count -= 1;
        } else {
            return NapiStatus::HandleScopeMismatch;
        }
    }
    NapiStatus::Ok
}

pub unsafe fn napi_open_callback_scope(
    env: *mut NapiEnv,
    _resource_object: NapiValue,
    _context: NapiValue,
    _result: *mut NapiValue,
) -> NapiStatus {
    unsafe {
        let mut count = (*env).open_callback_scopes.borrow_mut();
        *count += 1;
    }
    NapiStatus::Ok
}

pub unsafe fn napi_close_callback_scope(
    env: *mut NapiEnv,
    _scope: NapiValue,
) -> NapiStatus {
    unsafe {
        let mut count = (*env).open_callback_scopes.borrow_mut();
        if *count > 0 {
            *count -= 1;
        } else {
            return NapiStatus::CallbackScopeMismatch;
        }
    }
    NapiStatus::Ok
}
