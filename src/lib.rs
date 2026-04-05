//! KossJS - Embeddable JavaScript runtime via C ABI
//!
//! Provides isolated JS instances that can be created, used, and destroyed
//! from any language that supports C FFI (Java/JNI, Python/ctypes, C++, etc.)

mod runtime;

pub use runtime::*;
