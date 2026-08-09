// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "独立模块闭源组合例外" ("Independent Module Exception for Closed-Source Combinations")

use boa_engine::{
    js_string,
    object::ObjectInitializer,
    property::Attribute,
    Context, JsData, JsError, JsNativeError, JsObject, JsValue, NativeFunction,
};
use boa_gc::{Finalize, Trace};
use std::ffi::c_void;

#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct JsPointer {
    #[unsafe_ignore_trace]
    pub address: usize,
    #[unsafe_ignore_trace]
    pub size: usize,
    #[unsafe_ignore_trace]
    pub instance_ptr: *mut c_void,
}

impl JsPointer {
    pub fn new(address: usize, size: usize, instance_ptr: *mut c_void) -> Self {
        Self { address, size, instance_ptr }
    }
}

fn get_ptr_info(this: &JsValue) -> Result<(usize, usize, *mut c_void), JsError> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::error().with_message("'this' is not a Pointer object"))?;
    let ptr = obj
        .downcast_ref::<JsPointer>()
        .ok_or_else(|| JsError::from(JsNativeError::error().with_message("'this' is not a Pointer object")))?;
    let addr = ptr.address;
    let sz = ptr.size;
    let instance_ptr = ptr.instance_ptr;
    drop(ptr);
    Ok((addr, sz, instance_ptr))
}

fn authorize_pointer(
    this: &JsValue,
    target: &str,
    args: &[JsValue],
    ctx: &mut Context,
) -> Result<(usize, usize, *mut c_void), JsError> {
    let info = get_ptr_info(this)?;
    let instance = unsafe { &*(info.2 as *mut crate::runtime::KossInstance) };
    crate::runtime::authorize_operation(
        instance,
        crate::sandbox::FFI_ALLOC,
        target,
        args,
        ctx,
    )?;
    Ok(info)
}

fn read_bytes(address: usize, offset: isize, len: usize) -> Result<Vec<u8>, JsError> {
    let base = address as isize;
    let target = base.checked_add(offset).ok_or_else(|| {
        JsNativeError::error().with_message("pointer offset overflow")
    })?;
    if target < 0 {
        return Err(JsNativeError::error().with_message("negative pointer offset").into());
    }
    let mut buf = vec![0u8; len];
    unsafe {
        std::ptr::copy_nonoverlapping(target as *const u8, buf.as_mut_ptr(), len);
    }
    Ok(buf)
}

fn write_bytes(address: usize, offset: isize, data: &[u8]) -> Result<(), JsError> {
    let base = address as isize;
    let target = base.checked_add(offset).ok_or_else(|| {
        JsNativeError::error().with_message("pointer offset overflow")
    })?;
    if target < 0 {
        return Err(JsNativeError::error().with_message("negative pointer offset").into());
    }
    unsafe {
        std::ptr::copy_nonoverlapping(data.as_ptr(), target as *mut u8, data.len());
    }
    Ok(())
}

macro_rules! define_read_method {
    ($name:literal, $rust_type:ty, $from_fn:expr) => {
        |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
            let (addr, _, _) = authorize_pointer(_this, concat!("ffi.pointer.", $name), args, ctx)?;
            let offset: isize = args
                .first()
                .and_then(|v| v.as_number())
                .map(|n| n as isize)
                .unwrap_or(0);
            let bytes = read_bytes(addr, offset, std::mem::size_of::<$rust_type>())?;
            let val = $from_fn(bytes);
            Ok(JsValue::from(val))
        }
    };
}

macro_rules! define_write_method {
    ($name:literal, $rust_type:ty, $to_fn:expr) => {
        |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
            let (addr, _, _) = authorize_pointer(_this, concat!("ffi.pointer.", $name), args, ctx)?;
            let offset: isize = if args.len() > 1 {
                args[0].as_number().map(|n| n as isize).unwrap_or(0)
            } else {
                0
            };
            let val_idx = if args.len() > 1 { 1 } else { 0 };
            let val = args
                .get(val_idx)
                .ok_or_else(|| JsNativeError::error().with_message("missing value argument"))?;
            let rust_val: $rust_type = $to_fn(val)?;
            let bytes = rust_val.to_le_bytes().to_vec();
            write_bytes(addr, offset, &bytes)?;
            Ok(JsValue::undefined())
        }
    };
}

