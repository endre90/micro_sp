use crate::models::rovers::models::prop_explicit_parser::parser;
use super::*;

// macro_rules! new_bool_assign_c {
//     ($name:expr, $domain:expr, $val:expr) => { ... };
//     ($name:expr, $domain:expr, $val:expr, $r#type:expr) => { ... };
//     ($name:expr, $domain:expr, $val:expr, $r#type:expr, $param:expr) => { ... };
//     ($name:expr, $domain:expr, $val:expr, $r#type:expr, $param:expr, $life:expr) => { ... };
// }

pub fn model(name: &str) -> ParamPlanningProblem {

    let (parsed, objects) = parser(name);

    let rovers = objects.get("rover").unwrap_or(&vec!()).to_vec();
    let landers = objects.get("lander").unwrap_or(&vec!()).to_vec();
    let cameras = objects.get("camera").unwrap_or(&vec!()).to_vec();
    let objectives = objects.get("objective").unwrap_or(&vec!()).to_vec();
    let stores = objects.get("store").unwrap_or(&vec!()).to_vec();
    let waypoints = objects.get("waypoint").unwrap_or(&vec!()).to_vec();
    let modes = objects.get("mode").unwrap_or(&vec!()).to_vec();

    let mut transitions = vec!();

    // (:action navigate
    // :parameters (?x - rover ?y - waypoint ?z - waypoint) 
    // :precondition (and (can_traverse ?x ?y ?z) (available ?x) (at ?x ?y) 
    //                 (visible ?y ?z)
    //         )
    // :effect (and (not (at ?x ?y)) (at ?x ?z)
    //         )
    // )

    for rover in &rovers {
        for wp1 in &waypoints {
            for wp2 in &waypoints {
                if wp1 != wp2 {
                    transitions.push(
                        ParamTransition::new(
                            &format!("navigate_{}_{}_{}", rover, wp1, wp2),
                            &ppred!(
                                &pass!(&new_bool_assign_c!(&format!("can_traverse_{}_{}_{}", rover, wp1, wp2), true, "c")),
                                &pass!(&new_bool_assign_c!(&format!("available_{}", rover), true, "c")),
                                &pass!(&new_bool_assign_c!(&format!("at_{}_{}", rover, wp1), true, "c")),
                                &pass!(&new_bool_assign_c!(&format!("visible_{}_{}", wp1, wp2), true, "c"))
                            ),
                            &ppred!(
                                &pass!(&new_bool_assign_c!(&format!("at_{}_{}", rover, wp1), false, "c")),
                                &pass!(&new_bool_assign_c!(&format!("at_{}_{}", rover, wp2), true, "c"))
                            )
                        )
                    )
                }
            }
        }
    }

    // (:action sample_soil
    // :parameters (?x - rover ?s - store ?p - waypoint)
    // :precondition (and (at ?x ?p) (at_soil_sample ?p) (equipped_for_soil_analysis ?x) (store_of ?s ?x) (empty ?s)
    //         )
    // :effect (and (not (empty ?s)) (full ?s) (have_soil_analysis ?x ?p) (not (at_soil_sample ?p))
    //         )
    // )

    for rover in &rovers {
        for store in &stores {
            for waypoint in &waypoints {
                transitions.push(
                    ParamTransition::new(
                        &format!("sample_soil_{}_{}_{}", rover, store, waypoint),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("at_{}_{}", rover, waypoint), true, "c")),
                            &pass!(&new_bool_assign_c!(&format!("at_soil_sample_{}", waypoint), true, "c")),
                            &pass!(&new_bool_assign_c!(&format!("equipped_for_soil_analysis_{}", rover), true, "c")),
                            &pass!(&new_bool_assign_c!(&format!("store_of_{}_{}", store, rover), true, "c")),
                            &pass!(&new_bool_assign_c!(&format!("empty_{}", store), true, "c"))
                        ),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("empty_{}", store), false, "c")),
                            &pass!(&new_bool_assign_c!(&format!("full_{}", store), true, "c")),
                            &pass!(&new_bool_assign_c!(&format!("have_soil_analysis_{}_{}", rover, waypoint), true, "c")),
                            &pass!(&new_bool_assign_c!(&format!("at_soil_sample_{}", waypoint), false, "c"))
                        )
                    )
                )
            }
        }
    }

    // (:action sample_rock
    // :parameters (?x - rover ?s - store ?p - waypoint)
    // :precondition (and (at ?x ?p) (at_rock_sample ?p) (equipped_for_rock_analysis ?x) (store_of ?s ?x)(empty ?s)
    //         )
    // :effect (and (not (empty ?s)) (full ?s) (have_rock_analysis ?x ?p) (not (at_rock_sample ?p))
    //         )
    // )

    for rover in &rovers {
        for store in &stores {
            for waypoint in &waypoints {
                transitions.push(
                    ParamTransition::new(
                        &format!("sample_rock_{}_{}_{}", rover, store, waypoint),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("at_{}_{}", rover, waypoint), true, "c")),
                            &pass!(&new_bool_assign_c!(&format!("at_rock_sample_{}", waypoint), true, "c")),
                            &pass!(&new_bool_assign_c!(&format!("equipped_for_rock_analysis_{}", rover), true, "c")),
                            &pass!(&new_bool_assign_c!(&format!("store_of_{}_{}", store, rover), true, "c")),
                            &pass!(&new_bool_assign_c!(&format!("empty_{}", store), true, "c"))
                        ),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("empty_{}", store), false, "c")),
                            &pass!(&new_bool_assign_c!(&format!("full_{}", store), true, "c")),
                            &pass!(&new_bool_assign_c!(&format!("have_rock_analysis_{}_{}", rover, waypoint), true, "c")),
                            &pass!(&new_bool_assign_c!(&format!("at_rock_sample_{}", waypoint), false, "c"))
                        )
                    )
                )
            }
        }
    }

    // (:action drop
    // :parameters (?x - rover ?y - store)
    // :precondition (and (store_of ?y ?x) (full ?y)
    //         )
    // :effect (and (not (full ?y)) (empty ?y)
    //     )
    // )

    for rover in &rovers {
        for store in &stores {
            transitions.push(
                ParamTransition::new(
                    &format!("drop_storage_{}_{}", rover, store),
                    &ppred!(
                        &pass!(&new_bool_assign_c!(&format!("store_of_{}_{}", store, rover), true, "c")),
                        &pass!(&new_bool_assign_c!(&format!("full_{}", store), true, "c"))
                    ),
                    &ppred!(
                        &pass!(&new_bool_assign_c!(&format!("empty_{}", store), true, "c")),
                        &pass!(&new_bool_assign_c!(&format!("full_{}", store), false, "c"))
                    )
                )
            )
        }
    }

    // (:action calibrate
    //  :parameters (?r - rover ?i - camera ?t - objective ?w - waypoint)
    //  :precondition (and (equipped_for_imaging ?r) (calibration_target ?i ?t) (at ?r ?w) (visible_from ?t ?w)(on_board ?i ?r)
    //         )
    //  :effect (calibrated ?i ?r) 
    // )

    for rover in &rovers {
        for camera in &cameras {
            for objective in &objectives {
                for waypoint in &waypoints {
                    transitions.push(
                        ParamTransition::new(
                            &format!("calibrate_{}_{}_{}_{}", rover, camera, objective, waypoint),
                            &ppred!(
                                &pass!(&new_bool_assign_c!(&format!("equipped_for_imaging_{}", rover), true, "c")),
                                &pass!(&new_bool_assign_c!(&format!("calibration_target_{}_{}", camera, objective), true, "c")),
                                &pass!(&new_bool_assign_c!(&format!("at_{}_{}", rover, waypoint), true, "c")),
                                &pass!(&new_bool_assign_c!(&format!("visible_from_{}_{}", objective, waypoint), true, "c")),
                                &pass!(&new_bool_assign_c!(&format!("on_board_{}_{}", camera, rover), true, "c"))
                            ),
                            &ppred!(
                                &pass!(&new_bool_assign_c!(&format!("calibrated_{}_{}", camera, rover), true, "c"))
                            )
                        )
                    )
                }
            }
        }
    }

    // (:action take_image
    //  :parameters (?r - rover ?p - waypoint ?o - objective ?i - camera ?m - mode)
    //  :precondition (and (calibrated ?i ?r)
    //              (on_board ?i ?r)
    //                       (equipped_for_imaging ?r)
    //                       (supports ?i ?m)
    //               (visible_from ?o ?p)
    //                      (at ?r ?p)
    //                )
    //  :effect (and (have_image ?r ?o ?m)(not (calibrated ?i ?r))
    //         )
    // )

    for rover in &rovers {
        for waypoint in &waypoints {
            for objective in &objectives {
                for camera in &cameras {
                    for mode in &modes {
                        transitions.push(
                            ParamTransition::new(
                                &format!("take_image_{}_{}_{}_{}_{}", rover, waypoint, objective, camera, mode),
                                &ppred!(
                                    &pass!(&new_bool_assign_c!(&format!("calibrated_{}_{}", camera, rover), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("on_board_{}_{}", camera, rover), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("equipped_for_imaging_{}", rover), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("supports_{}_{}", camera, mode), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("visible_from_{}_{}", objective, waypoint), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("at_{}_{}", rover, waypoint), true, "c"))
                                ),
                                &ppred!(
                                    &pass!(&new_bool_assign_c!(&format!("have_image_{}_{}_{}", rover, objective, mode), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("calibrated_{}_{}", camera, rover), false, "c"))
                                )
                            )
                        )
                    }
                }
            }
        }
    }

    // (:action communicate_soil_data
    //  :parameters (?r - rover ?l - lander ?p - waypoint ?x - waypoint ?y - waypoint)
    //  :precondition (and (at ?r ?x)(at_lander ?l ?y)(have_soil_analysis ?r ?p) 
    //                    (visible ?x ?y)(available ?r)(channel_free ?l)
    //             )
    //  :effect (and (not (available ?r))(not (channel_free ?l))(channel_free ?l)
    //         (communicated_soil_data ?p)(available ?r)
    //     )
    // )

    for rover in &rovers {
        for lander in &landers {
            for rovwp in &waypoints {
                for lanwp in &waypoints {
                    for soilwp in &waypoints {
                        transitions.push(
                            ParamTransition::new(
                                &format!("communicate_soil_data_{}_{}_{}_{}_{}", rover, lander, rovwp, lanwp, soilwp),
                                &ppred!(
                                    &pass!(&new_bool_assign_c!(&format!("at_{}_{}", rover, rovwp), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("at_lander_{}_{}", lander, lanwp), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("have_soil_analysis_{}_{}", rover, soilwp), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("visible_{}_{}", rovwp, lanwp), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("available_{}", rover), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("channel_free_{}", lander), true, "c"))
                                ),
                                &ppred!(
                                    &pass!(&new_bool_assign_c!(&format!("available_{}", rover), false, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("channel_free_{}", lander), false, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("communicated_soil_data_{}", soilwp), true, "c"))
                                )
                            )
                        )
                    }
                }
            }
        }
    }

    // (:action communicate_rock_data
    //  :parameters (?r - rover ?l - lander ?p - waypoint ?x - waypoint ?y - waypoint)
    //  :precondition (and (at ?r ?x)(at_lander ?l ?y)(have_rock_analysis ?r ?p)
    //                    (visible ?x ?y)(available ?r)(channel_free ?l)
    //             )
    //  :effect (and (not (available ?r))(not (channel_free ?l))(channel_free ?l)(communicated_rock_data ?p)(available ?r)
    //           )
    // )

    for rover in &rovers {
        for lander in &landers {
            for rovwp in &waypoints {
                for lanwp in &waypoints {
                    for rockwp in &waypoints {
                        transitions.push(
                            ParamTransition::new(
                                &format!("communicate_rock_data_{}_{}_{}_{}_{}", rover, lander, rovwp, lanwp, rockwp),
                                &ppred!(
                                    &pass!(&new_bool_assign_c!(&format!("at_{}_{}", rover, rovwp), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("at_lander_{}_{}", lander, lanwp), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("have_rock_analysis_{}_{}", rover, rockwp), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("visible_{}_{}", rovwp, lanwp), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("available_{}", rover), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("channel_free_{}", lander), true, "c"))
                                ),
                                &ppred!(
                                    &pass!(&new_bool_assign_c!(&format!("available_{}", rover), false, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("channel_free_{}", lander), false, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("communicated_rock_data_{}", rockwp), true, "c"))
                                )
                            )
                        )
                    }
                }
            }
        }
    }

    // (:action communicate_image_data
    //  :parameters (?r - rover ?l - lander ?o - objective ?m - mode ?x - waypoint ?y - waypoint)
    //  :precondition (and (at ?r ?x)(at_lander ?l ?y)(have_image ?r ?o ?m)(visible ?x ?y)(available ?r)(channel_free ?l)
    //             )
    //  :effect (and (not (available ?r))(not (channel_free ?l))(channel_free ?l)(communicated_image_data ?o ?m)(available ?r)
    //           )
    // )

    for rover in &rovers {
        for lander in &landers {
            for objective in &objectives {
                for mode in &modes {
                    for rovwp in &waypoints {
                        for lanwp in &waypoints {
                            transitions.push(
                                ParamTransition::new(
                                    &format!("communicate_image_data_{}_{}_{}_{}_{}_{}", rover, lander, objective, mode, rovwp, lanwp),
                                    &ppred!(
                                        &pass!(&new_bool_assign_c!(&format!("at_{}_{}", rover, rovwp), true, "c")),
                                        &pass!(&new_bool_assign_c!(&format!("at_lander_{}_{}", lander, lanwp), true, "c")),
                                        &pass!(&new_bool_assign_c!(&format!("have_image_{}_{}_{}", rover, objective, mode), true, "c")),
                                        &pass!(&new_bool_assign_c!(&format!("visible_{}_{}", rovwp, lanwp), true, "c")),
                                        &pass!(&new_bool_assign_c!(&format!("available_{}", rover), true, "c")),
                                        &pass!(&new_bool_assign_c!(&format!("channel_free_{}", lander), true, "c"))
                                    ),
                                    &ppred!(
                                        &pass!(&new_bool_assign_c!(&format!("available_{}", rover), false, "c")),
                                        &pass!(&new_bool_assign_c!(&format!("channel_free_{}", lander), false, "c")),
                                        &pass!(&new_bool_assign_c!(&format!("communicated_image_data_{}_{}", objective, mode), true, "c"))
                                    )
                                )
                            )
                        }
                    }
                }
            }
        }
    }

    // missing free_channel transitions? or add to effects like they do...?
    for rover in &rovers {
        for lander in &landers {
            transitions.push(
                ParamTransition::new(
                    &format!("free_channel_{}_{}", rover, lander),
                    &ppred!(
                        &pass!(&new_bool_assign_c!(&format!("available_{}", rover), false, "c")),
                        &pass!(&new_bool_assign_c!(&format!("channel_free_{}", lander), false, "c"))
                    ),
                    &ppred!(
                        &pass!(&new_bool_assign_c!(&format!("available_{}", rover), true, "c")),
                        &pass!(&new_bool_assign_c!(&format!("channel_free_{}", lander), true, "c"))
                    )
                )
            )
        }
    }

    // Some additional invariants added in hope to reduce the search space:
    // 1. can not traverse wp1 wp2 if not visible wp1 wp2 - doesn't help
    // 2. a rover can only be at one wp at a certain time 
    // 3. store can't be full and empty at the same time
    // 4. rover can't be unequipped for imaging and have image
    // 5. rover can't be unequipped for imaging and be calibrated
    // 6. rover can't be unequipped for imaging and have an onboard camera

    let mut invariants = vec!();

    for rover in &rovers {
        for wp1 in &waypoints {
            for wp2 in &waypoints {
                if wp1 != wp2 {
                    invariants.push(
                        pnot!(
                            &pand!(
                                &pass!(&new_bool_assign_c!(&format!("can_traverse_{}_{}_{}", rover, wp1, wp2), true, "c")),
                                &pass!(&new_bool_assign_c!(&format!("visible_{}_{}", wp1, wp2), false, "c"))
                            )
                        )
                    );  
                }
            }
        }
    }

    for rover in &rovers {
        let mut local_vec = vec!();
        for wp in &waypoints {
            local_vec.push(pass!(&new_bool_assign_c!(&format!("at_{}_{}", rover, wp), true, "c")),)
        }
        invariants.push(Predicate::PBEQ(local_vec, 1))
    };  

    for rover in &rovers {
        for camera in &cameras{
            invariants.push(
                pnot!(
                    &pand!(
                        &pass!(&new_bool_assign_c!(&format!("equipped_for_imaging_{}", rover), false, "c")),
                        &pass!(&new_bool_assign_c!(&format!("on_board_{}_{}", camera, rover), true, "c"))
                    )
                )
            );  
            invariants.push(
                pnot!(
                    &pand!(
                        &pass!(&new_bool_assign_c!(&format!("equipped_for_imaging_{}", rover), false, "c")),
                        &pass!(&new_bool_assign_c!(&format!("calibrated_{}_{}", camera, rover), true, "c"))
                    )
                )
            )
        }
        for objective in &objectives {
            for mode in &modes {
                invariants.push(
                    pnot!(
                        &pand!(
                            &pass!(&new_bool_assign_c!(&format!("equipped_for_imaging_{}", rover), false, "c")),
                            &pass!(&new_bool_assign_c!(&format!("have_image_{}_{}_{}", rover, objective, mode), true, "c"))
                        )
                    )
                );  
            }
        }
    }

    for store in &stores {
        invariants.push(
            pnot!(
                &pand!(
                    &pass!(&new_bool_assign_c!(&format!("empty_{}", store), true, "c")),
                    &pass!(&new_bool_assign_c!(&format!("full_{}", store), true, "c"))
                )
            )
        )
    };

    let c = Parameter::new("c", &true);

    let problem = ParamPlanningProblem::new(
        &format!("rovers_prop_invariant_{}", parsed.name.as_str()), 
        &parsed.init,
        &parsed.goal,
        &transitions,
        &Predicate::AND(invariants),
        &vec!(c)
    );

    problem
}