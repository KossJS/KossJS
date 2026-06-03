use std::ffi::CString;

use super::super::env::{NapiEnv, NapiValue};
use super::super::status::NapiStatus;

pub unsafe fn napi_create_string_utf8(
    _env: *mut NapiEnv,
    str: *const u8,
    length: isize,
    result: *mut NapiValue,
) -> NapiStatus {
    if str.is_null() {
        *result = std::ptr::null_mut();
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
    if let Ok(s) = std::str::from_utf8(bytes) {
        let cstr = CString::new(s).unwrap_or_default();
        *result = cstr.into_raw() as NapiValue;
        NapiStatus::Ok
    } else {
        NapiStatus::GenericFailure
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
    if value.is_null() || (value as usize) < 4096 {
        return NapiStatus::StringExpected;
    }
    let cstr = unsafe { std::ffi::CStr::from_ptr(value as *const i8) };
    let s = cstr.to_string_lossy();
    let bytes = s.as_bytes();
    let copy_len = bytes.len().min(bufsize - 1);
    if !buf.is_null() && bufsize > 0 {
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, copy_len);
            *buf.add(copy_len) = 0;
        }
    }
    if !result.is_null() {
        *result = bytes.len();
    }
    NapiStatus::Ok
}
