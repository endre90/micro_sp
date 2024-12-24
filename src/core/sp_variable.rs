use serde::{Deserialize, Serialize};

use crate::*;
use std::fmt;

/// A SPVariable is a named unit of data of type SPValueType that can be assigned a value from its finite domain.
/// Current version of micro_sp doesn't support formal methods, thus domain specification is optional.
/// Use the macros v!, iv!, fv!, bv!, and av! to define new variables instead of SPVariable::new().
#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SPVariable {
    pub name: String,
    pub value_type: SPValueType,
    pub domain: Vec<SPValue>,
}

impl SPVariable {
    /// Use the macros v!, iv!, fv!, bv!, and av! instead.
    pub fn new(name: &str, value_type: SPValueType, domain: Vec<SPValue>) -> SPVariable {
        SPVariable {
            name: name.to_owned(),
            value_type,
            domain,
        }
    }
    /// Use the macro bv! instead.
    pub fn new_boolean(name: &str) -> SPVariable {
        SPVariable::new(
            name,
            SPValueType::Bool,
            vec![false.to_spvalue(), true.to_spvalue()],
        )
    }
    /// Use the macro iv! instead.
    pub fn new_integer(name: &str, domain: Vec<SPValue>) -> SPVariable {
        SPVariable::new(name, SPValueType::Int64, domain)
    }
    /// Use the macro fv! instead.
    pub fn new_float(name: &str, domain: Vec<SPValue>) -> SPVariable {
        SPVariable::new(name, SPValueType::Float64, domain)
    }
    /// Use the macro v! instead.
    pub fn new_string(name: &str, domain: Vec<SPValue>) -> SPVariable {
        SPVariable::new(name, SPValueType::String, domain)
    }
    /// Use the macro av! instead.
    pub fn new_array(name: &str, domain: Vec<SPValue>) -> SPVariable {
        SPVariable::new(name, SPValueType::Array, domain)
    }

    // Maybe actually we can only use the float64 and do something like instant.to_64()
    // /// Use the macro tv! instead.
    // pub fn new_timer(name: &str) -> SPVariable {
    //     SPVariable::new(name, SPValueType::Time, vec!())
    // }

    /// This is used to retrieve information about the type of the variable.
    pub fn has_type(&self) -> SPValueType {
        self.value_type
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

    use crate::*;
    use ordered_float::OrderedFloat;

    #[test]
    fn test_new_spvariable() {
        let name = "test_var";
        let value_type = SPValueType::Float64;
        let domain = vec![
            SPValue::Float64(OrderedFloat(1.0)),
            SPValue::Float64(OrderedFloat(2.0)),
        ];
        let spvar = SPVariable::new(name, value_type, domain.clone());

        assert_eq!(spvar.name, name);
        assert_eq!(spvar.value_type, value_type);
        assert_eq!(spvar.domain, domain);
    }

    #[test]
    fn test_new_boolean() {
        let variable = SPVariable::new_boolean("test_bool");
        assert_eq!(variable.name, "test_bool");
        assert_eq!(variable.value_type, SPValueType::Bool);
        assert_eq!(variable.domain, vec![false.to_spvalue(), true.to_spvalue()]);
    }

    #[test]
    fn test_new_integer() {
        let domain = vec![0.to_spvalue(), 1.to_spvalue(), 2.to_spvalue()];
        let variable = SPVariable::new_integer("test_int", domain.clone());
        assert_eq!(variable.name, "test_int");
        assert_eq!(variable.value_type, SPValueType::Int64);
        assert_eq!(variable.domain, domain);
    }

    #[test]
    fn test_new_float() {
        let domain = vec![0.0.to_spvalue(), 1.0.to_spvalue(), 2.0.to_spvalue()];
        let variable = SPVariable::new_float("test_float", domain.clone());
        assert_eq!(variable.name, "test_float");
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
        let variable = SPVariable::new_string("test_string", domain.clone());
        assert_eq!(variable.name, "test_string");
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
        let variable = SPVariable::new_array("test_array", domain.clone());
        assert_eq!(variable.name, "test_array");
        assert_eq!(variable.value_type, SPValueType::Array);
        assert_eq!(variable.domain, domain);
    }

    #[test]
    fn test_has_type() {
        let v1 = SPVariable::new_boolean("bool_var");
        assert_eq!(v1.has_type(), (SPValueType::Bool));

        let v2 = SPVariable::new_integer(
            "int_var",
            vec![1.to_spvalue(), 2.to_spvalue(), 3.to_spvalue()],
        );
        assert_eq!(v2.has_type(), SPValueType::Int64);

        let v3 = SPVariable::new_float("float_var", vec![0.1.to_spvalue(), 0.2.to_spvalue()]);
        assert_eq!(v3.has_type(), SPValueType::Float64);

        let v4 = SPVariable::new_string(
            "string_var",
            vec![
                String::from("hello").to_spvalue(),
                String::from("world").to_spvalue(),
            ],
        );
        assert_eq!(v4.has_type(), SPValueType::String);
    }
}
