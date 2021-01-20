use super::*;
use z3_sys::*;
use micro_z3_rust::*;

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
pub fn predicate_to_ast(ctx: &Z3_context, pred: &Predicate, step: u64) -> Z3_ast {
    match pred {
        Predicate::TRUE => new_bool_value_z3(&ctx, true),
        Predicate::FALSE => new_bool_value_z3(&ctx, false),
        Predicate::NOT(p) => not_z3(&ctx, &predicate_to_ast(&ctx, p, step)),
        Predicate::AND(p) => and_z3(&ctx, &p.iter().map(|x| predicate_to_ast(&ctx, x, step)).collect()),
        Predicate::OR(p) => or_z3(&ctx, &p.iter().map(|x| predicate_to_ast(&ctx, x, step)).collect()),
        Predicate::ASS(x) => {
            match x.val.has_type() {
                SPValueType::String => {
                    let enum_sort = new_enum_sort_z3(&ctx, &x.var.r#type, &x.var.domain.iter().map(|x| match x {
                        SPValue::String(z) => z.as_str(),
                        SPValue::Bool(_) => panic!("can't have different types in domain")
                    }).collect());
                    let elems = &enum_sort.1;
                    let index = x.var.domain.iter().position(|r| r == &x.val).unwrap_or_default();
                    eq_z3(&ctx, &new_var_z3(&ctx, &enum_sort.0, format!("{}_s{}", x.var.name.to_string(), step).as_str()), &elems[index])
                },
                SPValueType::Bool => match x.val {
                    SPValue::Bool(false) => not_z3(&ctx, 
                        &new_var_z3(&ctx, &new_bool_sort_z3(&ctx), &format!("{}_s{}", x.var.name.to_string(), step))),
                    SPValue::Bool(true) =>  
                        new_var_z3(&ctx, &new_bool_sort_z3(&ctx), &format!("{}_s{}", x.var.name.to_string(), step)),
                    _ => panic!("Impossible"),
                }
            }
        },
        Predicate::EQ(x, y) => {
            match x.r#type == y.r#type && x.value_type == y.value_type {
                true => {
                    match x.value_type {
                        SPValueType::String => {
                            let sort_1 = new_enum_sort_z3(&ctx, &x.r#type, &x.domain.iter().map(|x| match x {
                                SPValue::String(z) => z.as_str(),
                                SPValue::Bool(_) => panic!("can't have different types in domain")
                            }).collect());
                            let sort_2 = new_enum_sort_z3(&ctx, &y.r#type, &y.domain.iter().map(|x| match x {
                                SPValue::String(z) => z.as_str(),
                                SPValue::Bool(_) => panic!("can't have different types in domain")
                            }).collect());
                            let v_1 = new_var_z3(&ctx, &sort_1.0, format!("{}_s{}", x.name.to_string(), step).as_str());
                            let v_2 = new_var_z3(&ctx, &sort_2.0, format!("{}_s{}", y.name.to_string(), step).as_str());
                            eq_z3(&ctx, &v_1, &v_2)
                        },
                        SPValueType::Bool => {
                            println!("hasodfhoasidfh");
                            eq_z3(&ctx,
                                &new_var_z3(&ctx, &new_bool_sort_z3(&ctx), format!("{}_s{}", x.name.to_string(), step).as_str()),
                                &new_var_z3(&ctx, &new_bool_sort_z3(&ctx), format!("{}_s{}", y.name.to_string(), step).as_str()))
                        },
                    }
                    
                }
                false => panic!("Error c8022e33-ed30-43af-8e45-8cfdaf09e8a5: Sorts '{}' and '{}' are incompatible.", x.r#type, y.r#type)                
            }
        },
        Predicate::PBEQ(x, k) => pbeq_z3(&ctx, &x.iter().map(|z| predicate_to_ast(&ctx, z, step)).collect(), *k),
    }
}