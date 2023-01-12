#![allow(unused_imports)]
#![allow(dead_code)]
use crate::{
    a, assign, bv, bv_run, fv, fv_run, iv, iv_run, t, t_plus, v, v_run, Predicate, Transition, eq, ToSPWrappedVar,
};
use crate::{
    Action, SPAssignment, SPValue, SPValueType, SPVariable, SPVariableType, State, ToSPValue,
    ToSPWrapped,
};
use std::collections::{HashMap, HashSet};
use std::f32::consts::E;

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
fn test_transition_new() {
    let weight = fv!("weight", vec!(80.0, 82.5, 85.0));
    let a1 = a!(weight.clone(), 85.0.wrap());
    let t1 = Transition::new(
        "gains_weight",
        Predicate::TRUE,
        Predicate::TRUE,
        vec![a1.clone()],
        vec![],
    );
    let t2 = Transition::new(
        "gains_weight",
        Predicate::TRUE,
        Predicate::TRUE,
        vec![a1],
        vec![],
    );
    assert_eq!(t1, t2);
}

#[test]
fn test_transition_new_macro() {
    let weight = fv!("weight", vec!(80.0, 82.5, 85.0));
    let a1 = a!(weight.clone(), 85.0.wrap());
    let t1 = t!("gains_weight", Predicate::TRUE, vec!(a1.clone()));
    let t2 = t!("gains_weight", Predicate::TRUE, vec!(a1));
    assert_eq!(t1, t2);
}

#[test]
fn test_transition_eval_planning() {
    let s = State::from_vec(&john_doe());
    let weight = fv!("weight", vec!(80.0, 82.5, 85.0));
    let a1 = a!(weight.clone(), 85.0.wrap());
    let t1 = t!("gains_weight", Predicate::TRUE, vec!(a1.clone()));
    let t2 = t!("gains_weight", Predicate::FALSE, vec!(a1));
    assert!(t1.eval_planning(&s));
    assert!(!t2.eval_planning(&s));
}

#[test]
fn test_transition_eval_running() {
    let s = State::from_vec(&john_doe());
    let weight = fv!("weight", vec!(80.0, 82.5, 85.0));
    let a1 = a!(weight.clone(), 85.0.wrap());
    let t1 = t_plus!(
        "gains_weight",
        Predicate::TRUE,
        Predicate::TRUE,
        vec!(a1.clone()),
        Vec::<Action>::new()
    );
    let t2 = t_plus!(
        "gains_weight",
        Predicate::TRUE,
        Predicate::FALSE,
        vec!(a1),
        Vec::<Action>::new()
    );
    assert!(t1.eval_running(&s));
    assert!(!t2.eval_running(&s));
}

// #[test]
// fn test_transition_take_planning() {
//     let s = State::from_vec(&john_doe());
//     let weight = fv!("weight", vec!(80.0, 82.5, 85.0));
//     let a1 = a!(weight.clone(), 82.5.wrap());
//     let a2 = a!(weight.clone(), 85.0.wrap());
//     let t1 = t!("gains_weight", eq!(weight.wrap(), 80.0.wrap()), vec!(a1));
//     let t2 = t!("gains_weight_again", eq!(weight.wrap(), 82.5.wrap()), vec!(a1));
//     let s_next_1 = t1.take(&s);
//     let s_next_2 = t2.take(&s_next_1);
//     let new_state = s.clone().update("weight", &87.to_spval());
//     assert_eq!(s_next_2, new_state);
// }

// #[test]
// #[should_panic]
// fn test_transition_take_panic() {
//     let s = State::new(&john_doe());
//     let weight = SPVariable::new(
//         "weight",
//         &SPValueType::Int32,
//         &vec![80.to_spval(), 85.to_spval(), 90.to_spval(), 87.to_spval()],
//     );
//     let a1 = a!(&weight, 87.cl());
//     let t1 = t!("gains_weight", eq!(&weight.cr(), 85.cl()), vec!(a1));
//     t1.take(&s);
// }

// #[test]
// fn test_transition_ordering() {
//     let s = State::new(&john_doe());
//     let weight = SPVariable::new(
//         "weight",
//         &SPValueType::Int32,
//         &vec![80.to_spval(), 85.to_spval(), 90.to_spval(), 87.to_spval()],
//     );
//     let a1 = a!(&weight, 85.cl());
//     let a2 = a!(&weight, 87.cl());
//     let a3 = a!(&weight, 90.cl());
//     let t1 = t!("gains_weight_again", eq!(&weight.cr(), 80.cl()), vec!(a1, a2, a3));
//     let s_next_1 = t1.take(&s);
//     assert_eq!(s_next_1.state.get("weight").unwrap().val, 90.to_spval());
// }

