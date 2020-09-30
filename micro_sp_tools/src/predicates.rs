use super::*;
use z3_sys::*;
use z3_v2::*;

#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord)]
pub enum Predicate {
    TRUE,
    FALSE,
    AND(Vec<Predicate>),
    OR(Vec<Predicate>),
    NOT(Box<Predicate>),
    EQRL(EnumVariable, String),
    EQRR(EnumVariable, EnumVariable),
}

pub struct PredicateToAstZ3<'ctx> {
    pub ctx: &'ctx ContextZ3,
    pub pred: Predicate,
    pub step: u32,
    pub r: Z3_ast,
}

impl<'ctx> PredicateToAstZ3<'ctx> {
    pub fn new(ctx: &'ctx ContextZ3, pred: &Predicate, r#type: &str, step: &u32) -> Z3_ast {
        match pred {
            Predicate::TRUE => BoolZ3::new(&ctx, true),
            Predicate::FALSE => BoolZ3::new(&ctx, false),
            Predicate::NOT(p) => NOTZ3::new(&ctx, PredicateToAstZ3::new(&ctx, p, r#type, step)),
            Predicate::AND(p) => ANDZ3::new(&ctx, p.iter().map(|x| PredicateToAstZ3::new(&ctx, x, r#type, step)).collect()),
            Predicate::OR(p) => ORZ3::new(&ctx, p.iter().map(|x| PredicateToAstZ3::new(&ctx, x, r#type, step)).collect()),
            Predicate::EQRL(x, y) => {
                match x.domain.contains(y) {
                    true => {
                        let sort = EnumSortZ3::new(&ctx, &x.r#type, x.domain.iter().map(|x| x.as_str()).collect());
                        let elems = &sort.enum_asts;
                        let index = x.domain.iter().position(|r| r == y).unwrap();
                        EQZ3::new(&ctx, EnumVarZ3::new(&ctx, sort.r, format!("{}_s{}", x.name.to_string(), step).as_str()), elems[index])      
                    },
                    false => panic!("Error 6f789b86-7f6c-4426-ab0f-6b5b72dd2c55: Value '{}' not in the domain of variable '{}'.", y, x.name)
                }
            },
            Predicate::EQRR(x, y) => {
                match x.r#type == y.r#type {
                    true => {
                        match r#type {
                            "guard" | "state" | "specs" => {
                                let sort_1 = EnumSortZ3::new(&ctx, &x.r#type, x.domain.iter().map(|x| x.as_str()).collect());
                                let sort_2 = EnumSortZ3::new(&ctx, &y.r#type, y.domain.iter().map(|y| y.as_str()).collect());
                                let v_1 = EnumVarZ3::new(&ctx, sort_1.r, format!("{}_s{}", x.name.to_string(), step).as_str());
                                let v_2 = EnumVarZ3::new(&ctx, sort_2.r, format!("{}_s{}", y.name.to_string(), step).as_str());
                                EQZ3::new(&ctx, v_1, v_2)
                            },
                            "update" => {
                                let sort_1 = EnumSortZ3::new(&ctx, &x.r#type, x.domain.iter().map(|x| x.as_str()).collect());
                                let sort_2 = EnumSortZ3::new(&ctx, &y.r#type, y.domain.iter().map(|y| y.as_str()).collect());
                                let v_1 = EnumVarZ3::new(&ctx, sort_1.r, format!("{}_s{}", x.name.to_string(), step).as_str());
                                let v_2 = EnumVarZ3::new(&ctx, sort_2.r, format!("{}_s{}", y.name.to_string(), step - 1).as_str());
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
