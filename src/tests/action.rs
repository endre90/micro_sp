#![allow(unused_imports)]
#![allow(dead_code)]
use crate::{SPValue, SPValueType, SPVariable, State, ToSPValue, ToSPCommon, ToSPCommonVar, Action, a};
use std::collections::{HashMap, HashSet};

fn john_doe() -> HashMap<SPVariable, SPValue> {
    let name = SPVariable::new(
        "name",
        &SPValueType::String,
        &vec!["John".to_spval(), "Jack".to_spval()],
    );
    let surname = SPVariable::new(
        "surname",
        &SPValueType::String,
        &vec!["Doe".to_spval(), "Crawford".to_spval()],
    );
    let height = SPVariable::new(
        "height",
        &SPValueType::Int32,
        &vec![180.to_spval(), 185.to_spval(), 190.to_spval()],
    );
    let weight = SPVariable::new(
        "weight",
        &SPValueType::Int32,
        &vec![80.to_spval(), 85.to_spval(), 90.to_spval()],
    );
    let smart = SPVariable::new(
        "smart",
        &SPValueType::Bool,
        &vec![true.to_spval(), false.to_spval()],
    );
    HashMap::from([
        (name, "John".to_spval()),
        (surname, "Doe".to_spval()),
        (height, 185.to_spval()),
        (weight, 80.to_spval()),
        (smart, true.to_spval()),
    ])
}

#[test]
fn test_action_assign() {
    let john_doe = john_doe();
    let s = State::new(&john_doe);
    let weight = SPVariable::new(
        "weight",
        &SPValueType::Int32,
        &vec![80.to_spval(), 85.to_spval(), 90.to_spval()],
    );
    let a1 = Action::new(weight.clone(), 85.cl());
    let a2 = Action::new(weight.clone(), 90.cl());
    let s_next_1 = a1.assign(&s);
    let s_next_2 = a2.assign(&s_next_1);
    assert_eq!(s_next_1.state.get("weight").unwrap().val, 85.to_spval());
    assert_eq!(s_next_2.state.get("weight").unwrap().val, 90.to_spval());
}

#[test]
#[should_panic]
fn test_action_assign_panic() {
    let john_doe = john_doe();
    let s = State::new(&john_doe);
    let bitrhyear = SPVariable::new(
        "bitrhyear",
        &SPValueType::Int32,
        &vec![1967.to_spval(), 1966.to_spval()],
    );
    let a1 = Action::new(bitrhyear.clone(), 1967.cl());
    a1.assign(&s);
}

#[test]
fn test_action_assign_macro() {
    let john_doe = john_doe();
    let s = State::new(&john_doe);
    let weight = SPVariable::new(
        "weight",
        &SPValueType::Int32,
        &vec![80.to_spval(), 85.to_spval(), 90.to_spval()],
    );
    let a1 = a!(&weight, 85.cl());
    let a2 = a!(&weight, 90.cl());
    let s_next_1 = a1.assign(&s);
    let s_next_2 = a2.assign(&s_next_1);
    assert_eq!(s_next_1.state.get("weight").unwrap().val, 85.to_spval());
    assert_eq!(s_next_2.state.get("weight").unwrap().val, 90.to_spval());
}

#[test]
#[should_panic]
fn test_action_assign_macro_panic() {
    let john_doe = john_doe();
    let s = State::new(&john_doe);
    let bitrhyear = SPVariable::new(
        "bitrhyear",
        &SPValueType::Int32,
        &vec![1967.to_spval(), 1966.to_spval()],
    );
    let a1 = a!(&bitrhyear, 1967.cl());
    a1.assign(&s);
}