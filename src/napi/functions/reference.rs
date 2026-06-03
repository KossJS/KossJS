use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::super::env::{NapiEnv, NapiValue};
use super::super::status::NapiStatus;

thread_local! {
    static REF_COUNTS: RefCell<HashMap<usize, (NapiValue, u32)>> = RefCell::new(HashMap::new());
}

static NEXT_REF_ID: AtomicUsize = AtomicUsize::new(1);

pub unsafe fn napi_create_reference(
    _env: *mut NapiEnv,
    value: NapiValue,
    _initial_refcount: u32,
    result: *mut NapiValue,
) -> NapiStatus {
    let id = NEXT_REF_ID.fetch_add(1, Ordering::Relaxed);
    REF_COUNTS.with(|reg| {
        reg.borrow_mut().insert(id, (value, 1));
    });
    *result = id as NapiValue;
    NapiStatus::Ok
}

pub unsafe fn napi_delete_reference(
    _env: *mut NapiEnv,
    reference: NapiValue,
) -> NapiStatus {
    let id = reference as usize;
    REF_COUNTS.with(|reg| { reg.borrow_mut().remove(&id); });
    NapiStatus::Ok
}

pub unsafe fn napi_reference_ref(
    _env: *mut NapiEnv,
    reference: NapiValue,
    result: *mut u32,
) -> NapiStatus {
    let id = reference as usize;
    REF_COUNTS.with(|reg| {
        if let Some((_, count)) = reg.borrow_mut().get_mut(&id) {
            *count += 1;
            *result = *count;
        }
    });
    NapiStatus::Ok
}

pub unsafe fn napi_reference_unref(
    _env: *mut NapiEnv,
    reference: NapiValue,
    result: *mut u32,
) -> NapiStatus {
    let id = reference as usize;
    REF_COUNTS.with(|reg| {
        if let Some((_, count)) = reg.borrow_mut().get_mut(&id) {
            if *count > 0 {
                *count -= 1;
            }
            *result = *count;
        }
    });
    NapiStatus::Ok
}

pub unsafe fn napi_get_reference_value(
    _env: *mut NapiEnv,
    reference: NapiValue,
    result: *mut NapiValue,
) -> NapiStatus {
    let id = reference as usize;
    REF_COUNTS.with(|reg| {
        if let Some((val, _)) = reg.borrow().get(&id) {
            *result = *val;
        }
    });
    NapiStatus::Ok
}
