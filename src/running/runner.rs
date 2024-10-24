use crate::*;
use tokio::time::{interval, Duration};
use std::sync::{Arc, Mutex};

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
                let mut updated_state  = t.clone().take_running(&shared_state_local);
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

        let runner_plan_name =
            match shared_state_local.get_value(&&format!("{}_plan_name", name)) {
                SPValue::String(value) => value,
                _ => {
                    log::error!(target: &&format!("{}_runner", name), 
            "Couldn't get '{}_runner_plan_name' from the shared state.", name);
                    "unknown".to_string()
                }
            };

        let runner_plan_state =
            match shared_state_local.get_value(&&format!("{}_plan_state", name)) {
                SPValue::String(value) => value,
                _ => {
                    log::error!(target: &&format!("{}_runner", name), 
                "Couldn't get '{}_plan_state' from the shared state.", name);
                    "unknown".to_string()
                }
            };

        match PlanState::from_str(&runner_plan_state) {
            PlanState::Initial => {
                log::info!(target: &&format!("{}_runner", name), "Current state of plan '{}': Initial.", runner_plan_name);
                let shared_state_local = shared_state.lock().unwrap().clone();
                let updated_state = shared_state_local.update(
                    &&format!("{}_plan_state", name),
                    PlanState::Executing.to_spvalue(),
                );
                log::info!(target: &&format!("{}_runner", name), "Starting plan: '{}'.", runner_plan_name);
                *shared_state.lock().unwrap() = updated_state;
            }
            PlanState::Executing => {
                log::info!(target: &&format!("{}_runner", name), "Current state of plan '{}': Executing.", runner_plan_name);
                let runner_plan =
                    match shared_state_local.get_value(&&format!("{}_plan", name)) {
                        SPValue::Array(_sp_value_type, value_array) => value_array,
                        _ => {
                            log::error!(target: &&format!("{}_runner", name), 
                "Couldn't get '{}_plan' from the shared state.", name);
                            vec![]
                        }
                    };
                let runner_plan_current_step = match shared_state_local
                    .get_value(&&format!("{}_plan_current_step", name))
                {
                    SPValue::Int64(value) => value,
                    _ => {
                        log::error!(target: &&format!("{}_runner", name), 
                "Couldn't get '{}_current_step' from the shared state.", name);
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
                            let current_model_operation = model
                                .operations
                                .iter()
                                .find(|op| op.name == current_model_operation.name)
                                .unwrap()
                                .to_owned();

                            if current_model_operation
                                .clone()
                                .can_be_reset(&shared_state_local)
                            {
                                log::info!(target: &&format!("{}_runner", name), 
                "Reseting operation: '{}'.", current_model_operation.name);

                                let shared_state_local = shared_state.lock().unwrap().clone();
                                let updated_state = current_model_operation
                                    .clone()
                                    .reset_running(&shared_state_local);
                                *shared_state.lock().unwrap() = updated_state.clone();
                            }
                        }
                        OperationState::Timedout => todo!(),
                        OperationState::Failed => todo!(),
                        OperationState::UNKNOWN => (),
                    }
                } else {
                    log::info!(target: &&format!("{}_runner", name), 
                "Completed plan: '{}'.", runner_plan_name);
                }
            }
            PlanState::Paused => todo!(),
            PlanState::Failed => todo!(),
            PlanState::NotFound => todo!(),
            PlanState::Completed => todo!(),
            PlanState::Cancelled => todo!(),
            PlanState::UNKNOWN => {},
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

        let runner_replanned =
            match shared_state_local.get_value(&&format!("{}_replanned", name)) {
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
            (true, true) => shared_state_local
                .update(
                    &&format!("{}_replan_trigger", name),
                    false.to_spvalue(),
                )
                .update(&&format!("{}_replanned", name), false.to_spvalue())
                .update(&&format!("{}_replan_counter", name), 0.to_spvalue()),
            (true, false) => {
                let goal = extract_goal_from_state(name.to_string(), &shared_state_local);
                let updated_state = shared_state_local
                    .update(
                        &&format!("{}_plan_counter", name),
                        (runner_plan_counter + 1).to_spvalue(),
                    )
                    .update(
                        &&format!("{}_replan_counter", name),
                        (runner_replan_counter + 1).to_spvalue(),
                    );
                    // .update(&&format!("{}_runner_state", name), "planning".to_spvalue());
                let updated_state = reset_all_operations(&updated_state);
                *shared_state.lock().unwrap() = updated_state.clone();
                let new_plan = bfs_operation_planner(
                    updated_state.clone(),
                    goal,
                    model.operations.clone(),
                    30,
                );
                if !new_plan.found {
                    log::error!(target: &&format!("{}_runner", name), "No plan was found");
                    updated_state.update(
                        &&format!("{}_plan_state", name),
                        "not_found".to_spvalue(),
                    )
                } else {
                    if new_plan.length == 0 {
                        log::info!(target: &&format!("{}_runner", name), "We are already in the goal.");
                        updated_state.update(
                            &&format!("{}_plan_state", name),
                            "completed".to_spvalue(),
                        )
                    } else {
                        log::info!(target: &&format!("{}_runner", name), "A new plan was found:");
                        for step in &new_plan.plan {
                            log::info!(target: &&format!("{}_runner", name), "  {}", step);
                        }
                        updated_state
                            .update(
                                &&format!("{}_plan", name),
                                new_plan.plan.to_spvalue(),
                            )
                            .update(
                                &&format!("{}_plan_state", name),
                                "initial".to_spvalue(),
                            )
                    }
                }
                // *shared_state.lock().unwrap() = updated_state;
            }
            (false, _) => shared_state_local.update("replanned", false.to_spvalue()),
        };

        *shared_state.lock().unwrap() = updated_state.clone();
        interval.tick().await;
    }
}
