// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_void;

use boa_engine::{js_string, Context, JsError, JsNativeError, JsObject, JsValue, NativeFunction};

use super::super::env::{NapiCallback, NapiCallbackInfo, NapiEnv, NapiValue};
use super::super::status::NapiStatus;
use super::super::value::get_value_as;
use super::object::value_to_js;

thread_local! {
    static CALLBACK_REGISTRY: RefCell<HashMap<usize, NapiCallback>> = RefCell::new(HashMap::new());
    static CALLBACK_DATA: RefCell<HashMap<usize, *mut c_void>> = RefCell::new(HashMap::new());
}

pub unsafe fn napi_create_function(
    env: *mut NapiEnv,
    _utf8name: *const u8,
    _length: isize,
    cb: NapiCallback,
    data: *mut c_void,
    result: *mut NapiValue,
) -> NapiStatus {
    let ctx = unsafe { &mut *(*env).ctx };
    let cb_id: usize = unsafe { std::mem::transmute(cb) };
    CALLBACK_REGISTRY.with(|reg| {
        reg.borrow_mut().insert(cb_id, cb);
    });
    if !data.is_null() {
        CALLBACK_DATA.with(|reg| {
            reg.borrow_mut().insert(cb_id, data);
        });
    }

    let closure = unsafe {
        NativeFunction::from_closure(
            move |_this: &JsValue, args: &[JsValue], _ctx: &mut Context| -> Result<JsValue, JsError> {
                let cb = CALLBACK_REGISTRY.with(|reg| reg.borrow().get(&cb_id).copied());
                match cb {
                    Some(callback) => {
                        let mut napi_args: Vec<NapiValue> = args.iter().map(|a| {
                            if a.is_undefined() { std::ptr::null_mut() }
                            else if a.is_null() { 1usize as NapiValue }
                            else if let Some(b) = a.as_boolean() {
                                if b { 2usize as NapiValue } else { 3usize as NapiValue }
                            } else if let Some(n) = a.as_number() {
                                let boxed = Box::new(n);
                                Box::into_raw(boxed) as NapiValue
                            } else if let Some(s) = a.as_string() {
                                let cstr = std::ffi::CString::new(s.to_std_string_escaped()).unwrap_or_default();
                                cstr.into_raw() as NapiValue
                            } else if let Some(o) = a.as_object() {
                                Box::into_raw(Box::new(o.clone())) as NapiValue
                            } else {
                                std::ptr::null_mut()
                            }
                        }).collect();

                        let cb_data = CALLBACK_DATA.with(|reg| reg.borrow().get(&cb_id).copied().unwrap_or(std::ptr::null_mut()));

                        let info = NapiCallbackInfo {
                            env,
                            this: std::ptr::null_mut(),
                            new_target: std::ptr::null_mut(),
                            argc: napi_args.len(),
                            argv: napi_args.as_ptr(),
                            data: cb_data,
                        };

                        let ret = callback(env, &info as *const NapiCallbackInfo as *mut NapiCallbackInfo);
                        if ret.is_null() {
                            Ok(JsValue::undefined())
                        } else {
                            Ok(value_to_js(ret, _ctx))
                        }
                    }
                    None => Err(JsNativeError::error().with_message("callback not found").into()),
                }
            },
        )
    };
    let js_func = closure.to_js_function(ctx.realm());
    let obj = JsObject::from(js_func);
    let boxed = Box::new(obj);
    *result = Box::into_raw(boxed) as NapiValue;
    NapiStatus::Ok
}

pub unsafe fn napi_call_function(
    _env: *mut NapiEnv,
    _recv: NapiValue,
    func: NapiValue,
    argc: usize,
    argv: *const NapiValue,
    result: *mut NapiValue,
) -> NapiStatus {
    let obj = match get_value_as(func) {
        Some(o) => o,
        None => return NapiStatus::FunctionExpected,
    };
    let ctx = unsafe { &mut *(*_env).ctx };
    let mut args: Vec<JsValue> = Vec::with_capacity(argc);
    for i in 0..argc {
        let val = unsafe { *argv.add(i) };
        args.push(value_to_js(val, ctx));
    }
    let js_func = boa_engine::object::builtins::JsFunction::from_object(obj.clone())
        .ok_or(NapiStatus::FunctionExpected);
    match js_func {
        Ok(f) => {
            match f.call(&JsValue::undefined(), &args, ctx) {
                Ok(v) => {
                    *result = super::object::js_value_to_napi_value(&v, ctx);
                    NapiStatus::Ok
                }
                Err(_) => NapiStatus::GenericFailure,
            }
        }
        Err(s) => s,
    }
}

pub unsafe fn napi_get_cb_info(
    _env: *mut NapiEnv,
    _cbinfo: *mut NapiCallbackInfo,
    argc: *mut usize,
    argv: *mut NapiValue,
    this_arg: *mut NapiValue,
    data: *mut *mut c_void,
) -> NapiStatus {
    if _cbinfo.is_null() {
        return NapiStatus::InvalidArg;
    }
    let info = unsafe { &*_cbinfo };
    if !argc.is_null() {
        *argc = info.argc;
    }
    if !argv.is_null() {
        for i in 0..info.argc {
            unsafe {
                *argv.add(i) = *info.argv.add(i);
            }
        }
    }
    if !this_arg.is_null() {
        *this_arg = info.this;
    }
    if !data.is_null() {
        *data = info.data;
    }
    NapiStatus::Ok
}

pub unsafe fn napi_get_new_target(
    _env: *mut NapiEnv,
    _cbinfo: *mut NapiCallbackInfo,
    result: *mut NapiValue,
) -> NapiStatus {
    if _cbinfo.is_null() {
        return NapiStatus::InvalidArg;
    }
    let info = unsafe { &*_cbinfo };
    *result = info.new_target;
    NapiStatus::Ok
}

pub unsafe fn napi_new_instance(
    env: *mut NapiEnv,
    constructor: NapiValue,
    argc: usize,
    argv: *const NapiValue,
    result: *mut NapiValue,
) -> NapiStatus {
    let obj = match get_value_as(constructor) {
        Some(o) => o,
        None => return NapiStatus::FunctionExpected,
    };
    let ctx = unsafe { &mut *(*env).ctx };
    let mut args: Vec<JsValue> = Vec::with_capacity(argc);
    for i in 0..argc {
        let val = unsafe { *argv.add(i) };
        args.push(value_to_js(val, ctx));
    }
    let new_obj = JsObject::with_object_proto(ctx.intrinsics());
    let boxed = Box::new(new_obj);
    *result = Box::into_raw(boxed) as NapiValue;
    NapiStatus::Ok
}
