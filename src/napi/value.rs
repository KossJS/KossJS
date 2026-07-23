// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

//! Unified N-API value representation.
//!
//! Previously `NapiValue` was a raw pointer whose meaning was guessed from its
//! numeric address range (`< 4096` => boxed f64, `> 0x10000` => C string, etc.).
//! Because numbers, strings and objects are all heap pointers, those ranges
//! overlapped: a boxed number was routinely mis-read as a C string, producing
//! type confusion and out-of-bounds reads.
//!
//! Every non-trivial value is now a tagged [`NapiSlot`] allocated on the heap.
//! Small sentinel integers encode `undefined`/`null`/`true`/`false` so pointer
//! identity for those keeps working. Any other `NapiValue` is a `*mut NapiSlot`.

use std::ffi::{c_void, CString};

use boa_engine::{js_string, Context, JsError, JsObject, JsValue};

use super::env::{NapiEnv, NapiValue};
use super::status::NapiStatus;

// Sentinel encodings. These small integers are never valid heap pointers, so
// they can be distinguished from a `*mut NapiSlot` by numeric value.
pub const NAPI_UNDEFINED: usize = 0;
pub const NAPI_NULL: usize = 1;
pub const NAPI_TRUE: usize = 2;
pub const NAPI_FALSE: usize = 3;
const MAX_SENTINEL: usize = 3;

/// Tagged heap payload behind a non-sentinel [`NapiValue`].
pub enum NapiSlot {
    Number(f64),
    Str(CString),
    Object(JsObject),
    Buffer(Vec<u8>),
    External(*mut c_void),
}

#[inline]
pub fn alloc_slot(slot: NapiSlot) -> NapiValue {
    Box::into_raw(Box::new(slot)) as NapiValue
}

#[inline]
pub fn napi_undefined() -> NapiValue {
    NAPI_UNDEFINED as NapiValue
}

#[inline]
pub fn napi_null() -> NapiValue {
    NAPI_NULL as NapiValue
}

#[inline]
pub fn napi_bool(b: bool) -> NapiValue {
    (if b { NAPI_TRUE } else { NAPI_FALSE }) as NapiValue
}

/// Borrow the [`NapiSlot`] behind a non-sentinel value, if any.
///
/// # Safety
/// `v` must be either a sentinel or a pointer previously produced by
/// [`alloc_slot`] (which is the invariant maintained by every producer in this
/// crate). Sentinels return `None` without dereferencing.
#[inline]
pub unsafe fn as_slot<'a>(v: NapiValue) -> Option<&'a NapiSlot> {
    let addr = v as usize;
    if addr <= MAX_SENTINEL {
        return None;
    }
    Some(unsafe { &*(v as *const NapiSlot) })
}

/// Convert a Boa [`JsValue`] into a fresh [`NapiValue`].
pub unsafe fn js_to_napi(js: &JsValue, _ctx: &mut Context) -> NapiValue {
    if js.is_undefined() {
        return napi_undefined();
    }
    if js.is_null() {
        return napi_null();
    }
    if let Some(b) = js.as_boolean() {
        return napi_bool(b);
    }
    if let Some(n) = js.as_number() {
        return alloc_slot(NapiSlot::Number(n));
    }
    if let Some(s) = js.as_string() {
        let s = s.to_std_string_escaped();
        // Strings with interior NULs cannot round-trip through a C string; fall
        // back to storing the truncated form rather than silently losing all data.
        let cstr = CString::new(s.clone())
            .unwrap_or_else(|_| CString::new(s.replace('\0', "")).unwrap_or_default());
        return alloc_slot(NapiSlot::Str(cstr));
    }
    if let Some(obj) = js.as_object() {
        return alloc_slot(NapiSlot::Object(obj.clone()));
    }
    napi_undefined()
}

/// N-API `napi_valuetype`-ish code, preserving this crate's historical mapping
/// (undefined=0, null=1, boolean=4, number=5, everything-else=6).
pub fn get_napi_value_type(val: NapiValue) -> i32 {
    let addr = val as usize;
    match addr {
        NAPI_UNDEFINED => 0,
        NAPI_NULL => 1,
        NAPI_TRUE | NAPI_FALSE => 4,
        _ => match unsafe { as_slot(val) } {
            Some(NapiSlot::Number(_)) => 5,
            _ => 6,
        },
    }
}

