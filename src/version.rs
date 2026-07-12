// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

const VERSION:  &[u8] = b"0.1.0-dev.9\0";

pub fn get_version() ->  &'static [u8] {
    return  VERSION;
}
