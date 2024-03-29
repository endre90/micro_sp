use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::SystemTime;

/// Represents a variable value of a specific type.
#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SPValue {
    Bool(bool),
    Float64(OrderedFloat<f64>),
    Int32(i32),
    String(String),
    Time(SystemTime),
    Array(SPValueType, Vec<SPValue>),
    Unknown,
}

/// Used by SPVariables for defining their type. Must be the same as SPValue.
#[derive(Debug, PartialEq, Copy, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SPValueType {
    Bool,
    Float64,
    Int32,
    String,
    Time,
    Array,
    Unknown,
}

impl SPValue {
    /// Checks whether the value is of the specified type.
    pub fn is_type(&self, t: SPValueType) -> bool {
        match self {
            SPValue::Bool(_) => SPValueType::Bool == t,
            SPValue::Float64(_) => SPValueType::Float64 == t,
            SPValue::Int32(_) => SPValueType::Int32 == t,
            SPValue::String(_) => SPValueType::String == t,
            SPValue::Time(_) => SPValueType::Time == t,
            SPValue::Array(_, _) => SPValueType::Array == t,
            SPValue::Unknown => SPValueType::Unknown == t,
        }
    }

    /// Returns the type of the `SPValue`.
    pub fn has_type(&self) -> SPValueType {
        match self {
            SPValue::Bool(_) => SPValueType::Bool,
            SPValue::Float64(_) => SPValueType::Float64,
            SPValue::Int32(_) => SPValueType::Int32,
            SPValue::String(_) => SPValueType::String,
            SPValue::Time(_) => SPValueType::Time,
            SPValue::Array(_, _) => SPValueType::Array,
            SPValue::Unknown => SPValueType::Unknown,
        }
    }

    /// Checks whether the value is of the array type.
    pub fn is_array(&self) -> bool {
        match self {
            SPValue::Array(_, _) => true,
            _ => false,
        }
    }

    /// Returns a `String` representation of the `SPValue`.
    pub fn to_string(&self) -> String {
        match self {
            SPValue::Bool(x) => x.to_string(),
            SPValue::Float64(x) => x.to_string(),
            SPValue::Int32(x) => x.to_string(),
            SPValue::String(x) => x.to_string(),
            SPValue::Time(x) => format!("{:?}", x.elapsed().unwrap_or_default()),
            SPValue::Array(_, arr) => format!(
                "[{}]",
                arr.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            SPValue::Unknown => "[unknown]".to_string(),
        }
    }
}

/// This trait defines a set of conversions from some Rust primitive types and containers to `SPValue`.
pub trait ToSPValue {
    fn to_spvalue(&self) -> SPValue;
}

impl ToSPValue for bool {
    fn to_spvalue(&self) -> SPValue {
        SPValue::Bool(*self)
    }
}

impl ToSPValue for i32 {
    fn to_spvalue(&self) -> SPValue {
        SPValue::Int32(*self)
    }
}

impl ToSPValue for f64 {
    fn to_spvalue(&self) -> SPValue {
        SPValue::Float64(OrderedFloat(*self))
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

impl ToSPValue for std::time::SystemTime {
    fn to_spvalue(&self) -> SPValue {
        SPValue::Time(*self)
    }
}

impl<T> ToSPValue for Vec<T>
where
    T: ToSPValue,
{
    fn to_spvalue(&self) -> SPValue {
        let res = self
            .iter()
            .map(|x| x.to_spvalue())
            .collect::<Vec<SPValue>>();
        res.to_spvalue()
    }
}

impl ToSPValue for Vec<SPValue> {
    fn to_spvalue(&self) -> SPValue {
        if self.is_empty() {
            SPValue::Array(SPValueType::Unknown, self.clone())
        } else {
            let spvaltype = self[0].has_type();
            assert!(self.iter().all(|e| e.has_type() == spvaltype));
            SPValue::Array(spvaltype, self.clone())
        }
    }
}

/// Displaying the value of an SPValue instance in a user-friendly way.
impl fmt::Display for SPValue {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SPValue::Bool(b) if *b => write!(fmtr, "true"),
            SPValue::Bool(_) => write!(fmtr, "false"),
            SPValue::Float64(f) => write!(fmtr, "{}", f.0),
            SPValue::Int32(i) => write!(fmtr, "{}", i),
            SPValue::String(s) => write!(fmtr, "{}", s),
            SPValue::Time(t) => write!(fmtr, "{:?}", t.elapsed().unwrap_or_default()),
            SPValue::Array(_, a) => write!(fmtr, "{:?}", a),
            SPValue::Unknown => write!(fmtr, "[unknown]"),
        }
    }
}

/// Converting a SPValueType value to a human-readable string representation.
impl fmt::Display for SPValueType {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SPValueType::Bool => write!(fmtr, "Bool"),
            SPValueType::Float64 => write!(fmtr, "Float64"),
            SPValueType::Int32 => write!(fmtr, "Int32"),
            SPValueType::String => write!(fmtr, "String"),
            SPValueType::Time => write!(fmtr, "Time"),
            SPValueType::Array => write!(fmtr, "Array"),
            SPValueType::Unknown => write!(fmtr, "[unknown]"),
        }
    }
}
