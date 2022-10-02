#![allow(unused_imports)]
#![allow(dead_code)]
use micro_sp::{State, Action, SPValue, ToSPValue, ToVal, a};
use std::collections::{HashMap, HashSet};

fn john_doe() -> HashMap<String, SPValue> {
    HashMap::from([
        ("name".to_string(), "John".to_spvalue()),
        ("surname".to_string(), "Doe".to_spvalue()),
        ("height".to_string(), 185.to_spvalue()),
        ("weight".to_string(), 80.5.to_spvalue()),
        ("smart".to_string(), true.to_spvalue()),
    ])
}

#[test]
fn test_action_assign() {
    let john_doe = john_doe();
    let s = State::new(john_doe.clone());
    let a1 = Action::new("weight", 85.to_val());
    let a2 = Action::new("weight", 87.5.to_val());
    let s_next_1 = a1.assign(&s);
    let s_next_2 = a2.assign(&s_next_1);
    assert_eq!(s_next_1.get("weight"), 85.to_spvalue());
    assert_eq!(s_next_2.get("weight"), 87.5.to_spvalue());
}

#[test]
#[should_panic]
fn test_action_assign_panic() {
    let john_doe = john_doe();
    let s = State::new(john_doe.clone());
    let a1 = Action::new("bitrhyear", 1967.to_val());
    a1.assign(&s);
}

#[test]
fn test_action_assign_macro() {
    let john_doe = john_doe();
    let s = State::new(john_doe.clone());
    let a1 = a!("weight", 85.to_val());
    let a2 = a!("weight", 87.5.to_val());
    let s_next_1 = a1.assign(&s);
    let s_next_2 = a2.assign(&s_next_1);
    assert_eq!(s_next_1.get("weight"), 85.to_spvalue());
    assert_eq!(s_next_2.get("weight"), 87.5.to_spvalue());
}

#[test]
#[should_panic]
fn test_action_assign_macro_panic() {
    let john_doe = john_doe();
    let s = State::new(john_doe.clone());
    let a1 = a!("bitrhyear", 1967.to_val());
    a1.assign(&s);
}