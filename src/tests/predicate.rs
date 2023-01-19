#![allow(unused_imports)]
#![allow(dead_code)]
use crate::{
    and, assign, av_run, bv, bv_run, eq, fv, fv_run, iv, iv_run, neq, not, or, pred_parser, v,
    v_run, Predicate, ToSPWrapped, ToSPWrappedVar, get_predicate_vars_all, get_predicate_vars_planner, get_predicate_vars_runner,
};
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
fn test_predicate_eq() {
    let state = State::from_vec(&john_doe());
    let eq1 = Predicate::EQ(v!("name", vec!("John", "Jack")).wrap(), "John".wrap());
    let eq2 = Predicate::EQ(v!("name", vec!("John", "Jack")).wrap(), "Jack".wrap());
    assert!(eq1.eval(&state));
    assert_ne!(true, eq2.eval(&state));
}

#[test]
fn test_predicate_neq() {
    let state = State::from_vec(&john_doe());
    let neq1 = Predicate::NEQ(v!("name", vec!("John", "Jack")).wrap(), "John".wrap());
    let neq2 = Predicate::NEQ(v!("name", vec!("John", "Jack")).wrap(), "Jack".wrap());
    assert_ne!(true, neq1.eval(&state));
    assert!(neq2.eval(&state));
}

#[test]
#[should_panic]
fn test_predicate_eq_panic_not_in_state() {
    let state = State::from_vec(&john_doe());
    let eq1 = Predicate::EQ(v!("v1", vec!("John", "Jack")).wrap(), "John".wrap());
    eq1.eval(&state);
}

#[test]
#[should_panic]
fn test_predicate_eq_wrong_var() {
    let state = State::from_vec(&john_doe());
    let eq1 = Predicate::EQ(
        v!("name", vec!("John", "Jack")).wrap(),
        v!("surname", vec!("Doe", "Crawford")).wrap(),
    );
    assert!(eq1.eval(&state));
}

#[test]
fn test_predicate_not() {
    let s1 = State::from_vec(&john_doe());
    let not = Predicate::NOT(Box::new(Predicate::EQ(bv!("smart").wrap(), false.wrap())));
    let notf = Predicate::NOT(Box::new(Predicate::EQ(bv!("smart").wrap(), true.wrap())));
    assert!(not.eval(&s1));
    assert!(!notf.eval(&s1));
}

#[test]
fn test_predicate_and() {
    let john_doe = john_doe();
    let s1 = State::from_vec(&john_doe);
    let eq = Predicate::EQ(bv!("smart").wrap(), true.wrap());
    let eq2 = Predicate::EQ(fv!("weight", vec!(80.0, 82.5, 85.0)).wrap(), 80.0.wrap());
    let eqf = Predicate::EQ(iv!("height", vec!(180, 185, 190)).wrap(), 175.wrap());
    let and = Predicate::AND(vec![eq.clone(), eq2.clone()]);
    let andf = Predicate::AND(vec![eq, eq2, eqf]);
    assert!(and.eval(&s1));
    assert!(!andf.eval(&s1));
}

#[test]
fn test_predicate_or() {
    let john_doe = john_doe();
    let s1 = State::from_vec(&john_doe);
    let eq = Predicate::EQ(bv!("smart").wrap(), true.wrap());
    let eq2 = Predicate::EQ(fv!("weight", vec!(80.0, 82.5, 85.0)).wrap(), 80.0.wrap());
    let eqf = Predicate::EQ(iv!("height", vec!(180, 185, 190)).wrap(), 175.wrap());
    let or = Predicate::OR(vec![eq.clone(), eq2.clone()]);
    let or2 = Predicate::OR(vec![eq, eq2, eqf]);
    assert!(or.eval(&s1));
    assert!(or2.eval(&s1));
}

#[test]
fn test_predicate_complex() {
    let john_doe = john_doe();
    let s1 = State::from_vec(&john_doe);
    let eq = Predicate::EQ(bv!("smart").wrap(), true.wrap());
    let eq2 = Predicate::EQ(fv!("weight", vec!(80.0, 82.5, 85.0)).wrap(), 80.0.wrap());
    let eqf = Predicate::EQ(iv!("height", vec!(180, 185, 190)).wrap(), 175.wrap());
    let and = Predicate::AND(vec![eq.clone(), eq2.clone()]);
    let andf = Predicate::AND(vec![eq.clone(), eq2.clone(), eqf.clone()]);
    let or = Predicate::OR(vec![eq.clone(), eq2.clone()]);
    let or2 = Predicate::OR(vec![eq, eq2, eqf]);
    let not = Predicate::NOT(Box::new(or.clone()));
    let cmplx = Predicate::AND(vec![
        Predicate::NOT(Box::new(not.clone())),
        or,
        or2,
        and,
        Predicate::NOT(Box::new(andf)),
    ]);
    assert!(cmplx.eval(&s1));
}

#[test]
fn test_predicate_eq_macro() {
    let state = State::from_vec(&john_doe());
    let eq1 = eq!(v!("name", vec!("John", "Jack")).wrap(), "John".wrap());
    let eq2 = eq!(v!("name", vec!("John", "Jack")).wrap(), "Jack".wrap());
    assert!(eq1.eval(&state));
    assert_ne!(true, eq2.eval(&state));
}