pub fn create_pointer_object(
    address: usize,
    size: usize,
    instance_ptr: *mut c_void,
    ctx: &mut Context,
) -> JsObject {
    let address_fn = NativeFunction::from_copy_closure(
        |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
            let (addr, _, _) = authorize_pointer(_this, "ffi.pointer.address", args, ctx)?;
            Ok(JsValue::from(addr as f64))
        },
    )
    .to_js_function(ctx.realm());
    let mut builder =
        ObjectInitializer::with_native_data(JsPointer::new(address, size, instance_ptr), ctx);

    builder.function(
        NativeFunction::from_copy_closure(define_read_method!(
            "readInt8",
            i8,
            |b: Vec<u8>| i8::from_le_bytes(b.try_into().unwrap()) as f64
        )),
        js_string!("readInt8"),
        1,
    );
    builder.function(
        NativeFunction::from_copy_closure(define_write_method!("writeInt8", i8, |v: &JsValue| {
            v.as_number()
                .map(|n| n as i8)
                .ok_or_else(|| JsError::from(JsNativeError::error().with_message("expected number")))
        })),
        js_string!("writeInt8"),
        2,
    );

    builder.function(
        NativeFunction::from_copy_closure(define_read_method!(
            "readUint8",
            u8,
            |b: Vec<u8>| u8::from_le_bytes(b.try_into().unwrap()) as f64
        )),
        js_string!("readUint8"),
        1,
    );
    builder.function(
        NativeFunction::from_copy_closure(define_write_method!("writeUint8", u8, |v: &JsValue| {
            v.as_number()
                .map(|n| n as u8)
                .ok_or_else(|| JsError::from(JsNativeError::error().with_message("expected number")))
        })),
        js_string!("writeUint8"),
        2,
    );

    builder.function(
        NativeFunction::from_copy_closure(define_read_method!(
            "readInt16",
            i16,
            |b: Vec<u8>| i16::from_le_bytes(b.try_into().unwrap()) as f64
        )),
        js_string!("readInt16"),
        1,
    );
    builder.function(
        NativeFunction::from_copy_closure(define_write_method!("writeInt16", i16, |v: &JsValue| {
            v.as_number()
                .map(|n| n as i16)
                .ok_or_else(|| JsError::from(JsNativeError::error().with_message("expected number")))
        })),
        js_string!("writeInt16"),
        2,
    );

    builder.function(
        NativeFunction::from_copy_closure(define_read_method!(
            "readUint16",
            u16,
            |b: Vec<u8>| u16::from_le_bytes(b.try_into().unwrap()) as f64
        )),
        js_string!("readUint16"),
        1,
    );
    builder.function(
        NativeFunction::from_copy_closure(define_write_method!("writeUint16", u16, |v: &JsValue| {
            v.as_number()
                .map(|n| n as u16)
                .ok_or_else(|| JsError::from(JsNativeError::error().with_message("expected number")))
        })),
        js_string!("writeUint16"),
        2,
    );

    builder.function(
        NativeFunction::from_copy_closure(define_read_method!(
            "readInt32",
            i32,
            |b: Vec<u8>| i32::from_le_bytes(b.try_into().unwrap()) as f64
        )),
        js_string!("readInt32"),
        1,
    );
    builder.function(
        NativeFunction::from_copy_closure(define_write_method!("writeInt32", i32, |v: &JsValue| {
            v.as_number()
                .map(|n| n as i32)
                .ok_or_else(|| JsError::from(JsNativeError::error().with_message("expected number")))
        })),
        js_string!("writeInt32"),
        2,
    );

    builder.function(
        NativeFunction::from_copy_closure(define_read_method!(
            "readUint32",
            u32,
            |b: Vec<u8>| u32::from_le_bytes(b.try_into().unwrap()) as f64
        )),
        js_string!("readUint32"),
        1,
    );
    builder.function(
        NativeFunction::from_copy_closure(define_write_method!("writeUint32", u32, |v: &JsValue| {
            v.as_number()
                .map(|n| n as u32)
                .ok_or_else(|| JsError::from(JsNativeError::error().with_message("expected number")))
        })),
        js_string!("writeUint32"),
        2,
    );

    builder.function(
        NativeFunction::from_copy_closure(define_read_method!(
            "readInt64",
            i64,
            |b: Vec<u8>| i64::from_le_bytes(b.try_into().unwrap()) as f64
        )),
        js_string!("readInt64"),
        1,
    );
    builder.function(
        NativeFunction::from_copy_closure(define_write_method!("writeInt64", i64, |v: &JsValue| {
            v.as_number()
                .map(|n| n as i64)
                .ok_or_else(|| JsError::from(JsNativeError::error().with_message("expected number")))
        })),
        js_string!("writeInt64"),
        2,
    );

    builder.function(
        NativeFunction::from_copy_closure(define_read_method!(
            "readUint64",
            u64,
            |b: Vec<u8>| u64::from_le_bytes(b.try_into().unwrap()) as f64
        )),
        js_string!("readUint64"),
        1,
    );
    builder.function(
        NativeFunction::from_copy_closure(define_write_method!("writeUint64", u64, |v: &JsValue| {
            v.as_number()
                .map(|n| n as u64)
                .ok_or_else(|| JsError::from(JsNativeError::error().with_message("expected number")))
        })),
        js_string!("writeUint64"),
        2,
    );

    builder.function(
        NativeFunction::from_copy_closure(define_read_method!(
            "readFloat32",
            f32,
            |b: Vec<u8>| f32::from_le_bytes(b.try_into().unwrap()) as f64
        )),
        js_string!("readFloat32"),
        1,
    );
    builder.function(
        NativeFunction::from_copy_closure(define_write_method!("writeFloat32", f32, |v: &JsValue| {
            v.as_number()
                .map(|n| n as f32)
                .ok_or_else(|| JsError::from(JsNativeError::error().with_message("expected number")))
        })),
        js_string!("writeFloat32"),
        2,
    );

    builder.function(
        NativeFunction::from_copy_closure(define_read_method!(
            "readFloat64",
            f64,
            |b: Vec<u8>| f64::from_le_bytes(b.try_into().unwrap())
        )),
        js_string!("readFloat64"),
        1,
    );
    builder.function(
        NativeFunction::from_copy_closure(define_write_method!("writeFloat64", f64, |v: &JsValue| {
            v.as_number()
                .ok_or_else(|| JsError::from(JsNativeError::error().with_message("expected number")))
        })),
        js_string!("writeFloat64"),
        2,
    );

    builder.function(
        NativeFunction::from_copy_closure(
            |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
                let (addr, _, _) = authorize_pointer(_this, "ffi.pointer.readPointer", args, ctx)?;
                let offset: isize = args
                    .first()
                    .and_then(|v| v.as_number())
                    .map(|n| n as isize)
                    .unwrap_or(0);
                let bytes = read_bytes(addr, offset, std::mem::size_of::<usize>())?;
                let val = usize::from_le_bytes(bytes.try_into().unwrap());
                Ok(JsValue::from(val as f64))
            },
        ),
        js_string!("readPointer"),
        1,
    );
    builder.function(
        NativeFunction::from_copy_closure(
            |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
                let (addr, _, _) = authorize_pointer(_this, "ffi.pointer.writePointer", args, ctx)?;
                let offset: isize = if args.len() > 1 {
                    args[0].as_number().map(|n| n as isize).unwrap_or(0)
                } else {
                    0
                };
                let default_val = JsValue::undefined();
                let val = if args.len() > 1 {
                    &args[1]
                } else {
                    args.first().unwrap_or(&default_val)
                };
                let target_addr: usize = val
                    .as_number()
                    .map(|n| n as usize)
                    .or_else(|| {
                        val.as_object()
                            .and_then(|o| {
                                let p = o.downcast_ref::<JsPointer>();
                                p.map(|p| p.address)
                            })
                    })
                    .ok_or_else(|| {
                        JsNativeError::error().with_message("expected Pointer or number")
                    })?;
                let bytes = target_addr.to_le_bytes().to_vec();
                write_bytes(addr, offset, &bytes)?;
                Ok(JsValue::undefined())
            },
        ),
        js_string!("writePointer"),
        2,
    );

    builder.function(
        NativeFunction::from_copy_closure(
            |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
                let (addr, _, _) = authorize_pointer(_this, "ffi.pointer.readCString", args, ctx)?;
                let offset: isize = args
                    .first()
                    .and_then(|v| v.as_number())
                    .map(|n| n as isize)
                    .unwrap_or(0);
                let target = addr as isize + offset;
                if target < 0 {
                    return Err(
                        JsNativeError::error().with_message("negative pointer address").into()
                    );
                }
                let mut len = 0usize;
                unsafe {
                    let mut p = target as *const u8;
                    while *p != 0 {
                        len += 1;
                        p = p.add(1);
                    }
                }
                let bytes = read_bytes(addr, offset, len)?;
                let s = String::from_utf8_lossy(&bytes).to_string();
                Ok(JsValue::from(js_string!(s)))
            },
        ),
        js_string!("readCString"),
        1,
    );
    builder.function(
        NativeFunction::from_copy_closure(
            |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
                let (addr, _, _) = authorize_pointer(_this, "ffi.pointer.writeCString", args, ctx)?;
                let offset: isize = if args.len() > 1 {
                    args[0].as_number().map(|n| n as isize).unwrap_or(0)
                } else {
                    0
                };
                let default_val = JsValue::undefined();
                let str_val = if args.len() > 1 {
                    &args[1]
                } else {
                    args.first().unwrap_or(&default_val)
                };
                let s = str_val
                    .as_string()
                    .map(|s| s.to_std_string_escaped())
                    .ok_or_else(|| {
                        JsNativeError::error().with_message("expected string")
                    })?;
                let cstr = std::ffi::CString::new(s.as_bytes())
                    .map_err(|e| {
                        JsNativeError::error().with_message(format!("invalid C string: {e}"))
                    })?;
                let bytes = cstr.as_bytes_with_nul();
                write_bytes(addr, offset, bytes)?;
                Ok(JsValue::undefined())
            },
        ),
        js_string!("writeCString"),
        2,
    );

    builder.function(
        NativeFunction::from_copy_closure(
            |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
                let (addr, _, instance_ptr) = authorize_pointer(_this, "ffi.pointer.add", args, ctx)?;
                let offset: isize = args
                    .first()
                    .and_then(|v| v.as_number())
                    .map(|n| n as isize)
                    .unwrap_or(0);
                let new_addr = (addr as isize)
                    .checked_add(offset)
                    .ok_or_else(|| JsNativeError::error().with_message("pointer overflow"))?;
                if new_addr < 0 {
                    return Err(JsNativeError::error().with_message("negative pointer").into());
                }
                let new_ptr = create_pointer_object(new_addr as usize, 0, instance_ptr, ctx);
                Ok(new_ptr.into())
            },
        ),
        js_string!("add"),
        1,
    );

    builder.function(
        NativeFunction::from_copy_closure(
            |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> Result<JsValue, JsError> {
                let (addr, _, _) = authorize_pointer(_this, "ffi.pointer.toBigInt", args, ctx)?;
                Ok(JsValue::from(addr as f64))
            },
        ),
        js_string!("toBigInt"),
        0,
    );

    builder.accessor(
        js_string!("address"),
        Some(address_fn),
        None,
        Attribute::NON_ENUMERABLE,
    );

    builder.build()
}
