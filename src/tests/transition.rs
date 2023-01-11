#![allow(unused_imports)]
#![allow(dead_code)]
use crate::{State, Action, Transition, SPValue, SPVariable, SPValueType, ToSPValue, ToSPCommon, ToSPCommonVar, Predicate, a, t, eq};
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
    let weight = SPVariable::new(
        "weight",
        &SPValueType::Int32,
        &vec![80.to_spval(), 85.to_spval(), 90.to_spval(), 87.to_spval()],
    );
    let a1 = a!(&weight, 85.cl());
    let t1 = Transition::new("gains_weight", Predicate::TRUE, vec!(a1.clone()));
    let t2 = Transition::new("gains_weight", Predicate::TRUE, vec!(a1));
    assert_eq!(t1, t2);
}

#[test]
fn test_transition_new_macro() {
    let weight = SPVariable::new(
        "weight",
        &SPValueType::Int32,
        &vec![80.to_spval(), 85.to_spval(), 90.to_spval(), 87.to_spval()],
    );
    let a1 = a!(&weight, 85.cl());
    let t1 = t!("gains_weight", Predicate::TRUE, vec!(a1.clone()));
    let t2 = t!("gains_weight", Predicate::TRUE, vec!(a1));
    assert_eq!(t1, t2);
}

#[test]
fn test_transition_eval() {
    let s = State::new(&john_doe());
    let weight = SPVariable::new(
        "weight",
        &SPValueType::Int32,
        &vec![80.to_spval(), 85.to_spval(), 90.to_spval(), 87.to_spval()],
    );
    let a1 = a!(&weight, 85.cl());
    let t1 = t!("gains_weight", Predicate::TRUE, vec!(a1.clone()));
    let t2 = t!("gains_weight", Predicate::FALSE, vec!(a1));
    assert!(t1.eval(&s));
    assert!(!t2.eval(&s));
}

#[test]
fn test_transition_take() {
    let s = State::new(&john_doe());
    let weight = SPVariable::new(
        "weight",
        &SPValueType::Int32,
        &vec![80.to_spval(), 85.to_spval(), 90.to_spval(), 87.to_spval()],
    );
    let a1 = a!(&weight, 85.cl());
    let a2 = a!(&weight, 87.cl());
    let t1 = t!("gains_weight", eq!(&weight.cr(), 80.cl()), vec!(a1));
    let t2 = t!("gains_weight_again", eq!(&weight.cr(), 85.cl()), vec!(a2));
    let s_next_1 = t1.take(&s);
    let s_next_2 = t2.take(&s_next_1);
    let new_state = s.clone().update("weight", &87.to_spval());
    assert_eq!(s_next_2, new_state);
}

#[test]
#[should_panic]
fn test_transition_take_panic() {
    let s = State::new(&john_doe());
    let weight = SPVariable::new(
        "weight",
        &SPValueType::Int32,
        &vec![80.to_spval(), 85.to_spval(), 90.to_spval(), 87.to_spval()],
    );
    let a1 = a!(&weight, 87.cl());
    let t1 = t!("gains_weight", eq!(&weight.cr(), 85.cl()), vec!(a1));
    t1.take(&s);
}

#[test]
fn test_transition_ordering() {
    let s = State::new(&john_doe());
    let weight = SPVariable::new(
        "weight",
        &SPValueType::Int32,
        &vec![80.to_spval(), 85.to_spval(), 90.to_spval(), 87.to_spval()],
    );
    let a1 = a!(&weight, 85.cl());
    let a2 = a!(&weight, 87.cl());
    let a3 = a!(&weight, 90.cl());
    let t1 = t!("gains_weight_again", eq!(&weight.cr(), 80.cl()), vec!(a1, a2, a3));
    let s_next_1 = t1.take(&s);
    assert_eq!(s_next_1.state.get("weight").unwrap().val, 90.to_spval());
}

#[test]
#[should_panic]
fn test_transition_ordering_panic() {
    let s = State::new(&john_doe());
    let weight = SPVariable::new(
        "weight",
        &SPValueType::Int32,
        &vec![80.to_spval(), 85.to_spval(), 90.to_spval(), 87.to_spval()],
    );
    let a1 = a!(&weight, 85.cl());
    let a2 = a!(&weight, 87.cl());
    let a3 = a!(&weight, 90.cl());
    let t1 = t!("gains_weight_again", eq!(&weight.cr(), 80.cl()), vec!(a1, a3, a2));
    let s_next_1 = t1.take(&s);
    assert_eq!(s_next_1.state.get("weight").unwrap().val, 90.to_spval());
}

