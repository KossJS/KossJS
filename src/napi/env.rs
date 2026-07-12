// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

use std::cell::RefCell;
use std::ffi::c_void;

use boa_engine::Context;

use super::status::NapiStatus;

#[derive(Debug)]
pub struct NapiEnv {
    pub ctx: *mut Context,
    pub last_error: RefCell<Option<(NapiStatus, String)>>,
    pub instance_data: RefCell<Option<*mut c_void>>,
    pub instance_data_finalize:
        RefCell<Option<unsafe extern "C" fn(env_ptr: *mut NapiEnv, data: *mut c_void, hint: *mut c_void)>>,
    pub instance_data_hint: RefCell<*mut c_void>,
    pub open_handle_scopes: RefCell<u32>,
    pub open_callback_scopes: RefCell<u32>,
    pub async_work_count: RefCell<u32>,
}

impl NapiEnv {
    pub fn new(ctx: *mut Context) -> Self {
        Self {
            ctx,
            last_error: RefCell::new(None),
            instance_data: RefCell::new(None),
            instance_data_finalize: RefCell::new(None),
            instance_data_hint: RefCell::new(std::ptr::null_mut()),
            open_handle_scopes: RefCell::new(0),
            open_callback_scopes: RefCell::new(0),
            async_work_count: RefCell::new(0),
        }
    }

    pub fn set_error(&self, status: NapiStatus, msg: &str) {
        *self.last_error.borrow_mut() = Some((status, msg.to_string()));
    }

    pub fn clear_error(&self) {
        *self.last_error.borrow_mut() = None;
    }

    pub fn take_error(&self) -> Option<(NapiStatus, String)> {
        self.last_error.borrow_mut().take()
    }
}

pub unsafe fn wrap_env(ctx: &mut Context) -> Box<NapiEnv> {
    Box::new(NapiEnv::new(ctx as *mut Context))
}

pub unsafe fn get_context<'a>(env: *mut NapiEnv) -> &'a mut Context {
    unsafe { &mut *(*env).ctx }
}

pub type NapiValue = *mut c_void;

pub type NapiCallback = unsafe extern "C" fn(env: *mut NapiEnv, info: *mut NapiCallbackInfo) -> NapiValue;

#[derive(Debug)]
#[repr(C)]
pub struct NapiCallbackInfo {
    pub env: *mut NapiEnv,
    pub this: NapiValue,
    pub new_target: NapiValue,
    pub argc: usize,
    pub argv: *const NapiValue,
    pub data: *mut c_void,
}

#[derive(Debug)]
#[repr(C)]
pub struct NapiPropertyDescriptor {
    pub utf8name: *const u8,
    pub name: NapiValue,
    pub method: Option<NapiCallback>,
    pub getter: Option<NapiCallback>,
    pub setter: Option<NapiCallback>,
    pub value: NapiValue,
    pub attributes: u32,
    pub data: *mut c_void,
}

pub const NAPI_DEFAULT: u32 = 0;
pub const NAPI_WRITABLE: u32 = 1 << 0;
pub const NAPI_ENUMERABLE: u32 = 1 << 1;
pub const NAPI_CONFIGURABLE: u32 = 1 << 2;
pub const NAPI_STATIC: u32 = 1 << 10;

#[derive(Debug)]
pub struct NapiAsyncWork {
    pub env: *mut NapiEnv,
    pub execute: Option<unsafe extern "C" fn(env: *mut NapiEnv, data: *mut c_void)>,
    pub complete: Option<unsafe extern "C" fn(env: *mut NapiEnv, status: NapiStatus, data: *mut c_void)>,
    pub data: *mut c_void,
    pub result: *mut c_void,
    pub status: NapiStatus,
}
