#![allow(unused_imports)]
#![allow(dead_code)]
use crate::{
    av_command, av_estimated, av_measured, av_runner, bv_command, bv_estimated, bv_measured,
    bv_runner, fv_command, fv_estimated, fv_measured, fv_runner, iv_command, iv_estimated,
    iv_measured, iv_runner, v_command, v_estimated, v_measured, v_runner,
};
use crate::{
    SPAssignment, SPValue, SPValueType, SPVariable, SPVariableType, SPWrapped, ToSPValue,
    ToSPWrapped, ToSPWrappedVar,
};
use std::collections::{HashMap, HashSet};
use std::time::SystemTime;

#[test]
fn test_new_assignment() {
    // Test creating an assignment with the correct value type
    let bool_var = SPVariable::new_boolean("bool_var", SPVariableType::Measured);
    let bool_val = true.to_spvalue();
    let bool_assignment = SPAssignment::new(bool_var.clone(), bool_val.clone());
    assert_eq!(bool_assignment.var, bool_var);
    assert_eq!(bool_assignment.val, bool_val);

    let int_var = SPVariable::new_integer(
        "int_var",
        SPVariableType::Estimated,
        vec![1.to_spvalue(), 2.to_spvalue()],
    );
    let int_val = 1.to_spvalue();
    let int_assignment = SPAssignment::new(int_var.clone(), int_val.clone());
    assert_eq!(int_assignment.var, int_var);
    assert_eq!(int_assignment.val, int_val);

    let float_var = SPVariable::new_float(
        "float_var",
        SPVariableType::Command,
        vec![1.0.to_spvalue(), 2.0.to_spvalue()],
    );
    let float_val = 1.0.to_spvalue();
    let float_assignment = SPAssignment::new(float_var.clone(), float_val.clone());
    assert_eq!(float_assignment.var, float_var);
    assert_eq!(float_assignment.val, float_val);

    let string_var = SPVariable::new_string(
        "string_var",
        SPVariableType::Runner,
        vec!["foo".to_spvalue(), "bar".to_spvalue()],
    );
    let string_val = "foo".to_spvalue();
    let string_assignment = SPAssignment::new(string_var.clone(), string_val.clone());
    assert_eq!(string_assignment.var, string_var);
    assert_eq!(string_assignment.val, string_val);

    let array_var = SPVariable::new_array(
        "array_var",
        SPVariableType::Measured,
        vec![
            vec![1.to_spvalue()].to_spvalue(),
            vec![2.to_spvalue()].to_spvalue(),
        ],
    );
    let array_val = vec![1.to_spvalue()].to_spvalue();
    let array_assignment = SPAssignment::new(array_var.clone(), array_val.clone());
    assert_eq!(array_assignment.var, array_var);
    assert_eq!(array_assignment.val, array_val);

    // // Test creating an assignment with an unknown value type
    let unknown_var = SPVariable::new(
        "unknown_var",
        SPVariableType::Runner,
        SPValueType::Unknown,
        vec![],
    );
    let unknown_val = SPValue::Unknown;
    let unknown_assignment = SPAssignment::new(unknown_var.clone(), unknown_val.clone());
    assert_eq!(unknown_assignment.var, unknown_var);
    assert_eq!(unknown_assignment.val, unknown_val);
}

#[test]
#[should_panic]
fn test_new_assignment_should_panic() {
    let var = SPVariable::new_boolean("test_var", SPVariableType::Measured);
    let compatible_val = SPValue::Bool(true);
    let incompatible_val = SPValue::Int32(42);

    // Test creating a compatible assignment
    let assignment = SPAssignment::new(var.clone(), compatible_val.clone());
    assert_eq!(assignment.var, var.clone());
    assert_eq!(assignment.val, compatible_val);

    // Test creating an incompatible assignment, which should panic
    SPAssignment::new(var.clone(), incompatible_val);
}
