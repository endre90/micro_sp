#![allow(unused_imports)]
#![allow(dead_code)]
use micro_sp::{SPValue, SPValueType, SPVariable, State, ToSPValue};
use std::collections::{HashMap, HashSet};

#[test]
fn test_sp_value_is_type() {
    let int_val = 123.to_spval();
    let bool_val = false.to_spval();
    let string_val = "asdf".to_spval();
    assert_eq!(true, int_val.is_type(SPValueType::Int32));
    assert_eq!(true, bool_val.is_type(SPValueType::Bool));
    assert_eq!(true, string_val.is_type(SPValueType::String));
    assert_eq!(false, int_val.is_type(SPValueType::Bool));
    assert_eq!(false, string_val.is_type(SPValueType::Int32));
    assert_eq!(false, bool_val.is_type(SPValueType::String));
}

#[test]
fn test_sp_value_has_type() {
    let int_val = 123.to_spval();
    let bool_val = false.to_spval();
    let string_val = "asdf".to_spval();
    assert_eq!(SPValueType::Int32, int_val.has_type());
    assert_eq!(SPValueType::Bool, bool_val.has_type());
    assert_eq!(SPValueType::String, string_val.has_type());
    assert_ne!(SPValueType::Bool, int_val.has_type());
    assert_ne!(SPValueType::Int32, string_val.has_type());
    assert_ne!(SPValueType::String, bool_val.has_type());
}