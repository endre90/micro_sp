use std::time::SystemTime;

use crate::*;
use tokio::{
    sync::{mpsc, oneshot},
    time::{interval, Duration},
};

pub async fn goal_runner(
    name: &str,
    model: &Model,
    command_sender: mpsc::Sender<StateManagement>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = interval(Duration::from_millis(100));

    log::info!(target: &&format!("{}_goal_runner", name), "Online.");

    // For nicer logging
    let mut plan_current_step_old = 0;
    let mut planner_information_old = "".to_string();
    let mut operation_state_old = "".to_string();
    let mut operation_information_old = "".to_string();
    let mut current_goal_state_old = "".to_string();
    let mut plan_old: Vec<String> = vec![];

    loop {
        let (response_tx, response_rx) = oneshot::channel();
        command_sender
            .send(StateManagement::GetState(response_tx))
            .await?;
        let state = response_rx.await?;

        let mut current_goal_state = state.get_string_or_default_to_unknown(
            &format!("{}_goal_runner", name),
            &format!("{}_current_goal_state", name),
        );

        let current_goal_id = state.get_string_or_default_to_unknown(
            &format!("{}_goal_runner", name),
            &format!("{}_current_goal_id", name),
        );

        match CurrentGoalState::from_str(&current_goal_state.to_string()) {
            CurrentGoalState::Empty => {
                log::info!(target: &&format!("{}_goal_runner", name), 
                        "Current goal state is Empty.");
                // Load the first goal from the schedule to be executed
                // remove it from the schedule and move up the goals
                let scheduled_goals = state.get_array_or_default_to_empty(
                    &format!("{}_goal_runner", name),
                    &format!("{}_scheduled_goals", name),
                );
                if let Some((first_goal_id, rest_of_the_goals)) = scheduled_goals.split_first() {
                    let current_goal_id = first_goal_id.to_string();
                    let current_goal_predicate = state.get_string_or_default_to_unknown(
                        &format!("{}_goal_runner", name),
                        &format!("{}_current_goal_predicate", name),
                    );
                    let current_goal_state = CurrentGoalState::Initial;

                    let new_state = state
                        .update(
                            &format!("{}_current_goal_id", name),
                            current_goal_id.to_spvalue(),
                        )
                        .update(
                            &format!("{}_current_goal_predicate", name),
                            current_goal_predicate.to_spvalue(),
                        )
                        .update(
                            &format!("{}_current_goal_state", name),
                            current_goal_state.to_spvalue(),
                        )
                        .update(
                            &format!("{}_scheduled_goals", name),
                            SPValue::Array(ArrayOrUnknown::Array(rest_of_the_goals.to_vec())),
                        );

                    let modified_state = state.get_diff_partial_state(&new_state);
                    command_sender
                        .send(StateManagement::SetPartialState(modified_state))
                        .await?;
                }
            }
            CurrentGoalState::Initial => {
                log::info!(target: &&format!("{}_goal_runner", name), 
                    "Initializing goal: {}.", current_goal_id);

                let current_goal_state = CurrentGoalState::Planning;

                let new_state = state
                    .update(&format!("{}_replan_trigger", name), true.to_spvalue())
                    .update(&format!("{}_replanned", name), false.to_spvalue())
                    .update(
                        &format!("{}_current_goal_state", name),
                        current_goal_state.to_spvalue(),
                    );

                let modified_state = state.get_diff_partial_state(&new_state);
                command_sender
                    .send(StateManagement::SetPartialState(modified_state))
                    .await?;
            }
            CurrentGoalState::Planning => {
                let mut replan_trigger = state.get_bool_or_default_to_false(
                    &format!("{}_goal_runner", name),
                    &format!("{}_replan_trigger", name),
                );
                let mut replanned = state.get_bool_or_default_to_false(
                    &format!("{}_goal_runner", name),
                    &format!("{}_replanned", name),
                );
                let mut plan_counter = state.get_int_or_default_to_zero(
                    &format!("{}_goal_runner", name),
                    &format!("{}_plan_counter", name),
                );
                let mut replan_counter = state.get_int_or_default_to_zero(
                    &format!("{}_goal_runner", name),
                    &format!("{}_replan_counter", name),
                );
                let mut replan_counter_total = state.get_int_or_default_to_zero(
                    &format!("{}_goal_runner", name),
                    &format!("{}_replan_counter_total", name),
                );
                let mut plan_current_step = state.get_int_or_default_to_zero(
                    &format!("{}_goal_runner", name),
                    &format!("{}_plan_current_step", name),
                );
                let plan_of_sp_values = state.get_array_or_default_to_empty(
                    &format!("{}_goal_runner", name),
                    &format!("{}_plan", name),
                );

                let mut plan: Vec<String> = plan_of_sp_values
                    .iter()
                    .filter(|val| val.is_string())
                    .map(|y| y.to_string())
                    .collect();

                let mut planner_information = state.get_string_or_default_to_unknown(
                    &format!("{}_goal_runner", name),
                    &format!("{}_planner_information", name),
                );

                // Log only when something changes and not every tick
                if plan_current_step_old != plan_current_step {
                    log::info!(target: &format!("{}_goal_runner", name), "Plan current step: {plan_current_step}.");
                    plan_current_step_old = plan_current_step
                }

                if planner_information_old != planner_information {
                    log::info!(target: &format!("{}_goal_runner", name), "Planner info: {planner_information}");
                    planner_information_old = planner_information.clone()
                }

                if plan_old != plan {
                    log::info!(
                        target: &format!("{}_goal_runner", name),
                        "Got a plan:\n{}",
                        plan.iter()
                            .enumerate()
                            .map(|(index, step)| format!("       {} -> {}", index + 1, step))
                            .collect::<Vec<String>>()
                            .join("\n")
                    );
                    plan_old = plan.clone()
                }

                let mut current_goal_state = CurrentGoalState::Planning;

                match (replan_trigger, replanned) {
                    (true, true) => {
                        planner_information = "Planner triggered and (re)planned.".to_string();
                        replan_trigger = false;
                        replanned = false;
                    }
                    (true, false) => {
                        plan_current_step = 0;
                        if replan_counter < MAX_REPLAN_RETRIES {
                            let goal = state.extract_goal(name);
                            replan_counter = replan_counter + 1;
                            replan_counter_total = replan_counter_total + 1;
                            let state_clone = state.clone();
                            let new_plan = bfs_operation_planner(
                                state_clone,
                                goal,
                                model.operations.clone(),
                                30,
                            );
                            if !new_plan.found {
                                planner_information = format!(
                                    "Planner triggered (try {replan_counter}/{MAX_REPLAN_RETRIES}): No plan was found."
                                );
                            } else {
                                replan_counter = 0;
                                if new_plan.length == 0 {
                                    planner_information = format!(
                                        "Planner triggered (try {replan_counter}/{MAX_REPLAN_RETRIES}): We are already in the goal, no action will be taken."
                                    );
                                    current_goal_state = CurrentGoalState::Completed;
                                } else {
                                    planner_information = format!(
                                        "Planner triggered (try {replan_counter}/{MAX_REPLAN_RETRIES}): A new plan was found."
                                    );
                                    plan = new_plan.plan;
                                    current_goal_state = CurrentGoalState::Executing;
                                    replanned = true;
                                    plan_counter = plan_counter + 1;
                                }
                            }
                        } else {
                            planner_information = "Max allowed replan retries reached.".to_string();
                            current_goal_state = CurrentGoalState::PlanNotFound;
                            replan_trigger = false;
                            replanned = false;
                        }
                    }

                    (false, _) => {
                        planner_information = "Planner is not triggered".to_string();
                        replanned = false;
                    }
                }

                let new_state = state
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
                    .update(
                        &format!("{}_plan_current_step", name),
                        plan_current_step.to_spvalue(),
                    )
                    .update(&format!("{}_plan", name), plan.to_spvalue())
                    .update(
                        &format!("{}_planner_information", name),
                        planner_information.to_spvalue(),
                    )
                    .update(
                        &format!("{}_replan_counter_total", name),
                        replan_counter_total.to_spvalue(),
                    )
                    .update(
                        &format!("{}_current_goal_state", name),
                        current_goal_state.to_spvalue(),
                    );

                let modified_state = state.get_diff_partial_state(&new_state);
                command_sender
                    .send(StateManagement::SetPartialState(modified_state))
                    .await?;
            }
            CurrentGoalState::PlanNotFound => {
                log::error!(target: &&format!("{}_goal_runner", name), "Unable to find plan.");
                log::error!(target: &&format!("{}_goal_runner", name), 
                    "Try changing the state and set replan_trigger.");
            }
            CurrentGoalState::Executing => {
                let mut new_state = state.clone();
                let mut plan_current_step = state.get_int_or_default_to_zero(
                    &format!("{}_goal_runner", name),
                    &format!("{}_plan_current_step", name),
                );
                let plan_of_sp_values = state.get_array_or_default_to_empty(
                    &format!("{}_goal_runner", name),
                    &format!("{}_plan", name),
                );

                let plan: Vec<String> = plan_of_sp_values
                    .iter()
                    .filter(|val| val.is_string())
                    .map(|y| y.to_string())
                    .collect();

                // Log only when something changes and not every tick
                if current_goal_state_old != current_goal_state {
                    log::info!(target: &format!("{}_goal_runner", name), "Goal in current state: {current_goal_state}.");
                    current_goal_state_old = current_goal_state.clone()
                }

                if plan.len() > plan_current_step as usize {
                    let operation = model
                        .operations
                        .iter()
                        .find(|op| op.name == plan[plan_current_step as usize].to_string())
                        .unwrap()
                        .to_owned();

                    let operation_state = state.get_string_or_default_to_unknown(
                        &format!("{}_goal_runner", name),
                        &format!("operation_{}", operation.name),
                    );

                    let mut operation_information = state.get_string_or_default_to_unknown(
                        &format!("{}_goal_runner", name),
                        &format!("operation_{}_information", operation.name),
                    );

                    let mut operation_retry_counter = state.get_int_or_default_to_zero(
                        &format!("{}_goal_runner", name),
                        &format!("operation_{}_retry_counter", operation.name),
                    );

                    let mut operation_start_time = state.get_time_or_unknown(
                        &format!("{}_goal_runner", name),
                        &format!("operation_{}_start_time", operation.name),
                    );

                    // Log only when something changes and not every tick
                    if operation_state_old != operation_state {
                        log::info!(target: &format!("{}_goal_runner", name), "Current state of operation {}: {}.", operation.name, operation_state);
                        operation_state_old = operation_state.clone()
                    }

                    if operation_information_old != operation_information {
                        log::info!(target: &format!("{}_goal_runner", name), "Current operation '{}' info: {}.", operation.name, operation_information);
                        operation_information_old = operation_information.clone()
                    }

                    match OperationState::from_str(&operation_state) {
                        OperationState::Initial => {
                            let now = SystemTime::now();
                            if operation.eval_running(&state) {
                                new_state = operation.start_running(&new_state);
                                operation_start_time = TimeOrUnknown::Time(now);
                            }
                        }
                        OperationState::Disabled => {
                            operation_information = "Operation is disabled, waiting.".to_string();
                            if operation.eval_running(&state) {
                                new_state = operation.start_running(&new_state);
                            }
                        }
                        OperationState::Executing => {
                            if operation.can_be_completed(&state) {
                                new_state = operation.clone().complete_running(&new_state);
                                operation_information = "Completing operation.".to_string();
                            } else if operation.can_be_failed(&state) {
                                new_state = operation.clone().fail_running(&new_state);
                                operation_information = "Failing operation.".to_string();
                            } else {
                                operation_information = "Waiting to be completed.".to_string();
                                match operation_start_time {
                                    TimeOrUnknown::Time(start_time_result) => {
                                        match start_time_result.elapsed() {
                                            Ok(start_time) => match operation.timeout_ms {
                                                Some(timeout) => {
                                                    if start_time.as_millis() > timeout {
                                                        new_state =
                                                            operation.timeout_running(&new_state);
                                                    }
                                                }
                                                None => (),
                                            },
                                            Err(_) => {}
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        OperationState::Completed => {
                            operation_retry_counter = 0;
                            new_state = new_state.update(
                                &format!("operation_{}_retry_counter", operation.name),
                                operation_retry_counter.to_spvalue(),
                            );
                            plan_current_step = plan_current_step + 1;
                        }
                        OperationState::Timedout => {
                            operation_information =
                                format!("Operation '{}' timedout.", operation.name);
                        }
                        OperationState::Failed => {
                            if operation_retry_counter < operation.retries {
                                operation_retry_counter = operation_retry_counter + 1;
                                operation_information = format!(
                                    "Retrying. Retry nr. {} out of {}.",
                                    operation_retry_counter, operation.retries
                                );
                                new_state = operation.clone().retry_running(&new_state);
                                new_state = new_state.update(
                                    &format!("operation_{}_retry_counter", operation.name),
                                    operation_retry_counter.to_spvalue(),
                                );
                            } else {
                                operation_retry_counter = 0;
                                new_state = new_state.update(
                                    &format!("operation_{}_retry_counter", operation.name),
                                    operation_retry_counter.to_spvalue(),
                                );
                                operation_information = format!(
                                    "No more operation retries left. Failing the plan: {:?}",
                                    plan
                                );
                                current_goal_state = CurrentGoalState::Failed.to_string();
                            }
                        }
                        OperationState::UNKNOWN => (),
                    }

                    new_state = new_state
                        .update(
                            &format!("operation_{}_information", operation.name),
                            operation_information.to_spvalue(),
                        )
                        .update(
                            &format!("operation_{}_start_time", name),
                            operation_start_time.to_spvalue(),
                        );
                } else {
                    current_goal_state = CurrentGoalState::Completed.to_string();
                }

                new_state = new_state
                    .update(
                        &format!("{}_current_goal_state", name),
                        current_goal_state.to_spvalue(),
                    )
                    .update(
                        &format!("{}_plan_current_step", name),
                        plan_current_step.to_spvalue(),
                    )
                    .update(&format!("{}_plan", name), plan.to_spvalue());

                let modified_state = state.get_diff_partial_state(&new_state);
                command_sender
                    .send(StateManagement::SetPartialState(modified_state))
                    .await?;
            }
            CurrentGoalState::Paused => {
                log::warn!(target: &&format!("{}_goal_runner", name), 
                    "The goal runner is paused.");
            }
            CurrentGoalState::Failed => todo!(),
            CurrentGoalState::Aborted => todo!(),
            CurrentGoalState::Completed => todo!(),
        }

        interval.tick().await;
    }
}
