// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_void;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

use boa_engine::{js_string, Context, JsError, JsObject, JsValue};
use libffi::low;
use libffi::middle;

use super::pointer::{create_pointer_object, JsPointer};
use super::types::FfiType;

thread_local! {
    static CALLBACK_REGISTRY: RefCell<HashMap<usize, CallbackEntry>> = RefCell::new(HashMap::new());
}

struct CallbackEntry {
    _closure: middle::Closure<'static>,
}

struct CallbackData {
    alive: AtomicBool,
    engine_ctx: *mut Context,
    ret_type: Rc<FfiType>,
    arg_types: Vec<Rc<FfiType>>,
    func: RefCell<Option<boa_engine::object::builtins::JsFunction>>,
}

pub fn create_callback(
    ret_type: Rc<FfiType>,
    arg_types: Vec<Rc<FfiType>>,
    js_func: boa_engine::object::builtins::JsFunction,
    ctx: &mut Context,
) -> Result<JsObject, JsError> {
    let middle_arg_types: Vec<middle::Type> = arg_types.iter()
        .map(|t| t.to_middle_type())
        .collect();
    let ret_middle_type = ret_type.to_middle_type();

    let cif = middle::Cif::new(middle_arg_types, ret_middle_type);

    let data = Box::new(CallbackData {
        alive: AtomicBool::new(true),
        engine_ctx: ctx as *mut Context,
        ret_type: Rc::clone(&ret_type),
        arg_types: arg_types.clone(),
        func: RefCell::new(Some(js_func)),
    });

    let data_ref: &'static CallbackData = Box::leak(data);

    let closure = middle::Closure::new(cif, trampoline::<CallResultBuf>, data_ref);
    let code = *closure.code_ptr() as *const c_void;

    let addr = code as usize;
    CALLBACK_REGISTRY.with(|reg| {
        reg.borrow_mut().insert(addr, CallbackEntry { _closure: closure });
    });

    let ptr_obj = create_pointer_object(addr, 0, ctx);

    Ok(ptr_obj)
}

#[allow(dead_code)]
fn free_callback(addr: usize) -> bool {
    CALLBACK_REGISTRY.with(|reg| {
        if let Some(entry) = reg.borrow_mut().remove(&addr) {
            drop(entry);
            true
        } else {
            false
        }
    })
}

pub fn has_callback(addr: usize) -> bool {
    CALLBACK_REGISTRY.with(|reg| reg.borrow().contains_key(&addr))
}

type CallResultBuf = *mut c_void;

unsafe extern "C" fn trampoline<R>(
    _cif: &low::ffi_cif,
    result: &mut R,
    args: *const *const c_void,
    userdata: &CallbackData,
) {
    if !userdata.alive.load(Ordering::Acquire) {
        return;
    }

    let ctx = unsafe { &mut *userdata.engine_ctx };
    let func_opt = userdata.func.borrow_mut().take();
    if func_opt.is_none() {
        return;
    }
    let func = func_opt.unwrap();

    let mut js_args: Vec<JsValue> = Vec::with_capacity(userdata.arg_types.len());
    for i in 0..userdata.arg_types.len() {
        let arg_ptr = unsafe { *args.add(i) as *const u8 };
        let type_info = &userdata.arg_types[i];
        let val = c_ptr_to_js(arg_ptr, type_info);
        js_args.push(val);
    }

    let js_result = match func.call(&JsValue::undefined(), &js_args, ctx) {
        Ok(v) => v,
        Err(_e) => JsValue::undefined(),
    };

    userdata.func.borrow_mut().replace(func);

    let ret_bytes = js_result_to_bytes(&js_result, &userdata.ret_type);
    let ret_size = ret_bytes.len().min(std::mem::size_of::<CallResultBuf>());
    let dst = result as *mut R as *mut u8;
    unsafe {
        std::ptr::copy_nonoverlapping(ret_bytes.as_ptr(), dst, ret_size);
    }
}

