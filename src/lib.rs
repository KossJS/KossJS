//! KossJS - Embeddable JavaScript runtime via C ABI
//!
//! Provides isolated JS instances that can be created, used, and destroyed
//! from any language that supports C FFI (Java/JNI, Python/ctypes, C++, etc.)

#![recursion_limit = "1024"]

pub mod bindings;
pub mod module_loader;
pub mod resolver;
mod runtime;

pub use runtime::*;
