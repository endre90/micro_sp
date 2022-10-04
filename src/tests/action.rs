#![allow(unused_imports)]
#![allow(dead_code)]
use micro_sp::{SPValue, SPValueType, SPVariable, State, ToSPValue, ToSPCommon, ToSPCommonVar, Action, ToSPVariable};
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
    let a1 = Action::new("weight".to_spvar(&s), 85.to_comval());
    let a2 = Action::new("weight".to_spvar(&s), 90.to_comval());
    let s_next_1 = a1.assign(&s);
    let s_next_2 = a2.assign(&s_next_1);
    assert_eq!(s_next_1.get_spval("weight"), 85.to_spval());
    assert_eq!(s_next_2.get_spval("weight"), 90.to_spval());
}

#[test]
#[should_panic]
fn test_action_assign_panic() {
    let john_doe = john_doe();
    let s = State::new(&john_doe);
    let a1 = Action::new("bitrhyear".to_spvar(&s), 1967.to_comval());
    a1.assign(&s);
}
