/*
 * Copyright (c) 2026 TT23XR Studio
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::ffi::{c_char, CStr};
use std::os::raw::c_int;

#[no_mangle]
pub extern "C" fn add_int(a: c_int, b: c_int) -> c_int {
    a + b
}

#[no_mangle]
pub extern "C" fn multiply_int(a: c_int, b: c_int) -> c_int {
    a * b
}

#[no_mangle]
pub extern "C" fn negate_int(a: c_int) -> c_int {
    -a
}

#[no_mangle]
pub extern "C" fn add_float(a: f64, b: f64) -> f64 {
    a + b
}

#[no_mangle]
pub extern "C" fn multiply_float(a: f64, b: f64) -> f64 {
    a * b
}

#[no_mangle]
pub extern "C" fn is_positive(a: c_int) -> c_int {
    if a > 0 { 1 } else { 0 }
}

#[no_mangle]
pub extern "C" fn is_zero(a: c_int) -> c_int {
    if a == 0 { 1 } else { 0 }
}

#[repr(C)]
pub struct Point {
    pub x: c_int,
    pub y: c_int,
}

#[no_mangle]
pub extern "C" fn point_distance(a: *const Point, b: *const Point) -> f64 {
    unsafe {
        let dx = (*a).x - (*b).x;
        let dy = (*a).y - (*b).y;
        ((dx * dx + dy * dy) as f64).sqrt()
    }
}

#[no_mangle]
pub extern "C" fn point_new(x: c_int, y: c_int) -> Point {
    Point { x, y }
}

#[no_mangle]
pub extern "C" fn increment_point(ptr: *mut Point) {
    unsafe {
        (*ptr).x += 1;
        (*ptr).y += 1;
    }
}

#[no_mangle]
pub extern "C" fn string_length(s: *const c_char) -> c_int {
    unsafe { CStr::from_ptr(s).to_bytes().len() as c_int }
}

#[no_mangle]
pub extern "C" fn string_equals(a: *const c_char, b: *const c_char) -> c_int {
    unsafe {
        if CStr::from_ptr(a) == CStr::from_ptr(b) { 1 } else { 0 }
    }
}

#[no_mangle]
pub extern "C" fn string_uppercase_first(src: *const c_char, dst: *mut c_char, max_len: c_int) -> c_int {
    unsafe {
        let s = match CStr::from_ptr(src).to_str() {
            Ok(s) => s,
            Err(_) => return 0,
        };
        let mut first = true;
        let mut written = 0;
        for ch in s.chars() {
            if written >= (max_len - 1) as usize { break; }
            let c = if first { first = false; ch.to_ascii_uppercase() } else { ch };
            let len = c.len_utf8();
            let buf = &mut [0u8; 4];
            c.encode_utf8(buf);
            for i in 0..len {
                *dst.add(written) = buf[i] as c_char;
                written += 1;
            }
        }
        *dst.add(written) = 0;
        written as c_int
    }
}

pub type CompareFn = unsafe extern "C" fn(*const c_int, *const c_int) -> c_int;

#[no_mangle]
pub extern "C" fn find_max(arr: *const c_int, len: c_int, cmp: CompareFn) -> c_int {
    unsafe {
        if len <= 0 { return 0; }
        let mut best = *arr;
        for i in 1..len {
            let val = *arr.add(i as usize);
            if cmp(&val, &best) > 0 {
                best = val;
            }
        }
        best
    }
}

#[no_mangle]
pub extern "C" fn sum_array(arr: *const c_int, len: c_int) -> c_int {
    unsafe {
        let mut total = 0;
        for i in 0..len {
            total += *arr.add(i as usize);
        }
        total
    }
}

#[no_mangle]
pub extern "C" fn fill_buffer(buf: *mut u8, len: c_int, value: u8) {
    unsafe {
        for i in 0..len {
            *buf.add(i as usize) = value;
        }
    }
}

#[no_mangle]
pub extern "C" fn read_byte_at(buf: *const u8, offset: c_int) -> u8 {
    unsafe { *buf.add(offset as usize) }
}

#[no_mangle]
pub extern "C" fn write_byte_at(buf: *mut u8, offset: c_int, value: u8) {
    unsafe { *buf.add(offset as usize) = value; }
}

#[no_mangle]
pub extern "C" fn test_bigint_ops(a: i64, b: i64) -> i64 {
    a + b
}

#[no_mangle]
pub extern "C" fn test_uint64_max() -> u64 {
    u64::MAX
}

#[no_mangle]
pub extern "C" fn return_null() -> *const c_int {
    std::ptr::null()
}

#[no_mangle]
pub extern "C" fn is_null(ptr: *const c_int) -> c_int {
    if ptr.is_null() { 1 } else { 0 }
}
