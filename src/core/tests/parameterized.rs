use super::*;

#[test]
fn test_new_parameter() {
    assert_eq!(
        Parameter::new("some_name", &false),
        Parameter {
            name: String::from("some_name"),
            value: false
        }
    )
}

#[test]
fn test_none_parameter() {
    assert_eq!(
        Parameter::none(),
        Parameter {
            name: String::from("NONE"),
            value: true
        }
    )
}

#[test]
fn test_new_param_predicate() {
    let pp = ppred!(
        &pass!(&new_enum_assign_m!("var1_m", vec!("a", "b", "c"), "t1", "p1")
    )

    let pp = ParamPredicate::new(&vec![
        Predicate::ASS(Assignment::new(
            &Variable::new(
                "var1_m",
                &SPValueType::String,
                &vec!["a".to_spvalue(), "b", "c"],
                "t1",
                Some(&Parameter::new("p1", &true)),
                &Kind::Measured,
            ),
            "a",
            None,
        )),
        Predicate::SET(EnumValue::new(
            &EnumVariable::new(
                "var1_c",
                &vec!["a", "b", "c"],
                "t1",
                Some(&Parameter::new("p1", &true)),
                &Kind::Command,
            ),
            "b",
            None,
        )),
    ]);
    println!("{:?}", pp);
}

#[test]
fn test_generate_predicate() {
    let p1 = Parameter::new("p1", &true);
    let p2 = Parameter::new("p2", &false);
    let d = vec!["a", "b", "c"];

    let var1_m = EnumVariable::new("var1_m", &d, "t1", Some(&p1), &Kind::Measured);
    let var1_c = EnumVariable::new("var1_c", &d, "t1", Some(&p1), &Kind::Command);
    let var2_m = EnumVariable::new("var2_m", &d, "t2", Some(&p2), &Kind::Measured);
    let var2_c = EnumVariable::new("var2_c", &d, "t2", Some(&p2), &Kind::Command);

    let pp = ParamPredicate::new(&vec![
        pass!(&new_enum_assign_m!("var1_m", &d, )
        Predicate::SET(EnumValue::new(&var1_m, "a", None)),
        Predicate::SET(EnumValue::new(&var1_c, "b", None)),
        Predicate::SET(EnumValue::new(&var2_m, "c", None)),
        Predicate::SET(EnumValue::new(&var2_c, "a", None)),
    ]);

    let params = vec![p1, p2];
    println!("generated {:?}", generate_predicate(&pp, &params));
}

#[test]
fn test_parameterized() {
    let (problem, params) = models::dummy_robot::dummy_robot::parameterized_model();

    let d = deactivate_all(&params);
    println!("prms: {:?}", d);
    let result1 = parameterized(&problem, &d);
    pprint_result(&result1.result);
    

    let p1 = &activate_next(&d);
    println!("prms: {:?}", p1);
    let result2 = parameterized(&problem, &p1);
    pprint_result(&result2.result);
    

    let p2 = &activate_next(&p1);
    println!("prms: {:?}", p2);
    let result3 = parameterized(&problem, &p2);
    pprint_result(&result3.result);
}
