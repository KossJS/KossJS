// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

use std::rc::Rc;

use boa_engine::{
    js_string,
    object::ObjectInitializer,
    property::Attribute,
    Context, JsData, JsError, JsNativeError, JsObject, JsValue, NativeFunction,
};
use boa_gc::{Finalize, Trace};

use super::pointer::{create_pointer_object, JsPointer};
use super::types::{self, compute_struct_layout, FfiType};

#[derive(Debug, Trace, Finalize, JsData)]
struct StructInstance {
    #[unsafe_ignore_trace]
    buffer: Vec<u8>,
    /// CString fields store a `char*` into `buffer`; the owning CString must
    /// live as long as this instance, otherwise the stored pointer dangles
    /// (use-after-free) when the struct is later passed to native code.
    #[unsafe_ignore_trace]
    cstrings: Vec<std::ffi::CString>,
}

fn read_from_buffer(buf: &[u8], offset: usize, type_info: &FfiType) -> JsValue {
    let end = offset + type_info.sizeof();
    if end > buf.len() {
        return JsValue::undefined();
    }
    let bytes = &buf[offset..end];
    match type_info {
        FfiType::Void => JsValue::undefined(),
        FfiType::Int8 => {
            let v = i8::from_le_bytes(bytes.try_into().unwrap());
            JsValue::from(v as f64)
        }
        FfiType::Uint8 => {
            let v = u8::from_le_bytes(bytes.try_into().unwrap());
            JsValue::from(v as f64)
        }
        FfiType::Int16 => {
            let v = i16::from_le_bytes(bytes.try_into().unwrap());
            JsValue::from(v as f64)
        }
        FfiType::Uint16 => {
            let v = u16::from_le_bytes(bytes.try_into().unwrap());
            JsValue::from(v as f64)
        }
        FfiType::Int32 => {
            let v = i32::from_le_bytes(bytes.try_into().unwrap());
            JsValue::from(v as f64)
        }
        FfiType::Uint32 => {
            let v = u32::from_le_bytes(bytes.try_into().unwrap());
            JsValue::from(v as f64)
        }
        FfiType::Int64 => {
            let v = i64::from_le_bytes(bytes.try_into().unwrap());
            JsValue::from(v as f64)
        }
        FfiType::Uint64 => {
            let v = u64::from_le_bytes(bytes.try_into().unwrap());
            JsValue::from(v as f64)
        }
        FfiType::Float32 => {
            let v = f32::from_le_bytes(bytes.try_into().unwrap());
            JsValue::from(v as f64)
        }
        FfiType::Float64 => {
            let v = f64::from_le_bytes(bytes.try_into().unwrap());
            JsValue::from(v)
        }
        FfiType::Pointer | FfiType::Callback { .. } => {
            let addr = usize::from_le_bytes(bytes.try_into().unwrap());
            JsValue::from(addr as f64)
        }
        FfiType::CString => {
            let addr = usize::from_le_bytes(bytes.try_into().unwrap());
            if addr == 0 {
                JsValue::null()
            } else {
                unsafe {
                    let cstr = std::ffi::CStr::from_ptr(addr as *const std::ffi::c_char);
                    let s = cstr.to_string_lossy();
                    JsValue::from(js_string!(s))
                }
            }
        }
        FfiType::Struct { .. } => {
            JsValue::undefined()
        }
        _ => JsValue::undefined(),
    }
}

