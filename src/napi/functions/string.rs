// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

use std::ffi::CString;

use super::super::env::{NapiEnv, NapiValue};
use super::super::status::NapiStatus;
use super::super::value::{alloc_slot, as_slot, napi_undefined, NapiSlot};

pub unsafe fn napi_create_string_utf8(
    _env: *mut NapiEnv,
    str: *const u8,
    length: isize,
    result: *mut NapiValue,
) -> NapiStatus {
    if str.is_null() {
        *result = napi_undefined();
        return NapiStatus::InvalidArg;
    }
    let len = if length < 0 {
        unsafe {
            let mut end = str;
            while *end != 0 {
                end = end.add(1);
            }
            end.offset_from(str) as isize
        }
    } else {
        length
    };
    let bytes = unsafe { std::slice::from_raw_parts(str, len as usize) };
    match std::str::from_utf8(bytes) {
        Ok(s) => {
            let cstr = CString::new(s)
                .unwrap_or_else(|_| CString::new(s.replace('\0', "")).unwrap_or_default());
            *result = alloc_slot(NapiSlot::Str(cstr));
            NapiStatus::Ok
        }
        Err(_) => NapiStatus::GenericFailure,
    }
}

pub unsafe fn napi_create_string_latin1(
    _env: *mut NapiEnv,
    str: *const u8,
    length: isize,
    result: *mut NapiValue,
) -> NapiStatus {
    napi_create_string_utf8(_env, str, length, result)
}

pub unsafe fn napi_get_value_string_utf8(
    _env: *mut NapiEnv,
    value: NapiValue,
    buf: *mut u8,
    bufsize: usize,
    result: *mut usize,
) -> NapiStatus {
    let cstr = match unsafe { as_slot(value) } {
        Some(NapiSlot::Str(c)) => c,
        _ => return NapiStatus::StringExpected,
    };
    let bytes = cstr.as_bytes();

    // When no buffer is provided (or size 0) the caller only wants the required
    // length. Compute it without the `bufsize - 1` subtraction, which would
    // underflow (panic in debug) when bufsize == 0.
    if buf.is_null() || bufsize == 0 {
        if !result.is_null() {
            *result = bytes.len();
        }
        return NapiStatus::Ok;
    }

    let copy_len = bytes.len().min(bufsize - 1);
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, copy_len);
        *buf.add(copy_len) = 0;
    }
    if !result.is_null() {
        // Number of bytes written excluding the trailing NUL.
        *result = copy_len;
    }
    NapiStatus::Ok
}
