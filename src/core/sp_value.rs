use std::fmt;
use ordered_float::OrderedFloat;
use std::f32::NAN;

/// SPValue represent a variable value of a specific type.
#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord)]
pub enum SPValue {
    Bool(bool),
    Float64(OrderedFloat<f64>),
    Int32(i32),
    String(String),
}

/// Used by SPVariables for defining their type. Must be the same as SPValue.
#[derive(Debug, PartialEq, Copy, Clone, Hash, Eq, PartialOrd, Ord)]
pub enum SPValueType {
    Bool,
    Float64,
    Int32,
    String,
}

impl SPValue {
    pub fn is_type(&self, t: SPValueType) -> bool {
        match self {
            SPValue::Bool(_) => SPValueType::Bool == t,
            SPValue::Float64(_) => SPValueType::Float64 == t,
            SPValue::Int32(_) => SPValueType::Int32 == t,
            SPValue::String(_) => SPValueType::String == t,
        }
    }

    pub fn has_type(&self) -> SPValueType {
        match self {
            SPValue::Bool(_) => SPValueType::Bool,
            SPValue::Float64(_) => SPValueType::Float64,
            SPValue::Int32(_) => SPValueType::Int32,
            SPValue::String(_) => SPValueType::String,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            SPValue::Bool(x) => x.to_string(),
            SPValue::Float64(x) => x.to_string(),
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

impl ToSPValue for i32 {
    fn to_spval(&self) -> SPValue {
        SPValue::Int32(*self)
    }
}

impl ToSPValue for f64 {
    fn to_spval(&self) -> SPValue {
        SPValue::Float64(OrderedFloat(*self))
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
            SPValue::Float64(f) => write!(fmtr, "{}", f),
            SPValue::Int32(i) => write!(fmtr, "{}", i),
            SPValue::String(s) => write!(fmtr, "{}", s),
        }
    }
}
