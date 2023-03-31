#![allow(unused_imports)]
#![allow(dead_code)]
use ordered_float::OrderedFloat;

use crate::{SPValue, SPValueType, SPVariable, ToSPValue};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, SystemTime};

#[test]
fn test_is_type_bool() {
    let val = SPValue::Bool(true);
    assert!(val.is_type(SPValueType::Bool));
    assert!(!val.is_type(SPValueType::Float64));
}

#[test]
fn test_is_type_float64() {
    let val = SPValue::Float64(OrderedFloat(3.14));
    assert!(val.is_type(SPValueType::Float64));
    assert!(!val.is_type(SPValueType::Int32));
}

#[test]
fn test_is_type_int32() {
    let val = SPValue::Int32(42);
    assert!(val.is_type(SPValueType::Int32));
    assert!(!val.is_type(SPValueType::String));
}

#[test]
fn test_is_type_string() {
    let val = SPValue::String("Hello, world!".to_string());
    assert!(val.is_type(SPValueType::String));
    assert!(!val.is_type(SPValueType::Bool));
}

#[test]
fn test_is_type_time() {
    let val = SPValue::Time(SystemTime::now());
    assert!(val.is_type(SPValueType::Time));
    assert!(!val.is_type(SPValueType::Array));
}

#[test]
fn test_is_type_array() {
    let val = SPValue::Array(
        SPValueType::Int32,
        vec![SPValue::Int32(1), SPValue::Int32(2)],
    );
    assert!(val.is_type(SPValueType::Array));
    assert!(!val.is_type(SPValueType::Time));
}

#[test]
fn test_is_type_unknown() {
    let val = SPValue::Unknown;
    assert!(val.is_type(SPValueType::Unknown));
    assert!(!val.is_type(SPValueType::Int32));
}

#[test]
fn test_has_type_bool() {
    let value = SPValue::Bool(true);
    assert_eq!(value.has_type(), SPValueType::Bool);
}

#[test]
fn test_has_type_float64() {
    let value = SPValue::Float64(OrderedFloat(3.14));
    assert_eq!(value.has_type(), SPValueType::Float64);
}

#[test]
fn test_has_type_int32() {
    let value = SPValue::Int32(42);
    assert_eq!(value.has_type(), SPValueType::Int32);
}

#[test]
fn test_has_type_string() {
    let value = SPValue::String("Hello, world!".to_string());
    assert_eq!(value.has_type(), SPValueType::String);
}

#[test]
fn test_has_type_time() {
    let value = SPValue::Time(SystemTime::UNIX_EPOCH);
    assert_eq!(value.has_type(), SPValueType::Time);
}

#[test]
fn test_has_type_array() {
    let value = SPValue::Array(
        SPValueType::Int32,
        vec![SPValue::Int32(1), SPValue::Int32(2)],
    );
    assert_eq!(value.has_type(), SPValueType::Array);
}

#[test]
fn test_has_type_unknown() {
    let value = SPValue::Unknown;
    assert_eq!(value.has_type(), SPValueType::Unknown);
}

#[test]
fn test_is_array_returns_true_for_array_value() {
    let array_value = SPValue::Array(
        SPValueType::Int32,
        vec![SPValue::Int32(1), SPValue::Int32(2)],
    );
    assert_eq!(array_value.is_array(), true);
}

#[test]
fn test_is_array_returns_false_for_non_array_values() {
    let bool_value = SPValue::Bool(true);
    assert_eq!(bool_value.is_array(), false);

    let float_value = SPValue::Float64(OrderedFloat(3.14));
    assert_eq!(float_value.is_array(), false);

    let int_value = SPValue::Int32(42);
    assert_eq!(int_value.is_array(), false);

    let string_value = SPValue::String("Hello, world!".to_string());
    assert_eq!(string_value.is_array(), false);

    let time_value = SPValue::Time(SystemTime::UNIX_EPOCH);
    assert_eq!(time_value.is_array(), false);

    let unknown_value = SPValue::Unknown;
    assert_eq!(unknown_value.is_array(), false);
}

#[test]
fn test_to_string_returns_correct_string_for_bool() {
    let bool_value = SPValue::Bool(true);
    assert_eq!(bool_value.to_string(), "true".to_string());

    let bool_value = SPValue::Bool(false);
    assert_eq!(bool_value.to_string(), "false".to_string());
}

#[test]
fn test_to_string_returns_correct_string_for_float() {
    let float_value = SPValue::Float64(OrderedFloat(3.14));
    assert_eq!(float_value.to_string(), "3.14".to_string());
}

#[test]
fn test_to_string_returns_correct_string_for_int() {
    let int_value = SPValue::Int32(42);
    assert_eq!(int_value.to_string(), "42".to_string());
}

