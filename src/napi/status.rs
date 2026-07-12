// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum NapiStatus {
    Ok = 0,
    InvalidArg = 1,
    ObjectExpected = 2,
    StringExpected = 3,
    NameExpected = 4,
    FunctionExpected = 5,
    NumberExpected = 6,
    BooleanExpected = 7,
    ArrayExpected = 8,
    GenericFailure = 9,
    PendingException = 10,
    Cancelled = 11,
    EscapeCalledTwice = 12,
    HandleScopeMismatch = 13,
    CallbackScopeMismatch = 14,
    QueueFull = 15,
    Closing = 16,
    BigintExpected = 17,
    DateExpected = 18,
    ArrayBufferExpected = 19,
    DetachableArraybufferExpected = 20,
    WouldDeadlock = 21,
    NoExternalBuffersAllowed = 22,
    InvalidState = 23,
}

impl NapiStatus {
    pub fn from_i32(v: i32) -> Self {
        match v {
            0 => Self::Ok,
            1 => Self::InvalidArg,
            2 => Self::ObjectExpected,
            3 => Self::StringExpected,
            4 => Self::NameExpected,
            5 => Self::FunctionExpected,
            6 => Self::NumberExpected,
            7 => Self::BooleanExpected,
            8 => Self::ArrayExpected,
            9 => Self::GenericFailure,
            10 => Self::PendingException,
            11 => Self::Cancelled,
            12 => Self::EscapeCalledTwice,
            13 => Self::HandleScopeMismatch,
            14 => Self::CallbackScopeMismatch,
            15 => Self::QueueFull,
            16 => Self::Closing,
            17 => Self::BigintExpected,
            18 => Self::DateExpected,
            19 => Self::ArrayBufferExpected,
            20 => Self::DetachableArraybufferExpected,
            21 => Self::WouldDeadlock,
            22 => Self::NoExternalBuffersAllowed,
            23 => Self::InvalidState,
            _ => Self::GenericFailure,
        }
    }
}
