use std::alloc::{self, Layout};

use boa_engine::{
    js_string, object::ObjectInitializer, JsNativeError, JsValue, NativeFunction,
};
use boa_engine::object::builtins::JsArrayBuffer;

use super::pointer::{create_pointer_object, JsPointer};

pub fn alloc_impl(size: usize) -> *mut u8 {
    if size == 0 {
        return std::ptr::null_mut();
    }
    let layout =
        Layout::from_size_align(size, 16).unwrap_or_else(|_| Layout::array::<u8>(size).unwrap());
    unsafe { alloc::alloc(layout) }
}

pub fn free_impl(ptr: *mut u8, size: usize) {
    if ptr.is_null() || size == 0 {
        return;
    }
    let layout =
        Layout::from_size_align(size, 16).unwrap_or_else(|_| Layout::array::<u8>(size).unwrap());
    unsafe { alloc::dealloc(ptr, layout) };
}

pub fn register_memory_methods(obj: &mut ObjectInitializer) {
    let alloc_fn = NativeFunction::from_copy_closure(move |_this, args, ctx| {
        let size = args
            .first()
            .and_then(|v| v.as_number())
            .map(|n| n as usize)
            .unwrap_or(0);

        let ptr = alloc_impl(size);
        if ptr.is_null() && size > 0 {
            return Err(JsNativeError::error()
                .with_message("memory allocation failed")
                .into());
        }

        let ptr_obj = create_pointer_object(ptr as usize, size, ctx);
        Ok(ptr_obj.into())
    });
    obj.function(alloc_fn, js_string!("alloc"), 1);

    let alloc_type_fn = NativeFunction::from_copy_closure(move |_this, args, ctx| {
        let type_val = args.first().ok_or_else(|| {
            JsNativeError::error().with_message("type argument required")
        })?;
        let ffi_type = super::types::js_value_to_ffi_type(type_val)?;
        let count = args
            .get(1)
            .and_then(|v| v.as_number())
            .map(|n| n as usize)
            .unwrap_or(1);

        let total_size = ffi_type.sizeof() * count;
        let ptr = alloc_impl(total_size);
        if ptr.is_null() && total_size > 0 {
            return Err(JsNativeError::error()
                .with_message("memory allocation failed")
                .into());
        }

        let ptr_obj = create_pointer_object(ptr as usize, total_size, ctx);
        Ok(ptr_obj.into())
    });
    obj.function(alloc_type_fn, js_string!("allocType"), 2);

    let free_fn = NativeFunction::from_copy_closure(move |_this, args, _ctx| {
        if let Some(arg) = args.first() {
            if let Some(obj) = arg.as_object() {
                if let Some(ptr) = obj.downcast_ref::<JsPointer>() {
                    let addr = ptr.address;
                    let size = ptr.size;
                    drop(ptr);
                    if addr != 0 {
                        free_impl(addr as *mut u8, size);
                    }
                }
            }
        }
        Ok(JsValue::undefined())
    });
    obj.function(free_fn, js_string!("free"), 1);

    let address_of_fn = NativeFunction::from_copy_closure(move |_this, args, ctx| {
        let arg = args.first().ok_or_else(|| {
            JsNativeError::error().with_message("buffer argument required")
        })?;
        let obj = arg.as_object().ok_or_else(|| {
            JsNativeError::error().with_message("expected ArrayBuffer or TypedArray")
        })?;
        let buf = JsArrayBuffer::from_object(obj.clone()).map_err(|e| {
            JsNativeError::error().with_message(format!("not a valid ArrayBuffer: {e}"))
        })?;
        let byte_len = buf.byte_length();
        let addr = if let Some(data) = buf.data() {
            let slice: &[u8] = &data;
            let ptr = slice.as_ptr() as usize;
            drop(data);
            ptr
        } else {
            return Err(JsNativeError::error().with_message("ArrayBuffer is detached").into());
        };
        let ptr_obj = create_pointer_object(addr, byte_len, ctx);
        Ok(ptr_obj.into())
    });
    obj.function(address_of_fn, js_string!("addressOf"), 1);

    let errno_fn = NativeFunction::from_copy_closure(|_this, _args, _ctx| {
        Ok(JsValue::from(0i32))
    });
    obj.function(errno_fn, js_string!("errno"), 0);

    let strerror_fn = NativeFunction::from_copy_closure(|_this, args, _ctx| {
        let err = args
            .first()
            .and_then(|v| v.as_number())
            .map(|n| n as i32)
            .unwrap_or(0);
        let msg = if err == 0 {
            "Success".to_string()
        } else {
            format!("Error code: {err}")
        };
        Ok(JsValue::from(js_string!(msg)))
    });
    obj.function(strerror_fn, js_string!("strerror"), 1);
}
