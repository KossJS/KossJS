use std::ffi::c_void;

use super::super::async_runtime::AsyncRuntime;
use super::super::env::{NapiEnv, NapiAsyncWork};
use super::super::status::NapiStatus;

pub unsafe fn napi_create_async_work(
    env: *mut NapiEnv,
    _async_resource: *mut c_void,
    _async_resource_name: *const u8,
    execute: Option<unsafe extern "C" fn(env: *mut NapiEnv, data: *mut c_void)>,
    complete: Option<unsafe extern "C" fn(env: *mut NapiEnv, status: NapiStatus, data: *mut c_void)>,
    data: *mut c_void,
    result: *mut *mut NapiAsyncWork,
) -> NapiStatus {
    let work = Box::new(NapiAsyncWork {
        env,
        execute,
        complete,
        data,
        result: std::ptr::null_mut(),
        status: NapiStatus::Ok,
    });
    *result = Box::into_raw(work);
    NapiStatus::Ok
}

pub unsafe fn napi_delete_async_work(
    _env: *mut NapiEnv,
    work: *mut NapiAsyncWork,
) -> NapiStatus {
    if !work.is_null() {
        unsafe {
            drop(Box::from_raw(work));
        }
    }
    NapiStatus::Ok
}

pub unsafe fn napi_queue_async_work(
    env: *mut NapiEnv,
    work: *mut NapiAsyncWork,
) -> NapiStatus {
    if work.is_null() {
        return NapiStatus::InvalidArg;
    }
    let work_ref = unsafe { &*work };
    let execute_fn = match work_ref.execute {
        Some(f) => f,
        None => return NapiStatus::InvalidArg,
    };
    let complete_fn = match work_ref.complete {
        Some(f) => f,
        None => return NapiStatus::InvalidArg,
    };

    let runtime = get_async_runtime();
    let completions = runtime.completions.clone();

    let raw_env = env as usize;
    let raw_execute = unsafe { std::mem::transmute::<_, usize>(execute_fn) };
    let raw_complete = unsafe { std::mem::transmute::<_, usize>(complete_fn) };
    let raw_data = work_ref.data as usize;

    AsyncRuntime::spawn_async_work_safe(
        completions,
        raw_env,
        raw_execute,
        raw_complete,
        raw_data,
    );

    unsafe {
        let count = &mut (*(*env).async_work_count.borrow_mut());
        *count += 1;
    }

    NapiStatus::Ok
}

pub fn get_or_drain_async_completions() -> Vec<super::super::async_runtime::AsyncCompletion> {
    get_async_runtime().drain_completions()
}

fn get_async_runtime() -> AsyncRuntime {
    use std::sync::OnceLock;
    static RUNTIME: OnceLock<AsyncRuntime> = OnceLock::new();
    RUNTIME.get_or_init(|| AsyncRuntime::new()).clone()
}
