#![allow(unused_imports)]
#![allow(dead_code)]
use micro_sp::{State, Action, Transition, SPValue, ToSPValue, ToVal, ToVar, Predicate, a, t, eq};
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
fn test_transition_new() {
    let a1 = a!("weight", 85.to_val());
    let t1 = Transition::new("gains_weight", Predicate::TRUE, vec!(a1.clone()));
    let t2 = Transition::new("gains_weight", Predicate::TRUE, vec!(a1));
    assert_eq!(t1, t2);
}

#[test]
fn test_transition_new_macro() {
    let a1 = a!("weight", 85.to_val());
    let t1 = t!("gains_weight", Predicate::TRUE, vec!(a1.clone()));
    let t2 = t!("gains_weight", Predicate::TRUE, vec!(a1));
    assert_eq!(t1, t2);
}

#[test]
fn test_transition_eval() {
    let s = State::new(john_doe());
    let a1 = a!("weight", 85.to_val());
    let t1 = t!("gains_weight", Predicate::TRUE, vec!(a1.clone()));
    let t2 = t!("gains_weight", Predicate::FALSE, vec!(a1));
    assert!(t1.eval(&s));
    assert!(!t2.eval(&s));
}

#[test]
fn test_transition_take() {
    let s = State::new(john_doe());
    let a1 = a!("weight", 85.to_val());
    let a2 = a!("weight", 87.5.to_val());
    let t1 = t!("gains_weight", eq!("weight".to_var(), 80.5.to_val()), vec!(a1));
    let t2 = t!("gains_weight_again", eq!("weight".to_var(), 85.to_val()), vec!(a2));
    let s_next_1 = t1.take(&s);
    let s_next_2 = t2.take(&s_next_1);
    let new_state = s.clone().update("weight", 87.5.to_spvalue());
    assert_eq!(s_next_2, new_state);
}

#[test]
#[should_panic]
fn test_transition_take_panic() {
    let s = State::new(john_doe());
    let a1 = a!("weight", 87.5.to_val());
    let t1 = t!("gains_weight", eq!("weight".to_var(), 85.to_val()), vec!(a1));
    t1.take(&s);
}

#[test]
fn test_transition_ordering() {
    let s = State::new(john_doe());
    let a1 = a!("weight", 85.to_val());
    let a2 = a!("weight", 87.5.to_val());
    let a3 = a!("weight", 90.to_val());
    let t1 = t!("gains_weight_again", eq!("weight".to_var(), 80.5.to_val()), vec!(a1, a2, a3));
    let s_next_1 = t1.take(&s);
    assert_eq!(s_next_1.get("weight"), 90.to_spvalue());
}