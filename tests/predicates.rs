use lib::*;
use z3_v2::*;

#[test]
fn test_true_predicate() {
    let cfg = ConfigZ3::new();
    let ctx = ContextZ3::new(&cfg);
    let t = Predicate::TRUE;
    let pred = predicate_to_ast(&ctx, &t, &3);
    assert_eq!("true", ast_to_string_z3!(&ctx, pred));
}

#[test]
fn test_false_predicate() {
    let cfg = ConfigZ3::new();
    let ctx = ContextZ3::new(&cfg);
    let t = Predicate::FALSE;
    let pred = predicate_to_ast(&ctx, &t, &3);
    assert_eq!("false", ast_to_string_z3!(&ctx, pred));
}

#[test]
fn test_not_predicate() {
    let x = EnumVariable::new("x", &vec!["a", "b", "c", "d"], None, &Kind::Estimated);
    let cfg = ConfigZ3::new();
    let ctx = ContextZ3::new(&cfg);
    let n = Predicate::NOT(Box::new(Predicate::EQ(EnumValue::new(&x, "b", None))));
    let pred = predicate_to_ast(&ctx, &n, &3);
    assert_eq!("(not (= x_s3 b))", ast_to_string_z3!(&ctx, pred));
}

#[test]
fn test_and_predicate() {
    let x = EnumVariable::new("x", &vec!["a", "b", "c", "d"], None, &Kind::Estimated);
    let y = EnumVariable::new("y", &vec!["a", "b", "c", "d"], None, &Kind::Estimated);
    let z = EnumVariable::new("z", &vec!["a", "b", "c", "d"], None, &Kind::Estimated);

    let cfg = ConfigZ3::new();
    let ctx = ContextZ3::new(&cfg);
    let n = Predicate::AND(vec![
        Predicate::EQ(EnumValue::new(&x, "a", None)),
        Predicate::EQ(EnumValue::new(&y, "b", None)),
        Predicate::EQ(EnumValue::new(&z, "c", None)),
    ]);
    let pred = predicate_to_ast(&ctx, &n, &3);
    assert_eq!(
        "(and (= x_s3 a) (= y_s3 b) (= z_s3 c))",
        ast_to_string_z3!(&ctx, pred)
    );
}


#[test]
fn test_or_predicate() {
    let x = EnumVariable::new("x", &vec!["a", "b", "c", "d"], None, &Kind::Estimated);
    let y = EnumVariable::new("y", &vec!["a", "b", "c", "d"], None, &Kind::Estimated);
    let z = EnumVariable::new("z", &vec!["a", "b", "c", "d"], None, &Kind::Estimated);

    let cfg = ConfigZ3::new();
    let ctx = ContextZ3::new(&cfg);
    let n = Predicate::OR(vec![
        Predicate::EQ(EnumValue::new(&x, "a", None)),
        Predicate::EQ(EnumValue::new(&y, "b", None)),
        Predicate::EQ(EnumValue::new(&z, "c", None)),
    ]);
    let pred = predicate_to_ast(&ctx, &n, &3);
    assert_eq!(
        "(or (= x_s3 a) (= y_s3 b) (= z_s3 c))",
        ast_to_string_z3!(&ctx, pred)
    );
}