use std::ffi::c_void;

use boa_engine::{Context, JsObject, JsValue};

use super::super::env::{NapiEnv, NapiValue};
use super::super::status::NapiStatus;

pub unsafe fn napi_create_buffer(
    env: *mut NapiEnv,
    length: usize,
    data: *mut *mut c_void,
    result: *mut NapiValue,
) -> NapiStatus {
    let ctx = unsafe { &mut *(*env).ctx };
    let mut buffer = vec![0u8; length];
    let ptr = buffer.as_mut_ptr();
    let mut boxed = Box::new(buffer);
    if !data.is_null() {
        *data = boxed.as_mut_ptr() as *mut c_void;
    }

    let data_addr = boxed.as_ptr() as usize;
    *result = data_addr as NapiValue;
    std::mem::forget(boxed);
    NapiStatus::Ok
}

pub unsafe fn napi_create_buffer_copy(
    env: *mut NapiEnv,
    length: usize,
    data: *const c_void,
    _result_data: *mut *mut c_void,
    result: *mut NapiValue,
) -> NapiStatus {
    let ctx = unsafe { &mut *(*env).ctx };
    let mut buffer = vec![0u8; length];
    if !data.is_null() {
        unsafe {
            std::ptr::copy_nonoverlapping(data as *const u8, buffer.as_mut_ptr(), length);
        }
    }
    let ptr = buffer.as_ptr() as usize;
    let boxed = Box::new(buffer);
    std::mem::forget(boxed);
    *result = ptr as NapiValue;
    NapiStatus::Ok
}

pub unsafe fn napi_get_buffer_info(
    _env: *mut NapiEnv,
    value: NapiValue,
    data: *mut *mut c_void,
    length: *mut usize,
) -> NapiStatus {
    if value.is_null() || (value as usize) < 4096 {
        return NapiStatus::InvalidArg;
    }
    if !data.is_null() {
        *data = value;
    }
    if !length.is_null() {
        *length = 0;
    }
    NapiStatus::Ok
}
