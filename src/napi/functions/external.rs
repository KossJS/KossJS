// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

use std::ffi::c_void;

use super::super::env::{NapiEnv, NapiValue};
use super::super::status::NapiStatus;
use super::super::value::{alloc_slot, as_slot, NapiSlot};

pub unsafe fn napi_create_external(
    _env: *mut NapiEnv,
    data: *mut c_void,
    _finalize_cb: Option<unsafe extern "C" fn(env: *mut NapiEnv, data: *mut c_void, hint: *mut c_void)>,
    _finalize_hint: *mut c_void,
    result: *mut NapiValue,
) -> NapiStatus {
    // Wrap the opaque pointer in a tagged slot so it is never mistaken for a
    // number/string/object value when decoded elsewhere.
    *result = alloc_slot(NapiSlot::External(data));
    NapiStatus::Ok
}

pub unsafe fn napi_get_value_external(
    _env: *mut NapiEnv,
    value: NapiValue,
    result: *mut *mut c_void,
) -> NapiStatus {
    match unsafe { as_slot(value) } {
        Some(NapiSlot::External(d)) => {
            *result = *d;
            NapiStatus::Ok
        }
        _ => {
            *result = std::ptr::null_mut();
            NapiStatus::InvalidArg
        }
    }
}
