// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

pub mod call;
pub mod callback;
pub mod library;
pub mod memory;
pub mod pointer;
pub mod struct_def;
pub mod types;

use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use boa_engine::{
    js_string,
    object::ObjectInitializer,
    property::Attribute,
    Context, JsError, JsNativeError, JsValue, NativeFunction,
};
use boa_engine::object::builtins::JsPromise;
use libloading;

use call::async_defs::{self, FfiCallAsync, invoke_ffi_call_async};
use library::{create_library_handle, LibraryHandle};
use memory::register_memory_methods;
use types::{FfiType, OwnedFfiType, find_callback_args};

pub fn register_senri_ffi(context: &mut Context, instance_ptr: *mut c_void) {
    let types_obj = {
        let obj = boa_engine::JsObject::with_object_proto(context.intrinsics());
        let type_names: [&str; 14] = [
            "void", "int8", "uint8", "int16", "uint16", "int32", "uint32",
            "int64", "uint64", "float32", "float64", "pointer", "cstring", "...",
        ];
        for name in &type_names {
            obj.insert_property(
                js_string!(*name),
                boa_engine::property::PropertyDescriptor::builder()
                    .value(js_string!(*name))
                    .writable(false)
                    .enumerable(false)
                    .configurable(false),
            );
        }
        obj
    };

    let mut builder = ObjectInitializer::new(context);
    builder.property(
        js_string!("types"),
        types_obj,
        Attribute::READONLY | Attribute::NON_ENUMERABLE,
    );

    let open_fn = NativeFunction::from_copy_closure(
        move |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
            let path = args
                .first()
                .and_then(|v| v.as_string())
                .map(|s| s.to_std_string_escaped())
                .ok_or_else(|| {
                    JsNativeError::error().with_message("path (string) required")
                })?;

            let lib = unsafe { libloading::Library::new(&path) }.map_err(|e| {
                JsNativeError::error()
                    .with_message(format!("failed to open library '{path}': {e}"))
            })?;

            let handle = create_library_handle(lib, &path, ctx);

            // Add funcAsync method
            add_func_async_to_handle(&handle, instance_ptr, ctx);
            // Add closeAsync method
            add_close_async_to_handle(&handle, instance_ptr, ctx);

            Ok(handle.into())
        },
    );
    builder.function(open_fn, js_string!("open"), 1);

    let struct_fn = NativeFunction::from_copy_closure(
        move |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
            let fields_obj = args
                .first()
                .and_then(|v| v.as_object())
                .ok_or_else(|| {
                    JsNativeError::error().with_message("fields object required")
                })?;

            let packed: Option<u16> = args
                .get(1)
                .and_then(|v| v.as_object())
                .and_then(|o| {
                    let props = o.borrow();
                    props
                        .properties()
                        .get(&js_string!("packed").into())
                        .and_then(|d| d.value().cloned())
                        .and_then(|v| v.as_number().map(|n| n as u16))
                });

            let mut field_list: Vec<(String, Rc<FfiType>)> = Vec::new();
            let props = fields_obj.borrow();
            let index_props = props.properties().index_properties();
            let count = index_props.count();

            for i in 0..count {
                let key: boa_engine::property::PropertyKey =
                    boa_engine::js_string!(i.to_string()).into();
                if let Some(desc) = props.properties().get(&key) {
                    if let Some(val) = desc.value() {
                        let type_rc = types::type_from_js(val)?;
                        field_list.push((i.to_string(), type_rc));
                    }
                }
            }

            let field_names: Vec<String> = {
                let mut names = Vec::new();
                for i in 0..count {
                    let key: boa_engine::property::PropertyKey =
                        boa_engine::js_string!(i.to_string()).into();
                    if let Some(desc) = props.properties().get(&key) {
                        if let Some(val) = desc.value() {
                            if let Some(name) = val.as_string() {
                                names.push(name.to_std_string_escaped());
                            } else if let Some(obj) = val.as_object() {
                                let obj_props = obj.borrow();
                                if let Some(name_desc) = obj_props
                                    .properties()
                                    .get(&js_string!("name").into())
                                    .and_then(|d| d.value().cloned())
                                {
                                    if let Some(name) = name_desc.as_string() {
                                        names.push(name.to_std_string_escaped());
                                    }
                                }
                            }
                        }
                    }
                }
                names
            };

            if field_names.len() == field_list.len() && !field_names.is_empty() {
                field_list = field_list
                    .into_iter()
                    .enumerate()
                    .map(|(i, (_, t))| (field_names[i].clone(), t))
                    .collect();
            }

            let ctor = struct_def::create_struct_constructor(&field_list, packed, ctx);
            Ok(ctor.into())
        },
    );
    builder.function(struct_fn, js_string!("struct"), 2);

    let pointer_fn = NativeFunction::from_copy_closure(
        move |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
            let _inner_type = args.first();
            let obj = ObjectInitializer::new(ctx).build();
            let pointer_type = Rc::new(FfiType::Pointer);
            types::store_type_handle(
                &obj,
                &pointer_type,
                ctx,
            );
            Ok(obj.into())
        },
    );
    builder.function(pointer_fn, js_string!("pointer"), 1);

    let array_fn = NativeFunction::from_copy_closure(
        move |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
            let inner_type = types::type_from_js(
                args.first().unwrap_or(&JsValue::undefined()),
            )?;
            let count = args
                .get(1)
                .and_then(|v| v.as_number())
                .map(|n| n as usize)
                .unwrap_or(0);

            let arr_type = FfiType::Array {
                inner: Box::new((*inner_type).clone()),
                count,
            };
            let type_rc = Rc::new(arr_type);
            let type_rc2 = Rc::clone(&type_rc);

            let obj = ObjectInitializer::new(ctx).build();
            types::store_type_handle(&obj, &type_rc2, ctx);
            obj.insert_property(
                js_string!("sizeof"),
                boa_engine::property::PropertyDescriptor::builder()
                    .value(JsValue::from(type_rc.sizeof() as f64))
                    .writable(false)
                    .enumerable(true)
                    .configurable(false),
            );

            Ok(obj.into())
        },
    );
    builder.function(array_fn, js_string!("array"), 2);

    let callback_fn = NativeFunction::from_copy_closure(
        move |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
            let ret_type = types::type_from_js(
                args.first().unwrap_or(&JsValue::undefined()),
            )?;
            let mut arg_types: Vec<Rc<FfiType>> = Vec::new();
            if let Some(arr) = args.get(1).and_then(|v| v.as_object()) {
                let len = arr.borrow().properties().index_properties().count();
                for i in 0..len {
                    let key: boa_engine::property::PropertyKey =
                        boa_engine::js_string!(i.to_string()).into();
                    if let Some(desc) = arr.borrow().properties().get(&key) {
                        if let Some(val) = desc.value() {
                            if let Ok(t) = types::type_from_js(val) {
                                arg_types.push(t);
                            }
                        }
                    }
                }
            }

            let cb_type = FfiType::Callback {
                args: arg_types,
                ret: Box::new((*ret_type).clone()),
            };
            let type_rc = Rc::new(cb_type);

            let obj = ObjectInitializer::new(ctx).build();
            types::store_type_handle(&obj, &type_rc, ctx);

            Ok(obj.into())
        },
    );
    builder.function(callback_fn, js_string!("callback"), 2);

    let create_cb_fn = NativeFunction::from_copy_closure(
        move |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
            let ret_type = types::type_from_js(
                args.first().unwrap_or(&JsValue::undefined()),
            )?;
            let mut arg_types: Vec<Rc<FfiType>> = Vec::new();
            if let Some(arr) = args.get(1).and_then(|v| v.as_object()) {
                let len = arr.borrow().properties().index_properties().count();
                for i in 0..len {
                    let key: boa_engine::property::PropertyKey =
                        boa_engine::js_string!(i.to_string()).into();
                    if let Some(desc) = arr.borrow().properties().get(&key) {
                        if let Some(val) = desc.value() {
                            if let Ok(t) = types::type_from_js(val) {
                                arg_types.push(t);
                            }
                        }
                    }
                }
            }
            let js_func_val = args.get(2).ok_or_else(|| {
                JsNativeError::error().with_message("callback function required")
            })?;
            let js_func_obj = js_func_val.as_object().ok_or_else(|| {
                JsNativeError::error().with_message("expected a function")
            })?;
            let js_func = boa_engine::object::builtins::JsFunction::from_object(js_func_obj.clone())
                .ok_or_else(|| {
                    JsNativeError::error().with_message("not a valid function")
                })?;

            let ptr_obj = callback::create_callback(ret_type, arg_types, js_func, ctx)?;
            Ok(ptr_obj.into())
        },
    );
    builder.function(create_cb_fn, js_string!("createCallback"), 3);

    // freeCallback(ptr) -> bool : release a callback previously produced by
    // createCallback, reclaiming its libffi closure and CallbackData. Without
    // this the callback allocation would live for the entire process.
    let free_cb_fn = NativeFunction::from_copy_closure(
        move |_this: &JsValue, args: &[JsValue], _ctx: &mut Context| -> Result<JsValue, JsError> {
            let addr = args
                .first()
                .and_then(|v| {
                    if let Some(obj) = v.as_object() {
                        obj.downcast_ref::<crate::_senri_ffi::pointer::JsPointer>()
                            .map(|p| p.address)
                    } else {
                        v.as_number().map(|n| n as usize)
                    }
                })
                .unwrap_or(0);
            Ok(JsValue::from(callback::free_callback(addr)))
        },
    );
    builder.function(free_cb_fn, js_string!("freeCallback"), 1);

    register_memory_methods(&mut builder);

    let senri_obj = builder.build();

    let _ = context.register_global_property(
        js_string!("_senri_ffi"),
        senri_obj,
        Attribute::all(),
    );
}

