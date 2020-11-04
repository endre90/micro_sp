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
    let x = enum_c!("x", vec!("a", "b", "c", "d"), "letters");
    let cfg = ConfigZ3::new();
    let ctx = ContextZ3::new(&cfg);
    let n = Predicate::NOT(Box::new(Predicate::ASS(enum_assign!(x, "b"))));
    let pred = predicate_to_ast(&ctx, &n, &3);
    assert_eq!("(not (= x_s3 b))", ast_to_string_z3!(&ctx, pred));
}

#[test]
fn test_and_predicate() {
    let x = enum_e!("x", vec!("a", "b", "c", "d"), "letters");
    let y = enum_e!("y", vec!("a", "b", "c", "d"), "letters");
    let z = enum_e!("z", vec!("a", "b", "c", "d"), "letters");

    let cfg = ConfigZ3::new();
    let ctx = ContextZ3::new(&cfg);
    let n = Predicate::AND(vec![
        Predicate::ASS(enum_assign!(x, "a")),
        Predicate::ASS(enum_assign!(y, "b")),
        Predicate::ASS(enum_assign!(z, "c")),
    ]);
    let pred = predicate_to_ast(&ctx, &n, &3);
    assert_eq!(
        "(and (= x_s3 a) (= y_s3 b) (= z_s3 c))",
        ast_to_string_z3!(&ctx, pred)
    );
}

#[test]
fn test_or_predicate() {
    let x = enum_e!("x", vec!("a", "b", "c", "d"), "letters");
    let y = enum_e!("y", vec!("a", "b", "c", "d"), "letters");
    let z = enum_e!("z", vec!("a", "b", "c", "d"), "letters");

    let cfg = ConfigZ3::new();
    let ctx = ContextZ3::new(&cfg);
    let n = Predicate::OR(vec![
        Predicate::ASS(enum_assign!(x, "a")),
        Predicate::ASS(enum_assign!(y, "b")),
        Predicate::ASS(enum_assign!(z, "c")),
    ]);
    let pred = predicate_to_ast(&ctx, &n, &3);
    assert_eq!(
        "(or (= x_s3 a) (= y_s3 b) (= z_s3 c))",
        ast_to_string_z3!(&ctx, pred)
    );
}

#[test]
fn test_ass_predicate(){
    let x = enum_e!("x", vec!("a", "b", "c", "d"), "letters");

    let cfg = ConfigZ3::new();
    let ctx = ContextZ3::new(&cfg);
    let n = Predicate::ASS(enum_assign!(x, "b"));
    let pred = predicate_to_ast(&ctx, &n, &3);
    assert_eq!("(= x_s3 b)", ast_to_string_z3!(&ctx, pred));
}

#[test]
#[should_panic]
fn test_set_predicate_panic(){

    let x = enum_e!("x", vec!("a", "b", "c", "d"), "letters");

    let cfg = ConfigZ3::new();
    let ctx = ContextZ3::new(&cfg);
    let n = Predicate::ASS(enum_assign!(x, "e"));
    let pred = predicate_to_ast(&ctx, &n, &3);
    assert_eq!("(= x_s3 b)", ast_to_string_z3!(&ctx, pred));
}

#[test]
fn test_pbeq_predicate(){

    let x = enum_e!("x", vec!("a", "b", "c", "d"), "letters");
    let y = enum_e!("y", vec!("a", "b", "c", "d"), "letters");
    let z = enum_e!("z", vec!("a", "b", "c", "d"), "letters");
    
    let cfg = ConfigZ3::new();
    let ctx = ContextZ3::new(&cfg);
    let slv = SolverZ3::new(&ctx);

    let nb = Predicate::ASS(enum_assign!(x, "a"));
    let nc = Predicate::ASS(enum_assign!(y, "b"));
    let nd = Predicate::ASS(enum_assign!(z, "c"));
    let pbeq = Predicate::PBEQ(vec!(nb, nc, nd), 2);
    let pred = predicate_to_ast(&ctx, &pbeq, &4);

    slv_assert_z3!(&ctx, &slv, pred);
    slv_check_z3!(&ctx, &slv);

    let model = slv_get_model_z3!(&ctx, &slv);
    assert_eq!("z_s4 -> a\nx_s4 -> a\ny_s4 -> b\n", model_to_string_z3!(&ctx, model));
}