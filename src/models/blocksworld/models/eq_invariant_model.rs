use crate::models::blocksworld::models::eq_invariant_parser::parser;
use super::*;

// macro_rules! new_enum_assign_c {
//     ($name:expr, $domain:expr, $val:expr) => { ... };
//     ($name:expr, $domain:expr, $val:expr, $r#type:expr) => { ... };
//     ($name:expr, $domain:expr, $val:expr, $r#type:expr, $param:expr) => { ... };
//     ($name:expr, $domain:expr, $val:expr, $r#type:expr, $param:expr, $life:expr) => { ... };
// }

pub fn model(name: &str) -> ParamPlanningProblem {

    let (parsed, blocks) = parser(name);
    let on_domain_init = blocks.iter().chain(vec!(String::from("GRIP"), String::from("TABLE")).iter()).cloned().collect::<Vec<_>>();
    let on_domain: Vec<&str> = on_domain_init.iter().map(|x| x.as_str()).collect();

    let mut transitions = vec![];

    for block in &blocks {
        transitions.push(
            ParamTransition::new(
                &format!("pick_up_{}", block),
                &ppred!(
                    &pass!(&new_bool_assign_c!(&format!("clear_{}", block), true, "on")),
                    &pass!(&new_enum_assign_c!(&format!("{}_on", block), &on_domain, "TABLE", "on", "on"))
                ),
                &ppred!(
                    &pass!(&new_bool_assign_c!(&format!("clear_{}", block), false, "on")),
                    &pass!(&new_enum_assign_c!(&format!("{}_on", block), &on_domain, "GRIP", "on", "on"))
                )
            )
        )
    }

    for block in &blocks {
        transitions.push(
            ParamTransition::new(
                &format!("put_down_{}", block),
                &ppred!(
                    &pass!(&new_enum_assign_c!(&format!("{}_on", block), &on_domain, "GRIP", "on", "on"))
                ),
                &ppred!(
                    &pass!(&new_enum_assign_c!(&format!("{}_on", block), &on_domain, "TABLE", "on", "on")),
                    &pass!(&new_bool_assign_c!(&format!("clear_{}", block), true, "on"))
                )
            )
        )
    }

    for b1 in &blocks {
        for b2 in &blocks {
            if b1 != b2 {
                transitions.push(
                    ParamTransition::new(
                        &format!("stack_{}_on_{}", b1, b2),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("clear_{}", b2), true, "on")),
                            &pass!(&new_enum_assign_c!(&format!("{}_on", b1), &on_domain, "GRIP", "on", "on"))
                        ),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("clear_{}", b2), false, "on")),
                            &pass!(&new_bool_assign_c!(&format!("clear_{}", b1), true, "on")),
                            &pass!(&new_enum_assign_c!(&format!("{}_on", b1), &on_domain, &format!("{}", b2), "on", "on"))
                        )
                    )
                )
            }
        }
    }

    for b1 in &blocks {
        for b2 in &blocks {
            if b1 != b2 {
                transitions.push(
                    ParamTransition::new(
                        &format!("unstack_{}_from_{}", b1, b2),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("clear_{}", b1), true, "on")),
                            &pass!(&new_enum_assign_c!(&format!("{}_on", b1), &on_domain, &format!("{}", b2), "on", "on"))
                        ),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("clear_{}", b2), true, "on")),
                            &pass!(&new_bool_assign_c!(&format!("clear_{}", b1), false, "on")),
                            &pass!(&new_enum_assign_c!(&format!("{}_on", b1), &on_domain, "GRIP", "on", "on"))
                        )
                    )
                )
            }
        }
    }

    let mut invariants = vec![];
    let mut holding = vec![];

    // The problem here is modelled with help of additional invariants:
    // 1. block can't be on another block if that block is on the first block
    // 2. if holding any block, the gripper can't be empty
    // 3. at most one block can be held
    // 4. a block can't simultaneously be on several different blocks
    // 5. if block is on table, it is not on a block
    // 6. if b1 is on b2, b2 is not clear

    // // 1. block can't be on another block if that block is on the first block
    // for b1 in &blocks {
    //     for b2 in &blocks {
    //         if b1 != b2 {
    //             invariants.push(
    //                 pnot!(
    //                     &pand!(
    //                         &pass!(&new_enum_assign_c!(&format!("{}_on", b1), &on_domain, &format!("{}", b2), "on", "on")),
    //                         &pass!(&new_enum_assign_c!(&format!("{}_on", b2), &on_domain, &format!("{}", b1), "on", "on"))
    //                     )
    //                 )
    //             );
    //         }
    //     }
    // }

    // Invariant 1: at most one block can be held
    for b in &blocks {
        holding.push(
            pass!(&new_enum_assign_c!(&format!("{}_on", b), &on_domain, "GRIP", "on", "on"))
        )
    }
    invariants.push(
        por!(
            &Predicate::PBEQ(holding.clone(), 1),
            &Predicate::PBEQ(holding.clone(), 0)
        )
    );

    // // 4. a block can't simultaneously be on several different blocks
    // for b1 in &blocks {
    //     for b2 in &blocks {
    //         for b3 in &blocks {
    //             if b1 != b2 && b1 != b3 && b2 != b3 {
    //                 invariants.push(
    //                     pnot!(
    //                         &pand!(
    //                             &pass!(&new_enum_assign_c!(&format!("{}_on", b1), &on_domain, &format!("{}", b2), "on", "on")),
    //                             &pass!(&new_enum_assign_c!(&format!("{}_on", b1), &on_domain, &format!("{}", b3), "on", "on"))
    //                         )
    //                     )
    //                 );
    //             }
    //         }
    //     }
    // }

    // Invariant 6: if b1 is on b2, b2 is not clear
    for b1 in &blocks {
        for b2 in &blocks {
            if b1 != b2 {
                invariants.push(
                    pnot!(
                        &pand!(
                            &pass!(&new_enum_assign_c!(&format!("{}_on", b1), &on_domain, &format!("{}", b2), "on", "on")),
                            &pass!(&new_bool_assign_c!(&format!("clear_{}", b2), true, "on"))
                        )
                    )
                )
            }
        }
    }

    let on = Parameter::new("on", &true);
    let clear = Parameter::new("clear", &true);

    let problem = ParamPlanningProblem::new(
        &format!("blocksworld_enum_invariants_{}", parsed.name.as_str()), 
        &parsed.init,
        &parsed.goal,
        &transitions,
        &Predicate::AND(invariants),
        &vec!(on)
    );

    problem
}