#![allow(unused_imports)]
#![allow(dead_code)]
use micro_sp::{SPValue, State, ToSPValue, VarOrVal, Predicate, ToVal, ToVar, eq, not, and, or};
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
fn test_predicate_eq() {
    let s1 = State::new(john_doe());
    let eq = Predicate::EQ("name".to_var(), "name".to_var());
    let eq2 = Predicate::EQ("height".to_var(), 175.to_val());
    assert!(eq.eval(&s1));
    assert_ne!(true, eq2.eval(&s1));
}

#[test]
#[should_panic]
fn test_predicate_eq_panic() {
    let s1 = State::new(john_doe());
    let eq = Predicate::EQ("v10".to_var(), "v11".to_var());
    eq.eval(&s1);
}

#[test]
fn test_predicate_not() {
    let s1 = State::new(john_doe());
    let not = Predicate::NOT(Box::new(Predicate::EQ("smart".to_var(), false.to_val())));
    let notf = Predicate::NOT(Box::new(Predicate::EQ("smart".to_var(), true.to_val())));
    assert!(not.eval(&s1));
    assert!(!notf.eval(&s1));
}

#[test]
fn test_predicate_and() {
    let s1 = State::new(john_doe());
    let eq = Predicate::EQ("smart".to_var(), true.to_val());
    let eq2 = Predicate::EQ("name".to_var(), "name".to_var());
    let eq3 = Predicate::EQ("weight".to_var(), 80.5.to_val());
    let eqf = Predicate::EQ("height".to_var(), 175.to_val());
    let and = Predicate::AND(vec!(eq.clone(), eq2.clone(), eq3.clone()));
    let andf = Predicate::AND(vec!(eq, eq2, eq3, eqf));
    assert!(and.eval(&s1));
    assert!(!andf.eval(&s1));
}

#[test]
fn test_predicate_or() {
    let s1 = State::new(john_doe());
    let eq = Predicate::EQ("smart".to_var(), true.to_val());
    let eq2 = Predicate::EQ("name".to_var(), "name".to_var());
    let eq3 = Predicate::EQ("weight".to_var(), 80.5.to_val());
    let eqf = Predicate::EQ("height".to_var(), 175.to_val());
    let or = Predicate::OR(vec!(eq.clone(), eq2.clone(), eq3.clone()));
    let or2 = Predicate::OR(vec!(eq, eq2, eq3, eqf));
    assert!(or.eval(&s1));
    assert!(or2.eval(&s1));
}

#[test]
fn test_predicate_complex() {
    let s1 = State::new(john_doe());
    let eq = Predicate::EQ("smart".to_var(), true.to_val());
    let eq2 = Predicate::EQ("name".to_var(), "name".to_var());
    let eq3 = Predicate::EQ("weight".to_var(), 80.5.to_val());
    let eqf = Predicate::EQ("height".to_var(), 175.to_val());
    let and = Predicate::AND(vec!(eq.clone(), eq2.clone(), eq3.clone()));
    let andf = Predicate::AND(vec!(eq.clone(), eq2.clone(), eq3.clone(), eqf.clone()));
    let or = Predicate::OR(vec!(eq.clone(), eq2.clone(), eq3.clone()));
    let or2 = Predicate::OR(vec!(eq, eq2, eq3, eqf));
    let not = Predicate::NOT(Box::new(or.clone()));
    let cmplx = Predicate::AND(vec!(Predicate::NOT(Box::new(not.clone())), or, or2, and, Predicate::NOT(Box::new(andf))));
    assert!(cmplx.eval(&s1));
}

#[test]
fn test_predicate_eq_macro() {
    let s1 = State::new(john_doe());
    let eq = eq!("name".to_var(), "name".to_var());
    let eq2 = eq!("height".to_var(), 175.to_val());
    assert!(eq.eval(&s1));
    assert_ne!(true, eq2.eval(&s1));
}

#[test]
fn test_predicate_not_macro() {
    let s1 = State::new(john_doe());
    let not = not!(eq!("smart".to_var(), false.to_val()));
    let notf = not!(eq!("smart".to_var(), true.to_val()));
    assert!(not.eval(&s1));
    assert!(!notf.eval(&s1));
}

#[test]
fn test_predicate_and_macro() {
    let s1 = State::new(john_doe());
    let eq = eq!("smart".to_var(), true.to_val());
    let eq2 = eq!("name".to_var(), "name".to_var());
    let eq3 = eq!("weight".to_var(), 80.5.to_val());
    let eqf = eq!("height".to_var(), 175.to_val());
    let and = and!(eq, eq2, eq3);
    let andf = and!(eq, eq2, eq3, eqf);
    assert!(and.eval(&s1));
    assert!(!andf.eval(&s1));
}

#[test]
fn test_predicate_or_macro() {
    let s1 = State::new(john_doe());
    let eq = eq!("smart".to_var(), true.to_val());
    let eq2 = eq!("name".to_var(), "name".to_var());
    let eq3 = eq!("weight".to_var(), 80.5.to_val());
    let eqf = eq!("height".to_var(), 175.to_val());
    let or = or!(eq, eq2, eq3);
    let orf = or!(eq, eq2, eq3, eqf);
    assert!(or.eval(&s1));
    assert!(orf.eval(&s1));
}

#[test]
fn test_predicate_complex_macro() {
    let s1 = State::new(john_doe());
    let eq = eq!("smart".to_var(), true.to_val());
    let eq2 = eq!("name".to_var(), "name".to_var());
    let eq3 = eq!("weight".to_var(), 80.5.to_val());
    let eqf = eq!("height".to_var(), 175.to_val());
    let and = and!(eq, eq2, eq3);
    let andf = and!(eq, eq2, eq3, eqf);
    let or = or!(eq, eq2, eq3);
    let or2 = or!(eq, eq2, eq3, eqf);
    let not = not!(or);
    let cmplx = and!(not!(not), or, or2, and, not!(andf));
    assert!(cmplx.eval(&s1));
}