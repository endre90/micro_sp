use crate::*;
use std::sync::{Arc, Mutex};
use tokio::time::{interval, Duration};

pub async fn simple_operation_runner(
    name: &str,
    model: &Model,
    shared_state: &Arc<Mutex<State>>,
    coverability_tracking: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = interval(Duration::from_millis(100));
    let model = model.clone();

    // Add the variables that keep track of the runner state
    let planner_state_vars = generate_runner_state_variables(&model, name, coverability_tracking);
    let shared_state_local = shared_state.lock().unwrap().clone();
    let updated_state = shared_state_local.extend(planner_state_vars, true);
    *shared_state.lock().unwrap() = updated_state.clone();

    loop {
        let shared_state_local = shared_state.lock().unwrap().clone();

        // Auto transitions should be taken as soon as guard becomas true
        for t in &model.auto_transitions {
            if t.clone().eval_running(&shared_state_local) {
                let shared_state_local = shared_state.lock().unwrap().clone();
                let mut updated_state = t.clone().take_running(&shared_state_local);
                log::info!(target: &&format!("{}_runner", name), "Executed auto transition: '{}'.", t.name);
                if coverability_tracking {
                    let taken_auto_counter =
                        match shared_state_local.get_value(&&format!("{}taken", name)) {
                            SPValue::Int64(value) => value,
                            _ => {
                                log::error!(target: &&format!("{}_runner", name), 
                    "Couldn't get '{}_taken' from the shared state.", name);
                                0
                            }
                        };
                    updated_state = updated_state.update(
                        &format!("{}_taken", t.name),
                        (taken_auto_counter + 1).to_spvalue(),
                    );
                }
                *shared_state.lock().unwrap() = updated_state;
            }
        }

        // let name =
        //     match shared_state_local.get_value(&&format!("{}_plan_name", name)) {
        //         SPValue::String(value) => value,
        //         _ => {
        //             log::error!(target: &&format!("{}_runner", name),
        //     "Couldn't get '{}_plan_name' from the shared state.", name);
        //             "unknown".to_string()
        //         }
        //     };

        let runner_plan_state = match shared_state_local.get_value(&&format!("{}_plan_state", name))
        {
            SPValue::String(value) => value,
            _ => {
                log::error!(target: &&format!("{}_runner", name), 
                "Couldn't get '{}_plan_state' from the shared state.", name);
                "unknown".to_string()
            }
        };

        match PlanState::from_str(&runner_plan_state) {
            PlanState::Initial => {
                println!("Current state of plan '{}': Initial.", name);
                log::info!(target: &&format!("{}_runner", name), "Current state of plan '{}': Initial.", name);
                let shared_state_local = shared_state.lock().unwrap().clone();
                let updated_state = shared_state_local.update(
                    &&format!("{}_plan_state", name),
                    PlanState::Executing.to_spvalue(),
                );
                log::info!(target: &&format!("{}_runner", name), "Starting plan: '{}'.", name);
                *shared_state.lock().unwrap() = updated_state;
            }
            PlanState::Executing => {
                println!("Current state of plan '{}': Executing.", name);

                log::info!(target: &&format!("{}_runner", name), "Current state of plan '{}': Executing.", name);
                let runner_plan = match shared_state_local.get_value(&&format!("{}_plan", name)) {
                    SPValue::Array(_sp_value_type, value_array) => value_array,
                    _ => {
                        log::error!(target: &&format!("{}_runner", name), 
                "Couldn't get '{}_plan' from the shared state.", name);
                        vec![]
                    }
                };

                println!("Current plan '{:?}'.", runner_plan);
                let runner_plan_current_step =
                    match shared_state_local.get_value(&&format!("{}_plan_current_step", name)) {
                        SPValue::Int64(value) => value,
                        // SPValue::UNKNOWN => {
                        //     log::error!("ADSFASDFASDFASDF");
                        //     0},
                        _ => {
                            log::error!(target: &&format!("{}_runner", name), 
                "Couldn't get '{}_plan_current_step' from the shared state.", name);
                            0
                        }
                    };
                if runner_plan.len() > runner_plan_current_step as usize {
                    let current_model_operation = model
                        .operations
                        .iter()
                        .find(|op| {
                            op.name == runner_plan[runner_plan_current_step as usize].to_string()
                        })
                        .unwrap()
                        .to_owned();
                    let shared_state_local = shared_state.lock().unwrap().clone();
                    match OperationState::from_str(
                        &shared_state_local
                            .get_value(&current_model_operation.name)
                            .to_string(),
                    ) {
                        OperationState::Initial => {
                            log::info!(target: &&format!("{}_runner", name), "Current state of operation '{}': Initial.", current_model_operation.name);
                            if current_model_operation
                                .clone()
                                .eval_running(&shared_state_local)
                            {
                                let shared_state_local = shared_state.lock().unwrap().clone();
                                log::info!(target: &&format!("{}_runner", name), "Starting operation: '{}'.", current_model_operation.name);
                                let updated_state =
                                    current_model_operation.start_running(&shared_state_local);
                                *shared_state.lock().unwrap() = updated_state.clone();
                            }
                        }
                        OperationState::Disabled => todo!(),
                        OperationState::Executing => {
                            log::info!(target: &&format!("{}_runner", name), "Current state of operation '{}': Executing.", current_model_operation.name);
                            if current_model_operation
                                .clone()
                                .can_be_completed(&shared_state_local)
                            {
                                let shared_state_local = shared_state.lock().unwrap().clone();
                                let updated_state = current_model_operation
                                    .clone()
                                    .complete_running(&shared_state_local);
                                log::info!(target: &&format!("{}_runner", name), 
                "Completing operation: '{}'.", current_model_operation.name);
                                *shared_state.lock().unwrap() = updated_state.clone();
                            } else {
                                log::info!(target: &&format!("{}_runner", name), 
                "Waiting for operation: '{}' to be completed.", current_model_operation.name);
                            }
                        }
                        OperationState::Completed => {
                            log::info!(target: &&format!("{}_runner", name), "Current state of operation '{}': Completed.", current_model_operation.name);
                            // let current_model_operation = model
                            //     .operations
                            //     .iter()
                            //     .find(|op| op.name == current_model_operation.name)
                            //     .unwrap()
                            //     .to_owned();

                            let updated_state = updated_state.update(
                                &&format!("{}_plan_current_step", name),
                                (runner_plan_current_step + 1).to_spvalue(),
                            );
                            *shared_state.lock().unwrap() = updated_state.clone();

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
                    log::info!(target: &&format!("{}_runner", name), 
                "Completed plan: '{}'.", name);
                }
            }
            PlanState::Paused => {
                println!("Current state of plan '{}': Paused.", name);
                log::info!(target: &&format!("{}_runner", name), "Current state of plan '{}': Paused.", name)
            }
            PlanState::Failed => {
                println!("Current state of plan '{}': Failed.", name);
                log::info!(target: &&format!("{}_runner", name), "Current state of plan '{}': Failed.", name)
            }
            PlanState::NotFound => {
                println!("Current state of plan '{}': NotFound.", name);
                log::info!(target: &&format!("{}_runner", name), "Current state of plan '{}': NotFound.", name)
            }
            PlanState::Completed => {
                println!("Current state of plan '{}': Completed.", name);
                log::info!(target: &&format!("{}_runner", name), "Current state of plan '{}': Completed.", name)
            }
            PlanState::Cancelled => {
                println!("Current state of plan '{}': Cancelled.", name);
                log::info!(target: &&format!("{}_runner", name), "Current state of plan '{}': Cancelled.", name)
            }
            PlanState::UNKNOWN => {
                println!("Current state of plan '{}': UNKNOWN.", name);
                log::info!(target: &&format!("{}_runner", name), "Current state of plan '{}': Unknown.", name)
            }
        }

        interval.tick().await;
    }
}

pub fn extract_goal_from_state(name: String, state: &State) -> Predicate {
    match state.state.get(&format!("{}_goal", name)) {
        Some(g_spvalue) => match &g_spvalue.val {
            SPValue::String(g_value) => match pred_parser::pred(&g_value, &state) {
                Ok(goal_predicate) => goal_predicate,
                Err(_) => Predicate::TRUE,
            },
            _ => Predicate::TRUE,
        },
        None => Predicate::TRUE,
    }
}

pub async fn planner_ticker(
    name: &str,
    model: &Model,
    shared_state: &Arc<Mutex<State>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = interval(Duration::from_millis(100));
    let model = model.clone();

    loop {
        log::info!(target: &&format!("{}_planner_ticker", name), 
            "asdf");
        // println!("planner_ticker: asdf");
        let shared_state_local = shared_state.lock().unwrap().clone();
        // let runner_replan_trigger = bv!(&&format!("{}_runner_replan_trigger", name));
        let runner_replan_trigger =
            match shared_state_local.get_value(&&format!("{}_replan_trigger", name)) {
                SPValue::Bool(value) => value,
                _ => {
                    log::error!(target: &&format!("{}_runner", name), 
            "Couldn't get '{}_replan_trigger' from the shared state.", name);
                    false
                }
            };

        let runner_replanned = match shared_state_local.get_value(&&format!("{}_replanned", name)) {
            SPValue::Bool(value) => value,
            _ => {
                log::error!(target: &&format!("{}_runner", name), 
            "Couldn't get '{}_replanned' from the shared state.", name);
                false
            }
        };

        let runner_plan_counter =
            match shared_state_local.get_value(&&format!("{}_plan_counter", name)) {
                SPValue::Int64(value) => value,
                _ => {
                    log::error!(target: &&format!("{}_runner", name), 
            "Couldn't get '{}_plan_counter' from the shared state.", name);
                    0
                }
            };

        let runner_replan_counter =
            match shared_state_local.get_value(&&format!("{}_replan_counter", name)) {
                SPValue::Int64(value) => value,
                _ => {
                    log::error!(target: &&format!("{}_runner", name), 
            "Couldn't get '{}_replan_counter' from the shared state.", name);
                    0
                }
            };

        let updated_state = match (runner_replan_trigger, runner_replanned) {
            (true, true) => {
                log::info!(target: &&format!("{}_planner_ticker", name), 
            "replan = true, replanned = true");
                println!("planner_ticker: replan = true, replanned = true");
                shared_state_local
                    .update(&&format!("{}_replan_trigger", name), false.to_spvalue())
                    .update(&&format!("{}_replanned", name), false.to_spvalue())
                    .update(&&format!("{}_replan_counter", name), 0.to_spvalue())
            }
            (true, false) => {
                log::info!(target: &&format!("{}_planner_ticker", name), 
            "replan = true, replanned = false");
                println!("planner_ticker: replan = true, replanned = false");
                let goal = extract_goal_from_state(name.to_string(), &shared_state_local);
                let mut updated_state = shared_state_local
                    .update(
                        &&format!("{}_plan_counter", name),
                        (runner_plan_counter + 1).to_spvalue(),
                    )
                    .update(
                        &&format!("{}_replan_counter", name),
                        (runner_replan_counter + 1).to_spvalue(),
                    );
                // .update(&&format!("{}_runner_state", name), "planning".to_spvalue());
                // let updated_state = reset_all_operations(&updated_state);
                // *shared_state.lock().unwrap() = updated_state.clone();
                let new_plan = bfs_operation_planner(
                    updated_state.clone(),
                    goal,
                    model.operations.clone(),
                    30,
                );
                if !new_plan.found {
                    log::error!(target: &&format!("{}_runner", name), "No plan was found");
                    updated_state = updated_state.update(&&format!("{}_plan_state", name), "not_found".to_spvalue());
                    updated_state
                } else {
                    if new_plan.length == 0 {
                        log::info!(target: &&format!("{}_runner", name), "We are already in the goal.");
                        updated_state = updated_state
                            .update(&&format!("{}_plan_state", name), "completed".to_spvalue());
                        updated_state
                    } else {
                        log::info!(target: &&format!("{}_runner", name), "A new plan was found:");
                        for step in &new_plan.plan {
                            log::info!(target: &&format!("{}_runner", name), "  {}", step);
                        }
                        updated_state = updated_state
                            .update(&&format!("{}_plan", name), new_plan.plan.to_spvalue())
                            .update(&&format!("{}_plan_state", name), "initial".to_spvalue())
                            .update(&&format!("{}_replanned", name), true.to_spvalue())
                            .update(&&format!("{}_plan_current_step", name), 0.to_spvalue());
                        updated_state
                    }
                }
                // *shared_state.lock().unwrap() = updated_state;
            }
            (false, _) => {
                log::info!(target: &&format!("{}_planner_ticker", name), 
            "replan = false, replanned = _");
                println!("planner_ticker: replan = false, replanned = _");
                shared_state_local.update(&&format!("{}_replanned", name), false.to_spvalue())
            }
        };

        *shared_state.lock().unwrap() = updated_state.clone();
        interval.tick().await;
    }
}

// pub async fn ticke_the_planner(
//     node_id: &str,
//     model: &Model,
//     shared_state: &Arc<Mutex<State>>,
//     mut timer: r2r::Timer,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     // wait for the measured values to update the state
//     // tokio::time::sleep(std::time::Duration::from_millis(5000)).await;
//     let shared_state_local = shared_state.lock().unwrap().clone();
//     let mut old_state = shared_state_local.clone();
//     loop {
//         let shsl = shared_state.lock().unwrap().clone();
//         let replan = shsl.get_value("runner_replan");
//         let replanned = shsl.get_value("runner_replanned");
//         let replan_counter = match shsl.get_value("runner_replan_counter") {
//             SPValue::Int32(value) => value,
//             _ => 0,
//         };

//         let new_state = match (replan, replanned) {
//             (SPValue::Bool(true), SPValue::Bool(true)) => shsl
//                 .update("runner_replan", false.to_spvalue())
//                 .update("runner_replanned", false.to_spvalue()),
//             (SPValue::Bool(true), SPValue::Bool(false)) => {
//                 let goal = extract_goal_from_state(&shsl);
//                 let new_state = shsl
//                     .update("runner_replanned", true.to_spvalue())
//                     .update("runner_replan_counter", (replan_counter + 1).to_spvalue());
//                 let new_state = reset_all_operations(&new_state);
//                 r2r::log_warn!(node_id, "Re-plan triggered in the following state:");
//                 println!("{}", new_state);
//                 *shared_state.lock().unwrap() = shsl.update("runner_plan_state", "PLANNING".to_spvalue());
//                 let new_plan =
//                     bfs_operation_planner(new_state.clone(), goal, model.operations.clone(), 30);
//                 match new_plan.found {
//                     false => {
//                         r2r::log_warn!(node_id, "No plan was found.");
//                         new_state
//                             .update("runner_plan_info", "No plan found.".to_spvalue())
//                             .update("runner_plan_state", "failed".to_spvalue())
//                     }
//                     true => match new_plan.length == 0 {
//                         true => {
//                             r2r::log_warn!(node_id, "We are already in the goal.");
//                             new_state
//                                 .update("runner_plan_info", "Already in the goal.".to_spvalue())
//                                 .update("runner_plan_state", "done".to_spvalue())
//                         }
//                         false => {
//                             r2r::log_warn!(node_id, "A new plan was found: {:?}.", new_plan.plan);
//                             new_state
//                                 .update("runner_plan", new_plan.plan.to_spvalue())
//                                 .update("runner_plan_info", "A new plan was found.".to_spvalue())
//                                 .update("runner_plan_state", "ready".to_spvalue())
//                         }
//                     },
//                 }
//             }
//             (SPValue::Bool(false), _) => shsl.update("runner_replanned", false.to_spvalue()),
//             (_, _) => shsl,
//         };

//         let new_state = tick_the_runner(node_id, &model, &new_state).await;

//         if new_state != old_state {
//             println!("{}", new_state);
//         }

//         old_state = new_state.clone();

//         *shared_state.lock().unwrap() = new_state.clone();

//         timer.tick().await?;
//     }
// }

// async fn tick_the_runner(node_id: &str, model: &Model, shared_state: &State) -> State {
//     let mut shsl = shared_state.clone();

//     for t in &model.auto_transitions {
//         let taken_auto_counter = match shsl.get_value(&format!("taken_auto_{}", t.name)) {
//             SPValue::Int32(taken) => taken,
//             _ => 0,
//         };
//         if t.clone().eval_running(&shsl) {
//             r2r::log_warn!(node_id, "Taking the free transition: {}.", t.name);
//             let new_state = shsl.update(
//                 &format!("taken_auto_{}", t.name),
//                 (taken_auto_counter + 1).to_spvalue(),
//             );
//             shsl = t.clone().take_running(&new_state);
//         }
//     }

//     match shsl.get_value("runner_plan") {
//         SPValue::Array(_, plan) => match plan.is_empty() {
//             true => shsl
//                 .update("runner_plan_info", "The plan is empty.".to_spvalue())
//                 .update("runner_plan", SPValue::Unknown)
//                 .update("runner_plan_current_step", SPValue::Unknown),

//             // we have not started executing the plan so we start at position 0 in the plan
//             false => match shsl.get_value("runner_plan_current_step") {
//                 SPValue::Unknown => shsl.update("runner_plan_current_step", 0.to_spvalue()),
//                 SPValue::Int32(curr_step) => match plan.len() <= curr_step as usize {
//                     // we are done with the plan and will stop executing and we also
//                     // reset the current plan so we do not tries to run the same plan again
//                     true => shsl
//                         .update("runner_plan_info", "The plan is done.".to_spvalue())
//                         .update("runner_plan_state", "done".to_spvalue())
//                         .update("runner_goal", SPValue::Unknown)
//                         .update("runner_plan", SPValue::Unknown)
//                         .update("runner_plan_current_step", SPValue::Unknown),

//                     false => {
//                         let current_op_name = match plan[curr_step as usize].clone() {
//                             SPValue::String(op_name) => op_name.to_string(),
//                             _ => panic!("no such op name"),
//                         };
//                         let current_op_state = shsl.get_value(&current_op_name);
//                         let current_op = model
//                             .operations
//                             .iter()
//                             .find(|op| op.name == current_op_name)
//                             .unwrap();

//                         let next_step_in_plan = curr_step + 1;

//                         if current_op_state == "initial".to_spvalue() {
//                             if current_op.clone().eval_running(&shsl) {
//                                 // The operation can be started

//                                 let start = SystemTime::now();
//                                 let since_the_epoch = start
//                                     .duration_since(UNIX_EPOCH)
//                                     .expect("Time went backwards")
//                                     .as_secs_f64();
//                                 // let current_op_started =
//                                 //     match shsl.get_value(&format!("started_{}", current_op_name)) {
//                                 //         SPValue::Int32(started) => started,
//                                 //         _ => 0,
//                                 //     };
//                                 let shsl = shsl
//                                     .update(
//                                         &format!("timestamp_{}", current_op_name),
//                                         since_the_epoch.to_spvalue(),
//                                     );
//                                     // .update(
//                                     //     &format!("started_{}", current_op_name),
//                                     //     (current_op_started + 1).to_spvalue(),
//                                     // );

//                                 current_op.clone().start_running(&shsl)
//                             } else {
//                                 // The operation can be started but is not enabled
//                                 let disabled_current_op = match shsl
//                                     .get_value(&format!("disabled_{}", current_op_name))
//                                 {
//                                     SPValue::Int32(started) => started,
//                                     _ => 0,
//                                 };
//                                 shsl.update(
//                                     "runner_plan_info",
//                                     format!("Waiting for {current_op_name} to be enabled.")
//                                         .to_spvalue(),
//                                 )
//                                 .update(
//                                     &format!("disabled_{}", current_op_name),
//                                     (disabled_current_op + 1).to_spvalue(),
//                                 )
//                             }
//                         } else if current_op_state == "executing".to_spvalue() {
//                             if current_op.clone().can_be_completed(&shsl) {
//                                 // complete the operation and take a step in the plan
//                                 let shsl = current_op.clone().complete_running(&shsl);
//                                 let current_op_completed = match shsl
//                                     .get_value(&format!("completed_{}", current_op_name))
//                                 {
//                                     SPValue::Int32(completed) => completed,
//                                     _ => 0,
//                                 };
//                                 shsl.update(
//                                     "runner_plan_current_step",
//                                     next_step_in_plan.to_spvalue(),
//                                 )
//                                 .update(
//                                     "runner_plan_info",
//                                     format!("Completed step {curr_step}.").to_spvalue(),
//                                 )
//                                 .update(
//                                     &format!("completed_{}", current_op_name),
//                                     (current_op_completed + 1).to_spvalue(),
//                                 )
//                             } else {
//                                 // the operation is still executing, check if operation timeout is exceeded
//                                 let timestamp_current_op = match shsl
//                                     .get_value(&format!("timestamp_{}", current_op_name))
//                                 {
//                                     SPValue::Float64(OrderedFloat(timestamp)) => timestamp,
//                                     _ => 0.0,
//                                 };
//                                 let deadline_current_op = match shsl
//                                     .get_value(&format!("deadline_{}", current_op_name))
//                                 {
//                                     SPValue::Float64(OrderedFloat(deadline)) => deadline,
//                                     _ => 0.0,
//                                 };
//                                 let start = SystemTime::now();
//                                 let since_the_epoch = start
//                                     .duration_since(UNIX_EPOCH)
//                                     .expect("Time went backwards")
//                                     .as_secs_f64();
//                                 if (since_the_epoch - timestamp_current_op) > deadline_current_op {
//                                     let nr_timedout = match shsl
//                                         .get_value(&format!("timedout_{}", current_op_name))
//                                     {
//                                         SPValue::Int32(nr_timedout) => nr_timedout,
//                                         _ => 0,
//                                     };
//                                     shsl.update(
//                                         "runner_plan_info",
//                                         format!("Operation {current_op_name} timed out.")
//                                             .to_spvalue(),
//                                     )
//                                     .update(
//                                         &format!("timestamp_{}", current_op_name),
//                                         since_the_epoch.to_spvalue(),
//                                     )
//                                     .update(
//                                         &format!("timedout_{}", current_op_name),
//                                         (nr_timedout + 1).to_spvalue(),
//                                     )
//                                 } else {
//                                     let executing_current_op = match shsl.get_value(
//                                         &format!("executing_{}", current_op_name),
//                                     ) {
//                                         SPValue::Int32(completed) => completed,
//                                         _ => 0,
//                                     };
//                                     shsl.update(
//                                         "runner_plan_info",
//                                         format!("Waiting for {current_op_name} to complete.")
//                                             .to_spvalue(),
//                                     )
//                                     .update(
//                                         &format!("executing_{}", current_op_name),
//                                         (executing_current_op + 1).to_spvalue(),
//                                     )
//                                 }
//                             }
//                         } else {
//                             // this shouldn't really happen
//                             shsl.update("runner_plan_info", format!("Doing nothing.").to_spvalue())
//                         }
//                     }
//                 },
//                 _ => shsl.clone(),
//             },
//         },
//         SPValue::Unknown => shsl.clone(),
//         _ => panic!("runner_plan should be Array type."),
//     }
// }
