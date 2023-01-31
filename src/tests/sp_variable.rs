#![allow(unused_imports)]
#![allow(dead_code)]
use crate::{
    av_command, av_estimated, av_measured, av_runner, bv_command, bv_estimated, bv_measured,
    bv_runner, fv_command, fv_estimated, fv_measured, fv_runner, iv_command, iv_estimated,
    iv_measured, iv_runner, v_command, v_estimated, v_measured, v_runner,
};
use crate::{SPValue, SPValueType, SPVariable, SPVariableType, ToSPValue};
use std::collections::{HashMap, HashSet};
use std::time::SystemTime;

#[test]
fn test_new_var() {
    let int_var = SPVariable::new_integer(
        "counter",
        SPVariableType::Estimated,
        vec![1.to_spvalue(), 3.to_spvalue(), 5.to_spvalue()],
    );
    let bool_var = SPVariable::new_boolean("toggle", SPVariableType::Command);
    let float_var = SPVariable::new_float(
        "speed",
        SPVariableType::Measured,
        vec![0.1.to_spvalue(), 0.3.to_spvalue()],
    );
    let string_var = SPVariable::new_string(
        "position",
        SPVariableType::Measured,
        vec!["a".to_spvalue(), "b".to_spvalue(), "c".to_spvalue()],
    );
    let array_var = SPVariable::new_array("plan", SPVariableType::Runner, vec![]); // runner, flexible domain
    let string_var_2 = SPVariable::new_string("goal", SPVariableType::Runner, vec![]); // runner, flexible domain
    assert_eq!(
        (SPVariableType::Estimated, SPValueType::Int32),
        int_var.has_type()
    );
    assert_eq!(
        (SPVariableType::Command, SPValueType::Bool),
        bool_var.has_type()
    );
    assert_eq!(
        (SPVariableType::Measured, SPValueType::Float64),
        float_var.has_type()
    );
    assert_eq!(
        (SPVariableType::Measured, SPValueType::String),
        string_var.has_type()
    );
    assert_eq!(
        (SPVariableType::Runner, SPValueType::Array),
        array_var.has_type()
    );
    assert_eq!(
        (SPVariableType::Runner, SPValueType::String),
        string_var_2.has_type()
    );
}

#[test]
fn test_new_var_macros() {
    let string_var = v_estimated!("position", vec!("a", "b", "c"));
    let string_var_run = v_runner!("position");
    let int_var = iv_estimated!("counter", vec!(1, 2, 3));
    let int_var_run = iv_runner!("counter");
    let bool_var = bv_estimated!("toggle");
    let bool_var_run = bv_runner!("toggle");
    let float_var = fv_measured!("speed", vec!(0.1, 0.3));
    let float_var_run = fv_runner!("speed");
    let array_var = av_estimated!(
        "valid_lists",
        vec!(vec!("a", "b", "c"), vec!("c", "a", "b"))
    );
    let array_var_run = av_runner!("valid_lists");
    assert_eq!(
        (SPVariableType::Estimated, SPValueType::String),
        string_var.has_type()
    );
    assert_eq!(
        (SPVariableType::Runner, SPValueType::String),
        string_var_run.has_type()
    );
    assert_eq!(
        (SPVariableType::Estimated, SPValueType::Int32),
        int_var.has_type()
    );
    assert_eq!(
        (SPVariableType::Runner, SPValueType::Int32),
        int_var_run.has_type()
    );
    assert_eq!(
        (SPVariableType::Estimated, SPValueType::Bool),
        bool_var.has_type()
    );
    assert_eq!(
        (SPVariableType::Runner, SPValueType::Bool),
        bool_var_run.has_type()
    );
    assert_eq!(
        (SPVariableType::Measured, SPValueType::Float64),
        float_var.has_type()
    );
    assert_eq!(
        (SPVariableType::Runner, SPValueType::Float64),
        float_var_run.has_type()
    );
    assert_eq!(
        (SPVariableType::Estimated, SPValueType::Array),
        array_var.has_type()
    );
    assert_eq!(
        (SPVariableType::Runner, SPValueType::Array),
        array_var_run.has_type()
    );
}
