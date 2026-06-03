use std::fmt;
use std::rc::Rc;

use boa_engine::{js_string, JsError, JsNativeError, JsObject, JsValue};
use libffi::middle;

pub const TYPE_HANDLE_KEY: &str = "__ffi_type_handle__";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FfiType {
    Void,
    Int8,
    Uint8,
    Int16,
    Uint16,
    Int32,
    Uint32,
    Int64,
    Uint64,
    Float32,
    Float64,
    Pointer,
    CString,
    Struct {
        fields: Vec<FieldInfo>,
        packed: Option<u16>,
        size: usize,
        align: usize,
    },
    Array {
        inner: Box<FfiType>,
        count: usize,
    },
    Callback {
        args: Vec<Rc<FfiType>>,
        ret: Box<FfiType>,
    },
    VarArg,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldInfo {
    pub name: String,
    pub offset: usize,
    pub type_info: Rc<FfiType>,
}

impl FfiType {
    pub fn has_callback(&self) -> bool {
        match self {
            FfiType::Callback { .. } => true,
            FfiType::Struct { fields, .. } => fields.iter().any(|f| f.type_info.has_callback()),
            FfiType::Array { inner, .. } => inner.has_callback(),
            _ => false,
        }
    }
}

/// Find all callback argument indices and their type info.
pub fn find_callback_args(types: &[Rc<FfiType>]) -> Vec<(usize, Vec<FfiType>, FfiType)> {
    types
        .iter()
        .enumerate()
        .filter_map(|(i, t)| {
            if let FfiType::Callback { args, ret } = t.as_ref() {
                let owned_args: Vec<FfiType> = args.iter().map(|rc| rc.as_ref().clone()).collect();
                let owned_ret: FfiType = ret.as_ref().clone();
                // Normalize: the callback's args/ret may also contain Rc, deep clone
                Some((i, owned_args, owned_ret))
            } else {
                None
            }
        })
        .collect()
}

/// Deep clone an Rc<FfiType> removing all Rc wrappers (for Send-safe cross-thread transfer).
/// This produces a type description that can be used to call .to_middle_type() in any thread.
pub fn deep_clone_type(t: &FfiType) -> FfiType {
    match t {
        FfiType::Struct { fields, packed, size, align } => FfiType::Struct {
            fields: fields.iter().map(|f| FieldInfo {
                name: f.name.clone(),
                offset: f.offset,
                type_info: Rc::new(deep_clone_type(&f.type_info)),
            }).collect(),
            packed: *packed,
            size: *size,
            align: *align,
        },
        FfiType::Array { inner, count } => FfiType::Array {
            inner: Box::new(deep_clone_type(inner)),
            count: *count,
        },
        FfiType::Callback { args, ret } => FfiType::Callback {
            args: args.iter().map(|rc| Rc::new(deep_clone_type(rc))).collect(),
            ret: Box::new(deep_clone_type(ret)),
        },
        other => other.clone(),
    }
}

/// Deep clone Rc<FfiType> to owned FfiType (removes Rc for cross-thread Send safety).
/// Different from deep_clone_type: this returns FfiType directly; for Callback/Struct
/// variants, converts Vec<Rc<FfiType>> to Vec<Box<FfiType>>.
#[derive(Debug, Clone)]
pub enum OwnedFfiType {
    Void, Int8, Uint8, Int16, Uint16, Int32, Uint32,
    Int64, Uint64, Float32, Float64,
    Pointer, CString, VarArg,
    Struct { fields: Vec<OwnedFieldInfo>, packed: Option<u16>, size: usize, align: usize },
    Array { inner: Box<OwnedFfiType>, count: usize },
    Callback { args: Vec<OwnedFfiType>, ret: Box<OwnedFfiType> },
}

#[derive(Debug, Clone)]
pub struct OwnedFieldInfo {
    pub name: String,
    pub offset: usize,
    pub type_info: OwnedFfiType,
}

impl OwnedFfiType {
    pub fn from_rc(rc_type: &FfiType) -> Self {
        match rc_type {
            FfiType::Void => OwnedFfiType::Void,
            FfiType::Int8 => OwnedFfiType::Int8,
            FfiType::Uint8 => OwnedFfiType::Uint8,
            FfiType::Int16 => OwnedFfiType::Int16,
            FfiType::Uint16 => OwnedFfiType::Uint16,
            FfiType::Int32 => OwnedFfiType::Int32,
            FfiType::Uint32 => OwnedFfiType::Uint32,
            FfiType::Int64 => OwnedFfiType::Int64,
            FfiType::Uint64 => OwnedFfiType::Uint64,
            FfiType::Float32 => OwnedFfiType::Float32,
            FfiType::Float64 => OwnedFfiType::Float64,
            FfiType::Pointer => OwnedFfiType::Pointer,
            FfiType::CString => OwnedFfiType::CString,
            FfiType::VarArg => OwnedFfiType::VarArg,
            FfiType::Struct { fields, packed, size, align } => OwnedFfiType::Struct {
                fields: fields.iter().map(|f| OwnedFieldInfo {
                    name: f.name.clone(),
                    offset: f.offset,
                    type_info: OwnedFfiType::from_rc(&f.type_info),
                }).collect(),
                packed: *packed,
                size: *size,
                align: *align,
            },
            FfiType::Array { inner, count } => OwnedFfiType::Array {
                inner: Box::new(OwnedFfiType::from_rc(inner)),
                count: *count,
            },
            FfiType::Callback { args, ret } => OwnedFfiType::Callback {
                args: args.iter().map(|rc| OwnedFfiType::from_rc(rc)).collect(),
                ret: Box::new(OwnedFfiType::from_rc(ret)),
            },
        }
    }

    pub fn to_middle_type(&self) -> middle::Type {
        match self {
            OwnedFfiType::Void => middle::Type::void(),
            OwnedFfiType::Int8 => middle::Type::i8(),
            OwnedFfiType::Uint8 => middle::Type::u8(),
            OwnedFfiType::Int16 => middle::Type::i16(),
            OwnedFfiType::Uint16 => middle::Type::u16(),
            OwnedFfiType::Int32 => middle::Type::i32(),
            OwnedFfiType::Uint32 => middle::Type::u32(),
            OwnedFfiType::Int64 => middle::Type::i64(),
            OwnedFfiType::Uint64 => middle::Type::u64(),
            OwnedFfiType::Float32 => middle::Type::f32(),
            OwnedFfiType::Float64 => middle::Type::f64(),
            OwnedFfiType::Pointer | OwnedFfiType::CString | OwnedFfiType::Callback { .. } => middle::Type::pointer(),
            OwnedFfiType::Struct { fields, .. } => {
                let field_types: Vec<middle::Type> = fields.iter().map(|f| f.type_info.to_middle_type()).collect();
                middle::Type::structure(field_types)
            }
            OwnedFfiType::Array { inner, .. } => inner.to_middle_type(),
            OwnedFfiType::VarArg => middle::Type::pointer(),
        }
    }

    pub fn sizeof(&self) -> usize {
        match self {
            OwnedFfiType::Void => 0,
            OwnedFfiType::Int8 | OwnedFfiType::Uint8 => 1,
            OwnedFfiType::Int16 | OwnedFfiType::Uint16 => 2,
            OwnedFfiType::Int32 | OwnedFfiType::Uint32 | OwnedFfiType::Float32 => 4,
            OwnedFfiType::Int64 | OwnedFfiType::Uint64 | OwnedFfiType::Float64 => 8,
            OwnedFfiType::Pointer | OwnedFfiType::CString | OwnedFfiType::Callback { .. } => {
                std::mem::size_of::<*const std::ffi::c_void>()
            }
            OwnedFfiType::Struct { size, .. } => *size,
            OwnedFfiType::Array { inner, count } => inner.sizeof() * count,
            OwnedFfiType::VarArg => std::mem::size_of::<*const std::ffi::c_void>(),
        }
    }

    pub fn is_callback(&self) -> bool {
        matches!(self, OwnedFfiType::Callback { .. })
    }
}

impl FfiType {
    pub fn to_middle_type(&self) -> middle::Type {
        match self {
            Self::Void => middle::Type::void(),
            Self::Int8 => middle::Type::i8(),
            Self::Uint8 => middle::Type::u8(),
            Self::Int16 => middle::Type::i16(),
            Self::Uint16 => middle::Type::u16(),
            Self::Int32 => middle::Type::i32(),
            Self::Uint32 => middle::Type::u32(),
            Self::Int64 => middle::Type::i64(),
            Self::Uint64 => middle::Type::u64(),
            Self::Float32 => middle::Type::f32(),
            Self::Float64 => middle::Type::f64(),
            Self::Pointer | Self::CString | Self::Callback { .. } => middle::Type::pointer(),
            Self::Struct { fields, .. } => {
                let field_types: Vec<middle::Type> =
                    fields.iter().map(|f| f.type_info.to_middle_type()).collect();
                middle::Type::structure(field_types)
            }
            Self::Array { inner, .. } => inner.to_middle_type(),
            Self::VarArg => middle::Type::pointer(),
        }
    }

    pub fn sizeof(&self) -> usize {
        match self {
            Self::Void => 0,
            Self::Int8 | Self::Uint8 => 1,
            Self::Int16 | Self::Uint16 => 2,
            Self::Int32 | Self::Uint32 | Self::Float32 => 4,
            Self::Int64 | Self::Uint64 | Self::Float64 => 8,
            Self::Pointer | Self::CString | Self::Callback { .. } => {
                std::mem::size_of::<*const std::ffi::c_void>()
            }
            Self::Struct { size, .. } => *size,
            Self::Array { inner, count } => inner.sizeof() * count,
            Self::VarArg => std::mem::size_of::<*const std::ffi::c_void>(),
        }
    }

    pub fn alignment(&self) -> usize {
        match self {
            Self::Void => 1,
            Self::Int8 | Self::Uint8 => 1,
            Self::Int16 | Self::Uint16 => 2,
            Self::Int32 | Self::Uint32 | Self::Float32 => 4,
            Self::Int64 | Self::Uint64 | Self::Float64 => 8,
            Self::Pointer | Self::CString | Self::Callback { .. } => {
                std::mem::align_of::<*const std::ffi::c_void>()
            }
            Self::Struct { align, .. } => *align,
            Self::Array { inner, .. } => inner.alignment(),
            Self::VarArg => std::mem::align_of::<*const std::ffi::c_void>(),
        }
    }

    pub fn from_type_name(name: &str) -> Option<Self> {
        match name {
            "void" => Some(Self::Void),
            "int8" => Some(Self::Int8),
            "uint8" => Some(Self::Uint8),
            "int16" => Some(Self::Int16),
            "uint16" => Some(Self::Uint16),
            "int32" => Some(Self::Int32),
            "uint32" => Some(Self::Uint32),
            "int64" => Some(Self::Int64),
            "uint64" => Some(Self::Uint64),
            "float32" => Some(Self::Float32),
            "float64" => Some(Self::Float64),
            "pointer" => Some(Self::Pointer),
            "cstring" => Some(Self::CString),
            "..." => Some(Self::VarArg),
            _ => None,
        }
    }

    pub fn to_type_name(&self) -> &'static str {
        match self {
            Self::Void => "void",
            Self::Int8 => "int8",
            Self::Uint8 => "uint8",
            Self::Int16 => "int16",
            Self::Uint16 => "uint16",
            Self::Int32 => "int32",
            Self::Uint32 => "uint32",
            Self::Int64 => "int64",
            Self::Uint64 => "uint64",
            Self::Float32 => "float32",
            Self::Float64 => "float64",
            Self::Pointer => "pointer",
            Self::CString => "cstring",
            Self::Struct { .. } => "struct",
            Self::Array { .. } => "array",
            Self::Callback { .. } => "callback",
            Self::VarArg => "...",
        }
    }
}

