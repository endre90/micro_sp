#![allow(unused_imports)]
#![allow(dead_code)]
use crate::{
    av_command, av_estimated, av_measured, av_runner, bv_command, bv_estimated, bv_measured,
    bv_runner, fv_command, fv_estimated, fv_measured, fv_runner, iv_command, iv_estimated,
    iv_measured, iv_runner, v_command, v_estimated, v_measured, v_runner, a, assign, t, t_plan, eq
};
use crate::{
    Action, SPAssignment, SPValue, SPValueType, SPVariable, SPVariableType, State, ToSPValue,
    ToSPWrapped, Predicate, ToSPWrappedVar, Transition, pred_parser
};
use std::collections::{HashMap, HashSet};
use std::f32::consts::E;

fn john_doe() -> Vec<(SPVariable, SPValue)> {
    let name = v_estimated!("name", vec!("John", "Jack"));
    let surname = v_estimated!("surname", vec!("Doe", "Crawford"));
    let height = iv_estimated!("height", vec!(180, 185, 190));
    let weight = fv_estimated!("weight", vec!(80.0, 82.5, 85.0));
    let smart = bv_estimated!("smart");
    let alive = bv_runner!("alive");

    vec![
        (name, "John".to_spvalue()),
        (surname, "Doe".to_spvalue()),
        (height, 185.to_spvalue()),
        (weight, 80.0.to_spvalue()),
        (smart, true.to_spvalue()),
        (alive, true.to_spvalue()),
    ]
}

#[test]
fn test_transition_new() {
    let weight = fv_estimated!("weight", vec!(80.0, 82.5, 85.0));
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
    let weight = fv_estimated!("weight", vec!(80.0, 82.5, 85.0));
    let a1 = a!(weight.clone(), 85.0.wrap());
    let t1 = t_plan!("gains_weight", Predicate::TRUE, vec!(a1.clone()));
    let t2 = t_plan!("gains_weight", Predicate::TRUE, vec!(a1));
    assert_eq!(t1, t2);
}

#[test]
fn test_transition_eval_planning() {
    let s = State::from_vec(&john_doe());
    let weight = fv_estimated!("weight", vec!(80.0, 82.5, 85.0));
    let a1 = a!(weight.clone(), 85.0.wrap());
    let t1 = t_plan!("gains_weight", Predicate::TRUE, vec!(a1.clone()));
    let t2 = t_plan!("gains_weight", Predicate::FALSE, vec!(a1));
    assert!(t1.eval_planning(&s));
    assert!(!t2.eval_planning(&s));
}

#[test]
fn test_transition_eval_running() {
    let s = State::from_vec(&john_doe());
    let t1 = t!(
        "gains_weight",
        "true",
        "true",
        vec!("var:weight <- 85.0", "var:height <- 190"),
        Vec::<&str>::new(),
        &s
    );
    let t2 = t!(
        "gains_weight",
        "true",
        "false",
        vec!("var:weight <- 85.0"),
        Vec::<&str>::new(),
        &s
    );
    assert!(t1.eval_running(&s));
    assert!(!t2.eval_running(&s));
}

#[test]
#[should_panic]
fn test_transition_planner_var_in_runner_guard_panic() {
    let s = State::from_vec(&john_doe());
    let t1 = t!(
        "gains_weight",
        "true",
        "var:weight == 85.0",
        vec!("var:weight <- 85.0", "var:height <- 190"),
        Vec::<&str>::new(),
        &s
    );
    assert!(t1.eval_running(&s));
}

#[test]
#[should_panic]
fn test_transition_runner_var_in_planner_guard_panic() {
    let s = State::from_vec(&john_doe());
    let t1 = t!(
        "gains_weight",
        "var:alive == true",
        "true",
        vec!("var:weight <- 85.0", "var:height <- 190"),
        Vec::<&str>::new(),
        &s
    );
    assert!(t1.eval_running(&s));
}

#[test]
#[should_panic]
fn test_transition_planner_var_in_runner_action_panic() {
    let s = State::from_vec(&john_doe());
    let t1 = t!(
        "gains_weight",
        "true",
        "true",
        Vec::<&str>::new(),
        vec!("var:weight <- 85.0", "var:height <- 190"),
        &s
    );
    assert!(t1.eval_running(&s));
}

#[test]
#[should_panic]
fn test_transition_runner_var_in_planner_action_panic() {
    let s = State::from_vec(&john_doe());
    let t1 = t!(
        "gains_weight",
        "true",
        "true",
        vec!("var:alive <- false", "var:height <- 190"),
        Vec::<&str>::new(),
        &s
    );
    assert!(t1.eval_running(&s));
}

