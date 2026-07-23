// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

use std::ffi::c_void;

use boa_engine::{Context, JsValue};

use super::super::env::{NapiEnv, NapiValue};
use super::super::status::NapiStatus;
use super::super::value::{alloc_slot, get_value_as, NapiSlot};
use super::object::value_to_js;

pub unsafe fn napi_create_array(
    env: *mut NapiEnv,
    result: *mut NapiValue,
) -> NapiStatus {
    let ctx = unsafe { &mut *(*env).ctx };
    let arr = boa_engine::object::builtins::JsArray::new(ctx);
    let js_val: JsValue = arr.into();
    let obj = js_val.as_object().map(|o| o.clone()).unwrap_or_else(|| {
        boa_engine::JsObject::with_object_proto(ctx.intrinsics())
    });
    let boxed_obj = obj;
    *result = alloc_slot(NapiSlot::Object(boxed_obj));
    NapiStatus::Ok
}

pub unsafe fn napi_create_array_with_length(
    env: *mut NapiEnv,
    length: usize,
    result: *mut NapiValue,
) -> NapiStatus {
    let ctx = unsafe { &mut *(*env).ctx };
    let elements: Vec<JsValue> = (0..length).map(|_| JsValue::undefined()).collect();
    let arr = boa_engine::object::builtins::JsArray::from_iter(elements, ctx);
    let js_val: JsValue = arr.into();
    let obj = js_val.as_object().map(|o| o.clone()).unwrap_or_else(|| {
        boa_engine::JsObject::with_object_proto(ctx.intrinsics())
    });
    let boxed_obj = obj;
    *result = alloc_slot(NapiSlot::Object(boxed_obj));
    NapiStatus::Ok
}

pub unsafe fn napi_get_array_length(
    env: *mut NapiEnv,
    arr: NapiValue,
    result: *mut u32,
) -> NapiStatus {
    let obj = match get_value_as(arr) {
        Some(o) => o,
        None => return NapiStatus::ArrayExpected,
    };
    let props = obj.borrow();
    let count = props.properties().index_properties().count();
    *result = count as u32;
    NapiStatus::Ok
}

pub unsafe fn napi_set_element(
    env: *mut NapiEnv,
    arr: NapiValue,
    index: u32,
    value: NapiValue,
) -> NapiStatus {
    let obj = match get_value_as(arr) {
        Some(o) => o,
        None => return NapiStatus::ArrayExpected,
    };
    let ctx = unsafe { &mut *(*env).ctx };
    let js_val = value_to_js(value, ctx);
    obj.insert_property(
        boa_engine::js_string!(index.to_string()),
        boa_engine::property::PropertyDescriptor::builder()
            .value(js_val)
            .writable(true)
            .enumerable(true)
            .configurable(true),
    );
    NapiStatus::Ok
}

pub unsafe fn napi_get_element(
    env: *mut NapiEnv,
    arr: NapiValue,
    index: u32,
    result: *mut NapiValue,
) -> NapiStatus {
    let obj = match get_value_as(arr) {
        Some(o) => o,
        None => return NapiStatus::ArrayExpected,
    };
    let ctx = unsafe { &mut *(*env).ctx };
    let props = obj.borrow();
    let pk: boa_engine::property::PropertyKey = boa_engine::js_string!(index.to_string()).into();
    if let Some(desc) = props.properties().get(&pk) {
        if let Some(val) = desc.value() {
            *result = super::object::js_value_to_napi_value(val, ctx);
            return NapiStatus::Ok;
        }
    }
    *result = std::ptr::null_mut();
    NapiStatus::Ok
}
