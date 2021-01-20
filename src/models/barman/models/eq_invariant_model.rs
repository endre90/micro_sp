use crate::models::barman::models::eq_invariant_parser::parser;
use super::*;

// macro_rules! new_bool_assign_c {
//     ($name:expr, $domain:expr, $val:expr) => { ... };
//     ($name:expr, $domain:expr, $val:expr, $r#type:expr) => { ... };
//     ($name:expr, $domain:expr, $val:expr, $r#type:expr, $param:expr) => { ... };
//     ($name:expr, $domain:expr, $val:expr, $r#type:expr, $param:expr, $life:expr) => { ... };
// }

#[allow(dead_code)]
pub fn model(name: &str) -> ParamPlanningProblem {

    let (parsed, objects) = parser(name);

    let mut transitions = vec![];

    let hands = objects.get("hand").unwrap_or(&vec!()).to_vec();
    let levels = objects.get("level").unwrap_or(&vec!()).to_vec();
    let dispensers = objects.get("dispenser").unwrap_or(&vec!()).to_vec();
    let ingredients = objects.get("ingredient").unwrap_or(&vec!()).to_vec();
    let cocktails = objects.get("cocktail").unwrap_or(&vec!()).to_vec();
    let shots = objects.get("shot").unwrap_or(&vec!()).to_vec();
    let shakers = objects.get("shaker").unwrap_or(&vec!()).to_vec();
    let beverages = objects.get("beverage").unwrap_or(&vec!()).to_vec();
    let containers = objects.get("container").unwrap_or(&vec!()).to_vec();

    for o in &objects {
        println!("{:?}", o)
    }

    let pos_domain = vec!("left", "right", "table");

    let mut state_domain: Vec<&str> = vec!();
    let clean = vec!("clean");
    let empty = beverages.iter().map(|x| format!("empty_{}", x)).collect::<Vec<String>>();
    let contains = beverages.iter().map(|x| format!("contains_{}", x)).collect::<Vec<String>>();
    let mut contains_mix = vec!();
    for ingredient1 in &ingredients {
        for ingredient2 in &ingredients {
            contains_mix.push(
                format!("contains_{}_{}", ingredient1, ingredient2)
            )
        }
    }
    state_domain.extend(clean);
    state_domain.extend(empty.iter().map(|x| x.as_str()).collect::<Vec<&str>>());
    state_domain.extend(contains.iter().map(|x| x.as_str()).collect::<Vec<&str>>());
    state_domain.extend(contains_mix.iter().map(|x| x.as_str()).collect::<Vec<&str>>());
    
    // works
    for hand in &hands {
        for container in &containers {
            transitions.push(
                ParamTransition::new(
                    &format!("grasp_{}_{}", hand, container),
                    &ppred!(
                        &pass!(&new_enum_assign_c!(&format!("pos_{}", container), &pos_domain, &format!("table"), "pos", "c"))
                    ),
                    &ppred!(
                        &pass!(&new_enum_assign_c!(&format!("pos_{}", container), &pos_domain, &format!("{}", hand), "pos", "c"))
                    )
                )
            )
        }
    }
    // works
    for hand in &hands {
        for container in &containers {
            transitions.push(
                ParamTransition::new(
                    &format!("leave_{}_{}", hand, container),
                    &ppred!(
                        &pass!(&new_enum_assign_c!(&format!("pos_{}", container), &pos_domain, &format!("{}", hand), "pos", "c"))
                    ),
                    &ppred!(
                        &pass!(&new_enum_assign_c!(&format!("pos_{}", container), &pos_domain, &format!("table"), "pos", "c"))
                    )
                )
            )
        }
    }
    
    for shot in &shots {
        for ingredient in &ingredients {
            for hand in &hands {
                for dispenser in &dispensers {
                    transitions.push(
                        ParamTransition::new(
                            &format!("fill_shot_{}_{}_{}_{}", shot, ingredient, hand, dispenser),
                            &ppred!(
                                &pass!(&new_enum_assign_c!(&format!("pos_{}", shot), &pos_domain, &format!("{}", hand), "pos", "c")),
                                &pass!(&new_bool_assign_c!(&format!("dispenses_{}_{}", dispenser, ingredient), true, "c")),
                                &pass!(&new_enum_assign_c!(&format!("state_{}", shot), &state_domain, &format!("clean"), "state", "c"))
                            ),
                            &ppred!(
                                &pass!(&new_enum_assign_c!(&format!("state_{}", shot), &state_domain, &format!("contains_{}", ingredient), "state", "c"))
                            )
                        )  
                    )
                }
            }
        }
    }
    
    for shot in &shots {
        for ingredient in &ingredients {
            for hand in &hands {
                for dispenser in &dispensers {
                    transitions.push(
                        ParamTransition::new(
                            &format!("refill_shot_{}_{}_{}_{}", shot, ingredient, hand, dispenser),
                            &ppred!(
                                &pass!(&new_enum_assign_c!(&format!("pos_{}", shot), &pos_domain, &format!("{}", hand), "pos", "c")),
                                &pass!(&new_bool_assign_c!(&format!("dispenses_{}_{}", dispenser, ingredient), true, "c")),
                                &pass!(&new_enum_assign_c!(&format!("state_{}", shot), &state_domain, &format!("empty_{}", ingredient), "state", "c"))
                            ),
                            &ppred!(
                                &pass!(&new_enum_assign_c!(&format!("state_{}", shot), &state_domain, &format!("contains_{}", ingredient), "state", "c"))
                            )
                        )  
                    )
                }
            }
        }
    }
    
    for hand in &hands {
        for shot in &shots {
            for beverage in &beverages {
                transitions.push(
                    ParamTransition::new(
                        &format!("empty_shot_{}_{}_{}", hand, shot, beverage),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("pos_{}", shot), &pos_domain, &format!("{}", hand), "pos", "c")),
                            &pass!(&new_enum_assign_c!(&format!("state_{}", shot), &state_domain, &format!("contains_{}", beverage), "state", "c"))
                        ),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("state_{}", shot), &state_domain, &format!("empty_{}", beverage), "state", "c"))
                        )
                    )  
                )
            }
        }
    }
    
    for shot in &shots {
        for beverage in &beverages {
            for hand in &hands {
                transitions.push(
                    ParamTransition::new(
                        &format!("clean_shot_{}_{}_{}", shot, beverage, hand),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("pos_{}", shot), &pos_domain, &format!("{}", hand), "pos", "c")),
                            &pass!(&new_enum_assign_c!(&format!("state_{}", shot), &state_domain, &format!("empty_{}", beverage), "state", "c"))
                        ),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("state_{}", shot), &state_domain, &format!("clean"), "state", "c"))
                        )
                    )  
                )
            }
        }
    }
    
    for shot in &shots {
        for ingredient in &ingredients {
            for shaker in &shakers {
                for hand in &hands {
                    for level1 in &levels {
                        for level2 in &levels {
                            transitions.push(
                                ParamTransition::new(
                                    &format!("pour_shot_to_clean_shaker_{}_{}_{}_{}_{}_{}", shot, ingredient, shaker, hand, level1, level2),
                                    &ppred!(
                                        &pass!(&new_enum_assign_c!(&format!("pos_{}", shot), &pos_domain, &format!("{}", hand), "pos", "c")),
                                        &pass!(&new_enum_assign_c!(&format!("state_{}", shot), &state_domain, &format!("contains_{}", ingredient), "state", "c")),
                                        &pass!(&new_enum_assign_c!(&format!("state_{}", shot), &state_domain, &format!("clean"), "state", "c")),
                                        &pass!(&new_bool_assign_c!(&format!("shaker_level_{}_{}", shaker, level1), true, "c")),
                                        &pass!(&new_bool_assign_c!(&format!("next_{}_{}", level1, level2), true, "c"))
                                    ),
                                    &ppred!(
                                        &pass!(&new_enum_assign_c!(&format!("state_{}", shot), &state_domain, &format!("empty_{}", ingredient), "state", "c")),
                                        &pass!(&new_enum_assign_c!(&format!("state_{}", shaker), &state_domain, &format!("contains_{}", ingredient), "state", "c")),
                                        &pass!(&new_bool_assign_c!(&format!("shaked_{}", shaker), false, "c")),
                                        &pass!(&new_bool_assign_c!(&format!("shaker_level_{}_{}", shaker, level1), false, "c")),
                                        &pass!(&new_bool_assign_c!(&format!("shaker_level_{}_{}", shaker, level2), true, "c"))
                                    )
                                )  
                            )
                        }
                    }
                }
            }
        }
    }
    
    for shot in &shots {
        for ingredient in &ingredients {
            for shaker in &shakers {
                for hand in &hands {
                    for level1 in &levels {
                        for level2 in &levels {
                            transitions.push(
                                ParamTransition::new(
                                    &format!("pour_shot_to_used_shaker_{}_{}_{}_{}_{}_{}", shot, ingredient, shaker, hand, level1, level2),
                                    &ppred!(
                                        &pass!(&new_enum_assign_c!(&format!("pos_{}", shot), &pos_domain, &format!("{}", hand), "pos", "c")),
                                        &pass!(&new_enum_assign_c!(&format!("state_{}", shot), &state_domain, &format!("contains_{}", ingredient), "state", "c")),
                                        &pass!(&new_bool_assign_c!(&format!("shaked_{}", shaker), false, "c")),
                                        &pass!(&new_bool_assign_c!(&format!("shaker_level_{}_{}", shaker, level1), true, "c")),
                                        &pass!(&new_bool_assign_c!(&format!("next_{}_{}", level1, level2), true, "c"))
                                    ),
                                    &ppred!(
                                        &pass!(&new_enum_assign_c!(&format!("state_{}", shot), &state_domain, &format!("empty_{}", ingredient), "state", "c")),
                                        &pass!(&new_enum_assign_c!(&format!("state_{}", shaker), &state_domain, &format!("contains_{}", ingredient), "state", "c")),
                                        &pass!(&new_bool_assign_c!(&format!("shaker_level_{}_{}", shaker, level1), false, "c")),
                                        &pass!(&new_bool_assign_c!(&format!("shaker_level_{}_{}", shaker, level2), true, "c"))
                                    )
                                )  
                            )
                        }
                    }
                }
            }
        }
    }
    
    for hand in &hands {
        for shaker in &shakers {
            for cocktail in &cocktails {
                for level1 in &levels {
                    for level2 in &levels {
                        transitions.push(
                            ParamTransition::new(
                                &format!("empty_shaker_{}_{}_{}_{}_{}", hand, shaker, cocktail, level1, level2),
                                &ppred!(
                                    &pass!(&new_enum_assign_c!(&format!("pos_{}", shaker), &pos_domain, &format!("{}", hand), "pos", "c")),
                                    &pass!(&new_enum_assign_c!(&format!("state_{}", shaker), &state_domain, &format!("contains_{}", cocktail), "state", "c")),
                                    &pass!(&new_bool_assign_c!(&format!("shaked_{}", shaker), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("shaker_level_{}_{}", shaker, level1), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("shaker_empty_level_{}_{}", shaker, level2), true, "c"))
                                ),
                                &ppred!(
                                    &pass!(&new_bool_assign_c!(&format!("shaked_{}", shaker), false, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("shaker_level_{}_{}", shaker, level1), false, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("shaker_level_{}_{}", shaker, level2), true, "c")),
                                    &pass!(&new_enum_assign_c!(&format!("state_{}", shaker), &state_domain, &format!("empty_{}", cocktail), "state", "c"))
                                )
                            )
                        )
                    }
                }
            }
        }
    }
    
    for hand in &hands {
        for shaker in &shakers {
            for beverage in &beverages {
                transitions.push(
                    ParamTransition::new(
                        &format!("clean_shaker_{}_{}", hand, shaker),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("pos_{}", shaker), &pos_domain, &format!("{}", hand), "pos", "c")),
                            &pass!(&new_enum_assign_c!(&format!("state_{}", shaker), &state_domain, &format!("empty_{}", beverage), "state", "c"))
                        ),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("state_{}", shaker), &state_domain, &format!("clean"), "state", "c"))
                        )
                    )
                )
            }
        }
    }
    
    for cocktail in &cocktails {
        for ingredient1 in &ingredients {
            for ingredient2 in &ingredients {
                for shaker in &shakers {
                    for hand in &hands {
                        transitions.push(
                            ParamTransition::new(
                                &format!("shake_{}_{}_{}_{}_{}", cocktail, ingredient1, ingredient2, shaker, hand),
                                &ppred!(
                                    &pass!(&new_enum_assign_c!(&format!("pos_{}", shaker), &pos_domain, &format!("{}", hand), "pos", "c")),
                                    &pass!(&new_enum_assign_c!(&format!("state_{}", shaker), &state_domain, &format!("contains_{}_{}", ingredient1, ingredient2), "state", "c")),
                                    &pass!(&new_bool_assign_c!(&format!("cocktail_part1_{}_{}", cocktail, ingredient1), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("cocktail_part2_{}_{}", cocktail, ingredient2), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("shaked_{}", shaker), false, "c"))
                                ),
                                &ppred!(
                                    &pass!(&new_bool_assign_c!(&format!("shaked_{}", shaker), true, "c")),
                                    &pass!(&new_enum_assign_c!(&format!("state_{}", shaker), &state_domain, &format!("contains_{}", cocktail), "state", "c"))
                                )
                            )
                        )
                    }
                }
            }
        }
    }
    
    for beverage in &beverages {
        for shot in &shots {
            for hand in &hands {
                for shaker in &shakers {
                    for level1 in &levels {
                        for level2 in &levels {
                            transitions.push(
                                ParamTransition::new(
                                    &format!("pour_shaker_to_shot_{}_{}_{}_{}_{}_{}", beverage, shot, hand, shaker, level1, level2),
                                    &ppred!(
                                        &pass!(&new_enum_assign_c!(&format!("pos_{}", shaker), &pos_domain, &format!("{}", hand), "pos", "c")),
                                        &pass!(&new_bool_assign_c!(&format!("shaked_{}", shaker), true, "c")),
                                        &pass!(&new_enum_assign_c!(&format!("state_{}", shot), &state_domain, &format!("clean"), "state", "c")),
                                        &pass!(&new_enum_assign_c!(&format!("state_{}", shaker), &state_domain, &format!("contains_{}", beverage), "state", "c")),
                                        &pass!(&new_bool_assign_c!(&format!("shaker_level_{}_{}", shaker, level1), true, "c")),
                                        &pass!(&new_bool_assign_c!(&format!("next_{}_{}", level2, level1), true, "c"))
                                    ),
                                    &ppred!(
                                        &pass!(&new_enum_assign_c!(&format!("state_{}", shot), &state_domain, &format!("contains_{}", beverage), "state", "c")),
                                        &pass!(&new_bool_assign_c!(&format!("shaker_level_{}_{}", shaker, level1), false, "c")),
                                        &pass!(&new_bool_assign_c!(&format!("shaker_level_{}_{}", shaker, level2), true, "c"))
                                    )
                                )
                            )
                        }
                    }
                }
            }
        }
    }
    
    // let mut invariants = vec!();    

    // // WILL NEED invariant for ton more than one container in one hand
    // // one hand has to be emty when filling or refilling
    // // one hand has to be empty when cleaning a container
    // // one hand has to be empty when shaking a shaker



    let c = Parameter::new("c", &true);

    let problem = ParamPlanningProblem::new(
        &format!("barman_eq_invariant_{}", parsed.name.as_str()), 
        &parsed.init,
        &parsed.goal,
        &transitions,
        // &Predicate::AND(invariants),
        &Predicate::TRUE,
        &vec!(c)
    );

    problem
}