impl fmt::Display for FfiType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Struct { fields, .. } => {
                write!(f, "struct {{ ")?;
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", field.name, field.type_info)?;
                }
                write!(f, " }}")
            }
            Self::Array { inner, count } => write!(f, "{}[{}]", inner, count),
            Self::Callback { ret, args } => {
                write!(f, "fn(")?;
                for (i, a) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", a)?;
                }
                write!(f, ") -> {}", ret)
            }
            _ => write!(f, "{}", self.to_type_name()),
        }
    }
}

pub fn type_from_js(val: &JsValue) -> Result<Rc<FfiType>, JsError> {
    if let Some(s) = val.as_string() {
        let s = s.to_std_string_escaped();
        FfiType::from_type_name(&s)
            .map(Rc::new)
            .ok_or_else(|| JsNativeError::error().with_message(format!("unknown type: {s}")).into())
    } else if let Some(obj) = val.as_object() {
        let props = obj.borrow();
        if let Some(type_desc) = props
            .properties()
            .get(&js_string!("type").into())
            .and_then(|desc| desc.value().cloned())
        {
            return type_from_js(&type_desc);
        }
        if let Some(handle) = props
            .properties()
            .get(&js_string!(TYPE_HANDLE_KEY).into())
            .and_then(|desc| desc.value().cloned())
        {
            return type_from_js(&handle);
        }
        Err(JsNativeError::error()
            .with_message("object is not a valid type descriptor (missing __ffi_type_handle__)")
            .into())
    } else {
        Err(JsNativeError::error()
            .with_message("expected a type string or type descriptor object")
            .into())
    }
}

