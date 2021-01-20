use crate::models::barman::models::prop_explicit_parser::parser;
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

// (:action grasp
//     :parameters (?h - hand ?c - container)
//     :precondition (and (ontable ?c) (handempty ?h))
//     :effect (and (not (ontable ?c))
//                (not (handempty ?h))
//      (holding ?h ?c)
//      (increase (total-cost) 1)))

for hand in &hands {
    for container in &containers {
        transitions.push(
            ParamTransition::new(
                &format!("grasp_{}_{}", hand, container),
                &ppred!(
                    &pass!(&new_bool_assign_c!(&format!("ontable_{}", container), true, "c")),
                    &pass!(&new_bool_assign_c!(&format!("handempty_{}", hand), true, "c"))
                ),
                &ppred!(
                    &pass!(&new_bool_assign_c!(&format!("ontable_{}", container), false, "c")),
                    &pass!(&new_bool_assign_c!(&format!("handempty_{}", hand), false, "c")),
                    &pass!(&new_bool_assign_c!(&format!("holding_{}_{}", hand, container), true, "c"))
                )
            )
        )
    }
}

// (:action leave
//     :parameters (?h - hand ?c - container)
//     :precondition (holding ?h ?c)
//     :effect (and (not (holding ?h ?c))
//                (handempty ?h)
//      (ontable ?c)
//      (increase (total-cost) 1)))

for hand in &hands {
    for container in &containers {
        transitions.push(
            ParamTransition::new(
                &format!("leave_{}_{}", hand, container),
                &ppred!(
                    &pass!(&new_bool_assign_c!(&format!("holding_{}_{}", hand, container), true, "c"))
                ),
                &ppred!(
                    &pass!(&new_bool_assign_c!(&format!("ontable_{}", container), true, "c")),
                    &pass!(&new_bool_assign_c!(&format!("handempty_{}", hand), true, "c")),
                    &pass!(&new_bool_assign_c!(&format!("holding_{}_{}", hand, container), false, "c"))
                )
            )
        )
    }
}

// (:action fill-shot
//     :parameters (?s - shot ?i - ingredient ?h1 ?h2 - hand ?d - dispenser)
//     :precondition (and (holding ?h1 ?s)
//                        (handempty ?h2)
//               (dispenses ?d ?i)
//                        (empty ?s)
//            (clean ?s))
//     :effect (and (not (empty ?s))
//            (contains ?s ?i)
//            (not (clean ?s))
//      (used ?s ?i)
//      (increase (total-cost) 10)))

for shot in &shots {
    for ingredient in &ingredients {
        for hand1 in &hands {
            for hand2 in &hands {
                for dispenser in &dispensers {
                    if hand1 != hand2 {
                        transitions.push(
                            ParamTransition::new(
                                &format!("fill_shot_{}_{}_{}_{}_{}", shot, ingredient, hand1, hand2, dispenser),
                                &ppred!(
                                    &pass!(&new_bool_assign_c!(&format!("holding_{}_{}", hand1, shot), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("handempty_{}", hand2), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("dispenses_{}_{}", dispenser, ingredient), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("empty_{}", shot), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("clean_{}", shot), true, "c"))
                                ),
                                &ppred!(
                                    &pass!(&new_bool_assign_c!(&format!("empty_{}", shot), false, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("clean_{}", shot), false, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("contains_{}_{}", shot, ingredient), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("used_{}_{}", shot, ingredient), true, "c"))
                                )
                            )  
                        )
                    }
                }
            }
        }
    }
}

// (:action refill-shot
//     :parameters (?s - shot ?i - ingredient ?h1 ?h2 - hand ?d - dispenser)
//     :precondition (and (holding ?h1 ?s)	   		      
//                        (handempty ?h2)
//               (dispenses ?d ?i)
//                        (empty ?s)
//            (used ?s ?i))
//     :effect (and (not (empty ?s))
//                  (contains ?s ?i)
//      (increase (total-cost) 10)))

