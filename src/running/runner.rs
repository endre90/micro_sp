use crate::*;
use std::sync::{atomic::{AtomicUsize, Ordering}, Arc, Mutex};
use tokio::time::{interval, Duration};

// pub async fn auto_transition_runner(
//     name: &str,
//     model: &Model,
//     shared_state: &Arc<Mutex<State>>,
//     coverability_tracking: bool,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let mut interval = interval(Duration::from_millis(100));
//     let model = model.clone();
//     loop {
//         let state = shared_state.lock().unwrap().clone();

//         // Auto transitions should be taken as soon as guard becomas true
//         for t in &model.auto_transitions {
//             if t.clone().eval_running(&state) {
//                 let state = shared_state.lock().unwrap().clone();
//                 let mut updated_state = t.clone().take_running(&state);
//                 log::info!(target: &&format!("{}_runner", name), "Executed auto transition: '{}'.", t.name);
//                 if coverability_tracking {
//                     let taken_auto_counter =
//                         match state.get_value(&&format!("{}_taken", name)) {
//                             SPValue::Int64(value) => value,
//                             _ => {
//                                 log::error!(target: &&format!("{}_runner", name),
//                     "Couldn't get '{}_taken' from the shared state.", name);
//                                 0
//                             }
//                         };
//                     updated_state = updated_state.update(
//                         &format!("{}_taken", t.name),
//                         (taken_auto_counter + 1).to_spvalue(),
//                     );
//                 }
//                 *shared_state.lock().unwrap() = updated_state;
//             }
//         }
//         interval.tick().await;
//     }
// }