fn c_ptr_to_js(ptr: *const u8, type_info: &FfiType) -> JsValue {
    match type_info {
        FfiType::Void => JsValue::undefined(),
        FfiType::Int8 => {
            let val = unsafe { *(ptr as *const i8) };
            JsValue::from(val as f64)
        }
        FfiType::Uint8 => {
            let val = unsafe { *(ptr as *const u8) };
            JsValue::from(val as f64)
        }
        FfiType::Int16 => {
            let val = unsafe { *(ptr as *const i16) };
            JsValue::from(val as f64)
        }
        FfiType::Uint16 => {
            let val = unsafe { *(ptr as *const u16) };
            JsValue::from(val as f64)
        }
        FfiType::Int32 => {
            let val = unsafe { *(ptr as *const i32) };
            JsValue::from(val as f64)
        }
        FfiType::Uint32 => {
            let val = unsafe { *(ptr as *const u32) };
            JsValue::from(val as f64)
        }
        FfiType::Int64 => {
            let val = unsafe { *(ptr as *const i64) };
            JsValue::from(val as f64)
        }
        FfiType::Uint64 => {
            let val = unsafe { *(ptr as *const u64) };
            JsValue::from(val as f64)
        }
        FfiType::Float32 => {
            let val = unsafe { *(ptr as *const f32) };
            JsValue::from(val as f64)
        }
        FfiType::Float64 => {
            let val = unsafe { *(ptr as *const f64) };
            JsValue::from(val)
        }
        FfiType::Pointer | FfiType::Callback { .. } => {
            let addr = unsafe { *(ptr as *const usize) };
            JsValue::from(addr as f64)
        }
        FfiType::CString => {
            let addr = unsafe { *(ptr as *const *const std::ffi::c_char) };
            if addr.is_null() {
                JsValue::null()
            } else {
                let cstr = unsafe { std::ffi::CStr::from_ptr(addr) };
                let s = cstr.to_string_lossy().to_string();
                JsValue::from(js_string!(s))
            }
        }
        _ => JsValue::undefined(),
    }
}

fn js_result_to_bytes(val: &JsValue, type_info: &FfiType) -> Vec<u8> {
    match type_info {
        FfiType::Void => Vec::new(),
        FfiType::Int8 => {
            let v = val.as_number().map(|n| n as i8).unwrap_or(0);
            v.to_le_bytes().to_vec()
        }
        FfiType::Uint8 => {
            let v = val.as_number().map(|n| n as u8).unwrap_or(0);
            v.to_le_bytes().to_vec()
        }
        FfiType::Int16 => {
            let v = val.as_number().map(|n| n as i16).unwrap_or(0);
            v.to_le_bytes().to_vec()
        }
        FfiType::Uint16 => {
            let v = val.as_number().map(|n| n as u16).unwrap_or(0);
            v.to_le_bytes().to_vec()
        }
        FfiType::Int32 => {
            let v = val.as_number().map(|n| n as i32).unwrap_or(0);
            v.to_le_bytes().to_vec()
        }
        FfiType::Uint32 => {
            let v = val.as_number().map(|n| n as u32).unwrap_or(0);
            v.to_le_bytes().to_vec()
        }
        FfiType::Int64 => {
            let v = val.as_number().map(|n| n as i64).unwrap_or(0);
            v.to_le_bytes().to_vec()
        }
        FfiType::Uint64 => {
            let v = val.as_number().map(|n| n as u64).unwrap_or(0);
            v.to_le_bytes().to_vec()
        }
        FfiType::Float32 => {
            let v = val.as_number().map(|n| n as f32).unwrap_or(0.0);
            v.to_le_bytes().to_vec()
        }
        FfiType::Float64 => {
            let v = val.as_number().unwrap_or(0.0);
            v.to_le_bytes().to_vec()
        }
        FfiType::Pointer | FfiType::Callback { .. } => {
            let addr = val
                .as_number()
                .map(|n| n as usize)
                .or_else(|| {
                    val.as_object()
                        .and_then(|o| {
                            let ptr_ref = o.downcast_ref::<JsPointer>();
                            ptr_ref.map(|p| p.address)
                        })
                })
                .unwrap_or(0);
            addr.to_le_bytes().to_vec()
        }
        FfiType::CString => {
            if val.is_null() || val.is_undefined() {
                0usize.to_le_bytes().to_vec()
            } else if let Some(s) = val.as_string() {
                let cstr = std::ffi::CString::new(s.to_std_string_escaped().as_bytes())
                    .unwrap_or_default();
                let ptr = cstr.into_raw() as usize;
                ptr.to_le_bytes().to_vec()
            } else {
                0usize.to_le_bytes().to_vec()
            }
        }
        _ => {
            vec![0u8; type_info.sizeof()]
        }
    }
}
