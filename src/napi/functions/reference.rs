// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::super::env::{NapiEnv, NapiValue};
use super::super::status::NapiStatus;
use super::super::value::{clone_value, free_value, napi_undefined};

thread_local! {
    static REF_COUNTS: RefCell<HashMap<usize, (NapiValue, u32)>> = RefCell::new(HashMap::new());
}

static NEXT_REF_ID: AtomicUsize = AtomicUsize::new(1);

pub unsafe fn napi_create_reference(
    _env: *mut NapiEnv,
    value: NapiValue,
    initial_refcount: u32,
    result: *mut NapiValue,
) -> NapiStatus {
    let id = NEXT_REF_ID.fetch_add(1, Ordering::Relaxed);
    // Store an independent copy so the reference stays valid even after the
    // (callback-scoped) source value is freed.
    let owned = unsafe { clone_value(value) };
    REF_COUNTS.with(|reg| {
        reg.borrow_mut().insert(id, (owned, initial_refcount));
    });
    if !result.is_null() {
        *result = id as NapiValue;
    }
    NapiStatus::Ok
}

pub unsafe fn napi_delete_reference(
    _env: *mut NapiEnv,
    reference: NapiValue,
) -> NapiStatus {
    let id = reference as usize;
    let removed = REF_COUNTS.with(|reg| reg.borrow_mut().remove(&id));
    if let Some((owned, _)) = removed {
        unsafe { free_value(owned) };
    }
    NapiStatus::Ok
}

pub unsafe fn napi_reference_ref(
    _env: *mut NapiEnv,
    reference: NapiValue,
    result: *mut u32,
) -> NapiStatus {
    let id = reference as usize;
    REF_COUNTS.with(|reg| {
        let mut map = reg.borrow_mut();
        match map.get_mut(&id) {
            Some((_, count)) => {
                *count += 1;
                if !result.is_null() {
                    unsafe { *result = *count; }
                }
                NapiStatus::Ok
            }
            None => NapiStatus::InvalidArg,
        }
    })
}

pub unsafe fn napi_reference_unref(
    _env: *mut NapiEnv,
    reference: NapiValue,
    result: *mut u32,
) -> NapiStatus {
    let id = reference as usize;
    REF_COUNTS.with(|reg| {
        let mut map = reg.borrow_mut();
        match map.get_mut(&id) {
            Some((_, count)) => {
                if *count > 0 {
                    *count -= 1;
                }
                if !result.is_null() {
                    unsafe { *result = *count; }
                }
                NapiStatus::Ok
            }
            None => NapiStatus::InvalidArg,
        }
    })
}

pub unsafe fn napi_get_reference_value(
    _env: *mut NapiEnv,
    reference: NapiValue,
    result: *mut NapiValue,
) -> NapiStatus {
    if result.is_null() {
        return NapiStatus::InvalidArg;
    }
    let id = reference as usize;
    REF_COUNTS.with(|reg| {
        match reg.borrow().get(&id) {
            Some((val, _)) => {
                unsafe { *result = *val; }
            }
            None => {
                // Reference not found (e.g. already deleted): report undefined
                // rather than leaving the out-parameter uninitialized.
                unsafe { *result = napi_undefined(); }
            }
        }
    });
    NapiStatus::Ok
}
