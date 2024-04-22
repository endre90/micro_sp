use serde::{Deserialize, Serialize};

use crate::{SPValue, SPValueType, SPVariable};

/// Represents assigning a value to a variable.
#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SPAssignment {
    pub var: SPVariable,
    pub val: SPValue,
}

impl SPAssignment {
    /// Creates a new `SPAssignment` instance with the given variable and value.
    pub fn new(var: SPVariable, val: SPValue) -> SPAssignment {
        match val.has_type() {
            SPValueType::UNDEFINED => SPAssignment { var, val },
            _ => match var.has_type().1 == val.has_type() {
                true => SPAssignment { var, val },
                false => panic!(
                    "Wrong value type '{}' to be assigned to a variable with type '{}'.",
                    var.has_type().1,
                    val.has_type()
                ),
            },
        }
    }
}
