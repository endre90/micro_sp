use super::*;
use std::time::Instant;
use z3_sys::*;
use z3_v2::*;

/// Only the most basic connectives to form predicates.
#[derive(Debug, PartialEq, Clone, Eq, Ord, PartialOrd)]
pub enum Predicate {
    TRUE,
    FALSE,
    NOT(Box<Predicate>),
    AND(Vec<Predicate>),
    OR(Vec<Predicate>),
    /// Assignment
    ASS(Assignment),
    /// Equality
    EQ(Variable, Variable),
    /// Pseudo-boolean equality 
    PBEQ(Vec<Predicate>, i32)
}

/// Transforms a Predicate to an object that z3 can handle.
pub fn predicate_to_ast(ctx: &ContextZ3, pred: &Predicate, step: u64) -> Z3_ast {
    match pred {
        Predicate::TRUE => BoolZ3::new(&ctx, true),
        Predicate::FALSE => BoolZ3::new(&ctx, false),
        Predicate::NOT(p) => NOTZ3::new(&ctx, predicate_to_ast(&ctx, p, step)),
        Predicate::AND(p) => ANDZ3::new(&ctx, p.iter().map(|x| predicate_to_ast(&ctx, x, step)).collect()),
        Predicate::OR(p) => ORZ3::new(&ctx, p.iter().map(|x| predicate_to_ast(&ctx, x, step)).collect()),
        Predicate::ASS(x) => {
            match x.val.has_type() {
                SPValueType::String => {
                    let sort = EnumSortZ3::new(&ctx, &x.var.r#type, x.var.domain.iter().map(|x| match x {
                        SPValue::String(z) => z.as_str(),
                        SPValue::Bool(_) => panic!("can't have different types in domain")
                    }).collect());
                    let elems = &sort.enum_asts;
                    let index = x.var.domain.iter().position(|r| r == &x.val).unwrap_or_default();
                    EQZ3::new(&ctx, EnumVarZ3::new(&ctx, sort.r, format!("{}_s{}", x.var.name.to_string(), step).as_str()), elems[index])
                },
                SPValueType::Bool => match x.val {
                    SPValue::Bool(false) => EQZ3::new(&ctx, 
                        BoolVarZ3::new(&ctx, &BoolSortZ3::new(&ctx), &format!("{}_s{}", x.var.name.to_string(), step)), 
                        BoolZ3::new(&ctx, false)),
                    SPValue::Bool(true) => EQZ3::new(&ctx, 
                        BoolVarZ3::new(&ctx, &BoolSortZ3::new(&ctx), &format!("{}_s{}", x.var.name.to_string(), step)), 
                        BoolZ3::new(&ctx, true)),
                    _ => panic!("Impossible"),
                }
            }
        },
        Predicate::EQ(x, y) => {
            match x.r#type == y.r#type && x.value_type == y.value_type {
                true => {
                    match x.value_type {
                        SPValueType::String => {
                            let sort_1 = EnumSortZ3::new(&ctx, &x.r#type, x.domain.iter().map(|x| match x {
                                SPValue::String(z) => z.as_str(),
                                SPValue::Bool(_) => panic!("can't have different types in domain")
                            }).collect());
                            let sort_2 = EnumSortZ3::new(&ctx, &y.r#type, y.domain.iter().map(|x| match x {
                                SPValue::String(z) => z.as_str(),
                                SPValue::Bool(_) => panic!("can't have different types in domain")
                            }).collect());
                            let v_1 = EnumVarZ3::new(&ctx, sort_1.r, format!("{}_s{}", x.name.to_string(), step).as_str());
                            let v_2 = EnumVarZ3::new(&ctx, sort_2.r, format!("{}_s{}", y.name.to_string(), step).as_str());
                            EQZ3::new(&ctx, v_1, v_2)
                        },
                        SPValueType::Bool => {
                            println!("hasodfhoasidfh");
                            EQZ3::new(&ctx,
                                BoolVarZ3::new(&ctx, &BoolSortZ3::new(&ctx), format!("{}_s{}", x.name.to_string(), step).as_str()),
                                BoolVarZ3::new(&ctx, &BoolSortZ3::new(&ctx), format!("{}_s{}", y.name.to_string(), step).as_str()))
                        },
                    }
                    
                }
                false => panic!("Error c8022e33-ed30-43af-8e45-8cfdaf09e8a5: Sorts '{}' and '{}' are incompatible.", x.r#type, y.r#type)                
            }
        },
        Predicate::PBEQ(x, k) => PBEQZ3::new(&ctx, x.iter().map(|z| predicate_to_ast(&ctx, z, step)).collect(), *k),
    }
}