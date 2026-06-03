use std::ffi::c_void;

use super::super::env::NapiEnv;
use super::super::status::NapiStatus;

pub unsafe fn napi_set_instance_data(
    env: *mut NapiEnv,
    data: *mut c_void,
    finalize_cb: Option<unsafe extern "C" fn(env: *mut NapiEnv, data: *mut c_void, hint: *mut c_void)>,
    hint: *mut c_void,
) -> NapiStatus {
    unsafe {
        *(*env).instance_data.borrow_mut() = Some(data);
        *(*env).instance_data_finalize.borrow_mut() = finalize_cb;
        *(*env).instance_data_hint.borrow_mut() = hint;
    }
    NapiStatus::Ok
}

pub unsafe fn napi_get_instance_data(
    env: *mut NapiEnv,
    result: *mut *mut c_void,
) -> NapiStatus {
    unsafe {
        *result = (*env).instance_data.borrow().unwrap_or(std::ptr::null_mut());
    }
    NapiStatus::Ok
}
