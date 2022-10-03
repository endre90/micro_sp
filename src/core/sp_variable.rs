use serde::*;

use crate::{SPValue, SPValueType, State, SPCommon};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Hash, Eq)]
pub struct SPVariable {
    pub name: String,
    pub value_type: SPValueType,
    pub domain: Vec<SPValue>,
}

impl SPVariable {
    pub fn new(name: &str, value_type: &SPValueType, domain: &Vec<SPValue>) -> SPVariable {
        SPVariable {
            name: name.to_owned(),
            value_type: value_type.to_owned(),
            domain: domain.to_owned(),
        }
    }
    pub fn from_name(name: &str, state: &State) -> SPVariable {
        state.clone().get_var(name)
    }
    pub fn to_common(var: &SPVariable) -> SPCommon {
        SPCommon::SPVariable(var.clone())
    }
    pub fn to_common_from_name(name: &str, state: &State) -> SPCommon {
        SPCommon::SPVariable(state.clone().get_var(name).clone())
    }
}

pub trait ToSPVariable {
    fn to_spvar(&self) -> SPValue;
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