pub fn js_value_to_ffi_type(val: &JsValue) -> Result<FfiType, JsError> {
    let rc = type_from_js(val)?;
    Ok((*rc).clone())
}

pub fn compute_struct_layout(
    fields: &[(String, Rc<FfiType>)],
    packed: Option<u16>,
) -> (Vec<FieldInfo>, usize, usize) {
    let mut field_infos = Vec::with_capacity(fields.len());
    let mut current_offset: usize = 0;
    let mut max_align: usize = 1;

    for (name, type_info) in fields {
        let field_align = if let Some(p) = packed {
            type_info.alignment().min(p as usize)
        } else {
            type_info.alignment()
        };

        if current_offset % field_align != 0 {
            if packed.is_none() {
                current_offset = (current_offset + field_align - 1) / field_align * field_align;
            }
        }

        field_infos.push(FieldInfo {
            name: name.clone(),
            offset: current_offset,
            type_info: Rc::clone(type_info),
        });

        current_offset += type_info.sizeof();
        max_align = max_align.max(field_align);
    }

    let total_size = if packed.is_none() && current_offset % max_align != 0 {
        (current_offset + max_align - 1) / max_align * max_align
    } else {
        current_offset
    };

    (field_infos, total_size, max_align)
}

pub fn store_type_handle(obj: &JsObject, type_info: &Rc<FfiType>, context: &mut boa_engine::Context) {
    let type_val = make_type_descriptor_value(type_info, context);
    obj.insert_property(
        js_string!(TYPE_HANDLE_KEY),
        boa_engine::property::PropertyDescriptor::builder()
            .value(type_val)
            .writable(false)
            .enumerable(false)
            .configurable(false),
    );
}

