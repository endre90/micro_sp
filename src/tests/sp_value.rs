#![allow(unused_imports)]
#![allow(dead_code)]
use crate::{SPValue, SPValueType, SPVariable, ToSPValue};
use std::collections::{HashMap, HashSet};
use std::time::SystemTime;

#[test]
fn test_sp_value_is_type() {
    let int_val = 123.to_spvalue();
    let float_val = 0.123.to_spvalue();
    let bool_val = false.to_spvalue();
    let string_val = "asdf".to_spvalue();
    let time_val = SystemTime::now().to_spvalue();
    let array_val = vec!("asdf", "asdf2").to_spvalue();
    let array_val2 = vec!(1, 3, 5).to_spvalue();
    let unknown_val = SPValue::Unknown;
    assert_eq!(true, int_val.is_type(SPValueType::Int32));
    assert_eq!(true, bool_val.is_type(SPValueType::Bool));
    assert_eq!(true, string_val.is_type(SPValueType::String));
    assert_eq!(true, float_val.is_type(SPValueType::Float64));
    assert_eq!(true, time_val.is_type(SPValueType::Time));
    assert_eq!(true, array_val.is_type(SPValueType::String));
    assert_eq!(true, array_val.is_array());
    assert_eq!(true, array_val2.is_type(SPValueType::Int32));
    assert_eq!(true, unknown_val.is_type(SPValueType::Unknown));
    assert_eq!(false, int_val.is_type(SPValueType::Bool));
    assert_eq!(false, string_val.is_type(SPValueType::Int32));
    assert_eq!(false, bool_val.is_type(SPValueType::String));
}

#[test]
fn test_sp_value_has_type() {
    let int_val = 123.to_spvalue();
    let float_val = 0.123.to_spvalue();
    let bool_val = false.to_spvalue();
    let string_val = "asdf".to_spvalue();
    let time_val = SystemTime::now().to_spvalue();
    let array_val = vec!("asdf", "asdf2").to_spvalue();
    let array_val2 = vec!(1, 3, 5).to_spvalue();
    let unknown_val = SPValue::Unknown;
    assert_eq!(SPValueType::Int32, int_val.has_type());
    assert_eq!(SPValueType::Bool, bool_val.has_type());
    assert_eq!(SPValueType::String, string_val.has_type());
    assert_eq!(SPValueType::Float64, float_val.has_type());
    assert_eq!(SPValueType::Time, time_val.has_type());
    assert_eq!(SPValueType::String, array_val.has_type());
    assert_eq!(SPValueType::Int32, array_val2.has_type());
    assert_eq!(SPValueType::Unknown, unknown_val.has_type());
    assert_ne!(SPValueType::Bool, int_val.has_type());
    assert_ne!(SPValueType::Int32, string_val.has_type());
    assert_ne!(SPValueType::String, bool_val.has_type());
}


#[test]
fn test_sp_value_to_string() {
    let int_val = 123.to_spvalue();
    let float_val = 0.123.to_spvalue();
    let bool_val = false.to_spvalue();
    let string_val = "asdf".to_spvalue();
    let array_val = vec!("asdf", "asdf2").to_spvalue();
    let array_val2 = vec!(1, 3, 5).to_spvalue();
    let unknown_val = SPValue::Unknown;
    assert_eq!(int_val.to_string(), "123");
    assert_eq!(float_val.to_string(), "0.123");
    assert_eq!(bool_val.to_string(), "false");
    assert_eq!(string_val.to_string(), "asdf");
    assert_eq!(array_val.to_string(), "[asdf, asdf2]");
    assert_eq!(array_val2.to_string(), "[1, 3, 5]");
    assert_eq!(unknown_val.to_string(), "[unknown]");
}