#[test]
fn test_transition_take_planning() {
    let s = State::from_vec(&john_doe());
    let weight = fv_estimated!("weight", vec!(80.0, 82.5, 85.0));
    let a1 = a!(weight.clone(), 82.5.wrap());
    let a2 = a!(weight.clone(), 85.0.wrap());
    let t1 = t_plan!("gains_weight", eq!(weight.wrap(), 80.0.wrap()), vec!(a1));
    let t2 = t_plan!("gains_weight_again", eq!(weight.wrap(), 82.5.wrap()), vec!(a2));
    let s_next_1 = t1.take_planning(&s);
    let s_next_2 = t2.take_planning(&s_next_1);
    let new_state = s.clone().update("weight", 85.0.to_spvalue());
    assert_eq!(s_next_2, new_state);
}

#[test]
#[should_panic]
fn test_transition_take_planning_panic() {
    let s = State::from_vec(&john_doe());
    let weight = fv_estimated!("weight", vec!(80.0, 82.5, 85.0));
    let a1 = a!(weight.clone(), 87.0.wrap());
    let t1 = t_plan!("gains_weight", eq!(weight.wrap(), 80.0.wrap()), vec!(a1));
    t1.take_planning(&s);
}

// #[test]
// fn test_transition_take_planning_fail() {
//     let s = State::from_vec(&john_doe());
//     let weight = fv_estimated!("weight", vec!(80.0, 82.5, 85.0));
//     let a1 = a!(weight.clone(), 87.0.wrap());
//     let t1 = t_plan!("gains_weight", eq!(weight.wrap(), 82.5.wrap()), vec!(a1));
//     let next = t1.take_planning(&s);
//     assert_eq!(next, s);
// }

#[test]
fn test_transition_action_ordering() {
    let s = State::from_vec(&john_doe());
    let weight = fv_estimated!("weight", vec!(80.0, 82.5, 85.0, 87.5));
    let a1 = a!(weight.clone(), 82.5.wrap());
    let a2 = a!(weight.clone(), 85.0.wrap());
    let t1 = t_plan!("gains_weight", eq!(weight.wrap(), 80.0.wrap()), vec!(a1, a2));
    let s_next_1 = t1.take_planning(&s);
    assert_eq!(s_next_1.get_value("weight"), 85.0.to_spvalue());
}

#[test]
#[should_panic]
fn test_transition_action_ordering_panic() {
    let s = State::from_vec(&john_doe());
    let weight = fv_estimated!("weight", vec!(80.0, 82.5, 85.0, 87.5));
    let a1 = a!(weight.clone(), 82.5.wrap());
    let a2 = a!(weight.clone(), 85.0.wrap());
    let a3 = a!(weight.clone(), 87.5.wrap());
    let t1 = t_plan!("gains_weight", eq!(weight.wrap(), 80.0.wrap()), vec!(a1, a2, a3));
    let s_next_1 = t1.take_planning(&s);
    assert_eq!(s_next_1.get_value("weight"), 87.5.to_spvalue());
}

#[test]
fn test_transition_action_ordering_fail() {
    let s = State::from_vec(&john_doe());
    let weight = fv_estimated!("weight", vec!(80.0, 82.5, 85.0, 87.5));
    let a1 = a!(weight.clone(), 82.5.wrap());
    let a2 = a!(weight.clone(), 85.0.wrap());
    let t1 = t_plan!("gains_weight", eq!(weight.wrap(), 80.0.wrap()), vec!(a2, a1));
    let s_next_1 = t1.take_planning(&s);
    assert_ne!(s_next_1.get_value("weight"), 85.0.to_spvalue());
}

#[test]
fn test_transition_equality() {
    let weight = fv_estimated!("weight", vec!(80.0, 82.5, 85.0, 87.5));
    let a1 = a!(weight.clone(), 82.5.wrap());
    let a2 = a!(weight.clone(), 85.0.wrap());
    let a3 = a!(weight.clone(), 87.5.wrap());

    // Transitions should be equal even if they have a different name
    let t1 = t_plan!("gains_weight_again", eq!(&weight.wrap(), 80.0.wrap()), vec!(a1.clone(), a2.clone(), a3.clone()));
    let t2 = t_plan!("gains_weight_again", eq!(&weight.wrap(), 80.0.wrap()), vec!(a1.clone(), a2.clone(), a3.clone()));
    let t3 = t_plan!("loses_weight_again", eq!(&weight.wrap(), 80.0.wrap()), vec!(a1.clone(), a2.clone(), a3.clone()));
    let t4 = t_plan!("loses_weight_again", eq!(&weight.wrap(), 80.0.wrap()), vec!(a3.clone(), a2.clone()));
    let t5 = t_plan!("loses_weight_again", eq!(&weight.wrap(), 85.0.wrap()), vec!(a3.clone(), a2.clone()));
    assert_eq!(t1, t2);
    assert_eq!(t1, t3);
    assert_ne!(t3, t4);
    assert_ne!(t4, t5);
}

