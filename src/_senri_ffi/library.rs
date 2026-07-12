// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

use std::cell::RefCell;
use std::ffi::CString;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64};

use boa_engine::{
    js_string,
    object::ObjectInitializer,
    property::Attribute,
    Context, JsData, JsError, JsNativeError, JsObject, JsValue, NativeFunction,
};
use boa_gc::{Finalize, Trace};
use libffi::low::CodePtr;
use libffi::middle;

use super::call::{CallingConvention, FfiCall, invoke_ffi_call};
use super::types::FfiType;

#[derive(Debug, Trace, Finalize, JsData)]
#[allow(dead_code)]
pub struct LibraryHandle {
    #[unsafe_ignore_trace]
    pub(crate) inner: Option<usize>,
    #[unsafe_ignore_trace]
    pub(crate) path: String,
    #[unsafe_ignore_trace]
    pub(crate) closed: Rc<RefCell<bool>>,
    #[unsafe_ignore_trace]
    pub active_async_tasks: Arc<AtomicU64>,
    #[unsafe_ignore_trace]
    pub tainted: Arc<AtomicBool>,
}

struct LibraryWrapper {
    lib: libloading::Library,
}

pub(crate) fn get_library(handle: &LibraryHandle) -> Result<&libloading::Library, JsError> {
    let ptr = handle.inner.ok_or_else(|| {
        JsNativeError::error().with_message("library already closed")
    })?;
    let wrapper = unsafe { &*(ptr as *const LibraryWrapper) };
    Ok(&wrapper.lib)
}

