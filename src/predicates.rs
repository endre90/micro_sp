use super::*;
use z3_sys::*;
use z3_v2::*;

#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord)]
pub enum Predicate {
    TRUE,
    FALSE,
    NOT(Box<Predicate>),
    AND(Vec<Predicate>),
    OR(Vec<Predicate>),
    EQ(EnumValue)
}

pub fn predicate_to_ast(ctx: &ContextZ3, pred: &Predicate, step: &u32) -> Z3_ast {
    match pred {
        Predicate::TRUE => BoolZ3::new(&ctx, true),
        Predicate::FALSE => BoolZ3::new(&ctx, false),
        Predicate::NOT(p) => NOTZ3::new(&ctx, predicate_to_ast(&ctx, p, step)),
        Predicate::AND(p) => ANDZ3::new(&ctx, p.iter().map(|x| predicate_to_ast(&ctx, x, step)).collect()),
        Predicate::OR(p) => ORZ3::new(&ctx, p.iter().map(|x| predicate_to_ast(&ctx, x, step)).collect()),
        Predicate::EQ(x) => {
            let sort = EnumSortZ3::new(&ctx, &x.var.r#type, x.var.domain.iter().map(|x| x.as_str()).collect());
            let elems = &sort.enum_asts;
            let index = x.var.domain.iter().position(|r| r == &x.val).unwrap();
            EQZ3::new(&ctx, EnumVarZ3::new(&ctx, sort.r, format!("{}_s{}", x.var.name.to_string(), step).as_str()), elems[index])
        }
    }
}