pub async fn operation_runner(
    model: &Model,
    shared_state: &Arc<(Mutex<State>, Vec<AtomicUsize>)>, //HashMap<String, AtomicUsize>)>,
) -> Result<(), Box<dyn std::error::Error>> {
    let name = &model.name;
    let mut interval = interval(Duration::from_millis(100));
    let model = model.clone();

    // let mut ref_count: i64 = 0;
    let mut last_known_local_version = 0;

    loop {
        let mut current_local_version = shared_state.1[2].load(Ordering::SeqCst);

        if current_local_version != last_known_local_version {
            println!(
                "operation_runner: {} - {}",
                current_local_version, last_known_local_version
            );
            // state has been updated by the "gantry_interface" task
            log::warn!(target: &&format!("{}_operation_runner", name), "state has been updated by the 'operation_runner' task");
            last_known_local_version = current_local_version;

            let mut state = shared_state.0.lock().unwrap().clone();
            // let ref_counter = state.get_or_default_i64(
            //     &format!("{}_operation_runner", name),
            //     &format!("{}_runner_ref_counter", name),
            // );
            // if ref_counter > ref_count {
            //     ref_count = ref_counter;

            let mut plan_state = state.get_or_default_string(
                &format!("{}_operation_runner", name),
                &format!("{}_plan_state", name),
            );
            let mut plan_current_step = state.get_or_default_i64(
                &format!("{}_operation_runner", name),
                &format!("{}_plan_current_step", name),
            );
            let plan = state.get_or_default_array_of_strings(
                &format!("{}_operation_runner", name),
                &format!("{}_plan", name),
            );

            match PlanState::from_str(&plan_state) {
                PlanState::Initial => {
                    log::info!(target: &&format!("{}_operation_runner", name), "Current state of plan '{}': Initial.", name);
                    log::info!(target: &&format!("{}_operation_runner", name), "Starting plan: '{:?}'.", plan);
                    plan_state = PlanState::Executing.to_string();
                }
                PlanState::Executing => {
                    log::info!(target: &&format!("{}_operation_runner", name), "Current state of plan '{}': Executing.", name);
                    log::info!(target: &&format!("{}_operation_runner", name), "Executing plan: '{:?}'.", plan);

                    if plan.len() > plan_current_step as usize {
                        let operation = model
                            .operations
                            .iter()
                            .find(|op| op.name == plan[plan_current_step as usize].to_string())
                            .unwrap()
                            .to_owned();

                        let operation_state = state.get_or_default_string(
                            &format!("{}_operation_runner", name),
                            &format!("{}", operation.name),
                        );

                        match OperationState::from_str(&operation_state) {
                            OperationState::Initial => {
                                log::info!(target: &&format!("{}_operation_runner", name), 
                                "Current state of operation '{}': Initial.", operation.name);
                                if operation.eval_running(&state) {
                                    log::info!(target: &&format!("{}_operation_runner", name), 
                                    "Starting operation: '{}'.", operation.name);
                                    state = operation.start_running(&state);
                                }
                            }
                            OperationState::Disabled => todo!(),
                            OperationState::Executing => {
                                log::info!(target: &&format!("{}_operation_runner", name), 
                            "Current state of operation '{}': Executing.", operation.name);
                                if operation.can_be_completed(&state) {
                                    state = operation.clone().complete_running(&state);
                                    log::info!(target: &&format!("{}_operation_runner", name), 
                                    "Completing operation: '{}'.", operation.name);
                                } else {
                                    log::info!(target: &&format!("{}_operation_runner", name), 
                                    "Waiting for operation: '{}' to be completed.", operation.name);
                                }
                            }
                            OperationState::Completed => {
                                log::info!(target: &&format!("{}_runner", name), 
                                "Current state of operation '{}': Completed.", operation.name);
                                plan_current_step = plan_current_step + 1;
                                // let current_model_operation = model
                                //     .operations
                                //     .iter()
                                //     .find(|op| op.name == current_model_operation.name)
                                //     .unwrap()
                                //     .to_owned();

                                //             if current_model_operation
                                //                 .clone()
                                //                 .can_be_reset(&shared_state_local)
                                //             {
                                //                 log::info!(target: &&format!("{}_runner", name),
                                // "Reseting operation: '{}'.", current_model_operation.name);

                                //                 let shared_state_local = shared_state.lock().unwrap().clone();
                                //                 let updated_state = current_model_operation
                                //                     .clone()
                                //                     .reset_running(&shared_state_local);
                                //                 *shared_state.lock().unwrap() = updated_state.clone();
                                //             }
                            }
                            OperationState::Timedout => todo!(),
                            OperationState::Failed => todo!(),
                            OperationState::UNKNOWN => (),
                        }
                    } else {
                        log::info!(target: &&format!("{}_operation_runner", name), 
                "Completed plan: '{}'.", name);
                        plan_state = PlanState::Completed.to_string();
                    }
                }
                PlanState::Paused => {
                    log::info!(target: &&format!("{}_runner", name), "Current state of plan '{}': Paused.", name)
                }
                PlanState::Failed => {
                    log::info!(target: &&format!("{}_runner", name), "Current state of plan '{}': Failed.", name)
                }
                PlanState::NotFound => {
                    log::info!(target: &&format!("{}_runner", name), "Current state of plan '{}': NotFound.", name)
                }
                PlanState::Completed => {
                    log::info!(target: &&format!("{}_runner", name), "Current state of plan '{}': Completed.", name)
                }
                PlanState::Cancelled => {
                    log::info!(target: &&format!("{}_runner", name), "Current state of plan '{}': Cancelled.", name)
                }
                PlanState::UNKNOWN => {
                    log::info!(target: &&format!("{}_runner", name), "Current state of plan '{}': UNKNOWN.", name)
                }
            }

            let updated_state = state
                .update(&format!("{}_plan_state", name), plan_state.to_spvalue())
                .update(
                    &format!("{}_plan_current_step", name),
                    plan_current_step.to_spvalue(),
                )
                .update(&format!("{}_plan", name), plan.to_spvalue());
            // .update(
            //     &format!("{}_runner_ref_counter", name),
            //     (ref_counter + 1).to_spvalue(),
            // );
            shared_state.1[2].fetch_add(1, Ordering::SeqCst);
            *shared_state.0.lock().unwrap() = updated_state.clone();
            // } else {
            //     let updated_state = state.update(
            //         &format!("{}_runner_ref_counter", name),
            //         (ref_counter + 1).to_spvalue(),
            //     );
            //     *shared_state.lock().unwrap() = updated_state.clone();
            // }
        }
        interval.tick().await;
    }
}