use std::ffi::CString;
use std::os::raw::c_void;

fn add_func_async_to_handle(
    handle: &boa_engine::JsObject,
    instance_ptr: *mut c_void,
    ctx: &mut Context,
) {
    let handle_clone = handle.clone();

    let func_async = unsafe {
        NativeFunction::from_closure(
        move |_this: &JsValue, args: &[JsValue], call_ctx: &mut Context| -> Result<JsValue, JsError> {
            let lib_handle = handle_clone.downcast_ref::<LibraryHandle>().ok_or_else(|| {
                JsNativeError::error().with_message("not a LibraryHandle")
            })?;

            if *lib_handle.closed.borrow() {
                return Err(JsNativeError::error().with_message("library is closed").into());
            }
            if lib_handle.tainted.load(Ordering::Acquire) {
                return Err(JsNativeError::error().with_message("library is tainted").into());
            }

            let lib = crate::_senri_ffi::library::get_library(&*lib_handle)?;

            let symbol_name = args.first()
                .and_then(|v| v.as_string())
                .map(|s| s.to_std_string_escaped())
                .ok_or_else(|| JsNativeError::error().with_message("symbol name required"))?;

            let ret_type_rc = types::type_from_js(args.get(1).unwrap_or(&JsValue::undefined()))?;
            let ret_type = OwnedFfiType::from_rc(&ret_type_rc);

            let mut vararg_index: Option<usize> = None;
            let mut arg_type_rcs: Vec<Rc<FfiType>> = Vec::new();

            if args.len() > 2 {
                if let Some(arr) = args[2].as_object() {
                    let props = arr.borrow();
                    let index_props = props.properties().index_properties();
                    let len = index_props.count();
                    for i in 0..len {
                        let key: boa_engine::property::PropertyKey = boa_engine::js_string!(i.to_string()).into();
                        if let Some(desc) = props.properties().get(&key) {
                            if let Some(val) = desc.value() {
                                if let Some(s) = val.as_string() {
                                    if s.to_std_string_escaped() == "..." {
                                        vararg_index = Some(i);
                                        continue;
                                    }
                                }
                                if let Ok(t) = types::type_from_js(val) {
                                    arg_type_rcs.push(t);
                                }
                            }
                        }
                    }
                }
            }

            let effective_vararg_index = if let Some(vi) = vararg_index {
                if arg_type_rcs.len() > vi { arg_type_rcs.truncate(vi); }
                Some(vi)
            } else { None };

            let mut callback_timeout_ms: u64 = 30000;
            let mut allow_force_kill = false;
            if let Some(opts) = args.get(3).and_then(|v| v.as_object()) {
                let p = opts.borrow();
                if let Some(d) = p.properties().get(&js_string!("callbackTimeout").into()) {
                    callback_timeout_ms = d.value().and_then(|v| v.as_number()).map(|n| n as u64).unwrap_or(30000);
                }
                if let Some(d) = p.properties().get(&js_string!("allowForceKill").into()) {
                    allow_force_kill = d.value().and_then(|v| v.as_boolean()).unwrap_or(false);
                }
            }

            let callback_infos: Vec<(usize, Vec<OwnedFfiType>, OwnedFfiType)> =
                find_callback_args(&arg_type_rcs).into_iter()
                    .map(|(i, args, _ret)| {
                        let owned_args: Vec<OwnedFfiType> = args.iter().map(|t| OwnedFfiType::from_rc(&FfiType::Callback{
                            args: vec![Rc::new(t.clone())],
                            ret: Box::new(FfiType::Void),
                        })).collect();
                        (i, owned_args.into_iter().map(|t| t).collect(), OwnedFfiType::from_rc(&FfiType::Void))
                    })
                    .collect();

            let callback_indices: Vec<usize> = callback_infos.iter().map(|(i, _, _)| *i).collect();

            let owned_arg_types: Vec<OwnedFfiType> = arg_type_rcs.iter()
                .map(|rc| OwnedFfiType::from_rc(rc))
                .collect();

            let c_symbol_name = CString::new(symbol_name.as_bytes()).map_err(|_| {
                JsNativeError::error().with_message("invalid symbol name")
            })?;
            let fn_ptr_sym: libloading::Symbol<unsafe extern "C" fn()> =
                lib.get(c_symbol_name.as_bytes()).map_err(|e| {
                    JsNativeError::error().with_message(format!("symbol not found: {symbol_name}: {e}"))
                })?;
            let code_ptr = libffi::low::CodePtr(*fn_ptr_sym as *mut std::ffi::c_void);

            let ffi_async = Arc::new(FfiCallAsync {
                fn_ptr: code_ptr,
                vararg_index: effective_vararg_index,
                fixed_arg_count: if let Some(vi) = effective_vararg_index { vi } else { owned_arg_types.len() },
                arg_types: owned_arg_types.clone(),
                ret_type: ret_type.clone(),
                callback_indices: callback_indices.clone(),
            });

            let active_tasks = lib_handle.active_async_tasks.clone();
            let tainted = lib_handle.tainted.clone();

            let async_fn =
                NativeFunction::from_closure(
                    move |_this2: &JsValue, call_args: &[JsValue], ctx2: &mut Context| -> Result<JsValue, JsError> {
                        if tainted.load(Ordering::Acquire) {
                            return Err(JsNativeError::error().with_message("library is tainted").into());
                        }

                        let inst = &mut *(instance_ptr as *mut crate::runtime::KossInstance);
                        let event_loop = match inst.event_loop.as_mut() {
                            Some(el) => el,
                            None => return Err(JsNativeError::error().with_message("no event loop available").into()),
                        };

                        let actual_arg_count = call_args.len();

                        let mut arg_buffers: Vec<Vec<u8>> = Vec::with_capacity(ffi_async.arg_types.len());
                        for (i, owned_type) in ffi_async.arg_types.iter().enumerate() {
                            let js_val = if i < actual_arg_count { &call_args[i] } else { &JsValue::undefined() };
                            let buf = if ffi_async.callback_indices.contains(&i) {
                                async_defs::js_arg_to_bytes_send(js_val, &FfiType::Pointer, true)?
                            } else {
                                serialize_owned_arg(js_val, owned_type)?
                            };
                            arg_buffers.push(buf);
                        }

                        let (promise, resolvers) = JsPromise::new_pending(ctx2);
                        let promise_id = match event_loop.register_promise(
                            resolvers.resolve.clone(),
                            resolvers.reject.clone(),
                        ) {
                            Some(id) => id,
                            None => return Err(JsNativeError::error().with_message("too many pending promises").into()),
                        };

                        let task_canceled = Arc::new(AtomicBool::new(false));
                        let task_id = event_loop.register_ffi_task(
                            task_canceled.clone(),
                            allow_force_kill,
                            callback_timeout_ms,
                        );

                        for (cb_idx, _cb_args, _cb_ret) in &callback_infos {
                            let cb_js_val = if *cb_idx < call_args.len() {
                                &call_args[*cb_idx]
                            } else {
                                &JsValue::undefined()
                            };
                            if let Some(cb_obj) = cb_js_val.as_object() {
                                if let Some(cb_fn) = boa_engine::object::builtins::JsFunction::from_object(cb_obj.clone()) {
                                    event_loop.register_ffi_callback_fn(task_id, *cb_idx, cb_fn);
                                }
                            }
                        }

                        let io_tx = event_loop.io_tx.clone();
                        let callback_tx = event_loop.callback_tx_clone();
                        let ffi_data_clone = ffi_async.clone();
                        let active_tasks_spawn = active_tasks.clone();
                        active_tasks.fetch_add(1, Ordering::SeqCst);
                        let active_tasks_err = active_tasks.clone();

                        let thread_result = std::thread::Builder::new()
                            .name(format!("koss-ffi-async-{task_id}"))
                            .spawn(move || {
                                let result = invoke_ffi_call_async(
                                    &ffi_data_clone, &arg_buffers, task_id, &callback_tx, callback_timeout_ms,
                                );
                                active_tasks_spawn.fetch_sub(1, Ordering::SeqCst);
                                let _ = io_tx.send(crate::runtime::AsyncIoResult {
                                    promise_id,
                                    result: result.map_err(|e| format!("FFI error: {e}")),
                                });
                            });

                        match thread_result {
                            Ok(handle) => {
                                event_loop.set_ffi_task_thread(task_id, handle);
                            }
                            Err(e) => {
                                active_tasks_err.fetch_sub(1, Ordering::SeqCst);
                                event_loop.remove_ffi_task(task_id);
                                return Err(JsNativeError::error()
                                    .with_message(format!("failed to spawn thread: {e}"))
                                    .into());
                            }
                        }

                        Ok(promise.into())
                    },
                );

            let js_func = async_fn.to_js_function(call_ctx.realm());
            Ok(js_func.into())
        })
    };

    let _ = handle.insert_property(
        js_string!("funcAsync"),
        boa_engine::property::PropertyDescriptor::builder()
            .value(func_async.to_js_function(ctx.realm()))
            .writable(false)
            .enumerable(false)
            .configurable(true),
    );
}

