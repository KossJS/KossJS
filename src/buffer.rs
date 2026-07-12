// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

use boa_engine::{js_string, Context, JsError, JsNativeError, JsObject, JsValue, NativeFunction};

pub fn register_buffer_globals(ctx: &mut Context) {
    let alloc_fn = unsafe {
        NativeFunction::from_closure(
            move |_this: &JsValue, args: &[JsValue], _ctx: &mut Context| -> Result<JsValue, JsError> {
                let size = args.first()
                    .and_then(|v| v.as_number())
                    .map(|n| n as usize)
                    .unwrap_or(0);
                let buf = vec![0u8; size];
                let data_ptr = buf.as_ptr() as usize;
                let len = buf.len();
                std::mem::forget(buf);

                let obj = JsObject::with_object_proto(_ctx.intrinsics());
                obj.insert_property(js_string!("length"), boa_engine::property::PropertyDescriptor::builder().value(JsValue::from(len as f64)).writable(false).enumerable(true).configurable(false));
                obj.insert_property(js_string!("_data_ptr"), boa_engine::property::PropertyDescriptor::builder().value(JsValue::from(data_ptr as f64)).writable(false).enumerable(false).configurable(false));
                obj.insert_property(js_string!("__is_buffer__"), boa_engine::property::PropertyDescriptor::builder().value(JsValue::from(true)).writable(false).enumerable(false).configurable(false));
                Ok(obj.into())
            },
        )
    };

    let alloc_unsafe_fn = unsafe {
        NativeFunction::from_closure(
            move |_this: &JsValue, args: &[JsValue], _ctx: &mut Context| -> Result<JsValue, JsError> {
                let size = args.first()
                    .and_then(|v| v.as_number())
                    .map(|n| n as usize)
                    .unwrap_or(0);
                let buf = vec![0u8; size];
                let data_ptr = buf.as_ptr() as usize;
                let len = buf.len();
                std::mem::forget(buf);

                let obj = JsObject::with_object_proto(_ctx.intrinsics());
                obj.insert_property(js_string!("length"), boa_engine::property::PropertyDescriptor::builder().value(JsValue::from(len as f64)).writable(false).enumerable(true).configurable(false));
                obj.insert_property(js_string!("_data_ptr"), boa_engine::property::PropertyDescriptor::builder().value(JsValue::from(data_ptr as f64)).writable(false).enumerable(false).configurable(false));
                obj.insert_property(js_string!("__is_buffer__"), boa_engine::property::PropertyDescriptor::builder().value(JsValue::from(true)).writable(false).enumerable(false).configurable(false));
                Ok(obj.into())
            },
        )
    };

    let from_fn = unsafe {
        NativeFunction::from_closure(
            move |_this: &JsValue, args: &[JsValue], _ctx: &mut Context| -> Result<JsValue, JsError> {
                let input = args.first().ok_or_else(|| {
                    JsNativeError::error().with_message("Buffer.from requires an argument")
                })?;
                if let Some(s) = input.as_string() {
                    let s = s.to_std_string_escaped();
                    let buf = s.as_bytes().to_vec();
                    let len = buf.len();
                    let data_ptr = buf.as_ptr() as usize;
                    std::mem::forget(buf);
                    let obj = JsObject::with_object_proto(_ctx.intrinsics());
                    obj.insert_property(js_string!("length"), boa_engine::property::PropertyDescriptor::builder().value(JsValue::from(len as f64)).writable(false).enumerable(true).configurable(false));
                    obj.insert_property(js_string!("_data_ptr"), boa_engine::property::PropertyDescriptor::builder().value(JsValue::from(data_ptr as f64)).writable(false).enumerable(false).configurable(false));
                    obj.insert_property(js_string!("__is_buffer__"), boa_engine::property::PropertyDescriptor::builder().value(JsValue::from(true)).writable(false).enumerable(false).configurable(false));
                    return Ok(obj.into());
                }
                Err(JsNativeError::error().with_message("Buffer.from: unsupported input type").into())
            },
        )
    };

    let is_buffer_fn = NativeFunction::from_copy_closure(
        move |_this: &JsValue, args: &[JsValue], _ctx: &mut Context| -> Result<JsValue, JsError> {
            let is_buf = args.first()
                .and_then(|v| v.as_object())
                .and_then(|o| {
                    let props = o.borrow();
                    let pk: boa_engine::property::PropertyKey = js_string!("__is_buffer__").into();
                    props.properties().get(&pk)
                        .and_then(|d| d.value().and_then(|v| v.as_boolean()))
                })
                .unwrap_or(false);
            Ok(JsValue::from(is_buf))
        },
    );

    let buf_obj = boa_engine::object::ObjectInitializer::new(ctx)
        .function(alloc_fn, js_string!("alloc"), 1)
        .function(alloc_unsafe_fn, js_string!("allocUnsafe"), 1)
        .function(from_fn, js_string!("from"), 1)
        .function(is_buffer_fn, js_string!("isBuffer"), 1)
        .build();

    let _ = ctx.register_global_property(
        js_string!("Buffer"),
        buf_obj,
        boa_engine::property::Attribute::all(),
    );
}
