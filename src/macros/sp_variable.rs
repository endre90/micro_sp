
// use crate::SPVariable

#[macro_export]
macro_rules! v {
    ($a:expr) => {
        crate::SPVariable::new(
            $a.clone(),
            crate::SPValueType::String,
            vec![],
        )
    };
}

#[macro_export]
macro_rules! bv {
    ($a:expr) => {
        crate::SPVariable::new(
            $a.clone(),
            crate::SPValueType::Bool,
            vec![true.to_spvalue(), false.to_spvalue()],
        )
    };
}

#[macro_export]
macro_rules! iv {
    ($a:expr) => {
        crate::SPVariable::new(
            $a.clone(),
            crate::SPValueType::Int64,
            vec![],
            // $b.iter().map(|x| x.clone().to_spvalue()).collect(),
        )
    };
}

#[macro_export]
macro_rules! fv {
    ($a:expr) => {
        crate::SPVariable::new(
            $a.clone(),
            crate::SPValueType::Float64,
            vec![]
        )
    };
}

#[macro_export]
macro_rules! av {
    ($a:expr) => {
        crate::SPVariable::new(
            $a.clone(),
            crate::SPValueType::Array,
            vec![],
        )
    };
}
