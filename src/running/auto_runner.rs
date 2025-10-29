use crate::{
    ConnectionManager, Model, OPERAION_RUNNER_TICK_INTERVAL_MS, Operation, OperationState, State,
    StateManager, Transition,
    running::process_operation::{OperationProcessingType, process_operation},
};
use rand::prelude::*;
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

    let mut keys: Vec<String> = model
        .auto_operations
        .iter()
        .flat_map(|t| t.get_all_var_keys())
        .collect();

    keys.extend(
        model
            .auto_operations
            .iter()
            .flat_map(|op| {
                vec![
                    format!("{}", op.name),
                    format!("{}_information", op.name),
                    format!("{}_failure_retry_counter", op.name),
                    format!("{}_timeout_retry_counter", op.name),
                    format!("{}_elapsed_ms", op.name),
                ]
            })
            .collect::<Vec<String>>(),
    );

    loop {
        interval.tick().await;
        if let Err(_) = connection_manager.check_redis_health(&log_target).await {
            continue;
        }
        let mut con = connection_manager.get_connection().await;
        let state =
            match StateManager::get_state_for_keys(&mut con.clone(), &keys, &log_target).await {
                Some(s) => s,
                None => continue,
            };

        let mut enabled_operations = vec![];
        for o in &model.auto_operations {
            if o.eval(&state, &log_target) {
                enabled_operations.push(o);
            }
        }

        let maybe_random_op = {
            let mut rng = rand::rng();
            enabled_operations.choose(&mut rng).cloned()
        };

        let mut new_state = state.clone();

        let mut default_op = Operation::default();
        default_op.name = "asdf".to_string();

        //process all operations that are not in the initial state
        for o in &model.auto_operations {
            println!("{}, {}", o.name, o.state.to_string());
            if o.state == OperationState::Disabled
                || o.state == OperationState::Executing
                || o.state == OperationState::Bypassed
                || o.state == OperationState::Timedout
                || o.state == OperationState::Failed
                    && o.name != maybe_random_op.unwrap_or_else(|| &default_op).name
            {
                new_state = process_operation(
                    state.clone(),
                    o,
                    OperationProcessingType::Automatic,
                    None,
                    None,
                    con.clone(),
                    &log_target,
                )
                .await;

                // let modified_state = state.get_diff_partial_state(&new_state);
                // StateManager::set_state(&mut con, &modified_state).await;
            }
        }

        // process newly enabled operation
        match maybe_random_op {
            Some(random_operation) => {
                new_state = process_operation(
                    state.clone(),
                    random_operation,
                    OperationProcessingType::Automatic,
                    None,
                    None,
                    con.clone(),
                    &log_target,
                )
                .await;
            }
            None => {}
        }

        

        let modified_state = state.get_diff_partial_state(&new_state);
        StateManager::set_state(&mut con, &modified_state).await;
    }
}
