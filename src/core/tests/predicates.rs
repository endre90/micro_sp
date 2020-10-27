use super::*;
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
    let x = EnumVariable::new("x", &vec!["a", "b", "c", "d"], "type", None, &Kind::Estimated);
    let cfg = ConfigZ3::new();
    let ctx = ContextZ3::new(&cfg);
    let n = Predicate::NOT(Box::new(Predicate::SET(EnumValue::new(&x, "b", None))));
    let pred = predicate_to_ast(&ctx, &n, &3);
    assert_eq!("(not (= x_s3 b))", ast_to_string_z3!(&ctx, pred));
}

#[test]
fn test_and_predicate() {
    let x = EnumVariable::new("x", &vec!["a", "b", "c", "d"], "type", None, &Kind::Estimated);
    let y = EnumVariable::new("y", &vec!["a", "b", "c", "d"], "type", None, &Kind::Estimated);
    let z = EnumVariable::new("z", &vec!["a", "b", "c", "d"], "type", None, &Kind::Estimated);

    let cfg = ConfigZ3::new();
    let ctx = ContextZ3::new(&cfg);
    let n = Predicate::AND(vec![
        Predicate::SET(EnumValue::new(&x, "a", None)),
        Predicate::SET(EnumValue::new(&y, "b", None)),
        Predicate::SET(EnumValue::new(&z, "c", None)),
    ]);
    let pred = predicate_to_ast(&ctx, &n, &3);
    assert_eq!(
        "(and (= x_s3 a) (= y_s3 b) (= z_s3 c))",
        ast_to_string_z3!(&ctx, pred)
    );
}

#[test]
fn test_or_predicate() {
    let x = EnumVariable::new("x", &vec!["a", "b", "c", "d"], "type", None, &Kind::Estimated);
    let y = EnumVariable::new("y", &vec!["a", "b", "c", "d"], "type", None, &Kind::Estimated);
    let z = EnumVariable::new("z", &vec!["a", "b", "c", "d"], "type", None, &Kind::Estimated);

    let cfg = ConfigZ3::new();
    let ctx = ContextZ3::new(&cfg);
    let n = Predicate::OR(vec![
        Predicate::SET(EnumValue::new(&x, "a", None)),
        Predicate::SET(EnumValue::new(&y, "b", None)),
        Predicate::SET(EnumValue::new(&z, "c", None)),
    ]);
    let pred = predicate_to_ast(&ctx, &n, &3);
    assert_eq!(
        "(or (= x_s3 a) (= y_s3 b) (= z_s3 c))",
        ast_to_string_z3!(&ctx, pred)
    );
}

#[test]
fn test_set_predicate(){

    let x = EnumVariable::new("x", &vec!("a", "b", "c", "d"), "letters", None, &Kind::Estimated);

    let cfg = ConfigZ3::new();
    let ctx = ContextZ3::new(&cfg);
    let n = Predicate::SET(EnumValue::new(&x, "b", None));
    let pred = predicate_to_ast(&ctx, &n, &3);
    assert_eq!("(= x_s3 b)", ast_to_string_z3!(&ctx, pred));
}

#[test]
#[should_panic]
fn test_set_predicate_panic(){

    let x = EnumVariable::new("x", &vec!("a", "b", "c", "d"), "letters", None, &Kind::Estimated);

    let cfg = ConfigZ3::new();
    let ctx = ContextZ3::new(&cfg);
    let n = Predicate::SET(EnumValue::new(&x, "e", None));
    let pred = predicate_to_ast(&ctx, &n, &3);
    assert_eq!("(= x_s3 b)", ast_to_string_z3!(&ctx, pred));
}

#[test]
fn test_pbeq_predicate(){

    let x = EnumVariable::new("x", &vec!("a", "b", "c", "d"), "type", None, &Kind::Estimated);
    let y = EnumVariable::new("y", &vec!("a", "b", "c", "d"), "type", None, &Kind::Estimated);
    let z = EnumVariable::new("z", &vec!("a", "b", "c", "d"), "type", None, &Kind::Estimated);

    let cfg = ConfigZ3::new();
    let ctx = ContextZ3::new(&cfg);
    let slv = SolverZ3::new(&ctx);

    let nb = Predicate::SET(EnumValue::new(&x, "a", None));
    let nc = Predicate::SET(EnumValue::new(&y, "b", None));
    let nd = Predicate::SET(EnumValue::new(&z, "c", None));
    let pbeq = Predicate::PBEQ(vec!(nb, nc, nd), 2);
    let pred = predicate_to_ast(&ctx, &pbeq, &4);

    slv_assert_z3!(&ctx, &slv, pred);
    slv_check_z3!(&ctx, &slv);

    let model = slv_get_model_z3!(&ctx, &slv);
    assert_eq!("z_s4 -> a\nx_s4 -> a\ny_s4 -> b\n", model_to_string_z3!(&ctx, model));
}