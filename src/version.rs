// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "独立模块闭源组合例外" ("Independent Module Exception for Closed-Source Combinations")

const VERSION:  &[u8] = b"0.1.0-dev.10\0";

pub fn get_version() ->  &'static [u8] {
    return  VERSION;
}
