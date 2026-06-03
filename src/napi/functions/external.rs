use std::ffi::c_void;

use super::super::env::{NapiEnv, NapiValue};
use super::super::status::NapiStatus;

pub unsafe fn napi_create_external(
    _env: *mut NapiEnv,
    data: *mut c_void,
    _finalize_cb: Option<unsafe extern "C" fn(env: *mut NapiEnv, data: *mut c_void, hint: *mut c_void)>,
    _finalize_hint: *mut c_void,
    result: *mut NapiValue,
) -> NapiStatus {
    *result = data;
    NapiStatus::Ok
}

pub unsafe fn napi_get_value_external(
    _env: *mut NapiEnv,
    value: NapiValue,
    result: *mut *mut c_void,
) -> NapiStatus {
    *result = value;
    NapiStatus::Ok
}