fn make_type_descriptor_value(type_info: &FfiType, context: &mut boa_engine::Context) -> JsValue {
    match type_info {
        FfiType::Struct { fields, packed, size, align: _ } => {
            let fields_obj = {
                let inner = boa_engine::object::ObjectInitializer::new(context).build();
                for field in fields {
                    let desc = make_type_descriptor_value(&field.type_info, context);
                    inner.insert_property(
                        js_string!(field.name.clone()),
                        boa_engine::property::PropertyDescriptor::builder()
                            .value(desc)
                            .writable(false)
                            .enumerable(true)
                            .configurable(false),
                    );
                }
                inner
            };
            let obj = boa_engine::object::ObjectInitializer::new(context)
                .property(
                    js_string!("type"),
                    js_string!("struct"),
                    boa_engine::property::Attribute::READONLY | boa_engine::property::Attribute::NON_ENUMERABLE,
                )
                .property(
                    js_string!("fields"),
                    fields_obj,
                    boa_engine::property::Attribute::READONLY | boa_engine::property::Attribute::NON_ENUMERABLE,
                )
                .property(
                    js_string!("packed"),
                    if let Some(p) = packed { JsValue::from(*p as f64) } else { JsValue::null() },
                    boa_engine::property::Attribute::READONLY | boa_engine::property::Attribute::NON_ENUMERABLE,
                )
                .property(
                    js_string!("sizeof"),
                    JsValue::from(*size as f64),
                    boa_engine::property::Attribute::READONLY | boa_engine::property::Attribute::NON_ENUMERABLE,
                )
                .build();
            JsValue::from(obj)
        }
        FfiType::Array { inner, count } => {
            let inner_desc = make_type_descriptor_value(inner, context);
            let obj = boa_engine::object::ObjectInitializer::new(context)
                .property(
                    js_string!("type"),
                    js_string!("array"),
                    boa_engine::property::Attribute::READONLY | boa_engine::property::Attribute::NON_ENUMERABLE,
                )
                .property(
                    js_string!("inner"),
                    inner_desc,
                    boa_engine::property::Attribute::READONLY | boa_engine::property::Attribute::NON_ENUMERABLE,
                )
                .property(
                    js_string!("count"),
                    JsValue::from(*count as f64),
                    boa_engine::property::Attribute::READONLY | boa_engine::property::Attribute::NON_ENUMERABLE,
                )
                .build();
            JsValue::from(obj)
        }
        FfiType::Callback { args, ret } => {
            let mut elements: Vec<JsValue> = Vec::with_capacity(args.len());
            for a in args {
                elements.push(make_type_descriptor_value(a, context));
            }
            let args_arr = boa_engine::object::builtins::JsArray::from_iter(elements, context);
            let ret_desc = make_type_descriptor_value(ret, context);
            let obj = boa_engine::object::ObjectInitializer::new(context)
                .property(
                    js_string!("type"),
                    js_string!("callback"),
                    boa_engine::property::Attribute::READONLY | boa_engine::property::Attribute::NON_ENUMERABLE,
                )
                .property(
                    js_string!("args"),
                    args_arr,
                    boa_engine::property::Attribute::READONLY | boa_engine::property::Attribute::NON_ENUMERABLE,
                )
                .property(
                    js_string!("ret"),
                    ret_desc,
                    boa_engine::property::Attribute::READONLY | boa_engine::property::Attribute::NON_ENUMERABLE,
                )
                .build();
            JsValue::from(obj)
        }
        _ => JsValue::from(js_string!(type_info.to_type_name())),
    }
}
