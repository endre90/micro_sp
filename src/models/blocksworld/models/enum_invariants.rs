use crate::models::blocksworld::models::enum_invariants_parser::parser;
use super::*;

// macro_rules! new_enum_assign_c {
//     ($name:expr, $domain:expr, $val:expr) => { ... };
//     ($name:expr, $domain:expr, $val:expr, $r#type:expr) => { ... };
//     ($name:expr, $domain:expr, $val:expr, $r#type:expr, $param:expr) => { ... };
//     ($name:expr, $domain:expr, $val:expr, $r#type:expr, $param:expr, $life:expr) => { ... };
// }

/// Invariants:
/// 1. at most one block can be held
/// 2. if b1 is on b2, b2 is not clear
pub fn model(name: &str) -> ParamPlanningProblem {

    let (parsed, blocks) = parser(name);
    let on_domain_init = blocks.iter().chain(vec!(String::from("GRIP"), String::from("TABLE")).iter()).cloned().collect::<Vec<_>>();
    let on_domain: Vec<&str> = on_domain_init.iter().map(|x| x.as_str()).collect();

    let mut pick_up_transitions = vec![];
    let mut put_down_transitions = vec![];
    let mut stack_transitions = vec![];
    let mut unstack_transitions = vec![];

    for block in &blocks {
        pick_up_transitions.push(
            ParamTransition::new(
                &format!("pick_up_{}", block),
                &ppred!(
                    &pass!(&new_bool_assign_c!(&format!("clear_{}", block), true, "clear"))
                ),
                &ppred!(
                    &pass!(&new_bool_assign_c!(&format!("clear_{}", block), false, "clear")),
                    &pass!(&new_enum_assign_c!(&format!("{}_on", block), &on_domain, "GRIP", "on", "on"))
                )
            )
        )
    }

    for block in &blocks {
        put_down_transitions.push(
            ParamTransition::new(
                &format!("put_down_{}", block),
                &ppred!(
                    &pass!(&new_enum_assign_c!(&format!("{}_on", block), &on_domain, "GRIP", "on", "on"))
                ),
                &ppred!(
                    &pass!(&new_enum_assign_c!(&format!("{}_on", block), &on_domain, "TABLE", "on", "on")),
                    &pass!(&new_bool_assign_c!(&format!("clear_{}", block), true, "clear"))
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
                            &pass!(&new_enum_assign_c!(&format!("{}_on", b1), &on_domain, "GRIP", "on", "on"))
                        ),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("clear_{}", b2), false, "clear")),
                            &pass!(&new_bool_assign_c!(&format!("clear_{}", b1), true, "clear")),
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
                unstack_transitions.push(
                    ParamTransition::new(
                        &format!("unstack_{}_from_{}", b1, b2),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("clear_{}", b1), true, "clear")),
                            &pass!(&new_enum_assign_c!(&format!("{}_on", b1), &on_domain, &format!("{}", b2), "on", "on"))
                        ),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("clear_{}", b2), true, "clear")),
                            &pass!(&new_bool_assign_c!(&format!("clear_{}", b1), false, "clear")),
                            &pass!(&new_enum_assign_c!(&format!("{}_on", b1), &on_domain, "GRIP", "on", "on"))
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
    let mut holding = vec![];

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

     // Invariant 6: if b1 is on b2, b2 is not clear
     for b1 in &blocks {
        for b2 in &blocks {
            if b1 != b2 {
                invariants.push(
                    pnot!(
                        &pand!(
                            &pass!(&new_enum_assign_c!(&format!("{}_on", b1), &on_domain, &format!("{}", b2), "on", "on")),
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
        parsed.name.as_str(), 
        &parsed.init,
        &parsed.goal,
        &transitions,
        &Predicate::AND(invariants),
        &vec!(on, clear, ontable, hand, holding)
    );

    problem
}