pub fn create_library_handle(
    lib: libloading::Library,
    path: &str,
    ctx: &mut Context,
) -> JsObject {
    let wrapper = Box::new(LibraryWrapper { lib });
    let ptr = Box::into_raw(wrapper) as usize;
    let closed = Rc::new(RefCell::new(false));

    let handle = LibraryHandle {
        inner: Some(ptr),
        path: path.to_string(),
        closed: Rc::clone(&closed),
        active_async_tasks: Arc::new(AtomicU64::new(0)),
        tainted: Arc::new(AtomicBool::new(false)),
    };

    let mut builder = ObjectInitializer::with_native_data(handle, ctx);

    let func_closed = Rc::clone(&closed);
    let func_closure = move |_this: &JsValue,
                             args: &[JsValue],
                             _ctx: &mut Context|
          -> Result<JsValue, JsError> {
        if *func_closed.borrow() {
            return Err(JsNativeError::error()
                .with_message("library is closed")
                .into());
        }

        let this_obj = _this.as_object().ok_or_else(|| {
            JsNativeError::error().with_message("not a LibraryHandle")
        })?;
        let handle = this_obj.downcast_ref::<LibraryHandle>().ok_or_else(|| {
            JsNativeError::error().with_message("not a LibraryHandle")
        })?;
        let lib = get_library(&handle)?;

        let symbol_name = args
            .first()
            .and_then(|v| v.as_string())
            .map(|s| s.to_std_string_escaped())
            .ok_or_else(|| {
                JsNativeError::error()
                    .with_message("symbol name (string) required")
            })?;

        let ret_type = super::types::type_from_js(
            args.get(1).unwrap_or(&JsValue::undefined()),
        )?;

        let mut call_conv = CallingConvention::Cdecl;
        if args.len() > 3 {
            if let Some(opts) = args[3].as_object() {
                let props = opts.borrow();
                let pk: boa_engine::property::PropertyKey =
                    js_string!("callingConvention").into();
                if let Some(desc) = props.properties().get(&pk) {
                    if let Some(val) = desc.value() {
                        if let Some(s) = val.as_string() {
                            call_conv = CallingConvention::from_str(
                                &s.to_std_string_escaped(),
                            );
                        }
                    }
                }
            }
        }

        let mut vararg_index: Option<usize> = None;
        let mut arg_types: Vec<Rc<FfiType>> = Vec::new();

        if args.len() > 2 {
            if let Some(arr) = args[2].as_object() {
                let len = arr
                    .borrow()
                    .properties()
                    .index_properties()
                    .count();
                for i in 0..len {
                    let key: boa_engine::property::PropertyKey =
                        boa_engine::js_string!(i.to_string()).into();
                    let props = arr.borrow();
                    if let Some(desc) = props.properties().get(&key) {
                        if let Some(val) = desc.value() {
                            if let Some(s) = val.as_string() {
                                if s.to_std_string_escaped() == "..." {
                                    vararg_index = Some(i);
                                    continue;
                                }
                            }
                            match super::types::type_from_js(val) {
                                Ok(t) => arg_types.push(t),
                                Err(_) => {}
                            }
                        }
                    }
                }
            }
        }

        let effective_vararg_index = if let Some(vi) = vararg_index {
            if arg_types.len() > vi {
                arg_types.truncate(vi);
            }
            Some(vi)
        } else {
            None
        };

        let c_symbol_name = CString::new(symbol_name.as_bytes()).map_err(|_| {
            JsNativeError::error().with_message("invalid symbol name")
        })?;

        let fn_ptr: libloading::Symbol<unsafe extern "C" fn()> =
            unsafe {
                lib.get(c_symbol_name.as_bytes()).map_err(|e| {
                    JsNativeError::error().with_message(format!(
                        "symbol not found: {symbol_name}: {e}"
                    ))
                })?
            };

        let code_ptr = CodePtr(*fn_ptr as *mut std::ffi::c_void);

        let middle_types: Vec<middle::Type> = arg_types
            .iter()
            .map(|t| t.to_middle_type())
            .collect();

        let abi = call_conv.to_ffi_abi();

        let cif = if let Some(fixed_count) = effective_vararg_index {
            middle::Cif::new_variadic_with_abi(
                middle_types,
                fixed_count,
                ret_type.to_middle_type(),
                abi,
            )
        } else {
            middle::Cif::new_with_abi(middle_types, ret_type.to_middle_type(), abi)
        };

        let ffi_call = Rc::new(FfiCall {
            cif,
            fn_ptr: code_ptr,
            ret_type: Rc::new((*ret_type).clone()),
            arg_types,
            conv: CallingConvention::Cdecl,
            vararg_index: effective_vararg_index,
        });

        let ffi_call_for_js = Rc::clone(&ffi_call);
        let native = unsafe {
            NativeFunction::from_closure(
                move |_this2: &JsValue,
                      call_args: &[JsValue],
                      call_ctx: &mut Context|
                      -> Result<JsValue, JsError> {
                    invoke_ffi_call(&ffi_call_for_js, call_args, call_ctx)
                },
            )
        };
        let js_func = native.to_js_function(_ctx.realm());
        Ok(js_func.into())
    };

    let func_native = unsafe { NativeFunction::from_closure(func_closure) };
    builder.function(func_native, js_string!("func"), 3);

    let close_closed = Rc::clone(&closed);
    let close_closure = move |_this: &JsValue,
                              _args: &[JsValue],
                              _ctx: &mut Context|
          -> Result<JsValue, JsError> {
        *close_closed.borrow_mut() = true;
        let this_obj = _this.as_object().ok_or_else(|| {
            JsNativeError::error().with_message("not a LibraryHandle")
        })?;
        let mut handle = this_obj
            .downcast_mut::<LibraryHandle>()
            .ok_or_else(|| {
                JsNativeError::error().with_message("not a LibraryHandle")
            })?;
        if let Some(inner) = handle.inner.take() {
            unsafe {
                drop(Box::from_raw(inner as *mut LibraryWrapper));
            }
        }
        Ok(JsValue::undefined())
    };

    let close_native = unsafe { NativeFunction::from_closure(close_closure) };
    builder.function(close_native, js_string!("close"), 0);

    builder
        .property(
            js_string!("path"),
            JsValue::from(js_string!(path)),
            Attribute::READONLY,
        )
        .build()
}