fn write_to_buffer(
    buf: &mut [u8],
    offset: usize,
    type_info: &FfiType,
    val: &JsValue,
    keep: &mut Vec<std::ffi::CString>,
) -> Result<(), JsError> {
    let end = offset + type_info.sizeof();
    if end > buf.len() {
        return Err(JsNativeError::error().with_message("buffer overflow").into());
    }
    let target = &mut buf[offset..end];
    match type_info {
        FfiType::Void => Ok(()),
        FfiType::Int8 => {
            let v = val.as_number().map(|n| n as i8).ok_or_else(|| {
                JsNativeError::error().with_message("expected number (int8)")
            })?;
            target.copy_from_slice(&v.to_le_bytes());
            Ok(())
        }
        FfiType::Uint8 => {
            let v = val.as_number().map(|n| n as u8).ok_or_else(|| {
                JsNativeError::error().with_message("expected number (uint8)")
            })?;
            target.copy_from_slice(&v.to_le_bytes());
            Ok(())
        }
        FfiType::Int16 => {
            let v = val.as_number().map(|n| n as i16).ok_or_else(|| {
                JsNativeError::error().with_message("expected number (int16)")
            })?;
            target.copy_from_slice(&v.to_le_bytes());
            Ok(())
        }
        FfiType::Uint16 => {
            let v = val.as_number().map(|n| n as u16).ok_or_else(|| {
                JsNativeError::error().with_message("expected number (uint16)")
            })?;
            target.copy_from_slice(&v.to_le_bytes());
            Ok(())
        }
        FfiType::Int32 => {
            let v = val.as_number().map(|n| n as i32).ok_or_else(|| {
                JsNativeError::error().with_message("expected number (int32)")
            })?;
            target.copy_from_slice(&v.to_le_bytes());
            Ok(())
        }
        FfiType::Uint32 => {
            let v = val.as_number().map(|n| n as u32).ok_or_else(|| {
                JsNativeError::error().with_message("expected number (uint32)")
            })?;
            target.copy_from_slice(&v.to_le_bytes());
            Ok(())
        }
        FfiType::Int64 => {
            let v = val.as_number().map(|n| n as i64).ok_or_else(|| {
                JsNativeError::error().with_message("expected number (int64)")
            })?;
            target.copy_from_slice(&v.to_le_bytes());
            Ok(())
        }
        FfiType::Uint64 => {
            let v = val.as_number().map(|n| n as u64).ok_or_else(|| {
                JsNativeError::error().with_message("expected number (uint64)")
            })?;
            target.copy_from_slice(&v.to_le_bytes());
            Ok(())
        }
        FfiType::Float32 => {
            let v = val.as_number().map(|n| n as f32).ok_or_else(|| {
                JsNativeError::error().with_message("expected number (float32)")
            })?;
            target.copy_from_slice(&v.to_le_bytes());
            Ok(())
        }
        FfiType::Float64 => {
            let v = val.as_number().ok_or_else(|| {
                JsNativeError::error().with_message("expected number (float64)")
            })?;
            target.copy_from_slice(&v.to_le_bytes());
            Ok(())
        }
        FfiType::Pointer | FfiType::Callback { .. } => {
            let addr: usize = if let Some(obj) = val.as_object() {
                obj.downcast_ref::<JsPointer>()
                    .map(|p| p.address)
                    .unwrap_or_else(|| val.as_number().map(|n| n as usize).unwrap_or(0))
            } else {
                val.as_number().map(|n| n as usize).unwrap_or(0)
            };
            target.copy_from_slice(&addr.to_le_bytes());
            Ok(())
        }
        FfiType::CString => {
            if val.is_null() || val.is_undefined() {
                let zero: usize = 0;
                target.copy_from_slice(&zero.to_le_bytes());
            } else {
                let s = val.as_string().map(|s| s.to_std_string_escaped()).ok_or_else(|| {
                    JsNativeError::error().with_message("expected string for CString")
                })?;
                let cstr = std::ffi::CString::new(s.as_bytes()).map_err(|e| {
                    JsNativeError::error().with_message(format!("invalid C string: {e}"))
                })?;
                let ptr = cstr.as_ptr() as usize;
                target.copy_from_slice(&ptr.to_le_bytes());
                // Keep the CString alive for the lifetime of the owning buffer so
                // the stored char* does not dangle once passed to native code.
                keep.push(cstr);
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

pub fn create_struct_constructor(
    fields: &[(String, Rc<FfiType>)],
    packed: Option<u16>,
    ctx: &mut Context,
) -> JsObject {
    let (field_infos, total_size, _align) = compute_struct_layout(fields, packed);

    let ffi_type = FfiType::Struct {
        fields: field_infos.clone(),
        packed,
        size: total_size,
        align: _align,
    };
    let type_rc = Rc::new(ffi_type);

    let realm = ctx.realm();
    let mut accessors: Vec<(String, boa_engine::object::builtins::JsFunction, boa_engine::object::builtins::JsFunction)> = Vec::new();

    for field in &field_infos {
        let f_name = field.name.clone();
        let f_offset = field.offset;
        let f_type = Rc::clone(&field.type_info);
        let f_type2 = Rc::clone(&field.type_info);

        let getter = unsafe {
            NativeFunction::from_closure(
                move |_this: &JsValue, _args: &[JsValue], _ctx: &mut Context| -> Result<JsValue, JsError> {
                    let obj = _this.as_object().ok_or_else(|| {
                        JsNativeError::error().with_message("not a struct instance")
                    })?;
                    let inst = obj.downcast_ref::<StructInstance>().ok_or_else(|| {
                        JsNativeError::error().with_message("not a struct instance")
                    })?;
                    Ok(read_from_buffer(&inst.buffer, f_offset, &f_type))
                },
            )
        };
        let getter_func = getter.to_js_function(realm);

        let setter = unsafe {
            NativeFunction::from_closure(
                move |_this: &JsValue, args: &[JsValue], _ctx: &mut Context| -> Result<JsValue, JsError> {
                    let obj = _this.as_object().ok_or_else(|| {
                        JsNativeError::error().with_message("not a struct instance")
                    })?;
                    let mut inst = obj.downcast_mut::<StructInstance>().ok_or_else(|| {
                        JsNativeError::error().with_message("not a struct instance")
                    })?;
                    let default_val = JsValue::undefined();
                    let val = args.first().unwrap_or(&default_val);
                    // Split-borrow disjoint fields so the buffer and the CString
                    // collector can be borrowed mutably at the same time.
                    let inst_ref: &mut StructInstance = &mut inst;
                    let StructInstance { buffer, cstrings } = inst_ref;
                    write_to_buffer(buffer, f_offset, &f_type2, val, cstrings)?;
                    Ok(JsValue::undefined())
                },
            )
        };
        let setter_func = setter.to_js_function(realm);

        accessors.push((f_name, getter_func, setter_func));
    }
    // realm borrow ends here

    let to_pointer_fn = NativeFunction::from_copy_closure(
        move |_this: &JsValue, _args: &[JsValue], _ctx: &mut Context| -> Result<JsValue, JsError> {
            let obj = _this.as_object().ok_or_else(|| {
                JsNativeError::error().with_message("not a struct instance")
            })?;
            let inst = obj.downcast_ref::<StructInstance>().ok_or_else(|| {
                JsNativeError::error().with_message("not a struct instance")
            })?;
            let ptr = inst.buffer.as_ptr() as usize;
            let size = inst.buffer.len();
            let pointer_obj = create_pointer_object(ptr, size, _ctx);
            Ok(JsValue::from(pointer_obj))
        },
    );

    let mut proto_builder = ObjectInitializer::new(ctx);
    for (f_name, getter_func, setter_func) in accessors {
        proto_builder.accessor(
            js_string!(f_name),
            Some(getter_func),
            Some(setter_func),
            Attribute::all(),
        );
    }

    proto_builder.property(
        js_string!("sizeof"),
        JsValue::from(total_size as f64),
        Attribute::READONLY | Attribute::NON_ENUMERABLE,
    );

    proto_builder.function(to_pointer_fn, js_string!("toPointer"), 0);

    let prototype = proto_builder.build();

    let fields_for_ctor = field_infos.clone();
    let total_size_for_ctor = total_size;
    let proto_for_ctor = prototype.clone();

    let ctor_fn = unsafe {
        NativeFunction::from_closure(
            move |_this: &JsValue, args: &[JsValue], _ctx: &mut Context| -> Result<JsValue, JsError> {
                let mut buffer = vec![0u8; total_size_for_ctor];
                let mut cstrings: Vec<std::ffi::CString> = Vec::new();

                if let Some(init_obj) = args.first().and_then(|v| v.as_object()) {
                    for field in &fields_for_ctor {
                        let key: boa_engine::property::PropertyKey =
                            js_string!(field.name.clone()).into();
                        let props = init_obj.borrow();
                        if let Some(desc) = props.properties().get(&key) {
                            if let Some(val) = desc.value() {
                                write_to_buffer(
                                    &mut buffer,
                                    field.offset,
                                    &field.type_info,
                                    val,
                                    &mut cstrings,
                                )?;
                            }
                        }
                    }
                }

                let buffer_ptr = buffer.as_ptr() as usize;
                let instance = StructInstance { buffer, cstrings };
                let obj = JsObject::from_proto_and_data(
                    proto_for_ctor.clone(),
                    instance,
                );
                let ffi_buf = create_pointer_object(buffer_ptr, total_size_for_ctor, _ctx);
                obj.insert_property(
                    js_string!("_ffi_buffer"),
                    boa_engine::property::PropertyDescriptor::builder()
                        .value(ffi_buf)
                        .writable(false)
                        .enumerable(false)
                        .configurable(false),
                );
                Ok(obj.into())
            },
        )
    };
    let ctor = js_function_to_object(ctor_fn.to_js_function(ctx.realm()));

    ctor.insert_property(
        js_string!("prototype"),
        boa_engine::property::PropertyDescriptor::builder()
            .value(prototype)
            .writable(false)
            .enumerable(false)
            .configurable(false),
    );

    ctor.insert_property(
        js_string!("sizeof"),
        boa_engine::property::PropertyDescriptor::builder()
            .value(JsValue::from(total_size as f64))
            .writable(false)
            .enumerable(true)
            .configurable(false),
    );

    types::store_type_handle(&ctor, &Rc::clone(&type_rc), ctx);

    ctor
}

fn js_function_to_object(f: boa_engine::object::builtins::JsFunction) -> JsObject {
    JsObject::from(f)
}
