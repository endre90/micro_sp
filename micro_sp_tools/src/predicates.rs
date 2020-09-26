use z3_sys::*;
use z3_v2::*;
use super::*;
use arrayvec::ArrayString;

#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord)]
pub enum Predicate {
    TRUE,
    FALSE,
    AND(Vec<Predicate>),
    OR(Vec<Predicate>),
    NOT(Box<Predicate>),
    EQRL(EnumVariable, String),
    EQRR(EnumVariable, EnumVariable)
}

pub struct PredicateToAstZ3<'ctx> {
    pub ctx: &'ctx ContextZ3,
    pub pred: Predicate,
    pub step: u32,
    pub r: Z3_ast
}

impl <'ctx> PredicateToAstZ3<'ctx> {
    pub fn new(ctx: &'ctx ContextZ3, pred: &Predicate, r#type: &str, step: &u32) -> Z3_ast {
        match pred {
            Predicate::TRUE => BoolZ3::new(&ctx, true),
            Predicate::FALSE => BoolZ3::new(&ctx, false),
            Predicate::NOT(p) => NOTZ3::new(&ctx, PredicateToAstZ3::new(&ctx, p, r#type, step)),
            Predicate::AND(p) => ANDZ3::new(&ctx, p.iter().map(|x| PredicateToAstZ3::new(&ctx, x, r#type, step)).collect()),
            Predicate::OR(p) => ORZ3::new(&ctx, p.iter().map(|x| PredicateToAstZ3::new(&ctx, x, r#type, step)).collect()),
            Predicate::EQRL(x, y) => {
                let y = &ArrayString::<[_; 32]>::from(y).unwrap_or_default();
                match x.domain.contains(y) {
                    true => {
                        let sort = EnumSortZ3::new(&ctx, &x.r#type, x.domain.iter().map(|x| x.as_str()).collect());
                        let elems = &sort.enum_asts;
                        let index = x.domain.iter().position(|r| r == y).unwrap();
                        EQZ3::new(&ctx, EnumVarZ3::new(&ctx, sort.r, format!("{}_s{}", x.name.key.to_string(), step).as_str()), elems[index])      
                    },
                    false => panic!("Error 6f789b86-7f6c-4426-ab0f-6b5b72dd2c55: Value '{}' not in the domain of variable '{}'.", y, x.name.key)
                }
            },
            Predicate::EQRR(x, y) => {
                match x.r#type == y.r#type {
                    true => {
                        match r#type {
                            "guard" | "state" | "specs" => {
                                let sort_1 = EnumSortZ3::new(&ctx, &x.r#type, x.domain.iter().map(|x| x.as_str()).collect());
                                let sort_2 = EnumSortZ3::new(&ctx, &y.r#type, y.domain.iter().map(|y| y.as_str()).collect());
                                let v_1 = EnumVarZ3::new(&ctx, sort_1.r, format!("{}_s{}", x.name.key.to_string(), step).as_str());
                                let v_2 = EnumVarZ3::new(&ctx, sort_2.r, format!("{}_s{}", y.name.key.to_string(), step).as_str());
                                EQZ3::new(&ctx, v_1, v_2)
                            },
                            "update" => {
                                let sort_1 = EnumSortZ3::new(&ctx, &x.r#type, x.domain.iter().map(|x| x.as_str()).collect());
                                let sort_2 = EnumSortZ3::new(&ctx, &y.r#type, y.domain.iter().map(|y| y.as_str()).collect());
                                let v_1 = EnumVarZ3::new(&ctx, sort_1.r, format!("{}_s{}", x.name.key.to_string(), step).as_str());
                                let v_2 = EnumVarZ3::new(&ctx, sort_2.r, format!("{}_s{}", y.name.key.to_string(), step - 1).as_str());
                                EQZ3::new(&ctx, v_1, v_2)
                            },
                            _ => panic!("Error 53b0fd14-1ddd-4bf0-8dc7-d372d6ad8c99: Predicate type '{}' is not allowed.", r#type)
                        }
                    },
                    false => panic!("Error c8022e33-ed30-43af-8e45-8cfdaf09e8a5: Sorts '{}' and '{}' are incompatible.", x.r#type, y.r#type)
                }
            }
        }
    }
}

#[test]
fn test_true_predicate(){

    let cfg = ConfigZ3::new();
    let ctx = ContextZ3::new(&cfg);
    let t = Predicate::TRUE;
    let pred = PredicateToAstZ3::new(&ctx, &t, "guard", &3);
    assert_eq!("true", ast_to_string_z3!(&ctx, pred));
}

#[test]
fn test_false_predicate(){

    let cfg = ConfigZ3::new();
    let ctx = ContextZ3::new(&cfg);
    let t = Predicate::FALSE;
    let pred = PredicateToAstZ3::new(&ctx, &t, "guard", &3);
    assert_eq!("false", ast_to_string_z3!(&ctx, pred));
}

#[test]
fn test_not_predicate(){

    let x = EnumVariable::new("x", "letters", &vec!("a", "b", "c", "d"), None, &ControlKind::None);
    let y = EnumVariable::new("y", "letters", &vec!("a", "b", "c", "d"), None, &ControlKind::None);

    let cfg = ConfigZ3::new();
    let ctx = ContextZ3::new(&cfg);
    let n = Predicate::NOT(Box::new(Predicate::EQRR(x, y)));
    let pred = PredicateToAstZ3::new(&ctx, &n, "guard", &3);
    assert_eq!("(not (= x_s3 y_s3))", ast_to_string_z3!(&ctx, pred));
}

#[test]
fn test_and_predicate(){

    let x = EnumVariable::new("x", "letters", &vec!("a", "b", "c", "d"), None, &ControlKind::None);
    let y = EnumVariable::new("y", "letters", &vec!("a", "b", "c", "d"), None, &ControlKind::None);
    let z = EnumVariable::new("z", "letters", &vec!("a", "b", "c", "d"), None, &ControlKind::None);

    let cfg = ConfigZ3::new();
    let ctx = ContextZ3::new(&cfg);
    let n = Predicate::AND(vec!(Predicate::EQRR(x, y.clone()), Predicate::EQRR(y, z)));
    let pred = PredicateToAstZ3::new(&ctx, &n, "guard", &3);
    assert_eq!("(and (= x_s3 y_s3) (= y_s3 z_s3))", ast_to_string_z3!(&ctx, pred));
}

#[test]
fn test_or_predicate(){

    let x = EnumVariable::new("x", "letters", &vec!("a", "b", "c", "d"), None, &ControlKind::None);
    let y = EnumVariable::new("y", "letters", &vec!("a", "b", "c", "d"), None, &ControlKind::None);
    let z = EnumVariable::new("z", "letters", &vec!("a", "b", "c", "d"), None, &ControlKind::None);

    let cfg = ConfigZ3::new();
    let ctx = ContextZ3::new(&cfg);
    let n = Predicate::OR(vec!(Predicate::EQRR(x, y.clone()), Predicate::EQRR(y, z)));
    let pred = PredicateToAstZ3::new(&ctx, &n, "guard", &3);
    assert_eq!("(or (= x_s3 y_s3) (= y_s3 z_s3))", ast_to_string_z3!(&ctx, pred));
}

#[test]
fn test_eqrl_predicate(){

    let x = EnumVariable::new("x", "letters", &vec!("a", "b", "c", "d"), None, &ControlKind::None);
    let b = "b".to_string();

    let cfg = ConfigZ3::new();
    let ctx = ContextZ3::new(&cfg);
    let n = Predicate::EQRL(x, b);
    let pred = PredicateToAstZ3::new(&ctx, &n, "guard", &3);
    assert_eq!("(= x_s3 b)", ast_to_string_z3!(&ctx, pred));
}

#[test]
#[should_panic(expected = "Error 6f789b86-7f6c-4426-ab0f-6b5b72dd2c55: Value 'e' not in the domain of variable 'x'.")]
fn test_eqrl_predicate_panic(){

    let x = EnumVariable::new("x", "letters", &vec!("a", "b", "c", "d"), None, &ControlKind::None);
    let e = "e".to_string();

    let cfg = ConfigZ3::new();
    let ctx = ContextZ3::new(&cfg);
    let n = Predicate::EQRL(x, e);
    PredicateToAstZ3::new(&ctx, &n, "guard", &3);
}

#[test]
fn test_eqrr_predicate(){

    let x = EnumVariable::new("x", "letters", &vec!("a", "b", "c", "d"), None, &ControlKind::None);
    let y = EnumVariable::new("y", "letters", &vec!("a", "b", "c", "d"), None, &ControlKind::None);

    let cfg = ConfigZ3::new();
    let ctx = ContextZ3::new(&cfg);
    let n = Predicate::EQRR(x, y.clone());
    let pred = PredicateToAstZ3::new(&ctx, &n, "guard", &3);
    assert_eq!("(= x_s3 y_s3)", ast_to_string_z3!(&ctx, pred));
}

#[test]
#[should_panic(expected = "Error 53b0fd14-1ddd-4bf0-8dc7-d372d6ad8c99: Predicate type 'other' is not allowed.")]
fn test_eqrr_predicate_panic_1(){

    let x = EnumVariable::new("x", "letters", &vec!("a", "b", "c", "d"), None, &ControlKind::None);
    let y = EnumVariable::new("y", "letters", &vec!("a", "b", "c", "d"), None, &ControlKind::None);

    let cfg = ConfigZ3::new();
    let ctx = ContextZ3::new(&cfg);
    let n = Predicate::EQRR(x, y.clone());
    PredicateToAstZ3::new(&ctx, &n, "other", &3);
}

#[test]
#[should_panic(expected = "Error c8022e33-ed30-43af-8e45-8cfdaf09e8a5: Sorts 'letters' and 'numbers' are incompatible.")]
fn test_eqrr_predicate_panic_2(){

    let x = EnumVariable::new("x", "letters", &vec!("a", "b", "c", "d"), None, &ControlKind::None);
    let y = EnumVariable::new("y", "numbers", &vec!("1", "2", "3", "4"), None, &ControlKind::None);

    let cfg = ConfigZ3::new();
    let ctx = ContextZ3::new(&cfg);
    let n = Predicate::EQRR(x, y.clone());
    PredicateToAstZ3::new(&ctx, &n, "state", &3);
}