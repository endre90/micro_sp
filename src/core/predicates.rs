use super::*;
use z3_sys::*;
use z3_v2::*;

/// Only the most basic connectives to form predicates.
#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord)]
pub enum Predicate {
    TRUE,
    FALSE,
    NOT(Box<Predicate>),
    AND(Vec<Predicate>),
    OR(Vec<Predicate>),
    /// Assignment
    SET(EnumValue),
    /// Equality
    EQ(EnumVariable, EnumVariable),
    /// Pseudo-boolean equality 
    PBEQ(Vec<Predicate>, i32)
}

/// Transforms a Predicate to an object that z3 can handle.
pub fn predicate_to_ast(ctx: &ContextZ3, pred: &Predicate, step: &u32) -> Z3_ast {
    match pred {
        Predicate::TRUE => BoolZ3::new(&ctx, true),
        Predicate::FALSE => BoolZ3::new(&ctx, false),
        Predicate::NOT(p) => NOTZ3::new(&ctx, predicate_to_ast(&ctx, p, step)),
        Predicate::AND(p) => ANDZ3::new(&ctx, p.iter().map(|x| predicate_to_ast(&ctx, x, step)).collect()),
        Predicate::OR(p) => ORZ3::new(&ctx, p.iter().map(|x| predicate_to_ast(&ctx, x, step)).collect()),
        Predicate::SET(x) => {
            let sort = EnumSortZ3::new(&ctx, &x.var.r#type, x.var.domain.iter().map(|x| x.as_str()).collect());
            let elems = &sort.enum_asts;
            let index = x.var.domain.iter().position(|r| r == &x.val).unwrap_or_default();
            EQZ3::new(&ctx, EnumVarZ3::new(&ctx, sort.r, format!("{}_s{}", x.var.name.to_string(), step).as_str()), elems[index])
        },
        Predicate::EQ(x, y) => {
            match x.r#type == y.r#type {
                true => {
                    let sort_1 = EnumSortZ3::new(&ctx, &x.r#type, x.domain.iter().map(|x| x.as_str()).collect());
                    let sort_2 = EnumSortZ3::new(&ctx, &y.r#type, y.domain.iter().map(|y| y.as_str()).collect());
                    let v_1 = EnumVarZ3::new(&ctx, sort_1.r, format!("{}_s{}", x.name.to_string(), step).as_str());
                    let v_2 = EnumVarZ3::new(&ctx, sort_2.r, format!("{}_s{}", y.name.to_string(), step).as_str());
                    EQZ3::new(&ctx, v_1, v_2)
                }
                false => panic!("Error c8022e33-ed30-43af-8e45-8cfdaf09e8a5: Sorts '{}' and '{}' are incompatible.", x.r#type, y.r#type)                
            }
        },
        Predicate::PBEQ(x, k) => PBEQZ3::new(&ctx, x.iter().map(|z| predicate_to_ast(&ctx, z, step)).collect(), *k),
    }
}
