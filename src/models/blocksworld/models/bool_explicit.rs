use crate::models::blocksworld::models::bool_explicit_parser::parser;
use super::*;

// macro_rules! new_bool_assign_c {
//     ($name:expr, $domain:expr, $val:expr) => { ... };
//     ($name:expr, $domain:expr, $val:expr, $r#type:expr) => { ... };
//     ($name:expr, $domain:expr, $val:expr, $r#type:expr, $param:expr) => { ... };
//     ($name:expr, $domain:expr, $val:expr, $r#type:expr, $param:expr, $life:expr) => { ... };
// }

/// Explicitly generating negative predicates from diff(ojb/init)
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
                    &pass!(&new_bool_assign_c!(&format!("clear_{}", block), true, "block")),
                    &pass!(&new_bool_assign_c!(&format!("hand_empty"), true, "hand"))
                ),
                &ppred!(
                    &pass!(&new_bool_assign_c!(&format!("clear_{}", block), false, "block")),
                    &pass!(&new_bool_assign_c!(&format!("ontable_{}", block), false, "block")),
                    &pass!(&new_bool_assign_c!(&format!("holding_{}", block), true, "hand")),
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
                    &pass!(&new_bool_assign_c!(&format!("holding_{}", block), true, "hand"))
                ),
                &ppred!(
                    &pass!(&new_bool_assign_c!(&format!("holding_{}", block), false, "hand")),
                    &pass!(&new_bool_assign_c!(&format!("clear_{}", block), true, "block")),
                    &pass!(&new_bool_assign_c!(&format!("ontable_{}", block), true, "block")),
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
                            &pass!(&new_bool_assign_c!(&format!("clear_{}", b2), true, "block")),
                            &pass!(&new_bool_assign_c!(&format!("holding_{}", b1), true, "hand"))
                        ),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("clear_{}", b2), false, "block")),
                            &pass!(&new_bool_assign_c!(&format!("holding_{}", b1), false, "hand")),
                            &pass!(&new_bool_assign_c!(&format!("clear_{}", b1), true, "block")),
                            &pass!(&new_bool_assign_c!(&format!("{}_on_{}", b1, b2), true, "block")),
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
                            &pass!(&new_bool_assign_c!(&format!("clear_{}", b1), true, "block")),
                            &pass!(&new_bool_assign_c!(&format!("{}_on_{}", b1, b2), true, "block")),
                            &pass!(&new_bool_assign_c!(&format!("hand_empty"), true, "hand"))
                        ),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("holding_{}", b1), true, "hand")),
                            &pass!(&new_bool_assign_c!(&format!("clear_{}", b2), true, "block")),
                            &pass!(&new_bool_assign_c!(&format!("clear_{}", b1), false, "block")),
                            &pass!(&new_bool_assign_c!(&format!("hand_empty"), false, "hand")),
                            &pass!(&new_bool_assign_c!(&format!("{}_on_{}", b1, b2), false, "block"))
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

    let block = Parameter::new("block", &true);
    // let clear = Parameter::new("clear", &true);
    // let ontable = Parameter::new("ontable", &true);
    let hand = Parameter::new("hand", &true);
    // let holding = Parameter::new("holding", &true);

    let problem = ParamPlanningProblem::new(
        &format!("blocksworld_bool_explicit_{}", parsed.name.as_str()), 
        &parsed.init,
        &parsed.goal,
        &transitions,
        &Predicate::TRUE,
        &vec!(hand, block)
    );

    problem
}