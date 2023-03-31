use crate::{SPValue, SPVariable};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::fmt;

/// SPWrapped can either be a SPVariable or a SPValue.
#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub enum SPWrapped {
    SPVariable(SPVariable),
    SPValue(SPValue),
}

/// This trait defines a set of conversions from some Rust primitive types and containers to `SPWrapped`.
pub trait ToSPWrapped {
    fn wrap(&self) -> SPWrapped;
}

impl ToSPWrapped for SPValue {
    fn wrap(&self) -> SPWrapped {
        SPWrapped::SPValue(self.clone())
    }
}

impl ToSPWrapped for bool {
    fn wrap(&self) -> SPWrapped {
        SPWrapped::SPValue(SPValue::Bool(*self))
    }
}

impl ToSPWrapped for i32 {
    fn wrap(&self) -> SPWrapped {
        SPWrapped::SPValue(SPValue::Int32(*self))
    }
}

impl ToSPWrapped for f64 {
    fn wrap(&self) -> SPWrapped {
        SPWrapped::SPValue(SPValue::Float64(OrderedFloat(*self)))
    }
}

impl ToSPWrapped for String {
    fn wrap(&self) -> SPWrapped {
        SPWrapped::SPValue(SPValue::String(self.clone()))
    }
}

impl ToSPWrapped for &str {
    fn wrap(&self) -> SPWrapped {
        SPWrapped::SPValue(SPValue::String((*self).to_string()))
    }
}

/// This trait defines a set of conversions from `SPVariable` to `SPWrapped`.
pub trait ToSPWrappedVar {
    fn wrap(&self) -> SPWrapped;
}

impl ToSPWrappedVar for SPVariable {
    fn wrap(&self) -> SPWrapped {
        SPWrapped::SPVariable(self.clone())
    }
}

/// Displaying SPWrapped in a user-friendly way.
impl fmt::Display for SPWrapped {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SPWrapped::SPValue(val) => match val {
                SPValue::Bool(b) if *b => write!(fmtr, "true"),
                SPValue::Bool(_) => write!(fmtr, "false"),
                SPValue::Float64(f) => write!(fmtr, "{}", f),
                SPValue::Int32(i) => write!(fmtr, "{}", i),
                SPValue::String(s) => write!(fmtr, "{}", s),
                SPValue::Time(t) => write!(fmtr, "{:?} s", t.elapsed().unwrap_or_default()),
                SPValue::Array(_, a) => write!(fmtr, "{:?}", a),
                SPValue::Unknown => write!(fmtr, "[unknown]"),
            },
            SPWrapped::SPVariable(var) => write!(fmtr, "{}", var.name.to_owned()),
        }
    }
}
