use crate::models::barman::models::eq_invariant_parser::parser;
use super::*;

//     ($name:expr, $domain:expr, $val:expr, $r#type:expr, $param:expr) => { ... };

pub fn model(name: &str) -> ParamPlanningProblem {

    let (parsed, objects) = parser(name);

    let mut transitions = vec![];

    let shots = vec!("shot1", "shot2");
    let ingredients = vec!("ingredient1", "ingredient2");
    let containers = vec!("shot1", "shot2", "shaker1");
    let hands = vec!("left", "right");

    let pos_domain = vec!("left", "right", "table");
    let shot_contains_domain = vec!("a", "s", "d", "f");
    
    for hand in &hands {
        for container in &containers {
            transitions.push(
                ParamTransition::new(
                    &format!("grasp_{}_{}", hand, container),
                    &ppred!(
                        &pass!(&new_enum_assign_c!(&format!("pos_{}", container), &pos_domain, &format!("table"), "c", "c"))
                    ),
                    &ppred!(
                        &pass!(&new_enum_assign_c!(&format!("pos_{}", container), &pos_domain, &format!("{}", hand), "c", "c"))
                    )
                )
            )
        }
    }

    for hand in &hands {
        for container in &containers {
            transitions.push(
                ParamTransition::new(
                    &format!("leave_{}_{}", hand, container),
                    &ppred!(
                        &pass!(&new_enum_assign_c!(&format!("pos_{}", container), &pos_domain, &format!("{}", hand), "c", "c"))
                    ),
                    &ppred!(
                        &pass!(&new_enum_assign_c!(&format!("pos_{}", container), &pos_domain, &format!("table"), "c", "c"))
                    )
                )
            )
        }
    }
    
    for shot in &shots {
        for ingredient in &ingredients {
            for hand in &hands {
                transitions.push(
                    ParamTransition::new(
                        &format!("fill_shot_{}_{}_{}", shot, ingredient, hand),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("pos_{}", shot), &pos_domain, &format!("{}", hand), "c", "c")),
                            &pass!(&new_enum_assign_c!(&format!("state_{}", shot), &shot_contains_domain, &format!("a"), "d", "c"))
                        ),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("state_{}", shot), &shot_contains_domain, &format!("f"), "d", "c"))
                        )
                    )  
                )
            }
        }
    }

    let initial = vec!(
        pass!(&new_enum_assign_c!(&format!("pos_shaker1"), &pos_domain, &format!("table"), "c", "c")),
        pass!(&new_enum_assign_c!(&format!("pos_shot1"), &pos_domain, &format!("table"), "c", "c")),
        pass!(&new_enum_assign_c!(&format!("pos_shot2"), &pos_domain, &format!("table"), "c", "c")),
        pass!(&new_enum_assign_c!(&format!("state_shot1"), &shot_contains_domain, &format!("a"), "d", "c")),

    );

    let goal = vec!(
        pass!(&new_enum_assign_c!(&format!("state_shot1"), &shot_contains_domain, &format!("f"), "d", "c")),
        pass!(&new_enum_assign_c!(&format!("pos_shaker1"), &pos_domain, &format!("right"), "c", "c")),
    );

    let c = Parameter::new("c", &true);

    let problem = ParamPlanningProblem::new(
        &format!("barman_prop_invariant_{}", parsed.name.as_str()), 
        &ParamPredicate::new(&initial), 
        &ParamPredicate::new(&goal), 
        &transitions,
        // &Predicate::AND(invariants),
        &Predicate::TRUE,
        &vec!(c)
    );

    problem
}