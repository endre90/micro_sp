use serde::{Deserialize, Serialize};

use crate::{SPValue, SPValueType, ToSPValue};
use std::fmt;

/// An enum representing the different variable types which SPVariable can have.
#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SPVariableType {
    Measured,
    Estimated,
    Command,
    Runner,
    UNDEFINED,
}

impl Default for SPVariableType {
    fn default() -> Self {
        SPVariableType::UNDEFINED
    }
}

impl SPVariableType {
    pub fn from_str(x: &str) -> SPVariableType {
        match x {
            "measured" => SPVariableType::Measured,
            "estimated" => SPVariableType::Estimated,
            "command" => SPVariableType::Command,
            "runner" => SPVariableType::Runner,
            _ => SPVariableType::UNDEFINED,
        }
    }
}

impl fmt::Display for SPVariableType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SPVariableType::Measured => write!(f, "measured"),
            SPVariableType::Estimated => write!(f, "estimated"),
            SPVariableType::Command => write!(f, "command"),
            SPVariableType::Runner => write!(f, "runner"),
            SPVariableType::UNDEFINED => write!(f, "[UNDEFINED]"),
        }
    }
}

/// A SPVariable is a named unit of data of type SPValueType that can be assigned a value from its finite domain.
#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SPVariable {
    pub name: String,
    pub variable_type: SPVariableType,
    pub value_type: SPValueType,
    pub domain: Vec<SPValue>,
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
        SPVariable::new(name, variable_type, SPValueType::Int64, domain)
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
