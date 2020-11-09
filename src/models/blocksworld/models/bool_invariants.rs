use crate::models::blocksworld::models::bool_invariants_parser::parser;
use super::*;

// macro_rules! new_bool_assign_c {
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

    let mut pick_up_transitions = vec![];
    let mut put_down_transitions = vec![];
    let mut stack_transitions = vec![];
    let mut unstack_transitions = vec![];

    for block in &blocks {
        pick_up_transitions.push(
            ParamTransition::new(
                &format!("pick_up_{}", block),
                &ppred!(
                    &pass!(&new_bool_assign_c!(&format!("clear_{}", block), true, "clear")),
                    &pass!(&new_bool_assign_c!(&format!("hand_empty"), true, "hand"))
                ),
                &ppred!(
                    &pass!(&new_bool_assign_c!(&format!("clear_{}", block), false, "clear")),
                    &pass!(&new_bool_assign_c!(&format!("ontable_{}", block), false, "ontable")),
                    &pass!(&new_bool_assign_c!(&format!("holding_{}", block), true, "holding")),
                    &pass!(&new_bool_assign_c!(&format!("hand_empty"), false, "hand"))
                )
            )
        )
    }

    for block in &blocks {
        put_down_transitions.push(
            ParamTransition::new(
                &format!("put_down_{}", block),
                &ppred!(
                    &pass!(&new_bool_assign_c!(&format!("holding_{}", block), true, "holding"))
                ),
                &ppred!(
                    &pass!(&new_bool_assign_c!(&format!("holding_{}", block), false, "holding")),
                    &pass!(&new_bool_assign_c!(&format!("clear_{}", block), true, "clear")),
                    &pass!(&new_bool_assign_c!(&format!("ontable_{}", block), true, "ontable")),
                    &pass!(&new_bool_assign_c!(&format!("hand_empty"), true, "hand"))
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
                            &pass!(&new_bool_assign_c!(&format!("clear_{}", b2), true, "clear")),
                            &pass!(&new_bool_assign_c!(&format!("holding_{}", b1), true, "holding"))
                        ),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("clear_{}", b2), false, "clear")),
                            &pass!(&new_bool_assign_c!(&format!("holding_{}", b1), false, "holding")),
                            &pass!(&new_bool_assign_c!(&format!("clear_{}", b1), true, "clear")),
                            &pass!(&new_bool_assign_c!(&format!("{}_on_{}", b1, b2), true, "on")),
                            &pass!(&new_bool_assign_c!(&format!("hand_empty"), true, "hand"))
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
                            &pass!(&new_bool_assign_c!(&format!("clear_{}", b1), true, "clear")),
                            &pass!(&new_bool_assign_c!(&format!("{}_on_{}", b1, b2), true, "on")),
                            &pass!(&new_bool_assign_c!(&format!("hand_empty"), true, "hand"))
                        ),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("holding_{}", b1), true, "holding")),
                            &pass!(&new_bool_assign_c!(&format!("clear_{}", b2), true, "clear")),
                            &pass!(&new_bool_assign_c!(&format!("clear_{}", b1), false, "clear")),
                            &pass!(&new_bool_assign_c!(&format!("hand_empty"), false, "hand")),
                            &pass!(&new_bool_assign_c!(&format!("{}_on_{}", b1, b2), false, "on"))
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
                            &pass!(&new_bool_assign_c!(&format!("{}_on_{}", b1, b2), true, "on")),
                            &pass!(&new_bool_assign_c!(&format!("{}_on_{}", b2, b1), true, "on"))
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
            pass!(&new_bool_assign_c!(&format!("holding_{}", b), true, "holding"))
        )
    }
    invariants.push(
        pnot!(
            &pand!(
                &pass!(&new_bool_assign_c!(&format!("hand_empty"), true, "hand")),
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
                    pass!(&new_bool_assign_c!(&format!("{}_on_{}", b1, b2), true, "on"))
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
                    &pass!(&new_bool_assign_c!(&format!("ontable_{}", b1), true, "ontable")),
                    &Predicate::OR(local_vec.clone())
                )
            )
        );
    }

    // Invariant 6: if b1 is on b2, b2 is not clear
    for b1 in &blocks {
        for b2 in &blocks {
            if b1 != b2 {
                invariants.push(
                    pnot!(
                        &pand!(
                            &pass!(&new_bool_assign_c!(&format!("{}_on_{}", b1, b2), true, "on")),
                            &pass!(&new_bool_assign_c!(&format!("clear_{}", b2), true, "clear"))
                        )
                    )
                )
            }
        }
    }

    let on = Parameter::new("on", &true);
    let clear = Parameter::new("clear", &true);
    let ontable = Parameter::new("ontable", &true);
    let hand = Parameter::new("hand", &true);
    let holding = Parameter::new("holding", &true);

    let problem = ParamPlanningProblem::new(
        &format!("blocksworld_bool_invariants_{}", parsed.name.as_str()),  
        &parsed.init,
        &parsed.goal,
        &transitions,
        &Predicate::AND(invariants),
        &vec!(on, clear, ontable, hand, holding)
    );

    problem
}