#![allow(unused_imports)]
#![allow(dead_code)]
use crate::{SPValue, SPValueType, SPVariable, State, ToSPValue, ToSPCommon, ToSPCommonVar, Action};

#[macro_export]
macro_rules! v {
    ($a:expr, $b:expr) => {
        SPVariable::new($a.clone(), &SPValueType::String, &$b.iter().map(|x| x.clone().to_spval()).collect())
    };
}

#[macro_export]
macro_rules! bv {
    ($a:expr) => {
        SPVariable::new($a.clone(), &SPValueType::Bool, &vec!(true.to_spval(), false.to_spval()))
    };
}

#[macro_export]
macro_rules! iv {
    ($a:expr, $b:expr) => {
        SPVariable::new($a.clone(), &SPValueType::Int32, &$b.iter().map(|x| x.clone().to_spval()).collect())
    };
}