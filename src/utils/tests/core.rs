use super::*;

#[test]
fn test_get_predicate_vars() {
    let vars = get_predicate_vars(&Predicate::OR(vec![
        Predicate::SET(EnumValue::new(
            &EnumVariable::new(
                "x",
                &vec!["a", "b", "c", "d"],
                "type",
                None,
                &Kind::Estimated,
            ),
            "a",
            None,
        )),
        Predicate::SET(EnumValue::new(
            &EnumVariable::new(
                "y",
                &vec!["a", "b", "c", "d"],
                "type",
                None,
                &Kind::Estimated,
            ),
            "b",
            None,
        )),
        Predicate::SET(EnumValue::new(
            &EnumVariable::new(
                "z",
                &vec!["a", "b", "c", "d"],
                "type",
                None,
                &Kind::Estimated,
            ),
            "c",
            None,
        )),
    ]));
    assert_eq!(3, vars.len());
}

#[test]
fn test_get_param_predicate_vars() {
    let vars = get_param_predicate_vars(&ParamPredicate::new(&vec![
        Predicate::SET(EnumValue::new(
            &EnumVariable::new(
                "x",
                &vec!["a", "b", "c", "d"],
                "type",
                None,
                &Kind::Estimated,
            ),
            "a",
            None,
        )),
        Predicate::SET(EnumValue::new(
            &EnumVariable::new(
                "y",
                &vec!["a", "b", "c", "d"],
                "type",
                None,
                &Kind::Estimated,
            ),
            "b",
            None,
        )),
        Predicate::SET(EnumValue::new(
            &EnumVariable::new(
                "z",
                &vec!["a", "b", "c", "d"],
                "type",
                None,
                &Kind::Estimated,
            ),
            "c",
            None,
        )),
    ]));
    assert_eq!(3, vars.len());
}
