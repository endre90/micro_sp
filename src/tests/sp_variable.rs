#![allow(unused_imports)]
#![allow(dead_code)]
use ordered_float::OrderedFloat;

use crate::{
    av_command, av_estimated, av_measured, av_runner, bv_command, bv_estimated, bv_measured,
    bv_runner, fv_command, fv_estimated, fv_measured, fv_runner, iv_command, iv_estimated,
    iv_measured, iv_runner, v_command, v_estimated, v_measured, v_runner,
};
use crate::{SPValue, SPValueType, SPVariable, SPVariableType, ToSPValue};
use std::collections::{HashMap, HashSet};
use std::time::SystemTime;

#[test]
fn test_new_spvariable() {
    let name = "test_var";
    let variable_type = SPVariableType::Measured;
    let value_type = SPValueType::Float64;
    let domain = vec![
        SPValue::Float64(OrderedFloat(1.0)),
        SPValue::Float64(OrderedFloat(2.0)),
    ];
    let spvar = SPVariable::new(name, variable_type.clone(), value_type, domain.clone());

    assert_eq!(spvar.name, name);
    assert_eq!(spvar.variable_type, variable_type);
    assert_eq!(spvar.value_type, value_type);
    assert_eq!(spvar.domain, domain);
}

#[test]
fn test_new_boolean() {
    let variable = SPVariable::new_boolean("test_bool", SPVariableType::Measured);
    assert_eq!(variable.name, "test_bool");
    assert_eq!(variable.variable_type, SPVariableType::Measured);
    assert_eq!(variable.value_type, SPValueType::Bool);
    assert_eq!(variable.domain, vec![false.to_spvalue(), true.to_spvalue()]);
}

#[test]
fn test_new_integer() {
    let domain = vec![0.to_spvalue(), 1.to_spvalue(), 2.to_spvalue()];
    let variable = SPVariable::new_integer("test_int", SPVariableType::Estimated, domain.clone());
    assert_eq!(variable.name, "test_int");
    assert_eq!(variable.variable_type, SPVariableType::Estimated);
    assert_eq!(variable.value_type, SPValueType::Int32);
    assert_eq!(variable.domain, domain);
}

#[test]
fn test_new_float() {
    let domain = vec![0.0.to_spvalue(), 1.0.to_spvalue(), 2.0.to_spvalue()];
    let variable = SPVariable::new_float("test_float", SPVariableType::Command, domain.clone());
    assert_eq!(variable.name, "test_float");
    assert_eq!(variable.variable_type, SPVariableType::Command);
    assert_eq!(variable.value_type, SPValueType::Float64);
    assert_eq!(variable.domain, domain);
}

#[test]
fn test_new_string() {
    let domain = vec![
        "test1".to_spvalue(),
        "test2".to_spvalue(),
        "test3".to_spvalue(),
    ];
    let variable = SPVariable::new_string("test_string", SPVariableType::Runner, domain.clone());
    assert_eq!(variable.name, "test_string");
    assert_eq!(variable.variable_type, SPVariableType::Runner);
    assert_eq!(variable.value_type, SPValueType::String);
    assert_eq!(variable.domain, domain);
}

#[test]
fn test_new_array() {
    let domain = vec![
        SPValue::Array(
            SPValueType::Bool,
            vec![false.to_spvalue(), true.to_spvalue(), false.to_spvalue()],
        ),
        SPValue::Array(SPValueType::Int32, vec![0.to_spvalue(), 1.to_spvalue()]),
    ];
    let variable = SPVariable::new_array("test_array", SPVariableType::Measured, domain.clone());
    assert_eq!(variable.name, "test_array");
    assert_eq!(variable.variable_type, SPVariableType::Measured);
    assert_eq!(variable.value_type, SPValueType::Array);
    assert_eq!(variable.domain, domain);
}

