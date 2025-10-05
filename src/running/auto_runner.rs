use crate::{
    ConnectionManager, Model, State, StateManager, Transition,
    running::process_operation::{OperationProcessingType, process_operation},
};
use redis::aio::MultiplexedConnection;
use std::{sync::Arc, time::Duration};
use tokio::time::interval;

// Add automatic operations here as well that finish immediatelly, god for setting some values, triggering robot moves etc.

async fn process_transition(
    con: &mut MultiplexedConnection,
    transition: &Transition,
    state: &State,
    log_target: &str,
) {
    if !transition.to_owned().eval_running(state, &log_target) {
        return;
    }

    let new_state = transition.to_owned().take_running(state, &log_target);
    log::info!(target: &log_target, "Executed auto transition: '{}'.", transition.name);

    let modified_state = state.get_diff_partial_state(&new_state);
    StateManager::set_state(con, &modified_state).await;
}

pub async fn auto_transition_runner(
    name: &str,
    model: &Model,
    connection_manager: &Arc<ConnectionManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = interval(Duration::from_millis(100));
    let model = model.clone();
    let log_target = format!("{}_auto_trans_runner", name);
    let keys: Vec<String> = model
        .auto_transitions
        .iter()
        .flat_map(|t| t.get_all_var_keys())
        .collect();

    log::info!(target: &log_target, "Online.");

    // let last_known_state: Arc<RwLock<Option<State>>> = Arc::new(RwLock::new(None));

    let mut con = connection_manager.get_connection().await;
    loop {
        interval.tick().await;
        if let Err(_) = connection_manager.check_redis_health(&log_target).await {
            continue;
        }
        let state = match StateManager::get_state_for_keys(&mut con, &keys).await {
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
    let mut interval = interval(Duration::from_millis(100));
    let model = model.clone();
    let log_target = format!("{}_auto_op_runner", name);
    let con = connection_manager.get_connection().await;

    let keys: Vec<String> = model
        .auto_transitions
        .iter()
        .flat_map(|t| t.get_all_var_keys())
        .collect();

    loop {
        interval.tick().await;
        if let Err(_) = connection_manager.check_redis_health(&log_target).await {
            continue;
        }
        let state = match StateManager::get_state_for_keys(&mut con.clone(), &keys).await {
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
