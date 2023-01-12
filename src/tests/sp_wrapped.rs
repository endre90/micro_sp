#![allow(unused_imports)]
#![allow(dead_code)]
use crate::{av, av_run, bv, bv_run, fv, fv_run, iv, iv_run, v, v_run};
use crate::{
    SPValue, SPValueType, SPVariable, SPVariableType, SPWrapped, ToSPValue, ToSPWrapped,
    ToSPWrappedVar,
};
use std::collections::{HashMap, HashSet};
use std::time::SystemTime;

#[test]
fn test_wrap_values() {
    let int = 123;
    let float = 0.123;
    let bool = false;
    let string = "asdf";
    assert_eq!(SPWrapped::SPValue(SPValue::Int32(123)), int.wrap());
    assert_eq!(
        SPWrapped::SPValue(SPValue::Float64(ordered_float::OrderedFloat(0.123))),
        float.wrap()
    );
    assert_eq!(
        SPWrapped::SPValue(SPValue::Bool(false)),
        bool.wrap()
    );
    assert_eq!(
        SPWrapped::SPValue(SPValue::String("asdf".to_string())),
        string.wrap()
    );
}

#[test]
fn test_wrap_spvalues() {
    let int_val = 123.to_spvalue();
    let float_val = 0.123.to_spvalue();
    let bool_val = false.to_spvalue();
    let string_val = "asdf".to_spvalue();
    assert_eq!(
        SPWrapped::SPValue(SPValue::Int32(123)),
        int_val.wrap()
    );
    assert_eq!(
        SPWrapped::SPValue(SPValue::Float64(ordered_float::OrderedFloat(0.123))),
        float_val.wrap()
    );
    assert_eq!(
        SPWrapped::SPValue(SPValue::Bool(false)),
        bool_val.wrap()
    );
    assert_eq!(
        SPWrapped::SPValue(SPValue::String("asdf".to_string())),
        string_val.wrap()
    );
}

#[test]
fn test_wrap_variables() {
    let string_var = v!("position", vec!("a", "b", "c"));
    let string_var_run = v_run!("position");
    let int_var = iv!("counter", vec!(1, 2, 3));
    let int_var_run = iv_run!("counter");
    let bool_var = bv!("toggle");
    let bool_var_run = bv_run!("toggle");
    let float_var = fv!("speed", vec!(0.1, 0.3));
    let float_var_run = fv_run!("speed");
    assert_eq!(
        SPWrapped::SPVariable(string_var.clone()),
        string_var.wrap()
    );
    assert_eq!(
        SPWrapped::SPVariable(string_var_run.clone()),
        string_var_run.wrap()
    );
    assert_eq!(
        SPWrapped::SPVariable(int_var.clone()),
        int_var.wrap()
    );
    assert_eq!(
        SPWrapped::SPVariable(int_var_run.clone()),
        int_var_run.wrap()
    );
    assert_eq!(
        SPWrapped::SPVariable(bool_var.clone()),
        bool_var.wrap()
    );
    assert_eq!(
        SPWrapped::SPVariable(bool_var_run.clone()),
        bool_var_run.wrap()
    );
    assert_eq!(
        SPWrapped::SPVariable(float_var.clone()),
        float_var.wrap()
    );
    assert_eq!(
        SPWrapped::SPVariable(float_var_run.clone()),
        float_var_run.wrap()
    );
}
