use std::sync::Once;

static LICENSE_PRINTED: Once = Once::new();

pub fn output_license_once() {
    LICENSE_PRINTED.call_once(|| {
        eprintln!(
            "[TT23XR Info] This software uses KossJS (https://github.com/KossJS/) \
            under GNU AGPL v3.0 with additional permissions granted by TT23XR Studio."
        );
    });
}