#[test]
fn test_predicate_not_macro() {
    let s1 = State::from_vec(&john_doe());
    let not = not!(eq!(bv!("smart").wrap(), false.wrap()));
    let notf = not!(eq!(bv!("smart").wrap(), true.wrap()));
    assert!(not.eval(&s1));
    assert!(!notf.eval(&s1));
}

#[test]
fn test_predicate_neq_macro() {
    let state = State::from_vec(&john_doe());
    let neq1 = neq!(v!("name", vec!("John", "Jack")).wrap(), "John".wrap());
    let neq2 = neq!(v!("name", vec!("John", "Jack")).wrap(), "Jack".wrap());
    assert_ne!(true, neq1.eval(&state));
    assert!(neq2.eval(&state));
}

#[test]
fn test_predicate_and_macro() {
    let john_doe = john_doe();
    let s1 = State::from_vec(&john_doe);
    let eq = eq!(bv!("smart").wrap(), true.wrap());
    let eq2 = eq!(fv!("weight", vec!(80.0, 82.5, 85.0)).wrap(), 80.0.wrap());
    let eqf = eq!(iv!("height", vec!(180, 185, 190)).wrap(), 175.wrap());
    let and = and!(vec![eq.clone(), eq2.clone()]);
    let andf = and!(vec![eq, eq2, eqf]);
    assert!(and.eval(&s1));
    assert!(!andf.eval(&s1));
}

#[test]
fn test_predicate_or_macro() {
    let john_doe = john_doe();
    let s1 = State::from_vec(&john_doe);
    let eq = eq!(bv!("smart").wrap(), true.wrap());
    let eq2 = eq!(fv!("weight", vec!(80.0, 82.5, 85.0)).wrap(), 80.0.wrap());
    let eqf = eq!(iv!("height", vec!(180, 185, 190)).wrap(), 175.wrap());
    let or = or!(vec![eq.clone(), eq2.clone()]);
    let or2 = or!(vec![eq, eq2, eqf]);
    assert!(or.eval(&s1));
    assert!(or2.eval(&s1));
}

fn make_robot_initial_state() -> State {
    let state = State::new();
    let state = state.add(SPAssignment::new(
        v_run!("runner_goal"),
        "var:ur_current_pose == c".to_spvalue(),
    ));
    let state = state.add(SPAssignment::new(
        av_run!("runner_plan"),
        Vec::<String>::new().to_spvalue(),
    ));
    let state = state.add(SPAssignment::new(
        bv_run!("runner_replan"),
        true.to_spvalue(),
    ));
    let state = state.add(SPAssignment::new(
        bv_run!("runner_replanned"),
        false.to_spvalue(),
    ));
    let state = state.add(SPAssignment::new(
        bv_run!("ur_action_trigger"),
        false.to_spvalue(),
    ));
    let state = state.add(SPAssignment::new(
        v_run!("ur_action_state"),
        "initial".to_spvalue(),
    ));
    let state = state.add(SPAssignment::new(
        v!("ur_current_pose", vec!("a", "b", "c")),
        "a".to_spvalue(),
    ));
    let state = state.add(SPAssignment::new(
        v!("ur_command", vec!("movej", "movel")),
        "movej".to_spvalue(),
    ));
    let state = state.add(SPAssignment::new(
        fv!("ur_velocity", vec!(0.1, 0.2, 0.3)),
        0.2.to_spvalue(),
    ));
    let state = state.add(SPAssignment::new(
        fv!("ur_acceleration", vec!(0.2, 0.4, 0.6)),
        0.4.to_spvalue(),
    ));
    let state = state.add(SPAssignment::new(
        v!("ur_goal_feature_id", vec!("a", "b", "c")),
        "a".to_spvalue(),
    ));
    let state = state.add(SPAssignment::new(
        v!("ur_tcp_id", vec!("svt_tcp")),
        "svt_tcp".to_spvalue(),
    ));
    state
}

#[test]
fn test_predicate_get_all_variables() {
    let state = make_robot_initial_state();
    let pred = pred_parser::pred(
        "var:ur_action_trigger == false && var:ur_action_state == initial && var:ur_current_pose != a",
        &state,
    ).unwrap();
    let vars = get_predicate_vars_all(&pred);
    let vars_init = vec!(
        v_run!("ur_action_state"),
        bv_run!("ur_action_trigger"),
        v!("ur_current_pose", vec!("a", "b", "c"))
    );
    assert_eq!(vars, vars_init)
}

#[test]
fn test_predicate_get_planner_variables() {
    let state = make_robot_initial_state();
    let pred = pred_parser::pred(
        "var:ur_action_trigger == false && var:ur_action_state == initial && var:ur_current_pose != a",
        &state,
    ).unwrap();
    let vars = get_predicate_vars_planner(&pred);
    let vars_init = vec!(
        v!("ur_current_pose", vec!("a", "b", "c"))
    );
    assert_eq!(vars, vars_init)
}

#[test]
fn test_predicate_get_runner_variables() {
    let state = make_robot_initial_state();
    let pred = pred_parser::pred(
        "var:ur_action_trigger == false && var:ur_action_state == initial && var:ur_current_pose != a",
        &state,
    ).unwrap();
    let vars = get_predicate_vars_runner(&pred);
    let vars_init = vec!(
        v_run!("ur_action_state"),
        bv_run!("ur_action_trigger")
    );
    assert_eq!(vars, vars_init)
}