for shot in &shots {
    for ingredient in &ingredients {
        for hand1 in &hands {
            for hand2 in &hands {
                for dispenser in &dispensers {
                    if hand1 != hand2 {
                        transitions.push(
                            ParamTransition::new(
                                &format!("refill_shot_{}_{}_{}_{}_{}", shot, ingredient, hand1, hand2, dispenser),
                                &ppred!(
                                    &pass!(&new_bool_assign_c!(&format!("holding_{}_{}", hand1, shot), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("handempty_{}", hand2), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("dispenses_{}_{}", dispenser, ingredient), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("empty_{}", shot), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("used_{}_{}", shot, ingredient), true, "c"))
                                ),
                                &ppred!(
                                    &pass!(&new_bool_assign_c!(&format!("empty_{}", shot), false, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("contains_{}_{}", shot, ingredient), true, "c"))
                                )
                            )  
                        )
                    }
                }
            }
        }
    }
}

// (:action empty-shot
//     :parameters (?h - hand ?p - shot ?b - beverage)
//     :precondition (and (holding ?h ?p)
//                        (contains ?p ?b))
//     :effect (and (not (contains ?p ?b))
//            (empty ?p)
//      (increase (total-cost) 1)))

for hand in &hands {
    for shot in &shots {
        for beverage in &beverages {
            transitions.push(
                ParamTransition::new(
                    &format!("empty_shot_{}_{}_{}", hand, shot, beverage),
                    &ppred!(
                        &pass!(&new_bool_assign_c!(&format!("holding_{}_{}", hand, shot), true, "c")),
                        &pass!(&new_bool_assign_c!(&format!("contains_{}_{}", shot, beverage), true, "c"))
                    ),
                    &ppred!(
                        &pass!(&new_bool_assign_c!(&format!("empty_{}", shot), true, "c")),
                        &pass!(&new_bool_assign_c!(&format!("contains_{}_{}", shot, beverage), false, "c"))
                    )
                )  
            )
        }
    }
}

// (:action clean-shot
//     :parameters (?s - shot ?b - beverage ?h1 ?h2 - hand)
//       :precondition (and (holding ?h1 ?s)
//                          (handempty ?h2)	   		      
//              (empty ?s)
//                          (used ?s ?b))
//       :effect (and (not (used ?s ?b))
//              (clean ?s)
//        (increase (total-cost) 1)))

for shot in &shots {
    for beverage in &beverages {
        for hand1 in &hands {
            for hand2 in &hands {
                if hand1 != hand2 {
                    transitions.push(
                        ParamTransition::new(
                            &format!("clean_shot_{}_{}_{}_{}", shot, beverage, hand1, hand2),
                            &ppred!(
                                &pass!(&new_bool_assign_c!(&format!("holding_{}_{}", hand1, shot), true, "c")),
                                &pass!(&new_bool_assign_c!(&format!("handempty_{}", hand2), true, "c")),
                                &pass!(&new_bool_assign_c!(&format!("empty_{}", shot), true, "c")),
                                &pass!(&new_bool_assign_c!(&format!("used_{}_{}", shot, beverage), true, "c"))
                            ),
                            &ppred!(
                                &pass!(&new_bool_assign_c!(&format!("used_{}_{}", shot, beverage), false, "c")),
                                &pass!(&new_bool_assign_c!(&format!("clean_{}", shot), true, "c"))
                            )
                        )  
                    )
                }
            }
        }
    }
}

