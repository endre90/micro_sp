#![allow(unused_imports)]
#![allow(dead_code)]
use crate::{
    a, assign, bv, bv_run, fv, fv_run, iv, iv_run, t, t_plus, v, v_run, Predicate::*, Transition, eq, ToSPWrappedVar, pred_parser, SPWrapped,
};
use crate::{
    Action, SPAssignment, SPValue, SPValueType, SPVariable, SPVariableType, State, ToSPValue,
    ToSPWrapped,
};

fn john_doe() -> Vec<(SPVariable, SPValue)> {
    let name = v!("name", vec!("John", "Jack"));
    let surname = v!("surname", vec!("Doe", "Crawford"));
    let height = iv!("height", vec!(180, 185, 190));
    let weight = fv!("weight", vec!(80.0, 82.5, 85.0));
    let smart = bv!("smart");
    let alive = bv!("alive");

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
fn parse_values() {
    let s = State::new();
    assert_eq!(
        pred_parser::value("9", &s),
        Ok(SPWrapped::SPValue(9.to_spvalue()))
    );
    assert_eq!(
        pred_parser::value("19.123", &s),
        Ok(SPWrapped::SPValue(19.123.to_spvalue()))
    );
    assert_eq!(
        pred_parser::value("hej", &s),
        Ok(SPWrapped::SPValue("hej".to_spvalue()))
    );
    assert_eq!(
        pred_parser::value("true", &s),
        Ok(SPWrapped::SPValue(true.to_spvalue()))
    );
    assert_eq!(
        pred_parser::value("TRUE", &s),
        Ok(SPWrapped::SPValue(true.to_spvalue()))
    );
    assert_eq!(
        pred_parser::value("false", &s),
        Ok(SPWrapped::SPValue(false.to_spvalue()))
    );
    assert_eq!(
        pred_parser::value("FALSE", &s),
        Ok(SPWrapped::SPValue(false.to_spvalue()))
    );
}

#[test]
fn parse_variables() {
    let s = State::from_vec(&john_doe());
    assert_eq!(
        pred_parser::variable("var: height", &s),
        Ok(s.get_all("height").var)
    );
}

#[test]
#[should_panic]
fn parse_variables_panic() {
    let s = State::from_vec(&john_doe());
    let _ = pred_parser::variable("var: wealth", &s);
}

#[test]
fn parse_predicates() {
    let s = State::from_vec(&john_doe());
    let and = "TRUE && TRUE";
    let and2 = AND(vec![TRUE, TRUE]);
    assert_eq!(pred_parser::pred(and, &s), Ok(and2));

    let and = "TRUE  && TRUE && FALSE ";
    let and2 = AND(vec![TRUE, TRUE, FALSE]);
    assert_eq!(pred_parser::pred(and, &s), Ok(and2));

    let or = "TRUE || TRUE || FALSE";
    let or2 = OR(vec![TRUE, TRUE, FALSE]);
    assert_eq!(pred_parser::pred(or, &s), Ok(or2));

    let not_or = "TRUE || ! ( TRUE || FALSE && TRUE)";
    let not_or2 = OR(vec![
        TRUE,
        NOT(Box::new(OR(vec![TRUE, AND(vec![FALSE, TRUE])]))),
    ]);
    assert_eq!(pred_parser::pred(not_or, &s), Ok(not_or2));

    let eq1 = "TRUE == TRUE";
    let eq2 = EQ(
        SPWrapped::SPValue(true.to_spvalue()),
        SPWrapped::SPValue(true.to_spvalue()),
    );
    assert_eq!(pred_parser::eq(eq1, &s), Ok(eq2));

    let eq1 = "var:smart == FALSE";
    let eq2 = EQ(bv!("smart").wrap(), false.wrap());
    assert_eq!(pred_parser::eq(eq1, &s), Ok(eq2));

    let eq1 = "var:smart == true";
    let eq2 = EQ(bv!("smart").wrap(), true.wrap());
    assert_eq!(pred_parser::eq(eq1, &s), Ok(eq2));
    
    let neq1 = "var:smart != true";
    let neq2 = NEQ(bv!("smart").wrap(), true.wrap());
    assert_eq!(pred_parser::eq(neq1, &s), Ok(neq2));

    let eq1 = "TRUE == TRUE || FALSE != FALSE";
    let eq2 = EQ(
        SPWrapped::SPValue(true.to_spvalue()),
        SPWrapped::SPValue(true.to_spvalue()),
    );
    let eq3 = NEQ(
        SPWrapped::SPValue(false.to_spvalue()),
        SPWrapped::SPValue(false.to_spvalue()),
    );
    let or = OR(vec![eq2, eq3]);
    assert_eq!(pred_parser::pred(eq1, &s), Ok(or));

    let eq1 = "TRUE == TRUE || !(FALSE != FALSE)";
    let eq2 = EQ(
        SPWrapped::SPValue(true.to_spvalue()),
        SPWrapped::SPValue(true.to_spvalue()),
    );
    let eq3 = NEQ(
        SPWrapped::SPValue(false.to_spvalue()),
        SPWrapped::SPValue(false.to_spvalue()),
    );
    let or = OR(vec![eq2, NOT(Box::new(eq3))]);
    assert_eq!(pred_parser::pred(eq1, &s), Ok(or));

    let eq1 = "var:smart == TRUE || !(FALSE != var:smart)";
    let hej = s.get_all("smart").var.wrap();
    let eq2 = EQ(hej.clone(), true.to_spvalue().wrap());
    let eq3 = NEQ(false.to_spvalue().wrap(), hej);
    let or = OR(vec![eq2, NOT(Box::new(eq3))]);
    assert_eq!(pred_parser::pred(eq1, &s), Ok(or));

    let impl1 = " var:smart == TRUE ->  var:alive == FALSE || TRUE  ";
    let hej = s.get_all("smart").var.wrap();
    let hopp = s.get_all("alive").var.wrap();
    let eq1 = EQ(hej, true.to_spvalue().wrap());
    let eq2 = EQ(hopp, false.to_spvalue().wrap());
    let impl2 = OR(vec![NOT(Box::new(eq1)), OR(vec![eq2, TRUE])]);
    assert_eq!(pred_parser::pred(impl1, &s), Ok(impl2.clone()));
    let impl1 = "var:smart == TRUE -> (var:alive == FALSE || TRUE)";
    assert_eq!(pred_parser::pred(impl1, &s), Ok(impl2));
}

#[test]
fn parse_actions() {
    let s = State::from_vec(&john_doe());
    let weight = fv!("weight", vec!(80.0, 82.5, 85.0));
    let weight_2 = fv!("weight_2", vec!(85.0, 87.5));
    let s_new = s.add(SPAssignment::new(weight_2, 85.0.to_spvalue()));
    let a1 = a!(weight.clone(), 82.5.wrap());
    let a2 = a!(weight.clone(), 85.0.wrap());
    assert_eq!(pred_parser::action("var:weight <- 82.5", &s), Ok(a1));
    assert_eq!(pred_parser::action("var:weight <- var:weight_2", &s_new), Ok(a2));
}