use crate::*;
use std::sync::Arc;
use tokio::time::{interval, Duration};

pub async fn planned_operation_runner(
    model: &Model,
    connection_manager: &Arc<ConnectionManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    let sp_id = &model.name;
    let log_target = format!("{}_operation_runner", sp_id);
    let mut interval = interval(Duration::from_millis(250));

    // Get only the relevant keys from the state
    log::info!(target: &log_target, "Online.");
    let mut keys: Vec<String> = model
        .operations
        .iter()
        .flat_map(|t| t.get_all_var_keys())
        .collect();

    // We also need some of the planner vars
    keys.extend(vec![
        format!("{}_planner_state", sp_id),
        format!("{}_plan_state", sp_id),
        format!("{}_plan_current_step", sp_id),
        format!("{}_plan", sp_id),
    ]);

    // And the vars to keep trask of operation states
    keys.extend(
        model
            .operations
            .iter()
            .flat_map(|op| {
                vec![
                    format!("{}", op.name),
                    format!("{}_information", op.name),
                    format!("{}_retry_counter", op.name),
                ]
            })
            .collect::<Vec<String>>(),
    );

    // let last_known_state: Arc<RwLock<Option<State>>> = Arc::new(RwLock::new(None));

    let mut con: redis::aio::MultiplexedConnection = connection_manager.get_connection().await;
    
    loop {
        interval.tick().await;
        if let Err(_) = connection_manager.check_redis_health(&log_target).await {
            continue;
        }
        let state = match StateManager::get_state_for_keys(&mut con, &keys).await {
            Some(s) => s,
            None => continue,
        };
        let con_clone = con.clone();
        let new_state = process_plan_tick(sp_id, con_clone, &model, &state, &log_target).await;
        let modified_state = state.get_diff_partial_state(&new_state);
        // StateManager::set_state(con, &modified_state).await;
        StateManager::set_state(&mut con, &modified_state).await;
    }
}

async fn process_plan_tick(sp_id: &str, con: redis::aio::MultiplexedConnection, model: &Model, state: &State, log_target: &str) -> State {
    let mut new_state = state.clone();
    let mut planner_state =
        state.get_string_or_default_to_unknown(&format!("{}_planner_state", sp_id), &log_target);

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

    match PlanState::from_str(&plan_state_str) {
        PlanState::Initial => {
            if planner_state == PlannerState::Found.to_string() {
                plan_state_str = PlanState::Executing.to_string();
                plan_current_step = 0;
            }
        }
        PlanState::Executing => {
            if let Some(op_name) = plan.get(plan_current_step as usize) {
                process_operation(
                    &mut new_state,
                    &mut plan_state_str,
                    &mut plan_current_step,
                    op_name,
                    model,
                    state,
                    con,
                    log_target,
                ).await;
            } else {
                plan_state_str = PlanState::Completed.to_string();
            }
        }
        PlanState::Failed | PlanState::Completed | PlanState::UNKNOWN => {
            plan_current_step = 0;
            new_state = reset_all_operations(&new_state, &model);
            plan_state_str = PlanState::Initial.to_string();
            planner_state = PlannerState::Ready.to_string();
        }
    }

    new_state = new_state
        .update(
            &format!("{}_plan_state", sp_id),
            plan_state_str.to_spvalue(),
        )
        .update(
            &format!("{}_planner_state", sp_id),
            planner_state.to_spvalue(),
        )
        .update(
            &format!("{}_plan_current_step", sp_id),
            plan_current_step.to_spvalue(),
        );

    new_state
}

async fn process_operation(
    new_state: &mut State,
    plan_state_str: &mut String,
    plan_current_step: &mut i64,
    op_name: &str,
    model: &Model,
    state: &State,
    mut con: redis::aio::MultiplexedConnection,
    log_target: &str,
) {
    let Some(operation) = model.operations.iter().find(|op| op.name == op_name) else {
        log::error!("Operation '{}' not found in model!", op_name);
        *plan_state_str = PlanState::Failed.to_string();
        return;
    };

    let operation_state_str =
        state.get_string_or_default_to_unknown(&format!("{}", operation.name), &log_target);

    let old_operation_information = state
        .get_string_or_default_to_unknown(&format!("{}_information", operation.name), &log_target);

    let mut operation_retry_counter =
        state.get_int_or_default_to_zero(&format!("{}_retry_counter", operation.name), &log_target);

    let mut new_op_info = old_operation_information.clone();

    match OperationState::from_str(&operation_state_str) {
        OperationState::Initial => {
            if operation.eval_running(state, &log_target) {
                *new_state = operation.start_running(new_state, &log_target);
                new_op_info = format!("Operation '{}' started.", operation.name);
            }
        }
        OperationState::Executing => {
            if operation.can_be_completed(state, &log_target) {
                *new_state = operation.complete_running(new_state, &log_target);
                new_op_info = format!("Operation '{}' completing.", operation.name);
            } else if operation.can_be_failed(state, &log_target) {
                *new_state = operation.fail_running(new_state, &log_target);
                new_op_info = format!("Operation '{}' failing.", operation.name);
            }
        }
        OperationState::Completed => {
            *new_state =
                new_state.update(&format!("{}_retry_counter", operation.name), 0.to_spvalue());
            StateManager::remove_sp_value(&mut con, &operation.name).await;
            // *new_state = operation.reinitialize_running(&new_state, &log_target);
            *plan_current_step += 1;
            new_op_info = format!("Operation '{}' completed.", operation.name);
        }
        OperationState::Failed => {
            if operation_retry_counter < operation.retries {
                operation_retry_counter += 1;
                *new_state = operation.retry_running(new_state, &log_target);
                *new_state = new_state.update(
                    &format!("{}_retry_counter", operation.name),
                    operation_retry_counter.to_spvalue(),
                );
                new_op_info = format!(
                    "Operation '{}' retrying ({}/{}).",
                    operation.name, operation_retry_counter, operation.retries
                );
            } else {
                *new_state = operation.unrecover_running(new_state, &log_target);
                new_op_info = format!("Operation '{}' failed. No retries left.", operation.name);
            }
        }
        OperationState::Timedout => {
            *new_state = operation.unrecover_running(new_state, &log_target);
            new_op_info = format!("Operation '{}' timed out.", operation.name);
        }
        OperationState::Unrecoverable => {
            *plan_state_str = PlanState::Failed.to_string();
            new_op_info = format!(
                "Operation '{}' is unrecoverable. Failing plan.",
                operation.name
            );
        }
        OperationState::UNKNOWN => {
            *new_state = operation.initialize_running(&new_state, &log_target);
        },
    }

    if new_op_info != old_operation_information {
        log::info!(target: &log_target, "{}", new_op_info);
    }

    *new_state = new_state.update(
        &format!("{}_information", operation.name),
        new_op_info.to_spvalue(),
    );
}