#![allow(unused_imports)]
#![allow(dead_code)]
use crate::{SPValue, SPValueType, SPVariable, SPVariableType, ToSPValue};
use crate::{v, v_run, iv, iv_run, bv, bv_run, fv, fv_run, av, av_run};
use std::collections::{HashMap, HashSet};
use std::time::SystemTime;

#[test]
fn test_new_var() {
    let int_var = SPVariable::new_integer("counter", SPVariableType::Planner, vec!(1.to_spvalue(), 3.to_spvalue(), 5.to_spvalue()));
    let bool_var = SPVariable::new_boolean("toggle", SPVariableType::Planner);
    let float_var = SPVariable::new_float("speed", SPVariableType::Planner, vec!(0.1.to_spvalue(), 0.3.to_spvalue()));
    let string_var = SPVariable::new_string("position", SPVariableType::Planner, vec!("a".to_spvalue(), "b".to_spvalue(), "c".to_spvalue()));
    let array_var = SPVariable::new_array("plan", SPVariableType::Runner, vec!()); // runner, flexible domain
    let string_var_2 = SPVariable::new_string("goal", SPVariableType::Runner, vec!()); // runner, flexible domain
    assert_eq!((SPVariableType::Planner, SPValueType::Int32), int_var.has_type());
    assert_eq!((SPVariableType::Planner, SPValueType::Bool), bool_var.has_type());
    assert_eq!((SPVariableType::Planner, SPValueType::Float64), float_var.has_type());
    assert_eq!((SPVariableType::Planner, SPValueType::String), string_var.has_type());
    assert_eq!((SPVariableType::Runner, SPValueType::Array), array_var.has_type());
    assert_eq!((SPVariableType::Runner, SPValueType::String), string_var_2.has_type());
}

#[test]
fn test_new_var_macros() {
    let string_var = v!("position", vec!("a", "b", "c"));
    let string_var_run = v_run!("position");
    let int_var = iv!("counter", vec!(1, 2, 3));
    let int_var_run = iv_run!("counter");
    let bool_var = bv!("toggle");
    let bool_var_run = bv_run!("toggle");
    let float_var = fv!("speed", vec!(0.1, 0.3));
    let float_var_run = fv_run!("speed");
    let array_var = av!("valid_lists", vec!(vec!("a", "b", "c"), vec!("c", "a", "b")));
    let array_var_run = av_run!("valid_lists");
    assert_eq!((SPVariableType::Planner, SPValueType::String), string_var.has_type());
    assert_eq!((SPVariableType::Runner, SPValueType::String), string_var_run.has_type());
    assert_eq!((SPVariableType::Planner, SPValueType::Int32), int_var.has_type());
    assert_eq!((SPVariableType::Runner, SPValueType::Int32), int_var_run.has_type());
    assert_eq!((SPVariableType::Planner, SPValueType::Bool), bool_var.has_type());
    assert_eq!((SPVariableType::Runner, SPValueType::Bool), bool_var_run.has_type());
    assert_eq!((SPVariableType::Planner, SPValueType::Float64), float_var.has_type());
    assert_eq!((SPVariableType::Runner, SPValueType::Float64), float_var_run.has_type());
    assert_eq!((SPVariableType::Planner, SPValueType::Array), array_var.has_type());
    assert_eq!((SPVariableType::Runner, SPValueType::Array), array_var_run.has_type());
}