#[test]
fn test_transition_contained_in_vec() {
    let weight = fv_estimated!("weight", vec!(80.0, 82.5, 85.0, 87.5));
    let a1 = a!(weight.clone(), 82.5.wrap());
    let a2 = a!(weight.clone(), 85.0.wrap());
    let a3 = a!(weight.clone(), 87.5.wrap());

    // Transitions should be equal even if they have a different name
    let t1 = t_plan!("gains_weight_again", eq!(&weight.wrap(), 80.0.wrap()), vec!(a1.clone(), a2.clone(), a3.clone()));
    let t2 = t_plan!("gains_weight_again", eq!(&weight.wrap(), 80.0.wrap()), vec!(a1.clone(), a2.clone(), a3.clone()));
    let t3 = t_plan!("loses_weight_again", eq!(&weight.wrap(), 80.0.wrap()), vec!(a1.clone(), a2.clone(), a3.clone()));
    let t4 = t_plan!("loses_weight_again", eq!(&weight.wrap(), 80.0.wrap()), vec!(a3.clone(), a2.clone()));
    let t5 = t_plan!("loses_weight_again", eq!(&weight.wrap(), 85.0.wrap()), vec!(a3.clone(), a2.clone()));
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
    let weight = fv_estimated!("weight", vec!(80.0, 82.5, 85.0, 87.5));
    let a1 = a!(weight.clone(), 82.5.wrap());
    let a2 = a!(weight.clone(), 85.0.wrap());
    let a3 = a!(weight.clone(), 87.5.wrap());

    // Transitions should be equal even if they have a different name
    let t1 = t_plan!("gains_weight_again", eq!(&weight.wrap(), 80.0.wrap()), vec!(a1.clone(), a2.clone(), a3.clone()));
    let t2 = t_plan!("gains_weight_again", eq!(&weight.wrap(), 80.0.wrap()), vec!(a1.clone(), a2.clone(), a3.clone()));
    let t3 = t_plan!("loses_weight_again", eq!(&weight.wrap(), 80.0.wrap()), vec!(a1.clone(), a2.clone(), a3.clone()));
    let t4 = t_plan!("loses_weight_again", eq!(&weight.wrap(), 80.0.wrap()), vec!(a3.clone(), a2.clone()));
    let trans1 = vec!(t1.clone(), t3.clone());
    let trans2 = vec!(t2.clone(), t3.clone());
    let trans3 = vec!(t2.clone(), t4.clone());
    assert_eq!(trans1, trans2);
    assert_ne!(trans2, trans3);
}

// #[test]
// fn test_transition_get_vars_all() {
//     let s = State::from_vec(&john_doe());
//     let name = v_estimated!("name", vec!("John", "Jack"));
//     let surname = v_estimated!("surname", vec!("Doe", "Crawford"));
//     let height = iv_estimated!("height", vec!(180, 185, 190));
//     let weight = fv_estimated!("weight", vec!(80.0, 82.5, 85.0));
//     let smart = bv_estimated!("smart");
//     let alive = bv_runner!("alive");

//     let guard = pred_parser::pred("var:smart == TRUE -> (var:alive == FALSE || TRUE)", &s);

//     // Transitions should be equal even if they have a different name
//     let t1 = t_plan!("gains_weight_again", eq!(&weight.wrap(), 80.0.wrap()), vec!(a1.clone(), a2.clone(), a3.clone()));
//     let t2 = t_plan!("gains_weight_again", eq!(&weight.wrap(), 80.0.wrap()), vec!(a1.clone(), a2.clone(), a3.clone()));
//     let t3 = t_plan!("loses_weight_again", eq!(&weight.wrap(), 80.0.wrap()), vec!(a1.clone(), a2.clone(), a3.clone()));
//     let t4 = t_plan!("loses_weight_again", eq!(&weight.wrap(), 80.0.wrap()), vec!(a3.clone(), a2.clone()));
//     let trans1 = vec!(t1.clone(), t3.clone());
//     let trans2 = vec!(t2.clone(), t3.clone());
//     let trans3 = vec!(t2.clone(), t4.clone());
//     assert_eq!(trans1, trans2);
//     assert_ne!(trans2, trans3);
// }