use crate::{SPValue, SPVariable};
use std::fmt;

/// SPCommon can either be a SPVariable or a SPValue.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum SPCommon {
    SPVariable(SPVariable),
    SPValue(SPValue),
}

pub trait ToSPCommon {
    fn cl(&self) -> SPCommon;
}

impl ToSPCommon for SPValue {
    fn cl(&self) -> SPCommon {
        SPCommon::SPValue(self.clone())
    }
}

impl ToSPCommon for bool {
    fn cl(&self) -> SPCommon {
        SPCommon::SPValue(SPValue::Bool(*self))
    }
}

impl ToSPCommon for i32 {
    fn cl(&self) -> SPCommon {
        SPCommon::SPValue(SPValue::Int32(*self))
    }
}

impl ToSPCommon for String {
    fn cl(&self) -> SPCommon {
        SPCommon::SPValue(SPValue::String(self.clone()))
    }
}

impl ToSPCommon for &str {
    fn cl(&self) -> SPCommon {
        SPCommon::SPValue(SPValue::String((*self).to_string()))
    }
}

pub trait ToSPCommonVar {
    fn cr(&self) -> SPCommon;
}

impl ToSPCommonVar for SPVariable {
    fn cr(&self) -> SPCommon {
        SPCommon::SPVariable(self.clone())
    }
}

impl fmt::Display for SPCommon {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SPCommon::SPValue(val) => match val {
                SPValue::Bool(b) if *b => write!(fmtr, "true"),
                SPValue::Bool(_) => write!(fmtr, "false"),
                // SPValue::Float32(f) => write!(fmtr, "{}", f),
                SPValue::Int32(i) => write!(fmtr, "{}", i),
                SPValue::String(s) => write!(fmtr, "{}", s),
            },
            SPCommon::SPVariable(var) => write!(fmtr, "{}", var.name.to_owned()),
        }
    }
}