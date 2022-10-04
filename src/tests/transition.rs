#![allow(unused_imports)]
#![allow(dead_code)]
use micro_sp::{State, Action, Transition, SPValue, SPVariable, SPValueType, ToSPValue, ToSPCommon, ToSPCommonVar, Predicate, a, t, eq, ToSPVariable};
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
        &vec![80.to_spval(), 85.to_spval(), 90.to_spval(), 87.to_spval()],
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
fn test_transition_new() {
    let s = State::new(&john_doe());
    let a1 = a!("weight".to_spvar(&s), 85.to_comval());
    let t1 = Transition::new("gains_weight", Predicate::TRUE, vec!(a1.clone()));
    let t2 = Transition::new("gains_weight", Predicate::TRUE, vec!(a1));
    assert_eq!(t1, t2);
}

#[test]
fn test_transition_new_macro() {
    let s = State::new(&john_doe());
    let a1 = a!("weight".to_spvar(&s), 85.to_comval());
    let t1 = t!("gains_weight", Predicate::TRUE, vec!(a1.clone()));
    let t2 = t!("gains_weight", Predicate::TRUE, vec!(a1));
    assert_eq!(t1, t2);
}

#[test]
fn test_transition_eval() {
    let s = State::new(&john_doe());
    let a1 = a!("weight".to_spvar(&s), 85.to_comval());
    let t1 = t!("gains_weight", Predicate::TRUE, vec!(a1.clone()));
    let t2 = t!("gains_weight", Predicate::FALSE, vec!(a1));
    assert!(t1.eval(&s));
    assert!(!t2.eval(&s));
}

#[test]
fn test_transition_take() {
    let s = State::new(&john_doe());
    let a1 = a!("weight".to_spvar(&s), 85.to_comval());
    let a2 = a!("weight".to_spvar(&s), 87.to_comval());
    let t1 = t!("gains_weight", eq!("weight".to_comvar(&s), 80.to_comval()), vec!(a1));
    let t2 = t!("gains_weight_again", eq!("weight".to_comvar(&s), 85.to_comval()), vec!(a2));
    let s_next_1 = t1.take(&s);
    let s_next_2 = t2.take(&s_next_1);
    let new_state = s.clone().update("weight", &87.to_spval());
    assert_eq!(s_next_2, new_state);
}

#[test]
#[should_panic]
fn test_transition_take_panic() {
    let s = State::new(&john_doe());
    let a1 = a!("weight".to_spvar(&s), 87.to_comval());
    let t1 = t!("gains_weight", eq!("weight".to_comvar(&s), 85.to_comval()), vec!(a1));
    t1.take(&s);
}

#[test]
fn test_transition_ordering() {
    let s = State::new(&john_doe());
    let a1 = a!("weight".to_spvar(&s), 85.to_comval());
    let a2 = a!("weight".to_spvar(&s), 87.to_comval());
    let a3 = a!("weight".to_spvar(&s), 90.to_comval());
    let t1 = t!("gains_weight_again", eq!("weight".to_comvar(&s), 80.to_comval()), vec!(a1, a2, a3));
    let s_next_1 = t1.take(&s);
    assert_eq!(s_next_1.get_spval("weight"), 90.to_spval());
}