/// Shared decode used by both `napi_to_js` and `object::value_to_js`.
pub unsafe fn value_to_js(val: NapiValue, ctx: &mut Context) -> JsValue {
    let addr = val as usize;
    match addr {
        NAPI_UNDEFINED => JsValue::undefined(),
        NAPI_NULL => JsValue::null(),
        NAPI_TRUE => JsValue::from(true),
        NAPI_FALSE => JsValue::from(false),
        _ => match unsafe { as_slot(val) } {
            Some(NapiSlot::Number(n)) => JsValue::from(*n),
            Some(NapiSlot::Str(cstr)) => match cstr.to_str() {
                Ok(s) => JsValue::from(js_string!(s)),
                Err(_) => JsValue::undefined(),
            },
            Some(NapiSlot::Object(obj)) => JsValue::from(obj.clone()),
            // Buffer/External have no faithful JsValue representation here; the
            // dedicated accessors (napi_get_buffer_info / napi_get_value_external)
            // expose them safely. Returning undefined avoids unsafe reads.
            Some(NapiSlot::Buffer(_)) | Some(NapiSlot::External(_)) => JsValue::undefined(),
            None => {
                let _ = ctx;
                JsValue::undefined()
            }
        },
    }
}

pub unsafe fn napi_to_js(val: NapiValue, _env: *mut NapiEnv, ctx: &mut Context) -> Result<JsValue, JsError> {
    Ok(unsafe { value_to_js(val, ctx) })
}

/// Return the [`JsObject`] behind an object value, or `None` for anything else.
pub unsafe fn get_value_as(obj: NapiValue) -> Option<JsObject> {
    match unsafe { as_slot(obj) } {
        Some(NapiSlot::Object(o)) => Some(o.clone()),
        _ => None,
    }
}

/// Deep-copy a value into a freshly allocated, independently owned slot.
/// Sentinels are returned unchanged. Used so that persistent references do not
/// alias transient (callback-scoped) slots that are freed after the call.
pub unsafe fn clone_value(v: NapiValue) -> NapiValue {
    match unsafe { as_slot(v) } {
        None => v,
        Some(NapiSlot::Number(n)) => alloc_slot(NapiSlot::Number(*n)),
        Some(NapiSlot::Str(s)) => alloc_slot(NapiSlot::Str(s.clone())),
        Some(NapiSlot::Object(o)) => alloc_slot(NapiSlot::Object(o.clone())),
        Some(NapiSlot::Buffer(b)) => alloc_slot(NapiSlot::Buffer(b.clone())),
        Some(NapiSlot::External(p)) => alloc_slot(NapiSlot::External(*p)),
    }
}

/// Free a slot previously produced by [`alloc_slot`]. No-op for sentinels.
///
/// # Safety
/// The caller must own `v` (no other live alias). Intended for scope-bound
/// values such as per-call callback arguments.
pub unsafe fn free_value(v: NapiValue) {
    let addr = v as usize;
    if addr > MAX_SENTINEL {
        drop(unsafe { Box::from_raw(v as *mut NapiSlot) });
    }
}

#[allow(dead_code)]
pub fn napi_status_ok() -> NapiStatus {
    NapiStatus::Ok
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sentinels_distinct_from_slots() {
        assert_eq!(get_napi_value_type(napi_undefined()), 0);
        assert_eq!(get_napi_value_type(napi_null()), 1);
        assert_eq!(get_napi_value_type(napi_bool(true)), 4);
        assert_eq!(get_napi_value_type(napi_bool(false)), 4);
    }

    #[test]
    fn test_number_not_misclassified() {
        // A heap-boxed number must report as number (5), not object (6), and must
        // never be decoded as a string (the historical bug).
        let v = alloc_slot(NapiSlot::Number(42.5));
        assert_eq!(get_napi_value_type(v), 5);
        match unsafe { as_slot(v) } {
            Some(NapiSlot::Number(n)) => assert_eq!(*n, 42.5),
            _ => panic!("expected Number slot"),
        }
        // reclaim
        drop(unsafe { Box::from_raw(v as *mut NapiSlot) });
    }

    #[test]
    fn test_string_slot_roundtrip() {
        let v = alloc_slot(NapiSlot::Str(CString::new("hello").unwrap()));
        assert_eq!(get_napi_value_type(v), 6);
        match unsafe { as_slot(v) } {
            Some(NapiSlot::Str(s)) => assert_eq!(s.to_str().unwrap(), "hello"),
            _ => panic!("expected Str slot"),
        }
        drop(unsafe { Box::from_raw(v as *mut NapiSlot) });
    }

    #[test]
    fn test_buffer_slot_tracks_length() {
        let v = alloc_slot(NapiSlot::Buffer(vec![1, 2, 3, 4]));
        match unsafe { as_slot(v) } {
            Some(NapiSlot::Buffer(b)) => assert_eq!(b.len(), 4),
            _ => panic!("expected Buffer slot"),
        }
        drop(unsafe { Box::from_raw(v as *mut NapiSlot) });
    }

    #[test]
    fn test_as_slot_none_for_sentinels() {
        assert!(unsafe { as_slot(napi_undefined()) }.is_none());
        assert!(unsafe { as_slot(napi_null()) }.is_none());
        assert!(unsafe { as_slot(napi_bool(true)) }.is_none());
        assert!(unsafe { as_slot(napi_bool(false)) }.is_none());
    }
}
