use super::*;

#[test]
fn new_parameter() {
    assert_eq!(
        Parameter::new("some_name", &false),
        Parameter {
            name: String::from("some_name"),
            value: false
        }
    )
}

#[test]
fn none_parameter() {
    assert_eq!(
        Parameter::none(),
        Parameter {
            name: String::from("NONE"),
            value: true
        }
    )
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
        Predicate::EQ(EnumValue::new(&var1_m, "a", None)),
        Predicate::EQ(EnumValue::new(&var1_c, "b", None)),
        Predicate::EQ(EnumValue::new(&var2_m, "c", None)),
        Predicate::EQ(EnumValue::new(&var2_c, "a", None)),
    ]);

    let params = vec![p1, p2];
    println!("generated {:?}", generate_predicate(&pp, &params));
}
