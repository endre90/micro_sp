use serde::*;
use std::fmt;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Hash, Eq, PartialOrd, Ord)]
pub enum SPValue {
    Bool(bool),
    // Float32(f32), // can't eq or hash
    Int32(i32),
    String(String),
}

/// Used by Variables for defining type. Must be the same as SPValue
#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Hash, Eq, PartialOrd, Ord)]
pub enum SPValueType {
    Bool,
    // Float32, 
    Int32,
    String
}

impl SPValue {
    pub fn is_type(&self, t: SPValueType) -> bool {
        match self {
            SPValue::Bool(_) => SPValueType::Bool == t,
            // SPValue::Float32(_) => SPValueType::Float32 == t, 
            SPValue::Int32(_) => SPValueType::Int32 == t,
            SPValue::String(_) => SPValueType::String == t,
        }
    }

    pub fn has_type(&self) -> SPValueType {
        match self {
            SPValue::Bool(_) => SPValueType::Bool,
            // SPValue::Float32(_) => SPValueType::Float32, 
            SPValue::Int32(_) => SPValueType::Int32,
            SPValue::String(_) => SPValueType::String,
        }
    }

    pub fn value_as_string(&self) -> String {
        match self {
            SPValue::Bool(x) => x.to_string(),
            // SPValue::Float32(_) => SPValueType::Float32, 
            SPValue::Int32(x) => x.to_string(),
            SPValue::String(x) => x.to_string(),
        }
    }
}

pub trait ToSPValue {
    fn to_spval(&self) -> SPValue;
}

impl ToSPValue for bool {
    fn to_spval(&self) -> SPValue {
        SPValue::Bool(*self)
    }
}

// impl ToSPValue for f32 {
//     fn to_spval(&self) -> SPValue {
//         SPValue::Float32(*self)
//     }
// }

impl ToSPValue for i32 {
    fn to_spval(&self) -> SPValue {
        SPValue::Int32(*self)
    }
}

impl ToSPValue for String {
    fn to_spval(&self) -> SPValue {
        SPValue::String(self.clone())
    }
}

impl ToSPValue for &str {
    fn to_spval(&self) -> SPValue {
        SPValue::String((*self).to_string())
    }
}

impl fmt::Display for SPValue {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SPValue::Bool(b) if *b => write!(fmtr, "true"),
            SPValue::Bool(_) => write!(fmtr, "false"),
            // SPValue::Float32(f) => write!(fmtr, "{}", f),
            SPValue::Int32(i) => write!(fmtr, "{}", i),
            SPValue::String(s) => write!(fmtr, "{}", s),
        }
    }
}
