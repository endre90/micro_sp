use serde::*;

use crate::{SPValue, SPValueType, State, SPCommon};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Hash, Eq, PartialOrd, Ord)]
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
        state.clone().get_spvar(name)
    }
    pub fn to_common(var: &SPVariable) -> SPCommon {
        SPCommon::SPVariable(var.clone())
    }
    pub fn to_common_from_name(name: &str, state: &State) -> SPCommon {
        SPCommon::SPVariable(state.clone().get_spvar(name).clone())
    }
}

pub trait ToSPVariable {
    fn to_spvar(&self, state: &State) -> SPVariable;
}

impl ToSPVariable for String {
    fn to_spvar(&self, state: &State) -> SPVariable {
        SPVariable::from_name(self, state)
    }
}

impl ToSPVariable for &str {
    fn to_spvar(&self, state: &State) -> SPVariable {
        SPVariable::from_name(self, state)
    }
}