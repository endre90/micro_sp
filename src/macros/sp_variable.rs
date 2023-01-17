#![allow(unused_imports)]
#![allow(dead_code)]
use crate::{SPValue, SPValueType, SPVariable, SPVariableType, ToSPValue};

#[macro_export]
macro_rules! v {
    ($a:expr, $b:expr) => {
        SPVariable::new(
            $a.clone(),
            SPVariableType::Planner,
            SPValueType::String,
            $b.iter().map(|x| x.clone().to_spvalue()).collect(),
        )
    };
}

#[macro_export]
macro_rules! v_run {
    ($a:expr) => {
        SPVariable::new(
            $a.clone(),
            SPVariableType::Runner,
            SPValueType::String,
            vec!(),
        )
    };
}

#[macro_export]
macro_rules! bv {
    ($a:expr) => {
        SPVariable::new(
            $a.clone(),
            SPVariableType::Planner,
            SPValueType::Bool,
            vec![true.to_spvalue(), false.to_spvalue()],
        )
    };
}

#[macro_export]
macro_rules! bv_run {
    ($a:expr) => {
        SPVariable::new(
            $a.clone(),
            SPVariableType::Runner,
            SPValueType::Bool,
            vec![true.to_spvalue(), false.to_spvalue()],
        )
    };
}

#[macro_export]
macro_rules! iv {
    ($a:expr, $b:expr) => {
        SPVariable::new(
            $a.clone(),
            SPVariableType::Planner,
            SPValueType::Int32,
            $b.iter().map(|x| x.clone().to_spvalue()).collect(),
        )
    };
}

#[macro_export]
macro_rules! iv_run {
    ($a:expr) => {
        SPVariable::new(
            $a.clone(),
            SPVariableType::Runner,
            SPValueType::Int32,
            vec!(),
        )
    };
}

#[macro_export]
macro_rules! fv {
    ($a:expr, $b:expr) => {
        SPVariable::new(
            $a.clone(),
            SPVariableType::Planner,
            SPValueType::Float64,
            $b.iter().map(|x| x.clone().to_spvalue()).collect(),
        )
    };
}

#[macro_export]
macro_rules! fv_run {
    ($a:expr) => {
        SPVariable::new(
            $a.clone(),
            SPVariableType::Runner,
            SPValueType::Float64,
            vec!(),
        )
    };
}

#[macro_export]
macro_rules! av {
    ($a:expr, $b:expr) => {
        SPVariable::new(
            $a.clone(),
            SPVariableType::Planner,
            SPValueType::Array,
            $b.iter().map(|x| x.clone().to_spvalue()).collect(),
        )
    };
}

#[macro_export]
macro_rules! av_run {
    ($a:expr) => {
        SPVariable::new(
            $a.clone(),
            SPVariableType::Runner,
            SPValueType::Array,
            vec!(),
        )
    };
}

#[macro_export]
macro_rules! sav {
    ($a:expr, $b:expr) => {
        SPVariable::new(
            $a.clone(),
            SPVariableType::Planner,
            SPValueType::StringArray,
            $b.iter().map(|x| x.clone().to_spvalue()).collect(),
        )
    };
}

#[macro_export]
macro_rules! sav_run {
    ($a:expr) => {
        SPVariable::new(
            $a.clone(),
            SPVariableType::Runner,
            SPValueType::StringArray,
            vec!(),
        )
    };
}