#![allow(unused_imports)]
#![allow(dead_code)]
use micro_sp::{
    and, eq, not, or, Predicate, SPValue, SPValueType, SPVariable, State, ToSPValue, SPCommon, ToSPCommon, ToSPCommonVar
};
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
fn test_predicate_eq() {
    let john_doe = john_doe();
    let s1 = State::new(&john_doe);
    let s2 = State::new(&john_doe);
    let eq = Predicate::EQ(SPVariable::to_common_from_name("name", &s1), SPVariable::to_common_from_name("name", &s2));
    let eq2 = Predicate::EQ(SPVariable::to_common_from_name("height", &s1), 175.to_val());
    assert!(eq.eval(&s1));
    assert_ne!(true, eq2.eval(&s1));
}

#[test]
#[should_panic]
fn test_predicate_eq_panic() {
    let s1 = State::new(&john_doe());
    let eq = Predicate::EQ(SPVariable::to_common_from_name("v1", &s1), SPVariable::to_common_from_name("v2", &s1));
    eq.eval(&s1);
}

#[test]
fn test_predicate_not() {
    let s1 = State::new(&john_doe());
    let not = Predicate::NOT(Box::new(Predicate::EQ(SPVariable::to_common_from_name("smart", &s1), false.to_val())));
    let notf = Predicate::NOT(Box::new(Predicate::EQ(SPVariable::to_common_from_name("smart", &s1), true.to_val())));
    assert!(not.eval(&s1));
    assert!(!notf.eval(&s1));
}

#[test]
fn test_predicate_and() {
    let john_doe = john_doe();
    let s1 = State::new(&john_doe);
    let s2 = State::new(&john_doe);
    let eq = Predicate::EQ(SPVariable::to_common_from_name("smart", &s1), true.to_val());
    let eq2 = Predicate::EQ(SPVariable::to_common_from_name("name", &s1), SPVariable::to_common_from_name("name", &s2));
    let eq3 = Predicate::EQ(SPVariable::to_common_from_name("weight", &s1), 80.to_val());
    let eqf = Predicate::EQ(SPVariable::to_common_from_name("height", &s1), 175.to_val());
    let and = Predicate::AND(vec![eq.clone(), eq2.clone(), eq3.clone()]);
    let andf = Predicate::AND(vec![eq, eq2, eq3, eqf]);
    assert!(and.eval(&s1));
    assert!(!andf.eval(&s1));
}

#[test]
fn test_predicate_or() {
    let john_doe = john_doe();
    let s1 = State::new(&john_doe);
    let s2 = State::new(&john_doe);
    let eq = Predicate::EQ(SPVariable::to_common_from_name("smart", &s1), true.to_val());
    let eq2 = Predicate::EQ(SPVariable::to_common_from_name("name", &s1), SPVariable::to_common_from_name("name", &s2));
    let eq3 = Predicate::EQ(SPVariable::to_common_from_name("weight", &s1), 80.to_val());
    let eqf = Predicate::EQ(SPVariable::to_common_from_name("height", &s1), 175.to_val());
    let or = Predicate::OR(vec![eq.clone(), eq2.clone(), eq3.clone()]);
    let or2 = Predicate::OR(vec![eq, eq2, eq3, eqf]);
    assert!(or.eval(&s1));
    assert!(or2.eval(&s1));
}

#[test]
fn test_predicate_complex() {
    let john_doe = john_doe();
    let s1 = State::new(&john_doe);
    let s2 = State::new(&john_doe);
    let eq = Predicate::EQ(SPVariable::to_common_from_name("smart", &s1), true.to_val());
    let eq2 = Predicate::EQ(SPVariable::to_common_from_name("name", &s1), SPVariable::to_common_from_name("name", &s2));
    let eq3 = Predicate::EQ(SPVariable::to_common_from_name("weight", &s1), 80.to_val());
    let eqf = Predicate::EQ(SPVariable::to_common_from_name("height", &s1), 175.to_val());
    let and = Predicate::AND(vec![eq.clone(), eq2.clone(), eq3.clone()]);
    let andf = Predicate::AND(vec![eq.clone(), eq2.clone(), eq3.clone(), eqf.clone()]);
    let or = Predicate::OR(vec![eq.clone(), eq2.clone(), eq3.clone()]);
    let or2 = Predicate::OR(vec![eq, eq2, eq3, eqf]);
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
    let john_doe = john_doe();
    let s1 = State::new(&john_doe);
    let s2 = State::new(&john_doe);
    let eq = eq!("name".to_var(&s1), "name".to_var(&s2));
    let eq2 = eq!("height".to_var(&s1), 175.to_val());
    assert!(eq.eval(&s1));
    assert_ne!(true, eq2.eval(&s1));
}

#[test]
fn test_predicate_not_macro() {
    let john_doe = john_doe();
    let s1 = State::new(&john_doe);
    let not = not!(eq!("smart".to_var(&s1), false.to_val()));
    let notf = not!(eq!("smart".to_var(&s1), true.to_val()));
    assert!(not.eval(&s1));
    assert!(!notf.eval(&s1));
}

#[test]
fn test_predicate_and_macro() {
    let john_doe = john_doe();
    let s1 = State::new(&john_doe);
    let s2 = State::new(&john_doe);
    let eq = eq!("smart".to_var(&s1), true.to_val());
    let eq2 = eq!("name".to_var(&s1), "name".to_var(&s2));
    let eq3 = eq!("weight".to_var(&s1), 80.to_val());
    let eqf = eq!("height".to_var(&s1), 175.to_val());
    let and = and!(eq, eq2, eq3);
    let andf = and!(eq, eq2, eq3, eqf);
    assert!(and.eval(&s1));
    assert!(!andf.eval(&s1));
}

#[test]
fn test_predicate_or_macro() {
    let john_doe = john_doe();
    let s1 = State::new(&john_doe);
    let s2 = State::new(&john_doe);
    let eq = eq!("smart".to_var(&s1), true.to_val());
    let eq2 = eq!("name".to_var(&s1), "name".to_var(&s2));
    let eq3 = eq!("weight".to_var(&s1), 80.to_val());
    let eqf = eq!("height".to_var(&s1), 175.to_val());
    let or = or!(eq, eq2, eq3);
    let orf = or!(eq, eq2, eq3, eqf);
    assert!(or.eval(&s1));
    assert!(orf.eval(&s1));
}

#[test]
fn test_predicate_complex_macro() {
    let john_doe = john_doe();
    let s1 = State::new(&john_doe);
    let s2 = State::new(&john_doe);
    let eq = eq!("smart".to_var(&s1), true.to_val());
    let eq2 = eq!("name".to_var(&s1), "name".to_var(&s2));
    let eq3 = eq!("weight".to_var(&s1), 80.to_val());
    let eqf = eq!("height".to_var(&s1), 175.to_val());
    let and = and!(eq, eq2, eq3);
    let andf = and!(eq, eq2, eq3, eqf);
    let or = or!(eq, eq2, eq3);
    let or2 = or!(eq, eq2, eq3, eqf);
    let not = not!(or);
    let cmplx = and!(not!(not), or, or2, and, not!(andf));
    assert!(cmplx.eval(&s1));
}
