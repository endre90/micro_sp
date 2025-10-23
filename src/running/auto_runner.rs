use crate::{
    running::process_operation::{process_operation, OperationProcessingType}, ConnectionManager, Model, State, StateManager, Transition, OPERAION_RUNNER_TICK_INTERVAL_MS
};
use redis::aio::MultiplexedConnection;
use std::{sync::Arc, time::Duration};
use tokio::time::interval;

// Add automatic operations here as well that finish immediatelly, god for setting some values, triggering robot moves etc.
pub static TRANSITION_RUNNER_TICK_INTERVAL_MS: u64 = 100;

async fn process_transition(
    con: &mut MultiplexedConnection,
    transition: &Transition,
    state: &State,
    log_target: &str,
) {
    if !transition.to_owned().eval(state, &log_target) {
        return;
    }

    let new_state = transition.to_owned().take(state, &log_target);
    log::info!(target: &log_target, "Executed auto transition: '{}'.", transition.name);

    let modified_state = state.get_diff_partial_state(&new_state);
    StateManager::set_state(con, &modified_state).await;
}

pub async fn auto_transition_runner(
    name: &str,
    model: &Model,
    connection_manager: &Arc<ConnectionManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = interval(Duration::from_millis(TRANSITION_RUNNER_TICK_INTERVAL_MS));
    let model = model.clone();
    let log_target = format!("{}_auto_tr_runner", name);
    let keys: Vec<String> = model
        .auto_transitions
        .iter()
        .flat_map(|t| t.get_all_var_keys())
        .collect();

    log::info!(target: &log_target, "Online.");

    loop {
        interval.tick().await;
        if let Err(_) = connection_manager.check_redis_health(&log_target).await {
            continue;
        }
        let mut con = connection_manager.get_connection().await;
        let state = match StateManager::get_state_for_keys(&mut con, &keys, &log_target).await {
            Some(s) => s,
            None => continue,
        };

        for t in &model.auto_transitions {
            process_transition(&mut con, t, &state, &log_target).await;
        }
    }
}

pub async fn auto_operation_runner(
    name: &str,
    model: &Model,
    connection_manager: &Arc<ConnectionManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = interval(Duration::from_millis(OPERAION_RUNNER_TICK_INTERVAL_MS));
    let model = model.clone();
    let log_target = format!("{}_auto_op_runner", name);

    let keys: Vec<String> = model
        .auto_operations
        .iter()
        .flat_map(|t| t.get_all_var_keys())
        .collect();

    let op_names: Vec<String> = model.auto_operations.iter().map(|o| o.name.to_string()).collect();

    loop {
        interval.tick().await;
        if let Err(_) = connection_manager.check_redis_health(&log_target).await {
            continue;
        }
        let con = connection_manager.get_connection().await;
        let state = match StateManager::get_state_for_keys(&mut con.clone(), &op_names, &log_target).await {
            Some(s) => s,
            None => continue,
        };

        for o in &model.auto_operations {
            process_operation(
                state.clone(),
                o,
                OperationProcessingType::Automatic,
                None,
                None,
                con.clone(),
                &log_target,
            )
            .await;
        }
    }
}
