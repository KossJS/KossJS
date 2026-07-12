// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

#![allow(unsafe_op_in_unsafe_fn)]

use std::collections::VecDeque;
use std::ffi::c_void;
use std::sync::{Arc, Mutex};
use std::thread;

use super::env::NapiEnv;
use super::status::NapiStatus;

#[derive(Debug)]
pub struct AsyncCompletion {
    pub env: *mut NapiEnv,
    pub complete: unsafe extern "C" fn(env: *mut NapiEnv, status: NapiStatus, data: *mut c_void),
    pub data: *mut c_void,
    pub status: NapiStatus,
}

unsafe impl Send for AsyncCompletion {}
unsafe impl Sync for AsyncCompletion {}

#[derive(Debug, Clone)]
pub struct AsyncRuntime {
    pub completions: Arc<Mutex<VecDeque<AsyncCompletion>>>,
}

impl AsyncRuntime {
    pub fn new() -> Self {
        Self {
            completions: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn spawn_async_work_safe(
        completions: Arc<Mutex<VecDeque<AsyncCompletion>>>,
        env: usize,
        execute: usize,
        complete: usize,
        data: usize,
    ) {
        let execute_fn: unsafe extern "C" fn(*mut NapiEnv, *mut c_void) = unsafe { std::mem::transmute(execute) };
        let complete_fn: unsafe extern "C" fn(*mut NapiEnv, NapiStatus, *mut c_void) = unsafe { std::mem::transmute(complete) };

        thread::spawn(move || {
            unsafe {
                execute_fn(env as *mut NapiEnv, data as *mut c_void);
            }

            let completion = AsyncCompletion {
                env: env as *mut NapiEnv,
                complete: complete_fn,
                data: data as *mut c_void,
                status: NapiStatus::Ok,
            };

            if let Ok(mut q) = completions.lock() {
                q.push_back(completion);
            }
        });
    }

    pub fn drain_completions(&self) -> Vec<AsyncCompletion> {
        let mut result = Vec::new();
        if let Ok(mut q) = self.completions.lock() {
            while let Some(c) = q.pop_front() {
                result.push(c);
            }
        }
        result
    }
}
