//! Named, typed variables.
//!
//! An [`SPVariable`] is a name plus an [`SPValueType`]; pairing one with a
//! value of that type gives an [`SPAssignment`], and a set of assignments is a
//! [`State`]. [`SPVariableFormal`] adds an explicit domain, used where the
//! planner needs to enumerate a variable's possible values.

use serde::{Deserialize, Serialize};

use crate::*;
use std::fmt;

/// A named unit of data with a declared [`SPValueType`].
///
/// The name is the key the variable has in a [`State`] and in Redis, so it must
/// be unique across the model.
///
/// ```
/// use micro_sp::*;
///
/// let pos = SPVariable::new("pos", SPValueType::String);
/// assert_eq!(pos.has_type(), SPValueType::String);
///
/// // Type-specific constructors say the same thing more briefly.
/// assert_eq!(SPVariable::new_integer_var("count"), SPVariable::new("count", SPValueType::Int64));
/// ```
#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SPVariable {
    /// The variable's unique name, used as its state and Redis key.
    pub name: String,
    /// The type of value the variable may hold.
    pub value_type: SPValueType,
}

/// An [`SPVariable`] with an explicit domain of allowed values.
#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SPVariableFormal {
    /// The variable's unique name.
    pub name: String,
    /// The type of value the variable may hold.
    pub value_type: SPValueType,
    /// The values the variable is allowed to take.
    pub domain: Vec<SPValue>,
}

impl SPVariable {
    /// Creates a variable with the given name and type.
    pub fn new(name: &str, value_type: SPValueType) -> SPVariable {
        SPVariable {
            name: name.to_owned(),
            value_type,
        }
    }

    /// Creates a [`SPValueType::Bool`] variable. The `bv!` macro is shorter.
    pub fn new_boolean_var(name: &str) -> SPVariable {
        SPVariable::new(name, SPValueType::Bool)
    }

    /// Creates a [`SPValueType::Int64`] variable. The `iv!` macro is shorter.
    pub fn new_integer_var(name: &str) -> SPVariable {
        SPVariable::new(name, SPValueType::Int64)
    }

    /// Creates a [`SPValueType::Float64`] variable. The `fv!` macro is shorter.
    pub fn new_float_var(name: &str) -> SPVariable {
        SPVariable::new(name, SPValueType::Float64)
    }

    /// Creates a [`SPValueType::String`] variable. The `v!` macro is shorter.
    pub fn new_string_var(name: &str) -> SPVariable {
        SPVariable::new(name, SPValueType::String)
    }

    /// Creates a [`SPValueType::Array`] variable. The `av!` macro is shorter.
    pub fn new_array_var(name: &str) -> SPVariable {
        SPVariable::new(name, SPValueType::Array)
    }

    /// Creates a [`SPValueType::Map`] variable. The `mv!` macro is shorter.
    pub fn new_map_var(name: &str) -> SPVariable {
        SPVariable::new(name, SPValueType::Map)
    }

    /// Creates a [`SPValueType::Time`] variable. The `tv!` macro is shorter.
    pub fn new_time_var(name: &str) -> SPVariable {
        SPVariable::new(name, SPValueType::Time)
    }

    /// Creates a [`SPValueType::Transform`] variable. The `tfv!` macro is
    /// shorter.
    pub fn new_transform_var(name: &str) -> SPVariable {
        SPVariable::new(name, SPValueType::Transform)
    }

    /// Returns the variable's declared [`SPValueType`].
    pub fn has_type(&self) -> SPValueType {
        self.value_type
    }
}

/// Renders the variable's name.
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