// (:action pour-shot-to-clean-shaker
//     :parameters (?s - shot ?i - ingredient ?d - shaker ?h1 - hand ?l ?l1 - level)
//     :precondition (and (holding ?h1 ?s)
//            (contains ?s ?i)
//                        (empty ?d)
//               (clean ?d)                              
//                        (shaker-level ?d ?l)
//                        (next ?l ?l1))
//     :effect (and (not (contains ?s ?i))
//            (empty ?s)
//      (contains ?d ?i)
//                  (not (empty ?d))
//      (not (clean ?d))
//      (unshaked ?d)
//      (not (shaker-level ?d ?l))
//      (shaker-level ?d ?l1)
//      (increase (total-cost) 1)))

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
                                    &pass!(&new_bool_assign_c!(&format!("holding_{}_{}", hand, shot), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("contains_{}_{}", shot, ingredient), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("empty_{}", shaker), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("clean_{}", shaker), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("shaker_level_{}_{}", shaker, level1), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("next_{}_{}", level1, level2), true, "c"))
                                ),
                                &ppred!(
                                    &pass!(&new_bool_assign_c!(&format!("contains_{}_{}", shot, ingredient), false, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("empty_{}", shot), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("contains_{}_{}", shaker, ingredient), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("empty_{}", shaker), false, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("clean_{}", shaker), false, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("unshaked_{}", shaker), true, "c")),
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

// (:action pour-shot-to-used-shaker
//     :parameters (?s - shot ?i - ingredient ?d - shaker ?h1 - hand ?l ?l1 - level)
//     :precondition (and (holding ?h1 ?s)
//            (contains ?s ?i)
//                        (unshaked ?d)
//                        (shaker-level ?d ?l)
//                        (next ?l ?l1))
//     :effect (and (not (contains ?s ?i))
//                  (contains ?d ?i)
//            (empty ?s)     
//        (not (shaker-level ?d ?l))
//      (shaker-level ?d ?l1)
//      (increase (total-cost) 1)))

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
                                    &pass!(&new_bool_assign_c!(&format!("holding_{}_{}", hand, shot), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("contains_{}_{}", shot, ingredient), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("unshaked_{}", shaker), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("shaker_level_{}_{}", shaker, level1), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("next_{}_{}", level1, level2), true, "c"))
                                ),
                                &ppred!(
                                    &pass!(&new_bool_assign_c!(&format!("contains_{}_{}", shot, ingredient), false, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("empty_{}", shot), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("contains_{}_{}", shaker, ingredient), true, "c")),
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

// (:action empty-shaker
//     :parameters (?h - hand ?s - shaker ?b - cocktail ?l ?l1 - level)
//     :precondition (and (holding ?h ?s)
//                        (contains ?s ?b)
//            (shaked ?s)
//            (shaker-level ?s ?l)
//            (shaker-empty-level ?s ?l1))
//     :effect (and (not (shaked ?s))
//            (not (shaker-level ?s ?l))
//            (shaker-level ?s ?l1)
//      (not (contains ?s ?b))
//            (empty ?s)
//      (increase (total-cost) 1)))

for hand in &hands {
    for shaker in &shakers {
        for cocktail in &cocktails {
            for level1 in &levels {
                for level2 in &levels {
                    transitions.push(
                        ParamTransition::new(
                            &format!("empty_shaker_{}_{}_{}_{}_{}", hand, shaker, cocktail, level1, level2),
                            &ppred!(
                                &pass!(&new_bool_assign_c!(&format!("holding_{}_{}", hand, shaker), true, "c")),
                                &pass!(&new_bool_assign_c!(&format!("contains_{}_{}", shaker, cocktail), true, "c")),
                                &pass!(&new_bool_assign_c!(&format!("shaked_{}", shaker), true, "c")),
                                &pass!(&new_bool_assign_c!(&format!("shaker_level_{}_{}", shaker, level1), true, "c")),
                                &pass!(&new_bool_assign_c!(&format!("shaker_empty_level_{}_{}", shaker, level2), true, "c"))
                            ),
                            &ppred!(
                                &pass!(&new_bool_assign_c!(&format!("shaked_{}", shaker), false, "c")),
                                &pass!(&new_bool_assign_c!(&format!("shaker_level_{}_{}", shaker, level1), false, "c")),
                                &pass!(&new_bool_assign_c!(&format!("shaker_level_{}_{}", shaker, level2), true, "c")),
                                &pass!(&new_bool_assign_c!(&format!("contains_{}_{}", shaker, cocktail), false, "c")),
                                &pass!(&new_bool_assign_c!(&format!("empty_{}", shaker), true, "c"))
                            )
                        )
                    )
                }
            }
        }
    }
}

// (:action clean-shaker
//     :parameters (?h1 ?h2 - hand ?s - shaker)
//       :precondition (and (holding ?h1 ?s)
//                          (handempty ?h2)
//                          (empty ?s))
//       :effect (and (clean ?s)
//        (increase (total-cost) 1)))

for hand1 in &hands {
    for hand2 in &hands {
        for shaker in &shakers {
            if hand1 != hand2 {
                transitions.push(
                    ParamTransition::new(
                        &format!("clean_shaker_{}_{}_{}", hand1, hand2, shaker),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("holding_{}_{}", hand1, shaker), true, "c")),
                            &pass!(&new_bool_assign_c!(&format!("handempty_{}", hand2), true, "c")),
                            &pass!(&new_bool_assign_c!(&format!("empty_{}", shaker), true, "c"))
                        ),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("clean_{}", shaker), true, "c"))
                        )
                    )
                )
            }
        }
    }
}

