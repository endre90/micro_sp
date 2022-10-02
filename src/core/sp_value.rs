use serde::*;
use std::fmt;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Hash, Eq)]
pub enum SPValue {
    Bool(bool),
    // Float32(f32),
    Int32(i32),
    String(String),
}

pub trait ToSPValue {
    fn to_spvalue(&self) -> SPValue;
}

impl ToSPValue for bool {
    fn to_spvalue(&self) -> SPValue {
        SPValue::Bool(*self)
    }
}

// impl ToSPValue for f32 {
//     fn to_spvalue(&self) -> SPValue {
//         SPValue::Float32(*self)
//     }
// }

impl ToSPValue for i32 {
    fn to_spvalue(&self) -> SPValue {
        SPValue::Int32(*self)
    }
}

impl ToSPValue for String {
    fn to_spvalue(&self) -> SPValue {
        SPValue::String(self.clone())
    }
}

impl ToSPValue for &str {
    fn to_spvalue(&self) -> SPValue {
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
