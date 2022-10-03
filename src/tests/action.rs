#![allow(unused_imports)]
#![allow(dead_code)]
use micro_sp::{SPValue, SPValueType, SPVariable, State, ToSPValue, ToSPCommon, ToSPCommonVar, Action};
use std::collections::{HashMap, HashSet};

fn john_doe() -> HashMap<SPVariable, SPValue> {
    let name = SPVariable::new(
        "name",
        &SPValueType::String,
        &vec!["John".to_spvalue(), "Jack".to_spvalue()],
    );
    let surname = SPVariable::new(
        "surname",
        &SPValueType::String,
        &vec!["Doe".to_spvalue(), "Crawford".to_spvalue()],
    );
    let height = SPVariable::new(
        "height",
        &SPValueType::Int32,
        &vec![180.to_spvalue(), 185.to_spvalue(), 190.to_spvalue()],
    );
    let weight = SPVariable::new(
        "weight",
        &SPValueType::Int32,
        &vec![80.to_spvalue(), 85.to_spvalue(), 90.to_spvalue()],
    );
    let smart = SPVariable::new(
        "smart",
        &SPValueType::Bool,
        &vec![true.to_spvalue(), false.to_spvalue()],
    );
    HashMap::from([
        (name, "John".to_spvalue()),
        (surname, "Doe".to_spvalue()),
        (height, 185.to_spvalue()),
        (weight, 80.to_spvalue()),
        (smart, true.to_spvalue()),
    ])
}

#[test]
fn test_action_assign() {
    let john_doe = john_doe();
    let s = State::new(&john_doe);
    let a1 = Action::new("weight", 85.to_val());
    let a2 = Action::new("weight", 87.to_val());
    let s_next_1 = a1.assign(&s);
    let s_next_2 = a2.assign(&s_next_1);
    assert_eq!(s_next_1.get("weight"), 85.to_spvalue());
    assert_eq!(s_next_2.get("weight"), 87.to_spvalue());
}

#[test]
#[should_panic]
fn test_action_assign_panic() {
    let john_doe = john_doe();
    let s = State::new(john_doe.clone());
    let a1 = Action::new("bitrhyear", 1967.to_val());
    a1.assign(&s);
}