// #[test]
// #[should_panic]
// fn test_transition_ordering_panic() {
//     let s = State::new(&john_doe());
//     let weight = SPVariable::new(
//         "weight",
//         &SPValueType::Int32,
//         &vec![80.to_spval(), 85.to_spval(), 90.to_spval(), 87.to_spval()],
//     );
//     let a1 = a!(&weight, 85.cl());
//     let a2 = a!(&weight, 87.cl());
//     let a3 = a!(&weight, 90.cl());
//     let t1 = t!("gains_weight_again", eq!(&weight.cr(), 80.cl()), vec!(a1, a3, a2));
//     let s_next_1 = t1.take(&s);
//     assert_eq!(s_next_1.state.get("weight").unwrap().val, 90.to_spval());
// }

// #[test]
// fn test_transition_equality() {
//     let weight = SPVariable::new(
//         "weight",
//         &SPValueType::Int32,
//         &vec![80.to_spval(), 85.to_spval(), 90.to_spval(), 87.to_spval()],
//     );
//     let a1 = a!(&weight, 85.cl());
//     let a2 = a!(&weight, 87.cl());
//     let a3 = a!(&weight, 90.cl());

//     // Transitions should be equal even if they have a different name
//     let t1 = t!("gains_weight_again", eq!(&weight.cr(), 80.cl()), vec!(a1.clone(), a2.clone(), a3.clone()));
//     let t2 = t!("gains_weight_again", eq!(&weight.cr(), 80.cl()), vec!(a1.clone(), a2.clone(), a3.clone()));
//     let t3 = t!("loses_weight_again", eq!(&weight.cr(), 80.cl()), vec!(a1.clone(), a2.clone(), a3.clone()));
//     let t4 = t!("loses_weight_again", eq!(&weight.cr(), 80.cl()), vec!(a3.clone(), a2.clone()));
//     let t5 = t!("loses_weight_again", eq!(&weight.cr(), 85.cl()), vec!(a3.clone(), a2.clone()));
//     assert_eq!(t1, t2);
//     assert_eq!(t1, t3);
//     assert_ne!(t3, t4);
//     assert_ne!(t4, t5);
// }

// #[test]
// fn test_transition_contained_in_vec() {
//     let weight = SPVariable::new(
//         "weight",
//         &SPValueType::Int32,
//         &vec![80.to_spval(), 85.to_spval(), 90.to_spval(), 87.to_spval()],
//     );
//     let a1 = a!(&weight, 85.cl());
//     let a2 = a!(&weight, 87.cl());
//     let a3 = a!(&weight, 90.cl());

//     // Transitions should be equal even if they have a different name
//     let t1 = t!("gains_weight_again", eq!(&weight.cr(), 80.cl()), vec!(a1.clone(), a2.clone(), a3.clone()));
//     let t2 = t!("gains_weight_again", eq!(&weight.cr(), 80.cl()), vec!(a1.clone(), a2.clone(), a3.clone()));
//     let t3 = t!("loses_weight_again", eq!(&weight.cr(), 80.cl()), vec!(a1.clone(), a2.clone(), a3.clone()));
//     let t4 = t!("loses_weight_again", eq!(&weight.cr(), 80.cl()), vec!(a3.clone(), a2.clone()));
//     let t5 = t!("loses_weight_again", eq!(&weight.cr(), 85.cl()), vec!(a3.clone(), a2.clone()));
//     let trans2 = vec!(t2);
//     let trans3 = vec!(t3);
//     let trans4 = vec!(t4.clone());
//     let trans5 = vec!(t4, t5);
//     assert!(trans2.contains(&t1));
//     assert!(trans3.contains(&t1));
//     assert!(!trans4.contains(&t1));
//     assert!(!trans5.contains(&t1));
// }

// #[test]
// fn test_transition_vec_equality() {
//     let weight = SPVariable::new(
//         "weight",
//         &SPValueType::Int32,
//         &vec![80.to_spval(), 85.to_spval(), 90.to_spval(), 87.to_spval()],
//     );
//     let a1 = a!(&weight, 85.cl());
//     let a2 = a!(&weight, 87.cl());
//     let a3 = a!(&weight, 90.cl());

//     // Transitions should be equal even if they have a different name
//     let t1 = t!("gains_weight_again", eq!(&weight.cr(), 80.cl()), vec!(a1.clone(), a2.clone(), a3.clone()));
//     let t2 = t!("gains_weight_again", eq!(&weight.cr(), 80.cl()), vec!(a1.clone(), a2.clone(), a3.clone()));
//     let t3 = t!("loses_weight_again", eq!(&weight.cr(), 80.cl()), vec!(a1.clone(), a2.clone(), a3.clone()));
//     let t4 = t!("loses_weight_again", eq!(&weight.cr(), 80.cl()), vec!(a3.clone(), a2.clone()));
//     let trans1 = vec!(t1.clone(), t3.clone());
//     let trans2 = vec!(t2.clone(), t3.clone());
//     let trans3 = vec!(t2.clone(), t4.clone());
//     assert_eq!(trans1, trans2);
//     assert_ne!(trans2, trans3);
// }
