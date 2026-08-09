// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "独立模块闭源组合例外" ("Independent Module Exception for Closed-Source Combinations")

use std::sync::Once;

static LICENSE_PRINTED: Once = Once::new();

pub fn output_license_once() {
    LICENSE_PRINTED.call_once(|| {
        eprintln!(
            "[TT23XR Studio Info] This software uses KossJS (https://github.com/KossJS/) \
under GNU AGPL v3.0 with additional permission \"独立模块闭源组合例外\" \
(\"Independent Module Exception for Closed-Source Combinations\") granted by TT23XR Studio."
        );
    });
}
