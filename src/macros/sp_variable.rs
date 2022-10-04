#![allow(unused_imports)]
#![allow(dead_code)]
use crate::{SPValue, SPValueType, SPVariable, State, ToSPValue, ToSPCommon, ToSPCommonVar, Action, ToSPVariable};

#[macro_export]
macro_rules! v {
    ($a:expr, $b:expr) => {
        SPVariable::new($a.clone(), &SPValueType::String, &$b.clone())
    };
}