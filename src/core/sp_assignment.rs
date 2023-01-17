use serde::{Serialize, Deserialize};

use crate::{SPValue, SPVariable};

#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SPAssignment {
    pub var: SPVariable,
    pub val: SPValue
}

impl SPAssignment {
    pub fn new(var: SPVariable, val: SPValue) -> SPAssignment {
        match var.has_type().1 == val.has_type() {
            true => SPAssignment { var, val },
            false => panic!("Wrong value type {} to be assigned to a variable with type {}.", var.has_type().1, val.has_type())
        }
    }
}