#![allow(unused_imports)]
#![allow(dead_code)]
use crate::{bv, bv_run, fv, fv_run, iv, iv_run, v, v_run, assign, Action, ToSPWrapped, a};
use crate::{SPAssignment, SPValue, SPValueType, SPVariable, SPVariableType, State, ToSPValue};
use std::collections::{HashMap, HashSet};

fn john_doe() -> Vec<(SPVariable, SPValue)> {
    let name = v!("name", vec!("John", "Jack"));
    let surname = v!("surname", vec!("Doe", "Crawford"));
    let height = iv!("height", vec!(180, 185, 190));
    let weight = fv!("weight", vec!(80.0, 82.5, 85.0));
    let smart = bv!("smart");

    vec![
        (name, "John".to_spvalue()),
        (surname, "Doe".to_spvalue()),
        (height, 185.to_spvalue()),
        (weight, 80.0.to_spvalue()),
        (smart, true.to_spvalue()),
    ]
}

#[test]
fn test_action_assign() {
    let s = State::from_vec(&john_doe());
    let weight = fv!("weight", vec!(80.0, 82.5, 85.0));
    let a1 = Action::new(weight.clone(), 82.5.wrap());
    let a2 = Action::new(weight.clone(), 85.0.wrap());
    let s_next_1 = a1.assign(&s);
    let s_next_2 = a2.assign(&s_next_1);
    assert_eq!(s_next_1.get_value("weight"), 82.5.to_spvalue());
    assert_eq!(s_next_2.get_value("weight"), 85.0.to_spvalue());
}

#[test]
#[should_panic]
fn test_action_assign_panic() {
    let s = State::from_vec(&john_doe());
    let bitrhyear = iv!("bitrhyear", vec!(1967, 1966));
    let a1 = Action::new(bitrhyear.clone(), 1967.wrap());
    a1.assign(&s);
}

#[test]
fn test_action_assign_macro() {
    let s = State::from_vec(&john_doe());
    let weight = fv!("weight", vec!(80.0, 82.5, 85.0));
    let a1 = a!(weight.clone(), 82.5.wrap());
    let a2 = a!(weight.clone(), 85.0.wrap());
    let s_next_1 = a1.assign(&s);
    let s_next_2 = a2.assign(&s_next_1);
    assert_eq!(s_next_1.get_value("weight"), 82.5.to_spvalue());
    assert_eq!(s_next_2.get_value("weight"), 85.0.to_spvalue());
}

#[test]
#[should_panic]
fn test_action_assign_panic_macro() {
    let s = State::from_vec(&john_doe());
    let bitrhyear = iv!("bitrhyear", vec!(1967, 1966));
    let a1 = a!(bitrhyear.clone(), 1967.wrap());
    a1.assign(&s);
}