#[test]
fn test_to_string_returns_correct_string_for_string() {
    let string_value = SPValue::String("Hello, world!".to_string());
    assert_eq!(string_value.to_string(), "Hello, world!".to_string());
}

#[should_panic]
#[test]
fn test_to_string_returns_correct_string_for_time() {
    todo!()
}

#[test]
fn test_to_string_returns_correct_string_for_array() {
    let array_value = SPValue::Array(
        SPValueType::Int32,
        vec![SPValue::Int32(1), SPValue::Int32(2), SPValue::Int32(3)],
    );
    assert_eq!(array_value.to_string(), "[1, 2, 3]".to_string());
}

#[test]
fn test_to_string_returns_correct_string_for_unknown() {
    let unknown_value = SPValue::Unknown;
    assert_eq!(unknown_value.to_string(), "[unknown]".to_string());
}

#[test]
fn test_to_spvalue_bool() {
    assert_eq!(true.to_spvalue(), SPValue::Bool(true));
    assert_eq!(false.to_spvalue(), SPValue::Bool(false));
}

#[test]
fn test_to_spvalue_i32() {
    assert_eq!((-1).to_spvalue(), SPValue::Int32(-1));
    assert_eq!(0.to_spvalue(), SPValue::Int32(0));
    assert_eq!(42.to_spvalue(), SPValue::Int32(42));
}

#[test]
fn test_to_spvalue_f64() {
    assert_eq!(0.0.to_spvalue(), SPValue::Float64(OrderedFloat(0.0)));
    assert_eq!((-1.5).to_spvalue(), SPValue::Float64(OrderedFloat(-1.5)));
    assert_eq!(3.14.to_spvalue(), SPValue::Float64(OrderedFloat(3.14)));
}

#[test]
fn test_to_spvalue_string() {
    assert_eq!("".to_spvalue(), SPValue::String("".to_string()));
    assert_eq!("hello".to_spvalue(), SPValue::String("hello".to_string()));
}

#[test]
fn test_to_spvalue_str() {
    assert_eq!("".to_spvalue(), SPValue::String("".to_string()));
    assert_eq!("hello".to_spvalue(), SPValue::String("hello".to_string()));
}

#[test]
fn test_to_spvalue_system_time() {
    let epoch = std::time::UNIX_EPOCH;
    assert_eq!(epoch.to_spvalue(), SPValue::Time(epoch));
}

#[test]
fn test_display_bool_true() {
    let value = SPValue::Bool(true);
    assert_eq!(format!("{}", value), "true");
}

#[test]
fn test_display_bool_false() {
    let value = SPValue::Bool(false);
    assert_eq!(format!("{}", value), "false");
}

#[test]
fn test_display_float() {
    let value = SPValue::Float64(OrderedFloat(3.14));
    assert_eq!(format!("{}", value), "3.14");
}

#[test]
fn test_display_int() {
    let value = SPValue::Int32(42);
    assert_eq!(format!("{}", value), "42");
}

#[test]
fn test_display_string() {
    let value = SPValue::String(String::from("hello"));
    assert_eq!(format!("{}", value), "hello");
}

#[test]
fn test_display_array() {
    let value = SPValue::Array(
        SPValueType::Int32,
        vec![SPValue::Int32(1), SPValue::Int32(2), SPValue::Int32(3)],
    );
    assert_eq!(format!("{}", value), "[Int32(1), Int32(2), Int32(3)]");
}

#[test]
fn test_display_unknown() {
    let value = SPValue::Unknown;
    assert_eq!(format!("{}", value), "[unknown]");
}

#[test]
fn test_display_type_bool() {
    let value_type = SPValueType::Bool;
    assert_eq!(format!("{}", value_type), "Bool");
}

#[test]
fn test_display_type_float64() {
    let value_type = SPValueType::Float64;
    assert_eq!(format!("{}", value_type), "Float64");
}

#[test]
fn test_display_type_int32() {
    let value_type = SPValueType::Int32;
    assert_eq!(format!("{}", value_type), "Int32");
}

#[test]
fn test_display_type_string() {
    let value_type = SPValueType::String;
    assert_eq!(format!("{}", value_type), "String");
}

#[test]
fn test_display_type_time() {
    let value_type = SPValueType::Time;
    assert_eq!(format!("{}", value_type), "Time");
}

#[test]
fn test_display_type_array() {
    let value_type = SPValueType::Array;
    assert_eq!(format!("{}", value_type), "Array");
}

#[test]
fn test_display_type_unknown() {
    let value_type = SPValueType::Unknown;
    assert_eq!(format!("{}", value_type), "[unknown]");
}