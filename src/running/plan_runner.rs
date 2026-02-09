use crate::{running::process_operation::OperationProcessingType, *};
use redis::aio::MultiplexedConnection;
use std::sync::Arc;
// use redis::aio::MultiplexedConnection;
use tokio::{
    sync::mpsc,
    time::{Duration, interval},
};

pub static OPERAION_RUNNER_TICK_INTERVAL_MS: u64 = 200;

pub async fn planned_operation_runner(
    model: &Model,
    logging_tx: mpsc::Sender<LogMsg>,
    connection_manager: &Arc<ConnectionManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    let sp_id = &model.name;
    let log_target = format!("{}_op_runner", sp_id);
    let mut interval = interval(Duration::from_millis(OPERAION_RUNNER_TICK_INTERVAL_MS));

    // Get only the relevant keys from the state
    log::info!(target: &log_target, "Online.");

    loop {
        interval.tick().await;
        if let Err(_) = connection_manager.check_redis_health(&log_target).await {
            continue;
        }
        let mut con = connection_manager.get_connection().await;

        let state = match StateManager::get_full_state(&mut con).await {
            Some(s) => s,
            None => continue,
        };

        let con_clone = con.clone();
        let new_state = process_plan_tick(
            sp_id,
            con_clone,
            &model,
            &state,
            logging_tx.clone(),
            &log_target,
        )
        .await;
        let modified_state = state.get_diff_partial_state(&new_state);
        StateManager::set_state(&mut con, &modified_state).await;
    }
}

async fn process_plan_tick(
    sp_id: &str,
    mut con: MultiplexedConnection,
    model: &Model,
    state: &State,
    logging_tx: mpsc::Sender<LogMsg>,
    log_target: &str,
) -> State {
    let mut new_state = state.clone();
    let planner_state =
        state.get_string_or_default_to_unknown(&format!("{}_planner_state", sp_id), &log_target);

    let goal_state = state
        .get_string_or_default_to_unknown(&format!("{}_current_goal_state", sp_id), &log_target);

    let mut plan_state_str =
        state.get_string_or_default_to_unknown(&format!("{}_plan_state", sp_id), &log_target);
    let mut plan_current_step =
        state.get_int_or_default_to_zero(&format!("{}_plan_current_step", sp_id), &log_target);
    let plan_of_sp_values =
        state.get_array_or_default_to_empty(&format!("{}_plan", sp_id), &log_target);

    let plan: Vec<String> = plan_of_sp_values
        .iter()
        .filter(|val| val.is_string())
        .map(|y| y.to_string())
        .collect();

    // Operations ready to be removed from the state
    let mut terminated_operations = vec![];

    match PlanState::from_str(&plan_state_str) {
        PlanState::Initial => {
            if planner_state == PlannerState::Found.to_string() {
                plan_state_str = PlanState::Executing.to_string();
                plan_current_step = 0;
            }
        }
        PlanState::Executing => {
            if let Some(op_name) = plan.get(plan_current_step as usize) {
                match model
                    .operations
                    .iter()
                    .find(|op| op_name.starts_with(&op.name))
                {
                    Some(operation) => {
                        let mut uq_operation = operation.clone();
                        uq_operation.name = op_name.to_owned();
                        new_state = running::process_operation::process_operation(
                            &sp_id,
                            new_state,
                            &uq_operation,
                            OperationProcessingType::Planned,
                            Some(&mut plan_current_step),
                            Some(&mut plan_state_str),
                            logging_tx,
                            log_target,
                        )
                        .await;

                        let operation_state = new_state.get_string_or_default_to_unknown(
                            &format!("{}", uq_operation.name),
                            &log_target,
                        );

                        let op = uq_operation.clone();
                        match OperationState::from_str(&operation_state) {
                            OperationState::Terminated(_) => {
                                let mut terminated_operations_meta = vec![];
                                // for op in &terminated_operations {
                                    terminated_operations_meta.push(format!("{}_information", op.name));
                                    terminated_operations_meta
                                        .push(format!("{}_failure_retry_counter", op.name));
                                    terminated_operations_meta
                                        .push(format!("{}_timeout_retry_counter", op.name));
                                    terminated_operations_meta
                                        .push(format!("{}_elapsed_executing_ms", op.name));
                                    terminated_operations_meta
                                        .push(format!("{}_elapsed_disabled_ms", op.name));
                                // }
                                StateManager::remove_sp_values(&mut con, &terminated_operations)
                                    .await;
                                StateManager::remove_sp_values(
                                    &mut con,
                                    &terminated_operations_meta,
                                )
                                .await;
                                // terminated_operations.push(uq_operation.name.clone());
                            }
                            _ => (),
                        };
                    }
                    None => {
                        log::error!("Operation '{}' not found in model!", op_name);
                        plan_state_str = PlanState::Failed.to_string();
                    }
                }
            } else {
                plan_state_str = PlanState::Completed.to_string();
            }
        }
        // Maybe I also have to reset all operation here...?
        _ => {
            // new_state = reset_all_operations(&new_state, model);
            let mut terminated_operations_meta = vec![];
            for op in &terminated_operations {
                terminated_operations_meta.push(format!("{}_information", op));
                terminated_operations_meta.push(format!("{}_failure_retry_counter", op));
                terminated_operations_meta.push(format!("{}_timeout_retry_counter", op));
                terminated_operations_meta.push(format!("{}_elapsed_executing_ms", op));
                terminated_operations_meta.push(format!("{}_elapsed_disabled_ms", op));
            }
            StateManager::remove_sp_values(&mut con, &terminated_operations).await;
            StateManager::remove_sp_values(&mut con, &terminated_operations_meta).await;
        }
    }

    new_state = new_state
        .update(
            &format!("{}_plan_state", sp_id),
            plan_state_str.to_spvalue(),
        )
        .update(&format!("{}_plan", sp_id), plan.to_spvalue())
        .update(
            &format!("{}_planner_state", sp_id),
            planner_state.to_spvalue(),
        )
        .update(
            &format!("{}_current_goal_state", sp_id),
            goal_state.to_spvalue(),
        )
        .update(
            &format!("{}_plan_current_step", sp_id),
            plan_current_step.to_spvalue(),
        );

    new_state
}
