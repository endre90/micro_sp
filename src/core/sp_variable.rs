use crate::{SPValue, SPValueType, State, SPCommon};
use std::fmt;

/// An SPVariable is a named unit of data of type SPValueType that can be assigned a value from its finite domain.
#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord)]
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

impl fmt::Display for SPVariable {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmtr, "{}", self.name.to_owned())
    }
}
