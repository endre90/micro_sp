use crate::models::childsnack::models::eq_invariant_parser::parser;
use super::*;

#[allow(dead_code)]
pub fn model(name: &str) -> ParamPlanningProblem {

    let (parsed, objects) = parser(name);

    let mut transitions = vec![];

    let children = objects.get("child").unwrap_or(&vec!()).to_vec();
    let bread_portions = objects.get("bread_portion").unwrap_or(&vec!()).to_vec();
    let content_portions = objects.get("content_portion").unwrap_or(&vec!()).to_vec();
    let trays = objects.get("tray").unwrap_or(&vec!()).to_vec();
    let places = objects.get("place").unwrap_or(&vec!()).to_vec();
    let sandwiches = objects.get("sandwich").unwrap_or(&vec!()).to_vec();

    let sandwich_domain = objects.get("sandwich_domain").unwrap_or(&vec!()).to_vec();
    let tray_domain = objects.get("tray_domain").unwrap_or(&vec!()).to_vec();
    let child_domain = objects.get("child_domain").unwrap_or(&vec!()).to_vec();
    let tf_domain = objects.get("tf_domain").unwrap_or(&vec!()).to_vec();

    // for sandwich in &sandwiches {
    //     for bread_portion in &bread_portions {
    //         for content_portion in &content_portions {
    //             transitions.push(
    //                 ParamTransition::new(
    //                     &format!("make_sandwich_no_gluten_{}_{}_{}", sandwich, bread_portion, content_portion),
    //                     &ppred!(
    //                         &pass!(&new_bool_assign_c!(&format!("at_kitchen_bread_{}", bread_portion), true, "c")),
    //                         &pass!(&new_bool_assign_c!(&format!("at_kitchen_content_{}", content_portion), true, "c")),
    //                         &pass!(&new_bool_assign_c!(&format!("no_gluten_bread_{}", bread_portion), true, "c")),
    //                         &pass!(&new_bool_assign_c!(&format!("no_gluten_content_{}", content_portion), true, "c")),
    //                         &pass!(&new_bool_assign_c!(&format!("notexist_{}", sandwich), true, "c"))
    //                     ),
    //                     &ppred!(
    //                         &pass!(&new_bool_assign_c!(&format!("at_kitchen_bread_{}", bread_portion), false, "c")),
    //                         &pass!(&new_bool_assign_c!(&format!("at_kitchen_content_{}", content_portion), false, "c")),
    //                         &pass!(&new_bool_assign_c!(&format!("at_kitchen_sandwich_{}", sandwich), true, "c")),
    //                         &pass!(&new_bool_assign_c!(&format!("no_gluten_sandwich_{}", sandwich), true, "c")),
    //                         &pass!(&new_bool_assign_c!(&format!("notexist_{}", sandwich), false, "c"))
    //                     )
    //                 )
    //             )
    //         }
    //     }
    // }

    for sandwich in &sandwiches {
        for bread_portion in &bread_portions {
            for content_portion in &content_portions {
                transitions.push(
                    ParamTransition::new(
                        &format!("make_sandwich_no_gluten_{}_{}_{}", sandwich, bread_portion, content_portion),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("at_kitchen_bread_{}", bread_portion), &tf_domain, "true", "tf", "c")),
                            &pass!(&new_enum_assign_c!(&format!("at_kitchen_content_{}", content_portion), &tf_domain, "true", "tf", "c")),
                            &pass!(&new_enum_assign_c!(&format!("no_gluten_bread_{}", bread_portion), &tf_domain, "true", "tf", "c")),
                            &pass!(&new_enum_assign_c!(&format!("no_gluten_content_{}", content_portion), &tf_domain, "true", "tf", "c")),
                            &pass!(&new_enum_assign_c!(&format!("{}", sandwich), &sandwich_domain, "notexist", "sandwich", "c"))
                        ),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("at_kitchen_bread_{}", bread_portion), &tf_domain, "false", "tf", "c")),
                            &pass!(&new_enum_assign_c!(&format!("at_kitchen_content_{}", content_portion), &tf_domain, "false", "tf", "c")),
                            &pass!(&new_enum_assign_c!(&format!("no_gluten_sandwich_{}", sandwich), &tf_domain, "true", "tf", "c")),
                            &pass!(&new_enum_assign_c!(&format!("{}", sandwich), &sandwich_domain, "kitchen", "sandwich", "c"))
                        )
                    )
                )
            }
        }
    }


    for sandwich in &sandwiches {
        for bread_portion in &bread_portions {
            for content_portion in &content_portions {
                transitions.push(
                    ParamTransition::new(
                        &format!("make_sandwich_{}_{}_{}", sandwich, bread_portion, content_portion),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("at_kitchen_bread_{}", bread_portion), &tf_domain, "true", "tf", "c")),
                            &pass!(&new_enum_assign_c!(&format!("at_kitchen_content_{}", content_portion), &tf_domain, "true", "tf", "c")),
                            &pass!(&new_enum_assign_c!(&format!("{}", sandwich), &sandwich_domain, "notexist", "sandwich", "c"))
                        ),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("at_kitchen_bread_{}", bread_portion), &tf_domain, "false", "tf", "c")),
                            &pass!(&new_enum_assign_c!(&format!("at_kitchen_content_{}", content_portion), &tf_domain, "false", "tf", "c")),
                            &pass!(&new_enum_assign_c!(&format!("{}", sandwich), &sandwich_domain, "kitchen", "sandwich", "c"))
                        )
                    )
                )
            }
        }
    }

    for sandwich in &sandwiches {
        for tray in &trays {
            transitions.push(
                ParamTransition::new(
                    &format!("put_on_tray_{}_{}", sandwich, tray),
                    &ppred!(
                        &pass!(&new_enum_assign_c!(&format!("{}", sandwich), &sandwich_domain, "kitchen", "sandwich", "c")),
                        &pass!(&new_enum_assign_c!(&format!("{}", tray), &tray_domain, "kitchen", "tray", "c"))
                    ),
                    &ppred!(
                        &pass!(&new_enum_assign_c!(&format!("{}", sandwich), &sandwich_domain, &format!("{}", tray), "sandwich", "c"))
                    )
                )
            )
        }
    }  

    for sandwich in &sandwiches {
        for child in &children {
            for tray in &trays {
                for place in &places {
                    transitions.push(
                        ParamTransition::new(
                            &format!("serve_sandwich_no_gluten_{}_{}_{}_{}", sandwich, child, tray, place),
                            &ppred!(
                                &pass!(&new_enum_assign_c!(&format!("allergic_gluten_{}", child), &tf_domain, "true", "tf", "c")),
                                &pass!(&new_enum_assign_c!(&format!("{}", sandwich), &sandwich_domain, &format!("{}", tray), "sandwich", "c")),
                                &pass!(&new_enum_assign_c!(&format!("{}", child), &child_domain, &format!("{}", place), "child", "c")),
                                &pass!(&new_enum_assign_c!(&format!("no_gluten_sandwich_{}", sandwich), &tf_domain, "true", "tf", "c")),
                                &pass!(&new_enum_assign_c!(&format!("{}", tray), &tray_domain, &format!("{}", place), "tray", "c"))
                            ),
                            &ppred!(
                                &pass!(&new_enum_assign_c!(&format!("{}", sandwich), &sandwich_domain, "served", "sandwich", "c")),
                                &pass!(&new_enum_assign_c!(&format!("{}", child), &child_domain, "served", "child", "c"))
                            )
                        )
                    )
                }
            }
        }
    }    

    for sandwich in &sandwiches {
        for child in &children {
            for tray in &trays {
                for place in &places {
                    transitions.push(
                        ParamTransition::new(
                            &format!("serve_sandwich_{}_{}_{}_{}", sandwich, child, tray, place),
                            &ppred!(

                                &pass!(&new_enum_assign_c!(&format!("allergic_gluten_{}", child), &tf_domain, "false", "tf", "c")),
                                &pass!(&new_enum_assign_c!(&format!("{}", sandwich), &sandwich_domain, &format!("{}", tray), "sandwich", "c")),
                                &pass!(&new_enum_assign_c!(&format!("{}", child), &child_domain, &format!("{}", place), "child", "c")),
                                &pass!(&new_enum_assign_c!(&format!("{}", tray), &tray_domain, &format!("{}", place), "tray", "c"))
                            ),
                            &ppred!(
                                &pass!(&new_enum_assign_c!(&format!("{}", sandwich), &sandwich_domain, "served", "sandwich", "c")),
                                &pass!(&new_enum_assign_c!(&format!("{}", child), &child_domain, "served", "child", "c"))
                            )
                        )
                    )
                }
            }
        }
    } 

    for tray in &trays {
        for place1 in &places {
            for place2 in &places {
                if place1 != place2 {
                    transitions.push(
                        ParamTransition::new(
                            &format!("move_tray_{}_{}_{}", tray, place1, place2),
                            &ppred!(
                                &pass!(&new_enum_assign_c!(&format!("{}", tray), &tray_domain, &format!("{}", place1), "tray", "c"))
                            ),
                            &ppred!(
                                &pass!(&new_enum_assign_c!(&format!("{}", tray), &tray_domain, &format!("{}", place2), "tray", "c"))
                            )
                        )
                    )
                }
            }
        }
    }   

    let mut invariants = vec!();    

    // works
    for sandwich in &sandwiches {
        for tray in &trays {
            invariants.push(
                pnot!(
                    &pand!(
                        &pass!(&new_enum_assign_c!(&format!("{}", sandwich), &sandwich_domain, "notexist", "sandwich", "c")),
                        &pass!(&new_enum_assign_c!(&format!("{}", sandwich), &sandwich_domain, &format!("{}", tray), "sandwich", "c"))
                    )
                )
            );
        }
    }

    let c = Parameter::new("c", &true);

    let problem = ParamPlanningProblem::new(
        &format!("childsnack_eq_invariant_instance1"), 
        &parsed.init, 
        &parsed.goal, 
        &transitions,
        // &Predicate::TRUE,
        &Predicate::AND(invariants),
        &vec!(c)
    );

    problem
}