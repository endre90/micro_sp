use serde::{Deserialize, Serialize};

use crate::*;
use std::fmt;

// A SPVariable is a named unit of data of type SPValueType that can be assigned a value.
#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SPVariable {
    pub name: String,
    pub value_type: SPValueType,
}

#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SPVariableFormal {
    pub name: String,
    pub value_type: SPValueType,
    pub domain: Vec<SPValue>,
}

impl SPVariable {
    pub fn new(name: &str, value_type: SPValueType) -> SPVariable {
        SPVariable {
            name: name.to_owned(),
            value_type,
        }
    }
    // Use the macro bv! instead.
    pub fn new_boolean_var(name: &str) -> SPVariable {
        SPVariable::new(name, SPValueType::Bool)
    }
    // Use the macro iv! instead.
    pub fn new_integer_var(name: &str) -> SPVariable {
        SPVariable::new(name, SPValueType::Int64)
    }
    // Use the macro fv! instead.
    pub fn new_float_var(name: &str) -> SPVariable {
        SPVariable::new(name, SPValueType::Float64)
    }
    // Use the macro v! instead.
    pub fn new_string_var(name: &str) -> SPVariable {
        SPVariable::new(name, SPValueType::String)
    }
    // Use the macro av! instead.
    pub fn new_array_var(name: &str) -> SPVariable {
        SPVariable::new(name, SPValueType::Array)
    }
    // Use the macro mv! instead.
    pub fn new_map_var(name: &str) -> SPVariable {
        SPVariable::new(name, SPValueType::Map)
    }
    // Use the macro tv! instead.
    pub fn new_time_var(name: &str) -> SPVariable {
        SPVariable::new(name, SPValueType::Time)
    }
    // Use the macro tfv! instead.
    pub fn new_transform_var(name: &str) -> SPVariable {
        SPVariable::new(name, SPValueType::Transform)
    }

    // This is used to retrieve information about the type of the variable.
    pub fn has_type(&self) -> SPValueType {
        self.value_type
    }
}

// Displaying the variable name in a user-friendly way.
impl fmt::Display for SPVariable {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmtr, "{}", self.name.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_new_spvariable() {
        let name = "test_var";
        let value_type = SPValueType::Float64;
        let spvar = SPVariable::new(name, value_type);

        assert_eq!(spvar.name, name);
        assert_eq!(spvar.value_type, value_type);
    }

    #[test]
    fn test_new_boolean_var() {
        let variable = SPVariable::new_boolean_var("test_bool");
        assert_eq!(variable.name, "test_bool");
        assert_eq!(variable.value_type, SPValueType::Bool);
    }

    #[test]
    fn test_new_integer_var() {
        let variable = SPVariable::new_integer_var("test_int");
        assert_eq!(variable.name, "test_int");
        assert_eq!(variable.value_type, SPValueType::Int64);
    }

    #[test]
    fn test_new_float_var() {
        let variable = SPVariable::new_float_var("test_float");
        assert_eq!(variable.name, "test_float");
        assert_eq!(variable.value_type, SPValueType::Float64);
    }

    #[test]
    fn test_new_string_var() {
        let variable = SPVariable::new_string_var("test_string");
        assert_eq!(variable.name, "test_string");
        assert_eq!(variable.value_type, SPValueType::String);
    }

    #[test]
    fn test_new_array_var() {
        let variable = SPVariable::new_array_var("test_array");
        assert_eq!(variable.name, "test_array");
        assert_eq!(variable.value_type, SPValueType::Array);
    }

    #[test]
    fn test_new_map_var() {
        let variable = SPVariable::new_map_var("test_map");
        assert_eq!(variable.name, "test_map");
        assert_eq!(variable.value_type, SPValueType::Map);
    }

    #[test]
    fn test_new_time_var() {
        let variable = SPVariable::new_time_var("test_time");
        assert_eq!(variable.name, "test_time");
        assert_eq!(variable.value_type, SPValueType::Time);
    }

    #[test]
    fn test_new_transform_var() {
        let variable = SPVariable::new_transform_var("test_transform");
        assert_eq!(variable.name, "test_transform");
        assert_eq!(variable.value_type, SPValueType::Transform);
    }