pub async fn planner_ticker(
    model: &Model,
    shared_state: &Arc<(Mutex<State>, Vec<AtomicUsize>)>, //HashMap<String, AtomicUsize>)>,
) -> Result<(), Box<dyn std::error::Error>> {
    let name = &model.name;
    let mut interval = interval(Duration::from_millis(100));
    let model = model.clone();

    // let mut ref_count: i64 = 0;
    let mut last_known_local_version = 0;

    loop {
        let mut current_local_version = shared_state.1[3].load(Ordering::SeqCst);

        if current_local_version != last_known_local_version {
            println!(
                "planner_ticker: {} - {}",
                current_local_version, last_known_local_version
            );
            // state has been updated by the "gantry_interface" task
            log::warn!(target: &&format!("{}_operation_runner", name), "state has been updated by the 'operation_runner' task");
            last_known_local_version = current_local_version;

            let state = shared_state.0.lock().unwrap().clone();
            // let ref_counter = state.get_or_default_i64(
            //     &format!("{}_planner_ticker", name),
            //     &format!("{}_planner_ref_counter", name),
            // );
            // if ref_counter > ref_count {
            //     ref_count = ref_counter;
            let mut replan_trigger = state.get_or_default_bool(
                &format!("{}_planner_ticker", name),
                &format!("{}_replan_trigger", name),
            );
            let mut replanned = state.get_or_default_bool(
                &format!("{}_planner_ticker", name),
                &format!("{}_replanned", name),
            );
            let mut plan_counter = state.get_or_default_i64(
                &format!("{}_planner_ticker", name),
                &format!("{}_plan_counter", name),
            );
            let mut replan_counter = state.get_or_default_i64(
                &format!("{}_planner_ticker", name),
                &format!("{}_replan_counter", name),
            );
            let mut plan_state = state.get_or_default_string(
                &format!("{}_planner_ticker", name),
                &format!("{}_plan_state", name),
            );
            let mut plan_current_step = state.get_or_default_i64(
                &format!("{}_planner_ticker", name),
                &format!("{}_plan_current_step", name),
            );
            let mut plan = state.get_or_default_array_of_strings(
                &format!("{}_planner_ticker", name),
                &format!("{}_plan", name),
            );

            match (replan_trigger, replanned) {
                (true, true) => {
                    log::info!(target: &&format!("{}_planner_ticker", name), "Planner triggered and (re)planned.");
                    replan_trigger = false;
                    replanned = false;
                }
                (true, false) => {
                    log::info!(target: &&format!("{}_planner_ticker", name), 
            "Planner triggered, initiating (re)planning.");
                    let goal = state.extract_goal(name);
                    plan_counter = plan_counter + 1;
                    replan_counter = replan_counter + 1;
                    let state_clone = state.clone();
                    let new_plan =
                        bfs_operation_planner(state_clone, goal, model.operations.clone(), 30);
                    if !new_plan.found {
                        log::error!(target: &&format!("{}_planner_ticker", name), "No plan was found");
                        plan_state = PlanState::NotFound.to_string();
                        replan_counter = 0;
                    } else {
                        if new_plan.length == 0 {
                            log::info!(target: &&format!("{}_planner_ticker", name), "We are already in the goal.");
                            plan_state = PlanState::Completed.to_string();
                        } else {
                            log::info!(target: &&format!("{}_planner_ticker", name), "A new plan was found:");
                            log::info!(target: &&format!("{}_planner_ticker", name), "Plan: {:?}", new_plan.plan);
                            plan = new_plan.plan;
                            plan_state = PlanState::Initial.to_string();
                            replanned = true;
                            plan_current_step = 0;
                        }
                    }
                }
                (false, _) => {
                    log::info!(target: &&format!("{}_planner_ticker", name), 
            "Planner is not triggered.");
                    replanned = false;
                }
            };

            let updated_state = state
                .update(
                    &format!("{}_replan_trigger", name),
                    replan_trigger.to_spvalue(),
                )
                .update(&format!("{}_replanned", name), replanned.to_spvalue())
                .update(&format!("{}_plan_counter", name), plan_counter.to_spvalue())
                .update(
                    &format!("{}_replan_counter", name),
                    replan_counter.to_spvalue(),
                )
                .update(&format!("{}_plan_state", name), plan_state.to_spvalue())
                .update(
                    &format!("{}_plan_current_step", name),
                    plan_current_step.to_spvalue(),
                )
                .update(&format!("{}_plan", name), plan.to_spvalue());
            // .update(
            //     &format!("{}_planner_ref_counter", name),
            //     (ref_counter + 1).to_spvalue(),
            // );
            shared_state.1[3].fetch_add(1, Ordering::SeqCst);
            *shared_state.0.lock().unwrap() = updated_state.clone();
            // } else {
            //     let updated_state = state.update(
            //         &format!("{}_planner_ref_counter", name),
            //         (ref_counter + 1).to_spvalue(),
            //     );
            //     *shared_state.lock().unwrap() = updated_state.clone();
            // }
        }
        interval.tick().await;
    }
}
