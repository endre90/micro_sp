use crate::{SPValue, SPVariable};

#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord)]
pub struct SPAssignment {
    pub var: SPVariable,
    pub val: SPValue
}