#[test]
fn test_has_type() {
    let v1 = SPVariable::new_boolean("bool_var", SPVariableType::Measured);
    assert_eq!(v1.has_type(), (SPVariableType::Measured, SPValueType::Bool));

    let v2 = SPVariable::new_integer(
        "int_var",
        SPVariableType::Estimated,
        vec![1.to_spvalue(), 2.to_spvalue(), 3.to_spvalue()],
    );
    assert_eq!(
        v2.has_type(),
        (SPVariableType::Estimated, SPValueType::Int32)
    );

    let v3 = SPVariable::new_float(
        "float_var",
        SPVariableType::Command,
        vec![0.1.to_spvalue(), 0.2.to_spvalue()],
    );
    assert_eq!(
        v3.has_type(),
        (SPVariableType::Command, SPValueType::Float64)
    );

    let v4 = SPVariable::new_string(
        "string_var",
        SPVariableType::Runner,
        vec![
            String::from("hello").to_spvalue(),
            String::from("world").to_spvalue(),
        ],
    );
    assert_eq!(v4.has_type(), (SPVariableType::Runner, SPValueType::String));
}

// #[test]
// fn test_new_var() {
//     let int_var = SPVariable::new_integer(
//         "counter",
//         SPVariableType::Estimated,
//         vec![1.to_spvalue(), 3.to_spvalue(), 5.to_spvalue()],
//     );
//     let bool_var = SPVariable::new_boolean("toggle", SPVariableType::Command);
//     let float_var = SPVariable::new_float(
//         "speed",
//         SPVariableType::Measured,
//         vec![0.1.to_spvalue(), 0.3.to_spvalue()],
//     );
//     let string_var = SPVariable::new_string(
//         "position",
//         SPVariableType::Measured,
//         vec!["a".to_spvalue(), "b".to_spvalue(), "c".to_spvalue()],
//     );
//     let array_var = SPVariable::new_array("plan", SPVariableType::Runner, vec![]); // runner, flexible domain
//     let string_var_2 = SPVariable::new_string("goal", SPVariableType::Runner, vec![]); // runner, flexible domain
//     assert_eq!(
//         (SPVariableType::Estimated, SPValueType::Int32),
//         int_var.has_type()
//     );
//     assert_eq!(
//         (SPVariableType::Command, SPValueType::Bool),
//         bool_var.has_type()
//     );
//     assert_eq!(
//         (SPVariableType::Measured, SPValueType::Float64),
//         float_var.has_type()
//     );
//     assert_eq!(
//         (SPVariableType::Measured, SPValueType::String),
//         string_var.has_type()
//     );
//     assert_eq!(
//         (SPVariableType::Runner, SPValueType::Array),
//         array_var.has_type()
//     );
//     assert_eq!(
//         (SPVariableType::Runner, SPValueType::String),
//         string_var_2.has_type()
//     );
// }

// #[test]
// fn test_new_var_macros() {
//     let string_var = v_estimated!("position", vec!("a", "b", "c"));
//     let string_var_run = v_runner!("position");
//     let int_var = iv_estimated!("counter", vec!(1, 2, 3));
//     let int_var_run = iv_runner!("counter");
//     let bool_var = bv_estimated!("toggle");
//     let bool_var_run = bv_runner!("toggle");
//     let float_var = fv_measured!("speed", vec!(0.1, 0.3));
//     let float_var_run = fv_runner!("speed");
//     let array_var = av_estimated!(
//         "valid_lists",
//         vec!(vec!("a", "b", "c"), vec!("c", "a", "b"))
//     );
//     let array_var_run = av_runner!("valid_lists");
//     assert_eq!(
//         (SPVariableType::Estimated, SPValueType::String),
//         string_var.has_type()
//     );
//     assert_eq!(
//         (SPVariableType::Runner, SPValueType::String),
//         string_var_run.has_type()
//     );
//     assert_eq!(
//         (SPVariableType::Estimated, SPValueType::Int32),
//         int_var.has_type()
//     );
//     assert_eq!(
//         (SPVariableType::Runner, SPValueType::Int32),
//         int_var_run.has_type()
//     );
//     assert_eq!(
//         (SPVariableType::Estimated, SPValueType::Bool),
//         bool_var.has_type()
//     );
//     assert_eq!(
//         (SPVariableType::Runner, SPValueType::Bool),
//         bool_var_run.has_type()
//     );
//     assert_eq!(
//         (SPVariableType::Measured, SPValueType::Float64),
//         float_var.has_type()
//     );
//     assert_eq!(
//         (SPVariableType::Runner, SPValueType::Float64),
//         float_var_run.has_type()
//     );
//     assert_eq!(
//         (SPVariableType::Estimated, SPValueType::Array),
//         array_var.has_type()
//     );
//     assert_eq!(
//         (SPVariableType::Runner, SPValueType::Array),
//         array_var_run.has_type()
//     );
// }