// (:action shake
//     :parameters (?b - cocktail ?d1 ?d2 - ingredient ?s - shaker ?h1 ?h2 - hand)
//       :precondition (and (holding ?h1 ?s)
//                          (handempty ?h2)
//              (contains ?s ?d1)
//                          (contains ?s ?d2)
//                          (cocktail-part1 ?b ?d1)
//              (cocktail-part2 ?b ?d2)
//              (unshaked ?s))			      
//       :effect (and (not (unshaked ?s))
//            (not (contains ?s ?d1))
//                    (not (contains ?s ?d2))
//              (shaked ?s)
//                    (contains ?s ?b)
//        (increase (total-cost) 1)))

for cocktail in &cocktails {
    for ingredient1 in &ingredients {
        for ingredient2 in &ingredients {
            for shaker in &shakers {
                for hand1 in &hands {
                    for hand2 in &hands {
                        if hand1 != hand2 {
                            transitions.push(
                                ParamTransition::new(
                                    &format!("shake_{}_{}_{}_{}_{}_{}", cocktail, ingredient1, ingredient2, shaker, hand1, hand2),
                                    &ppred!(
                                        &pass!(&new_bool_assign_c!(&format!("holding_{}_{}", hand1, shaker), true, "c")),
                                        &pass!(&new_bool_assign_c!(&format!("handempty_{}", hand2), true, "c")),
                                        &pass!(&new_bool_assign_c!(&format!("contains_{}_{}", shaker, ingredient1), true, "c")),
                                        &pass!(&new_bool_assign_c!(&format!("contains_{}_{}", shaker, ingredient2), true, "c")),
                                        &pass!(&new_bool_assign_c!(&format!("cocktail_part1_{}_{}", cocktail, ingredient1), true, "c")),
                                        &pass!(&new_bool_assign_c!(&format!("cocktail_part2_{}_{}", cocktail, ingredient2), true, "c")),
                                        &pass!(&new_bool_assign_c!(&format!("unshaked_{}", shaker), true, "c"))
                                    ),
                                    &ppred!(
                                        &pass!(&new_bool_assign_c!(&format!("unshaked_{}", shaker), false, "c")),
                                        &pass!(&new_bool_assign_c!(&format!("contains_{}_{}", shaker, ingredient1), false, "c")),
                                        &pass!(&new_bool_assign_c!(&format!("contains_{}_{}", shaker, ingredient2), false, "c")),
                                        &pass!(&new_bool_assign_c!(&format!("shaked_{}", shaker), true, "c")),
                                        &pass!(&new_bool_assign_c!(&format!("contains_{}_{}", shaker, cocktail), true, "c"))
                                    )
                                )
                            )
                        }
                    }
                }
            }
        }
    }
}

// (:action pour-shaker-to-shot
//     :parameters (?b - beverage ?d - shot ?h - hand ?s - shaker ?l ?l1 - level)
//     :precondition (and (holding ?h ?s)
//            (shaked ?s)
//            (empty ?d)
//            (clean ?d)
//            (contains ?s ?b)
//                        (shaker-level ?s ?l)
//                        (next ?l1 ?l))
//     :effect (and (not (clean ?d))
//            (not (empty ?d))
//      (contains ?d ?b)
//      (shaker-level ?s ?l1)
//      (not (shaker-level ?s ?l))
//      (increase (total-cost) 1)))
// )

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
                                    &pass!(&new_bool_assign_c!(&format!("holding_{}_{}", hand, shaker), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("shaked_{}", shaker), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("empty_{}", shot), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("clean_{}", shot), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("contains_{}_{}", shaker, beverage), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("shaker_level_{}_{}", shaker, level1), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("next_{}_{}", level2, level1), true, "c"))
                                ),
                                &ppred!(
                                    &pass!(&new_bool_assign_c!(&format!("empty_{}", shot), false, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("clean_{}", shot), false, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("contains_{}_{}", shot, beverage), true, "c")),
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


    let c = Parameter::new("c", &true);

    let problem = ParamPlanningProblem::new(
        &format!("barman_bool_explicit_{}", parsed.name.as_str()), 
        &parsed.init,
        &parsed.goal,
        &transitions,
        &Predicate::TRUE,
        &vec!(c)
    );

    problem
}