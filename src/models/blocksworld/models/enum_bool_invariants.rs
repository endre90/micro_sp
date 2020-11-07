use crate::models::blocksworld::models::parser::parser;
use super::*;

// macro_rules! new_enum_assign_c {
//     ($name:expr, $domain:expr, $val:expr) => { ... };
//     ($name:expr, $domain:expr, $val:expr, $r#type:expr) => { ... };
//     ($name:expr, $domain:expr, $val:expr, $r#type:expr, $param:expr) => { ... };
//     ($name:expr, $domain:expr, $val:expr, $r#type:expr, $param:expr, $life:expr) => { ... };
// }

/// Instead of explicitly generating negative predicates from diff(ojb/init),
/// the problem here is modelled with help of additional invariants:
/// 1. block can't be on another block if that block is on the first block
/// 2. if holding any block, the gripper can't be empty
/// 3. at most one block can be held
/// 4. a block can't simultaneously be on several different blocks
/// 5. if block is on table, it is not on a block
/// 6. if b1 is on b2, b2 is not clear
pub fn model(name: &str) -> ParamPlanningProblem {

    let (parsed, blocks) = parser(name);
    let domain = vec!["true", "false"];

    let mut pick_up_transitions = vec![];
    let mut put_down_transitions = vec![];
    let mut stack_transitions = vec![];
    let mut unstack_transitions = vec![];

    for block in &blocks {
        pick_up_transitions.push(
            ParamTransition::new(
                &format!("pick_up_{}", block),
                &ppred!(
                    &pass!(&new_enum_assign_c!(&format!("clear_{}", block), &domain, "true", "bool", "clear")),
                    &pass!(&new_enum_assign_c!(&format!("hand_empty"), &domain, "true", "bool", "hand"))
                ),
                &ppred!(
                    &pass!(&new_enum_assign_c!(&format!("clear_{}", block), &domain, "false", "bool", "clear")),
                    &pass!(&new_enum_assign_c!(&format!("ontable_{}", block), &domain, "false", "bool", "ontable")),
                    &pass!(&new_enum_assign_c!(&format!("holding_{}", block), &domain, "true", "bool", "holding")),
                    &pass!(&new_enum_assign_c!(&format!("hand_empty"), &domain, "false", "bool", "hand"))
                )
            )
        )
    }

    for block in &blocks {
        put_down_transitions.push(
            ParamTransition::new(
                &format!("put_down_{}", block),
                &ppred!(
                    &pass!(&new_enum_assign_c!(&format!("holding_{}", block), &domain, "true", "bool", "holding"))
                ),
                &ppred!(
                    &pass!(&new_enum_assign_c!(&format!("holding_{}", block), &domain, "false", "bool", "holding")),
                    &pass!(&new_enum_assign_c!(&format!("clear_{}", block), &domain, "true", "bool", "clear")),
                    &pass!(&new_enum_assign_c!(&format!("ontable_{}", block), &domain, "true", "bool", "ontable")),
                    &pass!(&new_enum_assign_c!(&format!("hand_empty"), &domain, "true", "bool", "hand"))
                )
            )
        )
    }

    for b1 in &blocks {
        for b2 in &blocks {
            if b1 != b2 {
                stack_transitions.push(
                    ParamTransition::new(
                        &format!("stack_{}_on_{}", b1, b2),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("clear_{}", b2), &domain, "true", "bool", "clear")),
                            &pass!(&new_enum_assign_c!(&format!("holding_{}", b1), &domain, "true", "bool", "holding"))
                        ),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("clear_{}", b2), &domain, "false", "bool", "clear")),
                            &pass!(&new_enum_assign_c!(&format!("holding_{}", b1), &domain, "false", "bool", "holding")),
                            &pass!(&new_enum_assign_c!(&format!("clear_{}", b1), &domain, "true", "bool", "clear")),
                            &pass!(&new_enum_assign_c!(&format!("{}_on_{}", b1, b2), &domain, "true", "bool", "on")),
                            &pass!(&new_enum_assign_c!(&format!("hand_empty"), &domain, "true", "bool", "hand"))
                        )
                    )
                )
            }
        }
    }

    for b1 in &blocks {
        for b2 in &blocks {
            if b1 != b2 {
                unstack_transitions.push(
                    ParamTransition::new(
                        &format!("unstack_{}_from_{}", b1, b2),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("clear_{}", b1), &domain, "true", "bool", "clear")),
                            &pass!(&new_enum_assign_c!(&format!("{}_on_{}", b1, b2), &domain, "true", "bool", "on")),
                            &pass!(&new_enum_assign_c!(&format!("hand_empty"), &domain, "true", "bool", "hand"))
                        ),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("holding_{}", b1), &domain, "true", "bool", "holding")),
                            &pass!(&new_enum_assign_c!(&format!("clear_{}", b2), &domain, "true", "bool", "clear")),
                            &pass!(&new_enum_assign_c!(&format!("clear_{}", b1), &domain, "false", "bool", "clear")),
                            &pass!(&new_enum_assign_c!(&format!("hand_empty"), &domain, "false", "bool", "hand")),
                            &pass!(&new_enum_assign_c!(&format!("{}_on_{}", b1, b2), &domain, "false", "bool", "on"))
                        )
                    )
                )
            }
        }
    }

    let mut transitions = vec![];
    for t in vec![
        pick_up_transitions,
        put_down_transitions,
        stack_transitions,
        unstack_transitions,
    ] {
        transitions.extend(t)
    }

    let mut invariants = vec![];

    // Invariant 1: block can't be on another block if that block is on the first block
    for b1 in &blocks {
        for b2 in &blocks {
            if b1 != b2 {
                invariants.push(
                    pnot!(
                        &pand!(
                            &pass!(&new_enum_assign_c!(&format!("{}_on_{}", b1, b2), &domain, "true", "bool", "on")),
                            &pass!(&new_enum_assign_c!(&format!("{}_on_{}", b2, b1), &domain, "true", "bool", "on"))
                        )
                    )
                )
            }
        }
    }

    // Invariant 2: if holding any block, the gripper can't be empty
    let mut holding = vec![];
    for b in &blocks {
        holding.push(
            pass!(&new_enum_assign_c!(&format!("holding_{}", b), &domain, "true", "bool", "holding"))
        )
    }
    invariants.push(
        pnot!(
            &pand!(
                &pass!(&new_enum_assign_c!(&format!("hand_empty"), &domain, "true", "bool", "hand")),
                &Predicate::OR(holding.to_owned())
            )
        )
    );

    // Invariant 3: at most one block can be held
    invariants.push(
        por!(
            &Predicate::PBEQ(holding.clone(), 1),
            &Predicate::PBEQ(holding, 0)
        )
    );

    // Invariants 4 and 5
    for b1 in &blocks {
        let mut local_vec = vec![];
        for b2 in &blocks {
            if b1 != b2 {
                local_vec.push(
                    pass!(&new_enum_assign_c!(&format!("{}_on_{}", b1, b2), &domain, "true", "bool", "on"))
                )
            }
        }

        // Invariants 4: a block can't simultaneously be on several different blocks
        invariants.push(
            por!(
                &Predicate::PBEQ(local_vec.clone(), 1),
                &Predicate::PBEQ(local_vec.clone(), 0)
            )
        );

        // Invariant 5: if block is on table, it is not on a block
        invariants.push(
            pnot!(
                &pand!(
                    &pass!(&new_enum_assign_c!(&format!("ontable_{}", b1), &domain, "true", "bool", "ontable")),
                    &Predicate::OR(local_vec.clone())
                )
            )
        );
    }

    // Invariant 6: if b1 is on b2, b2 is not clear
    // for b1 in &blocks {
    //     for b2 in &blocks {
    //         if b1 != b2 {
    //             invariants.push(
    //                 pnot!(
    //                     &pand!(
    //                         &pass!(&new_enum_assign_c!(&format!("{}_on_{}", b1, b2), &domain, "true", "bool", "on")),
    //                         &pass!(&new_enum_assign_c!(&format!("clear_{}", b2), &domain, "true", "bool", "clear"))
    //                     )
    //                 )
    //             )
    //         }
    //     }
    // }

    let on = Parameter::new("on", &true);
    let clear = Parameter::new("clear", &true);
    let ontable = Parameter::new("ontable", &true);
    let hand = Parameter::new("hand", &true);
    let holding = Parameter::new("holding", &true);
    // let block = Parameter::new("block", &true);

    let problem = ParamPlanningProblem::new(
        parsed.name.as_str(), 
        &parsed.init,
        &parsed.goal,
        &transitions,
        &Predicate::AND(invariants),
        &vec!(on, clear, ontable, hand, holding)
    );

    println!("1234_INIT {:?}", problem.init);
    println!("1234_GOAL {:?}", problem.goal);

    problem
}