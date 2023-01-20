#![allow(unused_imports)]
#![allow(dead_code)]
use crate::{SPValue, SPValueType, SPVariable, SPVariableType, ToSPValue};

#[macro_export]
macro_rules! assign {
    ($a:expr, $b:expr) => {
        SPAssignment::new($a, $b)
    };
}