fn add_close_async_to_handle(
    handle: &boa_engine::JsObject,
    instance_ptr: *mut c_void,
    ctx: &mut Context,
) {
    let handle_clone = handle.clone();

    let close_async = unsafe {
        NativeFunction::from_closure(
        move |_this: &JsValue, _args: &[JsValue], call_ctx: &mut Context| -> Result<JsValue, JsError> {
            let obj = _this.as_object().map(|o| o.clone()).unwrap_or_else(|| handle_clone.clone());
            let mut lib_handle = obj.downcast_mut::<LibraryHandle>().ok_or_else(|| {
                JsNativeError::error().with_message("not a LibraryHandle")
            })?;

            *lib_handle.closed.borrow_mut() = true;
            let active_tasks = lib_handle.active_async_tasks.clone();

            let inst = &mut *(instance_ptr as *mut crate::runtime::KossInstance);
            let event_loop = match inst.event_loop.as_mut() {
                Some(el) => el,
                None => {
                    if let Some(inner) = lib_handle.inner.take() {
                        let wrapper = Box::from_raw(inner as *mut libloading::Library);
                        drop(wrapper);
                    }
                    return Ok(JsValue::undefined());
                }
            };

            let (promise, resolvers) = JsPromise::new_pending(call_ctx);
            let promise_id = match event_loop.register_promise(
                resolvers.resolve.clone(),
                resolvers.reject.clone(),
            ) {
                Some(id) => id,
                None => return Err(JsNativeError::error().with_message("too many pending promises").into()),
            };

            let io_tx = event_loop.io_tx.clone();
            event_loop.runtime.spawn(async move {
                loop {
                    if active_tasks.load(Ordering::SeqCst) == 0 {
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
                let _ = io_tx.send(crate::runtime::AsyncIoResult {
                    promise_id,
                    result: Ok("closed".to_string()),
                });
            });

            Ok(promise.into())
        })
    };

    let _ = handle.insert_property(
        js_string!("closeAsync"),
        boa_engine::property::PropertyDescriptor::builder()
            .value(close_async.to_js_function(ctx.realm()))
            .writable(false)
            .enumerable(false)
            .configurable(true),
    );
}

fn serialize_owned_arg(val: &JsValue, t: &OwnedFfiType) -> Result<Vec<u8>, JsError> {
    match t {
        OwnedFfiType::Void => Ok(Vec::new()),
        OwnedFfiType::Int8 => Ok((val.as_number().map(|n| n as i8).unwrap_or(0)).to_le_bytes().to_vec()),
        OwnedFfiType::Uint8 => Ok((val.as_number().map(|n| n as u8).unwrap_or(0)).to_le_bytes().to_vec()),
        OwnedFfiType::Int16 => Ok((val.as_number().map(|n| n as i16).unwrap_or(0)).to_le_bytes().to_vec()),
        OwnedFfiType::Uint16 => Ok((val.as_number().map(|n| n as u16).unwrap_or(0)).to_le_bytes().to_vec()),
        OwnedFfiType::Int32 => Ok((val.as_number().map(|n| n as i32).unwrap_or(0)).to_le_bytes().to_vec()),
        OwnedFfiType::Uint32 => Ok((val.as_number().map(|n| n as u32).unwrap_or(0)).to_le_bytes().to_vec()),
        OwnedFfiType::Int64 => Ok((val.as_number().map(|n| n as i64).unwrap_or(0)).to_le_bytes().to_vec()),
        OwnedFfiType::Uint64 => Ok((val.as_number().map(|n| n as u64).unwrap_or(0)).to_le_bytes().to_vec()),
        OwnedFfiType::Float32 => Ok((val.as_number().map(|n| n as f32).unwrap_or(0.0)).to_le_bytes().to_vec()),
        OwnedFfiType::Float64 => Ok((val.as_number().unwrap_or(0.0)).to_le_bytes().to_vec()),
        OwnedFfiType::Pointer | OwnedFfiType::Callback { .. } | OwnedFfiType::VarArg => {
            let addr = if let Some(obj) = val.as_object() {
                obj.downcast_ref::<crate::_senri_ffi::pointer::JsPointer>().map(|p| p.address).unwrap_or(0)
            } else {
                val.as_number().map(|n| n as usize).unwrap_or(0)
            };
            Ok(addr.to_le_bytes().to_vec())
        }
        OwnedFfiType::CString => {
            if val.is_null() || val.is_undefined() {
                Ok(0usize.to_le_bytes().to_vec())
            } else {
                let s = val.as_string().map(|s| s.to_std_string_escaped())
                    .ok_or_else(|| JsNativeError::error().with_message("expected string for CString"))?;
                let cstr = std::ffi::CString::new(s.as_bytes())
                    .map_err(|e| JsNativeError::error().with_message(format!("invalid C string: {e}")))?;
                let ptr = cstr.into_raw() as usize;
                Ok(ptr.to_le_bytes().to_vec())
            }
        }
        _ => Ok(vec![0u8; t.sizeof()]),
    }
}
