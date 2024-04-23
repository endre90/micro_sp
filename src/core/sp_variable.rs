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

#[cfg(test)]
mod tests {

    use ordered_float::OrderedFloat;
    use crate::{SPValue, SPValueType, SPVariable, SPVariableType, ToSPValue};

    #[test]
    fn test_new_spvariable() {
        let name = "test_var";
        let variable_type = SPVariableType::Measured;
        let value_type = SPValueType::Float64;
        let domain = vec![
            SPValue::Float64(OrderedFloat(1.0)),
            SPValue::Float64(OrderedFloat(2.0)),
        ];
        let spvar = SPVariable::new(name, variable_type.clone(), value_type, domain.clone());

        assert_eq!(spvar.name, name);
        assert_eq!(spvar.variable_type, variable_type);
        assert_eq!(spvar.value_type, value_type);
        assert_eq!(spvar.domain, domain);
    }

    #[test]
    fn test_new_boolean() {
        let variable = SPVariable::new_boolean("test_bool", SPVariableType::Measured);
        assert_eq!(variable.name, "test_bool");
        assert_eq!(variable.variable_type, SPVariableType::Measured);
        assert_eq!(variable.value_type, SPValueType::Bool);
        assert_eq!(variable.domain, vec![false.to_spvalue(), true.to_spvalue()]);
    }

    #[test]
    fn test_new_integer() {
        let domain = vec![0.to_spvalue(), 1.to_spvalue(), 2.to_spvalue()];
        let variable =
            SPVariable::new_integer("test_int", SPVariableType::Estimated, domain.clone());
        assert_eq!(variable.name, "test_int");
        assert_eq!(variable.variable_type, SPVariableType::Estimated);
        assert_eq!(variable.value_type, SPValueType::Int64);
        assert_eq!(variable.domain, domain);
    }

    #[test]
    fn test_new_float() {
        let domain = vec![0.0.to_spvalue(), 1.0.to_spvalue(), 2.0.to_spvalue()];
        let variable = SPVariable::new_float("test_float", SPVariableType::Command, domain.clone());
        assert_eq!(variable.name, "test_float");
        assert_eq!(variable.variable_type, SPVariableType::Command);
        assert_eq!(variable.value_type, SPValueType::Float64);
        assert_eq!(variable.domain, domain);
    }

    #[test]
    fn test_new_string() {
        let domain = vec![
            "test1".to_spvalue(),
            "test2".to_spvalue(),
            "test3".to_spvalue(),
        ];
        let variable =
            SPVariable::new_string("test_string", SPVariableType::Runner, domain.clone());
        assert_eq!(variable.name, "test_string");
        assert_eq!(variable.variable_type, SPVariableType::Runner);
        assert_eq!(variable.value_type, SPValueType::String);
        assert_eq!(variable.domain, domain);
    }

    #[test]
    fn test_new_array() {
        let domain = vec![
            SPValue::Array(
                SPValueType::Bool,
                vec![false.to_spvalue(), true.to_spvalue(), false.to_spvalue()],
            ),
            SPValue::Array(SPValueType::Int64, vec![0.to_spvalue(), 1.to_spvalue()]),
        ];
        let variable =
            SPVariable::new_array("test_array", SPVariableType::Measured, domain.clone());
        assert_eq!(variable.name, "test_array");
        assert_eq!(variable.variable_type, SPVariableType::Measured);
        assert_eq!(variable.value_type, SPValueType::Array);
        assert_eq!(variable.domain, domain);
    }

    #[test]
    fn test_has_type() {
        let v1 = SPVariable::new_boolean("bool_var", SPVariableType::Measured);
        assert_eq!(v1.has_type(), (SPVariableType::Measured, SPValueType::Bool));

        let v2 = SPVariable::new_integer(
            "int_var",
            SPVariableType::Estimated,
            vec![1.to_spvalue(), 2.to_spvalue(), 3.to_spvalue()],
        );
        assert_eq!(
            v2.has_type(),
            (SPVariableType::Estimated, SPValueType::Int64)
        );

        let v3 = SPVariable::new_float(
            "float_var",
            SPVariableType::Command,
            vec![0.1.to_spvalue(), 0.2.to_spvalue()],
        );
        assert_eq!(
            v3.has_type(),
            (SPVariableType::Command, SPValueType::Float64)
        );

        let v4 = SPVariable::new_string(
            "string_var",
            SPVariableType::Runner,
            vec![
                String::from("hello").to_spvalue(),
                String::from("world").to_spvalue(),
            ],
        );
        assert_eq!(v4.has_type(), (SPVariableType::Runner, SPValueType::String));
    }
}
