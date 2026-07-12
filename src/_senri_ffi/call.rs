// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

use std::ffi::CString;
use std::rc::Rc;
use std::sync::mpsc;

use boa_engine::{js_string, Context, JsError, JsNativeError, JsValue};
use libffi::low::CodePtr;
use libffi::middle;

use super::pointer::JsPointer;
use super::types::{FfiType, OwnedFfiType};

pub mod async_defs {
    use super::*;

    pub struct FfiCallAsync {
        pub fn_ptr: CodePtr,
        pub vararg_index: Option<usize>,
        pub fixed_arg_count: usize,
        pub arg_types: Vec<OwnedFfiType>,
        pub ret_type: OwnedFfiType,
        pub callback_indices: Vec<usize>,
    }

    unsafe impl Send for FfiCallAsync {}
    unsafe impl Sync for FfiCallAsync {}

    fn js_to_bytes(val: &JsValue, type_info: &FfiType) -> Result<Vec<u8>, JsError> {
        match type_info {
            FfiType::Void => Ok(Vec::new()),
            FfiType::Int8 => {
                let v = val.as_number().map(|n| n as i8)
                    .ok_or_else(|| JsNativeError::error().with_message("expected number (int8)"))?;
                Ok(v.to_le_bytes().to_vec())
            }
            FfiType::Uint8 => {
                let v = val.as_number().map(|n| n as u8)
                    .ok_or_else(|| JsNativeError::error().with_message("expected number (uint8)"))?;
                Ok(v.to_le_bytes().to_vec())
            }
            FfiType::Int16 => {
                let v = val.as_number().map(|n| n as i16)
                    .ok_or_else(|| JsNativeError::error().with_message("expected number (int16)"))?;
                Ok(v.to_le_bytes().to_vec())
            }
            FfiType::Uint16 => {
                let v = val.as_number().map(|n| n as u16)
                    .ok_or_else(|| JsNativeError::error().with_message("expected number (uint16)"))?;
                Ok(v.to_le_bytes().to_vec())
            }
            FfiType::Int32 => {
                let v = val.as_number().map(|n| n as i32)
                    .ok_or_else(|| JsNativeError::error().with_message("expected number (int32)"))?;
                Ok(v.to_le_bytes().to_vec())
            }
            FfiType::Uint32 => {
                let v = val.as_number().map(|n| n as u32)
                    .ok_or_else(|| JsNativeError::error().with_message("expected number (uint32)"))?;
                Ok(v.to_le_bytes().to_vec())
            }
            FfiType::Int64 => {
                let v = val.as_number().map(|n| n as i64)
                    .ok_or_else(|| JsNativeError::error().with_message("expected number (int64)"))?;
                Ok(v.to_le_bytes().to_vec())
            }
            FfiType::Uint64 => {
                let v = val.as_number().map(|n| n as u64)
                    .ok_or_else(|| JsNativeError::error().with_message("expected number (uint64)"))?;
                Ok(v.to_le_bytes().to_vec())
            }
            FfiType::Float32 => {
                let v = val.as_number().map(|n| n as f32)
                    .ok_or_else(|| JsNativeError::error().with_message("expected number (float32)"))?;
                Ok(v.to_le_bytes().to_vec())
            }
            FfiType::Float64 => {
                let v = val.as_number()
                    .ok_or_else(|| JsNativeError::error().with_message("expected number (float64)"))?;
                Ok(v.to_le_bytes().to_vec())
            }
            FfiType::Pointer | FfiType::Callback { .. } => {
                let addr = if let Some(obj) = val.as_object() {
                    if let Some(ptr_data) = obj.downcast_ref::<JsPointer>() {
                        ptr_data.address as usize
                    } else {
                        return Err(JsNativeError::error()
                            .with_message("expected Pointer object or number").into());
                    }
                } else if let Some(n) = val.as_number() {
                    n as usize
                } else {
                    return Err(JsNativeError::error()
                        .with_message("expected Pointer object or number").into());
                };
                Ok(addr.to_le_bytes().to_vec())
            }
            FfiType::CString => Err(JsNativeError::error()
                .with_message("CString must be handled with external lifetime").into()),
            FfiType::Struct { size, .. } => {
                if let Some(obj) = val.as_object() {
                    let mut buf = vec![0u8; *size];
                    let props = obj.borrow();
                    let pn = js_string!("_ffi_buffer");
                    let pk: boa_engine::property::PropertyKey = pn.into();
                    if let Some(desc) = props.properties().get(&pk) {
                        if let Some(buffer_val) = desc.value() {
                            if let Some(buffer_obj) = buffer_val.as_object() {
                                let buffer_ref = buffer_obj.borrow();
                                let bp = js_string!("_ffi_buffer");
                                let bpk: boa_engine::property::PropertyKey = bp.into();
                                if let Some(bdesc) = buffer_ref.properties().get(&bpk) {
                                    if let Some(data_val) = bdesc.value() {
                                        if let Some(data_obj) = data_val.as_object() {
                                            if let Some(ptr_info) = data_obj.downcast_ref::<JsPointer>() {
                                                unsafe {
                                                    std::ptr::copy_nonoverlapping(
                                                        ptr_info.address as *const u8,
                                                        buf.as_mut_ptr(), *size,
                                                    );
                                                }
                                                return Ok(buf);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    return Err(JsNativeError::error()
                        .with_message("expected a struct instance with _ffi_buffer").into());
                }
                Err(JsNativeError::error().with_message("expected a struct instance (object)").into())
            }
            FfiType::Array { inner, count } => {
                let elem_size = inner.sizeof();
                let total = elem_size * *count;
                let mut buf = vec![0u8; total];
                if let Some(obj) = val.as_object() {
                    let mut offset: usize = 0;
                    for i in 0..*count {
                        let key: boa_engine::property::PropertyKey = boa_engine::js_string!(i.to_string()).into();
                        let props = obj.borrow();
                        if let Some(desc) = props.properties().get(&key) {
                            if let Some(elem_val) = desc.value() {
                                let elem_bytes = js_to_bytes(elem_val, inner)?;
                                let len = elem_bytes.len().min(elem_size);
                                buf[offset..offset + len].copy_from_slice(&elem_bytes[..len]);
                            }
                        }
                        offset += elem_size;
                    }
                    Ok(buf)
                } else {
                    Ok(buf)
                }
            }
            FfiType::VarArg => {
                let addr = if let Some(obj) = val.as_object() {
                    if let Some(ptr_data) = obj.downcast_ref::<JsPointer>() {
                        ptr_data.address as usize
                    } else {
                        val.as_number().map(|n| n as usize).unwrap_or(0)
                    }
                } else if let Some(n) = val.as_number() {
                    n as usize
                } else {
                    val.as_string().map(|_| 0usize).unwrap_or(0)
                };
                Ok(addr.to_le_bytes().to_vec())
            }
        }
    }

    /// Send-safe: extract CString from JsValue to bytes (main thread only)
    pub fn js_arg_to_bytes_send(val: &JsValue, type_info: &FfiType, is_callback: bool) -> Result<Vec<u8>, JsError> {
        if is_callback {
            let addr = if let Some(obj) = val.as_object() {
                if let Some(ptr_data) = obj.downcast_ref::<JsPointer>() {
                    ptr_data.address as usize
                } else {
                    return Err(JsNativeError::error()
                        .with_message("callback argument must be a pointer (use createCallback)").into());
                }
            } else if let Some(n) = val.as_number() {
                n as usize
            } else {
                return Err(JsNativeError::error()
                    .with_message("callback argument must be a pointer").into());
            };
            return Ok(addr.to_le_bytes().to_vec());
        }

        if matches!(type_info, FfiType::CString) {
            if val.is_null() || val.is_undefined() {
                return Ok(0usize.to_le_bytes().to_vec());
            }
            let s = val.as_string().map(|s| s.to_std_string_escaped())
                .ok_or_else(|| JsNativeError::error().with_message("expected string for CString"))?;
            let cstr = CString::new(s.as_bytes())
                .map_err(|e| JsNativeError::error().with_message(format!("invalid C string: {e}")))?;
            let ptr = cstr.into_raw() as usize;
            return Ok(ptr.to_le_bytes().to_vec());
        }

        js_to_bytes(val, type_info)
    }

    /// Execute an FFI call in the blocking thread.
    /// Callback arguments are replaced with libffi Closures that proxy through callback_tx.
    pub(crate) fn invoke_ffi_call_async(
        data: &FfiCallAsync,
        arg_buffers: &[Vec<u8>],
        task_id: u64,
        callback_tx: &mpsc::Sender<crate::runtime::CallbackRequest>,
        callback_timeout_ms: u64,
    ) -> Result<String, String> {
        let middle_arg_types: Vec<middle::Type> = data.arg_types.iter()
            .map(|t| t.to_middle_type())
            .collect();
        let middle_ret_type = data.ret_type.to_middle_type();

        #[cfg(not(all(windows, target_arch = "x86")))]
        let abi = middle::ffi_abi_FFI_DEFAULT_ABI;
        #[cfg(all(windows, target_arch = "x86"))]
        let abi = middle::ffi_abi_FFI_DEFAULT_ABI;

        let cif = if let Some(fixed_count) = data.vararg_index {
            middle::Cif::new_variadic_with_abi(middle_arg_types, fixed_count, middle_ret_type, abi)
        } else {
            middle::Cif::new_with_abi(middle_arg_types, middle_ret_type, abi)
        };

        let mut closures: Vec<middle::Closure> = Vec::new();
        let mut adjusted_buffers: Vec<Vec<u8>> = Vec::with_capacity(arg_buffers.len());

        for (i, buf) in arg_buffers.iter().enumerate() {
            if data.callback_indices.contains(&i) {
                let cb_index = i;
                let cb_type = match &data.arg_types[i] {
                    OwnedFfiType::Callback { args, ret } => (args.clone(), ret.clone()),
                    _ => return Err(format!("callback at index {i} is not a Callback type")),
                };
                let (cb_args, cb_ret) = cb_type;
                let tx = callback_tx.clone();
                let tid = task_id;

                let proxy = create_channel_callback_closure(cb_args, *cb_ret, tx, tid, cb_index, callback_timeout_ms)?;
                let code_ptr = *proxy.code_ptr() as usize;
                closures.push(proxy);
                let addr_bytes = code_ptr.to_le_bytes().to_vec();
                adjusted_buffers.push(addr_bytes);
            } else {
                adjusted_buffers.push(buf.clone());
            }
        }

        let args_slice: Vec<middle::Arg> = adjusted_buffers
            .iter()
            .map(|buf| middle::arg(buf.as_slice()))
            .collect();

        let result = unsafe {
            match &data.ret_type {
                OwnedFfiType::Void => {
                    cif.call::<()>(data.fn_ptr, &args_slice);
                    "null".to_string()
                }
                OwnedFfiType::Int8 => format!("{}", cif.call::<i8>(data.fn_ptr, &args_slice)),
                OwnedFfiType::Uint8 => format!("{}", cif.call::<u8>(data.fn_ptr, &args_slice)),
                OwnedFfiType::Int16 => format!("{}", cif.call::<i16>(data.fn_ptr, &args_slice)),
                OwnedFfiType::Uint16 => format!("{}", cif.call::<u16>(data.fn_ptr, &args_slice)),
                OwnedFfiType::Int32 => format!("{}", cif.call::<i32>(data.fn_ptr, &args_slice)),
                OwnedFfiType::Uint32 => format!("{}", cif.call::<u32>(data.fn_ptr, &args_slice)),
                OwnedFfiType::Int64 => format!("{}", cif.call::<i64>(data.fn_ptr, &args_slice)),
                OwnedFfiType::Uint64 => format!("{}", cif.call::<u64>(data.fn_ptr, &args_slice)),
                OwnedFfiType::Float32 => format!("{}", cif.call::<f32>(data.fn_ptr, &args_slice)),
                OwnedFfiType::Float64 => format!("{}", cif.call::<f64>(data.fn_ptr, &args_slice)),
                OwnedFfiType::Pointer | OwnedFfiType::Callback { .. } => {
                    let addr: usize = cif.call::<usize>(data.fn_ptr, &args_slice);
                    format!("{addr}")
                }
                OwnedFfiType::CString => {
                    let addr: usize = cif.call::<usize>(data.fn_ptr, &args_slice);
                    if addr == 0 {
                        "null".to_string()
                    } else {
                        let cstr = std::ffi::CStr::from_ptr(addr as *const std::ffi::c_char);
                        let s = cstr.to_string_lossy();
                        serde_json::to_string(&*s).unwrap_or_else(|_| "\"\"".to_string())
                    }
                }
                OwnedFfiType::VarArg => {
                    let addr: usize = cif.call::<usize>(data.fn_ptr, &args_slice);
                    format!("{addr}")
                }
                OwnedFfiType::Struct { size, .. } => {
                    let mut ret_buffer = vec![0u8; if *size == 0 { 1 } else { *size }];
                    cif.call_return_into(data.fn_ptr, &args_slice, middle::ret(&mut ret_buffer));
                    "\"[binary data]\"".to_string()
                }
                OwnedFfiType::Array { inner, count } => {
                    let mut ret_buffer = vec![0u8; inner.sizeof() * count];
                    cif.call_return_into(data.fn_ptr, &args_slice, middle::ret(&mut ret_buffer));
                    "\"[binary data]\"".to_string()
                }
            }
        };

        Ok(result)
    }

    struct ChannelCallbackData {
        arg_count: usize,
        arg_types: Vec<OwnedFfiType>,
        ret_type: OwnedFfiType,
        callback_tx: mpsc::Sender<crate::runtime::CallbackRequest>,
        task_id: u64,
        cb_index: usize,
        ret_size: usize,
    }

    unsafe extern "C" fn channel_cb_trampoline<R>(
        _cif: &libffi::low::ffi_cif,
        result: &mut R,
        args: *const *const std::ffi::c_void,
        userdata: &ChannelCallbackData,
    ) {
        let d = userdata;
        let mut cb_args: Vec<Vec<u8>> = Vec::with_capacity(d.arg_count);
        for i in 0..d.arg_count {
            let arg_ptr = unsafe { *args.add(i) as *const u8 };
            let size = d.arg_types[i].sizeof();
            if size == 0 {
                cb_args.push(Vec::new());
                continue;
            }
            if arg_ptr.is_null() {
                cb_args.push(vec![0u8; size]);
                continue;
            }
            let mut buf = vec![0u8; size];
            unsafe {
                std::ptr::copy_nonoverlapping(arg_ptr, buf.as_mut_ptr(), size);
            }
            cb_args.push(buf);
        }

        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        let req = crate::runtime::CallbackRequest {
            task_id: d.task_id,
            cb_index: d.cb_index,
            args: cb_args,
            arg_types: d.arg_types.clone(),
            ret_type: d.ret_type.clone(),
            resp_tx,
        };

        if d.callback_tx.send(req).is_err() {
            if d.ret_size > 0 {
                unsafe {
                    std::ptr::write_bytes(result as *mut R as *mut u8, 0u8, d.ret_size);
                }
            }
            return;
        }

        let response = match resp_rx.blocking_recv() {
            Ok(Ok(data)) => data,
            _ => vec![0u8; d.ret_size],
        };

        if d.ret_size > 0 {
            let copy_len = response.len().min(d.ret_size);
            if copy_len > 0 {
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        response.as_ptr(),
                        result as *mut R as *mut u8,
                        copy_len,
                    );
                }
            }
        }
    }

    fn create_channel_callback_closure(
        arg_types: Vec<OwnedFfiType>,
        ret_type: OwnedFfiType,
        callback_tx: mpsc::Sender<crate::runtime::CallbackRequest>,
        task_id: u64,
        cb_index: usize,
        _timeout_ms: u64,
    ) -> Result<middle::Closure<'static>, String> {
        let middle_arg_types: Vec<middle::Type> = arg_types.iter().map(|t| t.to_middle_type()).collect();
        let middle_ret_type = ret_type.to_middle_type();
        let cif = middle::Cif::new(middle_arg_types, middle_ret_type);

        let ret_size = if ret_type.sizeof() == 0 { 1 } else { ret_type.sizeof() };

        let data = Box::new(ChannelCallbackData {
            arg_count: arg_types.len(),
            arg_types,
            ret_type,
            callback_tx,
            task_id,
            cb_index,
            ret_size,
        });
        let data_ref: &'static ChannelCallbackData = Box::leak(data);

        Ok(middle::Closure::new(cif, channel_cb_trampoline::<*mut std::ffi::c_void>, data_ref))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallingConvention {
    Cdecl,
    Stdcall,
    Fastcall,
    Thiscall,
}

impl CallingConvention {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "stdcall" => Self::Stdcall,
            "fastcall" => Self::Fastcall,
            "thiscall" => Self::Thiscall,
            _ => Self::Cdecl,
        }
    }

    pub fn to_ffi_abi(&self) -> libffi::middle::FfiAbi {
        match self {
            Self::Cdecl => libffi::middle::ffi_abi_FFI_DEFAULT_ABI,
            Self::Stdcall => {
                #[cfg(all(windows, target_arch = "x86"))]
                {
                    libffi::middle::ffi_abi_FFI_DEFAULT_ABI
                }
                #[cfg(not(all(windows, target_arch = "x86")))]
                {
                    libffi::middle::ffi_abi_FFI_DEFAULT_ABI
                }
            }
            Self::Fastcall | Self::Thiscall => {
                libffi::middle::ffi_abi_FFI_DEFAULT_ABI
            }
        }
    }
}

pub struct FfiCall {
    pub cif: middle::Cif,
    pub fn_ptr: CodePtr,
    pub ret_type: Rc<FfiType>,
    pub arg_types: Vec<Rc<FfiType>>,
    pub conv: CallingConvention,
    pub vararg_index: Option<usize>,
}

fn js_to_buffer(val: &JsValue, type_info: &FfiType) -> Result<Vec<u8>, JsError> {
    match type_info {
        FfiType::Void => Ok(Vec::new()),
        FfiType::Int8 => {
            let v = val
                .as_number()
                .map(|n| n as i8)
                .ok_or_else(|| JsNativeError::error().with_message("expected number (int8)"))?;
            Ok(v.to_le_bytes().to_vec())
        }
        FfiType::Uint8 => {
            let v = val
                .as_number()
                .map(|n| n as u8)
                .ok_or_else(|| JsNativeError::error().with_message("expected number (uint8)"))?;
            Ok(v.to_le_bytes().to_vec())
        }
        FfiType::Int16 => {
            let v = val
                .as_number()
                .map(|n| n as i16)
                .ok_or_else(|| JsNativeError::error().with_message("expected number (int16)"))?;
            Ok(v.to_le_bytes().to_vec())
        }
        FfiType::Uint16 => {
            let v = val
                .as_number()
                .map(|n| n as u16)
                .ok_or_else(|| JsNativeError::error().with_message("expected number (uint16)"))?;
            Ok(v.to_le_bytes().to_vec())
        }
        FfiType::Int32 => {
            let v = val
                .as_number()
                .map(|n| n as i32)
                .ok_or_else(|| JsNativeError::error().with_message("expected number (int32)"))?;
            Ok(v.to_le_bytes().to_vec())
        }
        FfiType::Uint32 => {
            let v = val
                .as_number()
                .map(|n| n as u32)
                .ok_or_else(|| JsNativeError::error().with_message("expected number (uint32)"))?;
            Ok(v.to_le_bytes().to_vec())
        }
        FfiType::Int64 => {
            let v = val
                .as_number()
                .map(|n| n as i64)
                .ok_or_else(|| JsNativeError::error().with_message("expected number (int64)"))?;
            Ok(v.to_le_bytes().to_vec())
        }
        FfiType::Uint64 => {
            let v = val
                .as_number()
                .map(|n| n as u64)
                .ok_or_else(|| JsNativeError::error().with_message("expected number (uint64)"))?;
            Ok(v.to_le_bytes().to_vec())
        }
        FfiType::Float32 => {
            let v = val
                .as_number()
                .map(|n| n as f32)
                .ok_or_else(|| JsNativeError::error().with_message("expected number (float32)"))?;
            Ok(v.to_le_bytes().to_vec())
        }
        FfiType::Float64 => {
            let v = val
                .as_number()
                .ok_or_else(|| {
                    JsNativeError::error().with_message("expected number (float64)")
                })?;
            Ok(v.to_le_bytes().to_vec())
        }
        FfiType::Pointer | FfiType::Callback { .. } => {
            let addr = if let Some(obj) = val.as_object() {
                if let Some(ptr_data) = obj.downcast_ref::<JsPointer>() {
                    ptr_data.address as usize
                } else {
                    return Err(JsNativeError::error()
                        .with_message("expected Pointer object or number")
                        .into());
                }
            } else if let Some(n) = val.as_number() {
                n as usize
            } else {
                return Err(JsNativeError::error()
                    .with_message("expected Pointer object or number")
                    .into());
            };
            Ok(addr.to_le_bytes().to_vec())
        }
        FfiType::CString => Err(JsNativeError::error()
            .with_message("CString must be handled with external lifetime")
            .into()),
        FfiType::Struct { size, .. } => {
            if let Some(obj) = val.as_object() {
                let mut buf = vec![0u8; *size];
                let props = obj.borrow();
                let pn = js_string!("_ffi_buffer");
                let pk: boa_engine::property::PropertyKey = pn.into();
                if let Some(desc) = props.properties().get(&pk) {
                    if let Some(buffer_val) = desc.value() {
                        if let Some(buffer_obj) = buffer_val.as_object() {
                            let buffer_ref = buffer_obj.borrow();
                            let bp = js_string!("_ffi_buffer");
                            let bpk: boa_engine::property::PropertyKey = bp.into();
                            if let Some(bdesc) = buffer_ref.properties().get(&bpk) {
                                if let Some(data_val) = bdesc.value() {
                                    if let Some(data_obj) = data_val.as_object() {
                                        if let Some(ptr_info) =
                                            data_obj.downcast_ref::<JsPointer>()
                                        {
                                            unsafe {
                                                std::ptr::copy_nonoverlapping(
                                                    ptr_info.address as *const u8,
                                                    buf.as_mut_ptr(),
                                                    *size,
                                                );
                                            }
                                            return Ok(buf);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                return Err(JsNativeError::error()
                    .with_message("expected a struct instance with _ffi_buffer")
                    .into());
            }
            Err(JsNativeError::error()
                .with_message("expected a struct instance (object)")
                .into())
        }
        FfiType::Array { inner, count } => {
            let elem_size = inner.sizeof();
            let total = elem_size * *count;
            let mut buf = vec![0u8; total];
            if let Some(obj) = val.as_object() {
                let mut offset: usize = 0;
                for i in 0..*count {
                    let key: boa_engine::property::PropertyKey =
                        boa_engine::js_string!(i.to_string()).into();
                    let props = obj.borrow();
                    if let Some(desc) = props.properties().get(&key) {
                        if let Some(elem_val) = desc.value() {
                            let elem_bytes = js_to_buffer(elem_val, inner)?;
                            let len = elem_bytes.len().min(elem_size);
                            buf[offset..offset + len].copy_from_slice(&elem_bytes[..len]);
                        }
                    }
                    offset += elem_size;
                }
                Ok(buf)
            } else {
                Ok(buf)
            }
        }
        FfiType::VarArg => {
            let addr = if let Some(obj) = val.as_object() {
                if let Some(ptr_data) = obj.downcast_ref::<JsPointer>() {
                    ptr_data.address as usize
                } else {
                    val.as_number()
                        .map(|n| n as usize)
                        .unwrap_or(0)
                }
            } else if let Some(n) = val.as_number() {
                n as usize
            } else {
                val.as_string()
                    .map(|_| 0usize)
                    .unwrap_or(0)
            };
            Ok(addr.to_le_bytes().to_vec())
        }
    }
}

pub fn invoke_ffi_call(
    data: &FfiCall,
    args: &[JsValue],
    _ctx: &mut Context,
) -> Result<JsValue, JsError> {
    if data.vararg_index.is_none() && args.len() != data.arg_types.len() {
        return Err(JsNativeError::error()
            .with_message(format!(
                "expected {} arguments, got {}",
                data.arg_types.len(),
                args.len()
            ))
            .into());
    }

    if let Some(fixed) = data.vararg_index {
        if args.len() < fixed {
            return Err(JsNativeError::error()
                .with_message(format!(
                    "expected at least {fixed} fixed arguments, got {}",
                    args.len()
                ))
                .into());
        }
    }

    let mut buffers: Vec<Vec<u8>> = Vec::with_capacity(args.len());
    let mut cstrings: Vec<CString> = Vec::new();

    for (i, arg) in args.iter().enumerate() {
        let type_info = if let Some(fixed) = data.vararg_index {
            if i < fixed {
                &data.arg_types[i]
            } else {
                &FfiType::VarArg
            }
        } else {
            &data.arg_types[i]
        };

        if matches!(type_info, FfiType::CString) {
            if arg.is_null() || arg.is_undefined() {
                let null_bytes = 0usize.to_le_bytes().to_vec();
                buffers.push(null_bytes);
            } else {
                let s = arg
                    .as_string()
                    .map(|s| s.to_std_string_escaped())
                    .ok_or_else(|| {
                        JsNativeError::error().with_message("expected string for CString")
                    })?;
                let cstr = CString::new(s.as_bytes()).map_err(|e| {
                    JsNativeError::error().with_message(format!("invalid C string: {e}"))
                })?;
                let ptr = cstr.as_ptr() as usize;
                cstrings.push(cstr);
                buffers.push(ptr.to_le_bytes().to_vec());
            }
        } else {
            let buf = js_to_buffer(arg, type_info)?;
            buffers.push(buf);
        }
    }

    let args_slice: Vec<middle::Arg> = buffers
        .iter()
        .map(|buf| middle::arg(buf.as_slice()))
        .collect();

    let result = unsafe {
        match &*data.ret_type {
            FfiType::Void => {
                data.cif.call::<()>(data.fn_ptr, &args_slice);
                JsValue::undefined()
            }
            FfiType::Int8 => JsValue::from(data.cif.call::<i8>(data.fn_ptr, &args_slice) as f64),
            FfiType::Uint8 => JsValue::from(data.cif.call::<u8>(data.fn_ptr, &args_slice) as f64),
            FfiType::Int16 => JsValue::from(data.cif.call::<i16>(data.fn_ptr, &args_slice) as f64),
            FfiType::Uint16 => JsValue::from(data.cif.call::<u16>(data.fn_ptr, &args_slice) as f64),
            FfiType::Int32 => JsValue::from(data.cif.call::<i32>(data.fn_ptr, &args_slice) as f64),
            FfiType::Uint32 => JsValue::from(data.cif.call::<u32>(data.fn_ptr, &args_slice) as f64),
            FfiType::Int64 => JsValue::from(data.cif.call::<i64>(data.fn_ptr, &args_slice) as f64),
            FfiType::Uint64 => JsValue::from(data.cif.call::<u64>(data.fn_ptr, &args_slice) as f64),
            FfiType::Float32 => JsValue::from(data.cif.call::<f32>(data.fn_ptr, &args_slice) as f64),
            FfiType::Float64 => JsValue::from(data.cif.call::<f64>(data.fn_ptr, &args_slice)),
            FfiType::Pointer | FfiType::Callback { .. } => {
                let addr: usize = data.cif.call::<usize>(data.fn_ptr, &args_slice);
                JsValue::from(addr as f64)
            }
            FfiType::CString => {
                let addr: usize = data.cif.call::<usize>(data.fn_ptr, &args_slice);
                if addr == 0 {
                    JsValue::null()
                } else {
                    let cstr = std::ffi::CStr::from_ptr(addr as *const std::ffi::c_char);
                    let s = cstr.to_string_lossy().to_string();
                    JsValue::from(js_string!(s))
                }
            }
            FfiType::Struct { size, .. } => {
                let mut ret_buffer = vec![0u8; *size];
                data.cif.call_return_into(data.fn_ptr, &args_slice, middle::ret(&mut ret_buffer));
                JsValue::from(js_string!("[binary data]"))
            }
            FfiType::Array { inner, count } => {
                let mut ret_buffer = vec![0u8; inner.sizeof() * count];
                data.cif.call_return_into(data.fn_ptr, &args_slice, middle::ret(&mut ret_buffer));
                JsValue::from(js_string!("[binary data]"))
            }
            FfiType::VarArg => {
                let addr: usize = data.cif.call::<usize>(data.fn_ptr, &args_slice);
                JsValue::from(addr as f64)
            }
        }
    };

    Ok(result)
}