#[test]
fn test_transition_equality() {
    let weight = SPVariable::new(
        "weight",
        &SPValueType::Int32,
        &vec![80.to_spval(), 85.to_spval(), 90.to_spval(), 87.to_spval()],
    );
    let a1 = a!(&weight, 85.cl());
    let a2 = a!(&weight, 87.cl());
    let a3 = a!(&weight, 90.cl());

    // Transitions should be equal even if they have a different name
    let t1 = t!("gains_weight_again", eq!(&weight.cr(), 80.cl()), vec!(a1.clone(), a2.clone(), a3.clone()));
    let t2 = t!("gains_weight_again", eq!(&weight.cr(), 80.cl()), vec!(a1.clone(), a2.clone(), a3.clone()));
    let t3 = t!("loses_weight_again", eq!(&weight.cr(), 80.cl()), vec!(a1.clone(), a2.clone(), a3.clone()));
    let t4 = t!("loses_weight_again", eq!(&weight.cr(), 80.cl()), vec!(a3.clone(), a2.clone()));
    let t5 = t!("loses_weight_again", eq!(&weight.cr(), 85.cl()), vec!(a3.clone(), a2.clone()));
    assert_eq!(t1, t2);
    assert_eq!(t1, t3);
    assert_ne!(t3, t4);
    assert_ne!(t4, t5);
}

#[test]
fn test_transition_contained_in_vec() {
    let weight = SPVariable::new(
        "weight",
        &SPValueType::Int32,
        &vec![80.to_spval(), 85.to_spval(), 90.to_spval(), 87.to_spval()],
    );
    let a1 = a!(&weight, 85.cl());
    let a2 = a!(&weight, 87.cl());
    let a3 = a!(&weight, 90.cl());

    // Transitions should be equal even if they have a different name
    let t1 = t!("gains_weight_again", eq!(&weight.cr(), 80.cl()), vec!(a1.clone(), a2.clone(), a3.clone()));
    let t2 = t!("gains_weight_again", eq!(&weight.cr(), 80.cl()), vec!(a1.clone(), a2.clone(), a3.clone()));
    let t3 = t!("loses_weight_again", eq!(&weight.cr(), 80.cl()), vec!(a1.clone(), a2.clone(), a3.clone()));
    let t4 = t!("loses_weight_again", eq!(&weight.cr(), 80.cl()), vec!(a3.clone(), a2.clone()));
    let t5 = t!("loses_weight_again", eq!(&weight.cr(), 85.cl()), vec!(a3.clone(), a2.clone()));
    let trans2 = vec!(t2);
    let trans3 = vec!(t3);
    let trans4 = vec!(t4.clone());
    let trans5 = vec!(t4, t5);
    assert!(trans2.contains(&t1));
    assert!(trans3.contains(&t1));
    assert!(!trans4.contains(&t1));
    assert!(!trans5.contains(&t1));
}

#[test]
fn test_transition_vec_equality() {
    let weight = SPVariable::new(
        "weight",
        &SPValueType::Int32,
        &vec![80.to_spval(), 85.to_spval(), 90.to_spval(), 87.to_spval()],
    );
    let a1 = a!(&weight, 85.cl());
    let a2 = a!(&weight, 87.cl());
    let a3 = a!(&weight, 90.cl());

    // Transitions should be equal even if they have a different name
    let t1 = t!("gains_weight_again", eq!(&weight.cr(), 80.cl()), vec!(a1.clone(), a2.clone(), a3.clone()));
    let t2 = t!("gains_weight_again", eq!(&weight.cr(), 80.cl()), vec!(a1.clone(), a2.clone(), a3.clone()));
    let t3 = t!("loses_weight_again", eq!(&weight.cr(), 80.cl()), vec!(a1.clone(), a2.clone(), a3.clone()));
    let t4 = t!("loses_weight_again", eq!(&weight.cr(), 80.cl()), vec!(a3.clone(), a2.clone()));
    let trans1 = vec!(t1.clone(), t3.clone());
    let trans2 = vec!(t2.clone(), t3.clone());
    let trans3 = vec!(t2.clone(), t4.clone());
    assert_eq!(trans1, trans2);
    assert_ne!(trans2, trans3);
}