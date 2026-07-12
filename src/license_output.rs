// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

use std::sync::Once;

static LICENSE_PRINTED: Once = Once::new();

pub fn output_license_once() {
    LICENSE_PRINTED.call_once(|| {
        eprintln!(
            "[TT23XR Info] This software uses KossJS (https://github.com/KossJS/) \
            under GNU AGPL v3.0 with additional permissions \"非本软件模块的源代码公开义务例外\" granted by TT23XR Studio."
        );
    });
}