    #[test]
    fn test_has_type() {
        let v_bool = SPVariable::new_boolean_var("bool_var");
        assert_eq!(v_bool.has_type(), SPValueType::Bool);

        let v_int = SPVariable::new_integer_var("int_var");
        assert_eq!(v_int.has_type(), SPValueType::Int64);

        let v_float = SPVariable::new_float_var("float_var");
        assert_eq!(v_float.has_type(), SPValueType::Float64);

        let v_string = SPVariable::new_string_var("string_var");
        assert_eq!(v_string.has_type(), SPValueType::String);

        let v_array = SPVariable::new_array_var("array_var");
        assert_eq!(v_array.has_type(), SPValueType::Array);

        let v_map = SPVariable::new_map_var("map_var");
        assert_eq!(v_map.has_type(), SPValueType::Map);

        let v_time = SPVariable::new_time_var("time_var");
        assert_eq!(v_time.has_type(), SPValueType::Time);

        let v_transform = SPVariable::new_transform_var("transform_var");
        assert_eq!(v_transform.has_type(), SPValueType::Transform);
    }

    #[test]
    fn test_display_for_spvariable() {
        let var = SPVariable::new("my_variable", SPValueType::Bool);
        assert_eq!(format!("{}", var), "my_variable");
    }
}

// OLD TESTS
// #[cfg(test)]
// mod tests {

//     use crate::*;

//     #[test]
//     fn test_new_spvariable() {
//         let name = "test_var";
//         let value_type = SPValueType::Float64;
//         // let domain = vec![
//         //     SPValue::Float64(FloatOrUnknown::Float64(OrderedFloat(1.0))),
//         //     SPValue::Float64(FloatOrUnknown::Float64(OrderedFloat(2.0))),
//         // ];
//         let spvar = SPVariable::new(name, value_type);

//         assert_eq!(spvar.name, name);
//         assert_eq!(spvar.value_type, value_type);
//     }

//     #[test]
//     fn test_new_boolean() {
//         let variable = SPVariable::new_boolean("test_bool");
//         assert_eq!(variable.name, "test_bool");
//         assert_eq!(variable.value_type, SPValueType::Bool);
//         // assert_eq!(variable.domain, vec![false.to_spvalue(), true.to_spvalue()]);
//     }

//     #[test]
//     fn test_new_integer() {
//         // let domain = vec![0.to_spvalue(), 1.to_spvalue(), 2.to_spvalue()];
//         let variable = SPVariable::new_integer("test_int");
//         assert_eq!(variable.name, "test_int");
//         assert_eq!(variable.value_type, SPValueType::Int64);
//         // assert_eq!(variable.domain, domain);
//     }

//     #[test]
//     fn test_new_float() {
//         // let domain = vec![0.0.to_spvalue(), 1.0.to_spvalue(), 2.0.to_spvalue()];
//         let variable = SPVariable::new_float("test_float");
//         assert_eq!(variable.name, "test_float");
//         assert_eq!(variable.value_type, SPValueType::Float64);
//         // assert_eq!(variable.domain, domain);
//     }

//     #[test]
//     fn test_new_string() {
//         // let domain = vec![
//         //     "test1".to_spvalue(),
//         //     "test2".to_spvalue(),
//         //     "test3".to_spvalue(),
//         // ];
//         let variable = SPVariable::new_string("test_string");
//         assert_eq!(variable.name, "test_string");
//         assert_eq!(variable.value_type, SPValueType::String);
//         // assert_eq!(variable.domain, domain);
//     }

//     #[test]
//     fn test_new_array() {
//         // let domain = vec![
//         //     SPValue::Array(
//         //         vec![false.to_spvalue(), true.to_spvalue(), false.to_spvalue()],
//         //     ),
//         //     SPValue::Array(SPValueType::Int64, vec![0.to_spvalue(), 1.to_spvalue()]),
//         // ];
//         let variable = SPVariable::new_array("test_array");
//         assert_eq!(variable.name, "test_array");
//         assert_eq!(variable.value_type, SPValueType::Array);
//         // assert_eq!(variable.domain, domain);
//     }

//     #[test]
//     fn test_has_type() {
//         let v1 = SPVariable::new_boolean("bool_var");
//         assert_eq!(v1.has_type(), (SPValueType::Bool));

//         let v2 = SPVariable::new_integer(
//             "int_var",
//             // vec![1.to_spvalue(), 2.to_spvalue(), 3.to_spvalue()],
//         );
//         assert_eq!(v2.has_type(), SPValueType::Int64);

//         let v3 = SPVariable::new_float("float_var");
//         assert_eq!(v3.has_type(), SPValueType::Float64);

//         let v4 = SPVariable::new_string(
//             "string_var",
//             // vec![
//             //     String::from("hello").to_spvalue(),
//             //     String::from("world").to_spvalue(),
//             // ],
//         );
//         assert_eq!(v4.has_type(), SPValueType::String);
//     }
// }
