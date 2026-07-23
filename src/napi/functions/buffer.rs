// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

use std::ffi::c_void;

use super::super::env::{NapiEnv, NapiValue};
use super::super::status::NapiStatus;
use super::super::value::{alloc_slot, as_slot, NapiSlot};

pub unsafe fn napi_create_buffer(
    _env: *mut NapiEnv,
    length: usize,
    data: *mut *mut c_void,
    result: *mut NapiValue,
) -> NapiStatus {
    let mut vec = vec![0u8; length];
    let ptr = vec.as_mut_ptr();
    if !data.is_null() {
        *data = ptr as *mut c_void;
    }
    // The Vec is owned by the slot; moving the Vec does not move its heap
    // allocation, so `ptr` (and the length) stay valid for the slot's lifetime.
    *result = alloc_slot(NapiSlot::Buffer(vec));
    NapiStatus::Ok
}

pub unsafe fn napi_create_buffer_copy(
    _env: *mut NapiEnv,
    length: usize,
    data: *const c_void,
    result_data: *mut *mut c_void,
    result: *mut NapiValue,
) -> NapiStatus {
    let mut vec = vec![0u8; length];
    if !data.is_null() {
        unsafe {
            std::ptr::copy_nonoverlapping(data as *const u8, vec.as_mut_ptr(), length);
        }
    }
    let ptr = vec.as_mut_ptr();
    if !result_data.is_null() {
        *result_data = ptr as *mut c_void;
    }
    *result = alloc_slot(NapiSlot::Buffer(vec));
    NapiStatus::Ok
}

pub unsafe fn napi_get_buffer_info(
    _env: *mut NapiEnv,
    value: NapiValue,
    data: *mut *mut c_void,
    length: *mut usize,
) -> NapiStatus {
    match unsafe { as_slot(value) } {
        Some(NapiSlot::Buffer(v)) => {
            if !data.is_null() {
                *data = v.as_ptr() as *mut c_void;
            }
            if !length.is_null() {
                *length = v.len();
            }
            NapiStatus::Ok
        }
        _ => NapiStatus::InvalidArg,
    }
}
