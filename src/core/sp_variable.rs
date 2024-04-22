use serde::{Deserialize, Serialize};

use crate::{SPValue, SPValueType, ToSPValue};
use std::fmt;

/// A SPVariable is a named unit of data of type SPValueType that can be assigned a value from its finite domain.
#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SPVariable {
    pub name: String,
    pub variable_type: SPVariableType,
    pub value_type: SPValueType,
    pub domain: Vec<SPValue>,
}

/// An enum representing the different variable types which SPVariable can have
#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SPVariableType {
    Undefined,
    Measured,
    Estimated,
    Command,
    Runner,
}

impl SPVariable {
    pub fn new(
        name: &str,
        variable_type: SPVariableType,
        value_type: SPValueType,
        domain: Vec<SPValue>,
    ) -> SPVariable {
        SPVariable {
            name: name.to_owned(),
            variable_type,
            value_type,
            domain,
        }
    }
    pub fn new_boolean(name: &str, variable_type: SPVariableType) -> SPVariable {
        SPVariable::new(
            name,
            variable_type,
            SPValueType::Bool,
            vec![false.to_spvalue(), true.to_spvalue()],
        )
    }
    pub fn new_integer(
        name: &str,
        variable_type: SPVariableType,
        domain: Vec<SPValue>,
    ) -> SPVariable {
        SPVariable::new(name, variable_type, SPValueType::Int32, domain)
    }
    pub fn new_float(
        name: &str,
        variable_type: SPVariableType,
        domain: Vec<SPValue>,
    ) -> SPVariable {
        SPVariable::new(name, variable_type, SPValueType::Float64, domain)
    }
    pub fn new_string(
        name: &str,
        variable_type: SPVariableType,
        domain: Vec<SPValue>,
    ) -> SPVariable {
        SPVariable::new(name, variable_type, SPValueType::String, domain)
    }
    pub fn new_array(
        name: &str,
        variable_type: SPVariableType,
        domain: Vec<SPValue>,
    ) -> SPVariable {
        SPVariable::new(name, variable_type, SPValueType::Array, domain)
    }

    /// This is used to retrieve information about the type of the variable.
    pub fn has_type(&self) -> (SPVariableType, SPValueType) {
        (self.variable_type.clone(), self.value_type)
    }
}

/// Displaying the variable name in a user-friendly way.
impl fmt::Display for SPVariable {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmtr, "{}", self.name.to_owned())
    }
}
