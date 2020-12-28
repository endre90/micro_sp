use crate::models::childsnack::models::bool_explicit_parser::parser;
use super::*;

// macro_rules! new_bool_assign_c {
//     ($name:expr, $domain:expr, $val:expr) => { ... };
//     ($name:expr, $domain:expr, $val:expr, $r#type:expr) => { ... };
//     ($name:expr, $domain:expr, $val:expr, $r#type:expr, $param:expr) => { ... };
//     ($name:expr, $domain:expr, $val:expr, $r#type:expr, $param:expr, $life:expr) => { ... };
// }

/// Some invariants added
/// 1. a tray can only be at one place at a time
/// 2. don't serve a sandwich to a served child
/// 3. if a sandwich exists, it is either in the kitchen or on one of the trays
/// 4. if a sandwich does not exists or it has been served, it is neither in the kitchen nor on one of the trays
pub fn model(name: &str) -> ParamPlanningProblem {

    let (parsed, objects) = parser(name);

    let mut transitions = vec![];

    let children = objects.get("child").unwrap_or(&vec!()).to_vec();
    let bread_portions = objects.get("bread_portion").unwrap_or(&vec!()).to_vec();
    let content_portions = objects.get("content_portion").unwrap_or(&vec!()).to_vec();
    let trays = objects.get("tray").unwrap_or(&vec!()).to_vec();
    let places = objects.get("place").unwrap_or(&vec!()).to_vec();
    let sandwiches = objects.get("sandwich").unwrap_or(&vec!()).to_vec();

    //     (:action make_sandwich_no_gluten 
    //         :parameters (?s - sandwich ?b - bread-portion ?c - content-portion)
    //         :precondition (and (at_kitchen_bread ?b)
    //                    (at_kitchen_content ?c)
    //                    (no_gluten_bread ?b)
    //                    (no_gluten_content ?c)
    //                    (notexist ?s))
    //         :effect (and
    //               (not (at_kitchen_bread ?b))
    //               (not (at_kitchen_content ?c))
    //               (at_kitchen_sandwich ?s)
    //               (no_gluten_sandwich ?s)
    //                       (not (notexist ?s))
    //               )) 

    for sandwich in &sandwiches {
        for bread_portion in &bread_portions {
            for content_portion in &content_portions {
                transitions.push(
                    ParamTransition::new(
                        &format!("make_sandwich_no_gluten_{}_{}_{}", sandwich, bread_portion, content_portion),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("at_kitchen_bread_{}", bread_portion), true, "c")),
                            &pass!(&new_bool_assign_c!(&format!("at_kitchen_content_{}", content_portion), true, "c")),
                            &pass!(&new_bool_assign_c!(&format!("no_gluten_bread_{}", bread_portion), true, "c")),
                            &pass!(&new_bool_assign_c!(&format!("no_gluten_content_{}", content_portion), true, "c")),
                            &pass!(&new_bool_assign_c!(&format!("notexist_{}", sandwich), true, "c"))
                        ),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("at_kitchen_bread_{}", bread_portion), false, "c")),
                            &pass!(&new_bool_assign_c!(&format!("at_kitchen_content_{}", content_portion), false, "c")),
                            &pass!(&new_bool_assign_c!(&format!("at_kitchen_sandwich_{}", sandwich), true, "c")),
                            &pass!(&new_bool_assign_c!(&format!("no_gluten_sandwich_{}", sandwich), true, "c")),
                            &pass!(&new_bool_assign_c!(&format!("notexist_{}", sandwich), false, "c"))
                        )
                    )
                )
            }
        }
    }
    
    
    //    (:action make_sandwich
    //         :parameters (?s - sandwich ?b - bread-portion ?c - content-portion)
    //         :precondition (and (at_kitchen_bread ?b)
    //                    (at_kitchen_content ?c)
    //                                (notexist ?s)
    //                    )
    //         :effect (and
    //               (not (at_kitchen_bread ?b))
    //               (not (at_kitchen_content ?c))
    //               (at_kitchen_sandwich ?s)
    //                       (not (notexist ?s))
    //               )) 

    for sandwich in &sandwiches {
        for bread_portion in &bread_portions {
            for content_portion in &content_portions {
                transitions.push(
                    ParamTransition::new(
                        &format!("make_sandwich_{}_{}_{}", sandwich, bread_portion, content_portion),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("at_kitchen_bread_{}", bread_portion), true, "c")),
                            &pass!(&new_bool_assign_c!(&format!("at_kitchen_content_{}", content_portion), true, "c")),
                            &pass!(&new_bool_assign_c!(&format!("notexist_{}", sandwich), true, "c"))
                        ),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("at_kitchen_bread_{}", bread_portion), false, "c")),
                            &pass!(&new_bool_assign_c!(&format!("at_kitchen_content_{}", content_portion), false, "c")),
                            &pass!(&new_bool_assign_c!(&format!("at_kitchen_sandwich_{}", sandwich), true, "c")),
                            &pass!(&new_bool_assign_c!(&format!("notexist_{}", sandwich), false, "c"))
                        )
                    )
                )
            }
        }
    }
    
    
    //    (:action put_on_tray
    //         :parameters (?s - sandwich ?t - tray)
    //         :precondition (and  (at_kitchen_sandwich ?s)
    //                     (at ?t kitchen))
    //         :effect (and
    //               (not (at_kitchen_sandwich ?s))
    //               (ontray ?s ?t)))   

    for sandwich in &sandwiches {
        for tray in &trays {
            transitions.push(
                ParamTransition::new(
                    &format!("put_on_tray_{}_{}", sandwich, tray),
                    &ppred!(
                        &pass!(&new_bool_assign_c!(&format!("at_kitchen_sandwich_{}", sandwich), true, "c")),
                        &pass!(&new_bool_assign_c!(&format!("at_{}_kitchen", tray), true, "c"))
                    ),
                    &ppred!(
                        &pass!(&new_bool_assign_c!(&format!("at_kitchen_sandwich_{}", sandwich), false, "c")),
                        &pass!(&new_bool_assign_c!(&format!("ontray_{}_{}", sandwich, tray), true, "c"))
                    )
                )
            )
        }
    }
    
    //    (:action serve_sandwich_no_gluten
    //         :parameters (?s - sandwich ?c - child ?t - tray ?p - place)
    //        :precondition (and
    //                   (allergic_gluten ?c)
    //                   (ontray ?s ?t)
    //                   (waiting ?c ?p)
    //                   (no_gluten_sandwich ?s)
    //                           (at ?t ?p)
    //                   )
    //        :effect (and (not (ontray ?s ?t))
    //                 (served ?c)))    

    for sandwich in &sandwiches {
        for child in &children {
            for tray in &trays {
                for place in &places {
                    transitions.push(
                        ParamTransition::new(
                            &format!("serve_sandwich_no_gluten_{}_{}_{}_{}", sandwich, child, tray, place),
                            &ppred!(
                                &pass!(&new_bool_assign_c!(&format!("allergic_gluten_{}", child), true, "c")),
                                &pass!(&new_bool_assign_c!(&format!("ontray_{}_{}", sandwich, tray), true, "c")),
                                &pass!(&new_bool_assign_c!(&format!("waiting_{}_{}", child, place), true, "c")),
                                &pass!(&new_bool_assign_c!(&format!("no_gluten_sandwich_{}", sandwich), true, "c")),
                                &pass!(&new_bool_assign_c!(&format!("at_{}_{}", tray, place), true, "c"))
                            ),
                            &ppred!(
                                &pass!(&new_bool_assign_c!(&format!("ontray_{}_{}", sandwich, tray), false, "c")),
                                &pass!(&new_bool_assign_c!(&format!("served_{}", child), true, "c"))
                            )
                        )
                    )
                }
            }
        }
    }
    
    //    (:action serve_sandwich
    //        :parameters (?s - sandwich ?c - child ?t - tray ?p - place)
    //        :precondition (and (not_allergic_gluten ?c)
    //                           (waiting ?c ?p)
    //                   (ontray ?s ?t)
    //                   (at ?t ?p))
    //        :effect (and (not (ontray ?s ?t))
    //                 (served ?c)))    

    for sandwich in &sandwiches {
        for child in &children {
            for tray in &trays {
                for place in &places {
                    transitions.push(
                        ParamTransition::new(
                            &format!("serve_sandwich_{}_{}_{}_{}", sandwich, child, tray, place),
                            &ppred!(
                                &pass!(&new_bool_assign_c!(&format!("not_allergic_gluten_{}", child), true, "c")),
                                &pass!(&new_bool_assign_c!(&format!("ontray_{}_{}", sandwich, tray), true, "c")),
                                &pass!(&new_bool_assign_c!(&format!("waiting_{}_{}", child, place), true, "c")),
                                &pass!(&new_bool_assign_c!(&format!("at_{}_{}", tray, place), true, "c"))
                            ),
                            &ppred!(
                                &pass!(&new_bool_assign_c!(&format!("ontray_{}_{}", sandwich, tray), false, "c")),
                                &pass!(&new_bool_assign_c!(&format!("served_{}", child), true, "c"))
                            )
                        )
                    )
                }
            }
        }
    }
    
    //    (:action move_tray
    //         :parameters (?t - tray ?p1 ?p2 - place)
    //         :precondition (and (at ?t ?p1))
    //         :effect (and (not (at ?t ?p1))
    //                  (at ?t ?p2)))   

    for tray in &trays {
        for place1 in &places {
            for place2 in &places {
                if place1 != place2 {
                    transitions.push(
                        ParamTransition::new(
                            &format!("move_tray_{}_{}_{}", tray, place1, place2),
                            &ppred!(
                                &pass!(&new_bool_assign_c!(&format!("at_{}_{}", tray, place1), true, "c"))
                            ),
                            &ppred!(
                                &pass!(&new_bool_assign_c!(&format!("at_{}_{}", tray, place1), false, "c")),
                                &pass!(&new_bool_assign_c!(&format!("at_{}_{}", tray, place2), true, "c"))
                            )
                        )
                    )
                }
            }
        }
    }   

    let mut invariants = vec!();    

    /// 1. a tray can only be at one place at a time
    /// 2. don't serve a sandwich to a served child
    /// 3. if a sandwich exists, it is either in the kitchen or on one of the trays
    /// 4. if a sandwich does not exists or it has been served, it is neither in the kitchen nor on one of the trays
    /// 
    
    for tray in &trays {
        let mut local_vec = vec!();
        for place in &places {
            local_vec.push(pass!(&new_bool_assign_c!(&format!("at_{}_{}", tray, place), true, "c")))
        }
        invariants.push(Predicate::PBEQ(local_vec, 1))
    }

    // 1. one rover can only be at one waypoint at a time
    for rover in &rovers {
        let mut local_vec = vec!();
        for waypoint in &waypoints {
            local_vec.push(pass!(&new_bool_assign_c!(&format!("at_{}_{}", rover, waypoint), true, "c")))
        }
        invariants.push(Predicate::PBEQ(local_vec, 1))
    }   

    // 2. one lander can only be at one waypoint at a time
    for lander in &landers {
        let mut local_vec = vec!();
        for waypoint in &waypoints {
            local_vec.push(pass!(&new_bool_assign_c!(&format!("at_lander_{}_{}", lander, waypoint), true, "c")))
        }
        invariants.push(Predicate::PBEQ(local_vec, 1))
    }   

    // 3. one store can only be full or empty at one time
    for store in &stores {
        let full = pass!(&new_bool_assign_c!(&format!("empty_{}", store), true, "c"));
        let empty = pass!(&new_bool_assign_c!(&format!("full_{}", store), true, "c"));
        invariants.push(pand!(&por!(&full, &empty), &pnot!(&pand!(&full, &empty))));
    }   

    // 4. lander communication channel is not free while some robot is communicating (rock, soil, image) data (not available)
    for lander in &landers {
        let mut local_vec = vec!();
        for rover in &rovers {
            local_vec.push(pass!(&new_bool_assign_c!(&format!("available_{}", rover), false, "c")));
        }
        invariants.push(pnot!(&pand!(&Predicate::OR(local_vec), &pass!(&new_bool_assign_c!(&format!("channel_free_{}", lander), true, "c"))))); 

    }

    let c = Parameter::new("c", &true);

    let problem = ParamPlanningProblem::new(
        &format!("childsnack_bool_explicit_{}", parsed.name.as_str()), 
        &parsed.init,
        &parsed.goal,
        &transitions,
        &Predicate::TRUE,
        &vec!(c)
    );

    problem
}