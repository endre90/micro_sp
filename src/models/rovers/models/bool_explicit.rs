use crate::models::rovers::models::bool_explicit_parser::parser;
use super::*;

// macro_rules! new_bool_assign_c {
//     ($name:expr, $domain:expr, $val:expr) => { ... };
//     ($name:expr, $domain:expr, $val:expr, $r#type:expr) => { ... };
//     ($name:expr, $domain:expr, $val:expr, $r#type:expr, $param:expr) => { ... };
//     ($name:expr, $domain:expr, $val:expr, $r#type:expr, $param:expr, $life:expr) => { ... };
// }

/// Explicitly generating negative predicates from diff(ojb/init)
pub fn model(name: &str) -> ParamPlanningProblem {

    let (parsed, rovers, landers, waypoints, objectives, cameras, modes, stores) = parser(name);

    let mut navigate_transitions = vec![];
    let mut sample_soil_transitions = vec![];
    let mut sample_rock_transitions = vec![];
    let mut drop_transitions = vec![];
    let mut calibrate_transitions = vec![];
    let mut take_image_transitions = vec![];
    let mut communicate_soil_data_transitions = vec![];
    let mut communicate_rock_data_transitions = vec![];
    let mut communicate_image_data_transitions = vec![];
    let mut free_channel_transitions = vec![];

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
                    navigate_transitions.push(
                        ParamTransition::new(
                            &format!("navigate_{}(rov)_{}(wp1)_{}(wp2)", rover, wp1, wp2),
                            &ppred!(
                                &pass!(&new_bool_assign_c!(&format!("{}_can_traverse_from_{}_to_{}", rover, wp1, wp2), true, "c")),
                                &pass!(&new_bool_assign_c!(&format!("{}_available", rover), true, "c")),
                                &pass!(&new_bool_assign_c!(&format!("{}_at_{}", rover, wp1), true, "c"))
                            ),
                            &ppred!(
                                &pass!(&new_bool_assign_c!(&format!("{}_at_{}", rover, wp1), false, "c")),
                                &pass!(&new_bool_assign_c!(&format!("{}_at_{}", rover, wp2), true, "c"))
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
                sample_soil_transitions.push(
                    ParamTransition::new(
                        &format!("sample_soil_{}(rov)_{}(str)_{}(wp)", rover, store, waypoint),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("{}_at_{}", rover, waypoint), true, "c")),
                            &pass!(&new_bool_assign_c!(&format!("{}_at_soil_sample", waypoint), true, "c")),
                            &pass!(&new_bool_assign_c!(&format!("{}_equipped_for_soil_analysis", rover), true, "c")),
                            &pass!(&new_bool_assign_c!(&format!("{}_store_of_{}", store, rover), true, "c")),
                            &pass!(&new_bool_assign_c!(&format!("{}_empty", store), true, "c"))
                        ),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("{}_empty", store), false, "c")),
                            &pass!(&new_bool_assign_c!(&format!("{}_full", store), true, "c")),
                            &pass!(&new_bool_assign_c!(&format!("{}_have_soil_analysis_{}", rover, waypoint), true, "c")),
                            &pass!(&new_bool_assign_c!(&format!("{}_at_soil_sample", waypoint), false, "c"))
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
                sample_rock_transitions.push(
                    ParamTransition::new(
                        &format!("sample_rock_{}(rov)_{}(str)_{}(wp)", rover, store, waypoint),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("{}_at_{}", rover, waypoint), true, "c")),
                            &pass!(&new_bool_assign_c!(&format!("{}_at_rock_sample", waypoint), true, "c")),
                            &pass!(&new_bool_assign_c!(&format!("{}_equipped_for_rock_analysis", rover), true, "c")),
                            &pass!(&new_bool_assign_c!(&format!("{}_store_of_{}", store, rover), true, "c")),
                            &pass!(&new_bool_assign_c!(&format!("{}_empty", store), true, "c"))
                        ),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("{}_empty", store), false, "c")),
                            &pass!(&new_bool_assign_c!(&format!("{}_full", store), true, "c")),
                            &pass!(&new_bool_assign_c!(&format!("{}_have_rock_analysis_{}", rover, waypoint), true, "c")),
                            &pass!(&new_bool_assign_c!(&format!("{}_at_rock_sample", waypoint), false, "c"))
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
            drop_transitions.push(
                ParamTransition::new(
                    &format!("drop_storage_{}(rov)_{}(str)", rover, store),
                    &ppred!(
                        &pass!(&new_bool_assign_c!(&format!("{}_store_of_{}", store, rover), true, "c")),
                        &pass!(&new_bool_assign_c!(&format!("{}_full", store), true, "c"))
                    ),
                    &ppred!(
                        &pass!(&new_bool_assign_c!(&format!("{}_empty", store), true, "c")),
                        &pass!(&new_bool_assign_c!(&format!("{}_full", store), false, "c"))
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
                    calibrate_transitions.push(
                        ParamTransition::new(
                            &format!("calibrate_{}(rov)_{}(cam)_{}(obj)_{}(wp)", rover, camera, objective, waypoint),
                            &ppred!(
                                &pass!(&new_bool_assign_c!(&format!("{}_equipped_for_imaging", rover), true, "c")),
                                &pass!(&new_bool_assign_c!(&format!("{}_calibration_target_{}", camera, objective), true, "c")),
                                &pass!(&new_bool_assign_c!(&format!("{}_at_{}", rover, waypoint), true, "c")),
                                &pass!(&new_bool_assign_c!(&format!("{}_visible_from_{}", objective, waypoint), true, "c")),
                                &pass!(&new_bool_assign_c!(&format!("{}_on_board_{}", camera, rover), true, "c"))
                            ),
                            &ppred!(
                                &pass!(&new_bool_assign_c!(&format!("{}_calibrated_{}", camera, rover), true, "c"))
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
                        take_image_transitions.push(
                            ParamTransition::new(
                                &format!("take_image_{}(rov)_{}(wp)_{}(obj)_{}(cam)_{}(mod)", rover, waypoint, objective, camera, mode),
                                &ppred!(
                                    &pass!(&new_bool_assign_c!(&format!("{}_calibrated_{}", camera, rover), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("{}_on_board_{}", camera, rover), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("{}_equipped_for_imaging", rover), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("{}_supports_mode_{}", camera, mode), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("{}_visible_from_{}", objective, waypoint), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("{}_at_{}", rover, waypoint), true, "c"))
                                ),
                                &ppred!(
                                    &pass!(&new_bool_assign_c!(&format!("{}_have_image_{}_{}", rover, objective, mode), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("{}_calibrated_{}", camera, rover), false, "c"))
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
                        communicate_soil_data_transitions.push(
                            ParamTransition::new(
                                &format!("communicate_soil_data_{}(rov)_{}(lan)_{}(rovwp)_{}(lanwp)_{}(soilwp)", rover, lander, rovwp, lanwp, soilwp),
                                &ppred!(
                                    &pass!(&new_bool_assign_c!(&format!("{}_at_{}", rover, rovwp), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("{}_at_lander_{}", lander, lanwp), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("{}_have_soil_analysis_{}", rover, soilwp), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("{}_visible_{}", rovwp, lanwp), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("{}_available", rover), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("{}_channel_free", lander), true, "c"))
                                ),
                                &ppred!(
                                    &pass!(&new_bool_assign_c!(&format!("{}_available", rover), false, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("{}_channel_free", lander), false, "c")),
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
                        communicate_rock_data_transitions.push(
                            ParamTransition::new(
                                &format!("communicate_rock_data_{}(rov)_{}(lan)_{}(rovwp)_{}(lanwp)_{}(rockwp)", rover, lander, rovwp, lanwp, rockwp),
                                &ppred!(
                                    &pass!(&new_bool_assign_c!(&format!("{}_at_{}", rover, rovwp), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("{}_at_lander_{}", lander, lanwp), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("{}_have_rock_analysis_{}", rover, rockwp), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("{}_visible_{}", rovwp, lanwp), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("{}_available", rover), true, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("{}_channel_free", lander), true, "c"))
                                ),
                                &ppred!(
                                    &pass!(&new_bool_assign_c!(&format!("{}_available", rover), false, "c")),
                                    &pass!(&new_bool_assign_c!(&format!("{}_channel_free", lander), false, "c")),
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
                            communicate_image_data_transitions.push(
                                ParamTransition::new(
                                    &format!("communicate_image_data_{}(rov)_{}(lan)_{}(obj)_{}(mod)_{}(rovwp)_{}(lanwp)", rover, lander, objective, mode, rovwp, lanwp),
                                    &ppred!(
                                        &pass!(&new_bool_assign_c!(&format!("{}_at_{}", rover, rovwp), true, "c")),
                                        &pass!(&new_bool_assign_c!(&format!("{}_at_lander_{}", lander, lanwp), true, "c")),
                                        &pass!(&new_bool_assign_c!(&format!("have_image_{}_{}_{}", rover, objective, mode), true, "c")),
                                        &pass!(&new_bool_assign_c!(&format!("{}_visible_{}", rovwp, lanwp), true, "c")),
                                        &pass!(&new_bool_assign_c!(&format!("{}_available", rover), true, "c")),
                                        &pass!(&new_bool_assign_c!(&format!("{}_channel_free", lander), true, "c"))
                                    ),
                                    &ppred!(
                                        &pass!(&new_bool_assign_c!(&format!("{}_available", rover), false, "c")),
                                        &pass!(&new_bool_assign_c!(&format!("{}_channel_free", lander), false, "c")),
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

    // missing free_channel transitions?
    for rover in &rovers {
        for lander in &landers {
            free_channel_transitions.push(
                ParamTransition::new(
                    &format!("free_channel_{}(rov)_{}(lan)", rover, lander),
                    &ppred!(
                        &pass!(&new_bool_assign_c!(&format!("{}_available", rover), false, "c")),
                        &pass!(&new_bool_assign_c!(&format!("{}_channel_free", lander), false, "c"))
                    ),
                    &ppred!(
                        &pass!(&new_bool_assign_c!(&format!("{}_available", rover), true, "c")),
                        &pass!(&new_bool_assign_c!(&format!("{}_channel_free", lander), true, "c"))
                    )
                )
            )
        }
    }

    let mut transitions = vec![];
    for t in vec![
        navigate_transitions,
        sample_soil_transitions,
        sample_rock_transitions,
        drop_transitions,
        calibrate_transitions,
        take_image_transitions,
        communicate_soil_data_transitions,
        communicate_rock_data_transitions,
        communicate_image_data_transitions,
        free_channel_transitions,
    ] {
        transitions.extend(t)
    }

    let c = Parameter::new("c", &true);
    // let clear = Parameter::new("clear", &true);
    // let ontable = Parameter::new("ontable", &true);
    // let hand = Parameter::new("hand", &true);
    // let holding = Parameter::new("holding", &true);

    let problem = ParamPlanningProblem::new(
        &format!("blocksworld_bool_explicit_{}", parsed.name.as_str()), 
        &parsed.init,
        &parsed.goal,
        &transitions,
        &Predicate::TRUE,
        &vec!(c)
    );

    problem
}