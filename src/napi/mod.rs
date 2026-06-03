#![allow(unsafe_op_in_unsafe_fn)]
#![allow(unused)]

pub mod env;
pub mod status;
pub mod value;
pub mod functions;
pub mod async_runtime;

use std::ffi::c_void;

use boa_engine::{js_string, Context, JsError, JsNativeError, JsObject, JsValue, NativeFunction};

use self::env::NapiEnv;
use self::status::NapiStatus;

pub fn register_napi_functions_to_obj(obj: &mut boa_engine::object::ObjectInitializer) {
    macro_rules! reg {
        ($name:expr, $len:expr) => {
            let err_fn = unsafe {
                NativeFunction::from_closure(
                    move |_t: &JsValue, _a: &[JsValue], _c: &mut Context| -> Result<JsValue, JsError> {
                        Err(JsNativeError::error()
                            .with_message(format!("N-API function '{}' is not yet wired to JS side", $name))
                            .into())
                    },
                )
            };
            obj.function(err_fn, js_string!($name), $len);
        };
    }

    reg!("napi_create_number", 3);
    reg!("napi_create_int32", 3);
    reg!("napi_create_uint32", 3);
    reg!("napi_create_int64", 3);
    reg!("napi_create_double", 3);
    reg!("napi_create_string_utf8", 4);
    reg!("napi_create_string_latin1", 4);
    reg!("napi_create_bool", 3);
    reg!("napi_create_null", 2);
    reg!("napi_create_undefined", 2);
    reg!("napi_create_object", 2);
    reg!("napi_create_array", 2);
    reg!("napi_create_array_with_length", 3);
    reg!("napi_create_buffer", 4);
    reg!("napi_create_buffer_copy", 5);
    reg!("napi_create_external", 5);
    reg!("napi_create_function", 5);
    reg!("napi_create_error", 4);
    reg!("napi_create_type_error", 4);
    reg!("napi_create_reference", 4);
    reg!("napi_delete_reference", 2);
    reg!("napi_reference_ref", 3);
    reg!("napi_reference_unref", 3);
    reg!("napi_get_reference_value", 3);
    reg!("napi_get_value_int32", 3);
    reg!("napi_get_value_int64", 3);
    reg!("napi_get_value_double", 3);
    reg!("napi_get_value_bool", 3);
    reg!("napi_get_value_string_utf8", 5);
    reg!("napi_get_buffer_info", 4);
    reg!("napi_get_value_external", 3);
    reg!("napi_set_property", 4);
    reg!("napi_get_property", 4);
    reg!("napi_has_property", 4);
    reg!("napi_set_named_property", 4);
    reg!("napi_get_named_property", 4);
    reg!("napi_define_properties", 4);
    reg!("napi_define_class", 7);
    reg!("napi_wrap", 6);
    reg!("napi_unwrap", 3);
    reg!("napi_instanceof", 4);
    reg!("napi_call_function", 6);
    reg!("napi_get_cb_info", 6);
    reg!("napi_get_new_target", 3);
    reg!("napi_new_instance", 5);
    reg!("napi_throw", 2);
    reg!("napi_throw_error", 3);
    reg!("napi_throw_type_error", 3);
    reg!("napi_is_exception_pending", 2);
    reg!("napi_get_and_clear_last_exception", 2);
    reg!("napi_typeof", 3);
    reg!("napi_strict_equals", 4);
    reg!("napi_get_boolean", 3);
    reg!("napi_get_null", 2);
    reg!("napi_get_undefined", 2);
    reg!("napi_get_global", 2);
    reg!("napi_set_instance_data", 4);
    reg!("napi_get_instance_data", 2);
    reg!("napi_create_async_work", 7);
    reg!("napi_delete_async_work", 2);
    reg!("napi_queue_async_work", 2);
    reg!("napi_open_handle_scope", 2);
    reg!("napi_close_handle_scope", 2);
    reg!("napi_open_callback_scope", 4);
    reg!("napi_close_callback_scope", 2);
    reg!("napi_add_finalizer", 6);
    reg!("napi_fatal_error", 2);
    reg!("napi_adjust_external_memory", 3);
}

pub unsafe fn create_napi_env(ctx: &mut Context) -> Box<NapiEnv> {
    unsafe { env::wrap_env(ctx) }
}

pub fn drain_async_completions() -> Vec<async_runtime::AsyncCompletion> {
    functions::async_work::get_or_drain_async_completions()
}
