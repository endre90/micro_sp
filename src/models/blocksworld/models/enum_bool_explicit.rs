use crate::models::blocksworld::models::enum_bool_explicit_parser::parser;
use super::*;

// macro_rules! new_enum_assign_c {
//     ($name:expr, $domain:expr, $val:expr) => { ... };
//     ($name:expr, $domain:expr, $val:expr, $r#type:expr) => { ... };
//     ($name:expr, $domain:expr, $val:expr, $r#type:expr, $param:expr) => { ... };
//     ($name:expr, $domain:expr, $val:expr, $r#type:expr, $param:expr, $life:expr) => { ... };
// }

/// Explicitly generating negative predicates from diff(ojb/init)
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

    let on = Parameter::new("on", &true);
    let clear = Parameter::new("clear", &true);
    let ontable = Parameter::new("ontable", &true);
    let hand = Parameter::new("hand", &true);
    let holding = Parameter::new("holding", &true);

    let problem = ParamPlanningProblem::new(
        &format!("blocksworld_enum_bool_invariants_{}", parsed.name.as_str()), 
        &parsed.init,
        &parsed.goal,
        &transitions,
        &Predicate::TRUE,
        &vec!(on, clear, ontable, hand, holding)
    );

    problem
}