use crate::{
    ConnectionManager, Model, State, Transition, handle_redis_error, redis_get_state_for_keys,
    redis_set_state, restore_state_from_snapshot,
};
use redis::{AsyncCommands, aio::MultiplexedConnection};
use std::{sync::Arc, time::Duration};
use tokio::{sync::RwLock, time::interval};

async fn process_single_transition(
    con: &mut MultiplexedConnection,
    transition: &Transition,
    state: &State,
    log_target: &str,
) {
    if !transition.to_owned().eval_running(state) {
        return;
    }

    let new_state = transition.to_owned().take_running(state);
    log::info!(target: log_target, "Executed auto transition: '{}'.", transition.name);

    let modified_state = state.get_diff_partial_state(&new_state);
    redis_set_state(con, modified_state).await;
}

pub async fn auto_transition_runner(
    name: &str,
    model: &Model,
    connection_manager: &Arc<ConnectionManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = interval(Duration::from_millis(100));
    let model = model.clone();
    let log_target = format!("{}_auto_runner", name);
    let keys: Vec<String> = model
        .auto_transitions
        .iter()
        .flat_map(|t| t.get_all_var_keys())
        .collect();

    log::info!(target: &log_target, "Online.");

    let last_known_state: Arc<RwLock<Option<State>>> = Arc::new(RwLock::new(None));

    loop {
        interval.tick().await;
        let mut con = connection_manager.get_connection().await;

        if let Err(e) = con.set::<_, _, ()>("heartbeat", "alive").await {
            handle_redis_error(&e, &log_target, connection_manager).await;
            continue;
        }
        match redis_get_state_for_keys(&mut con, &keys).await {
            Some(current_state) => {
                *last_known_state.write().await = Some(current_state.clone());
                for t in &model.auto_transitions {
                    process_single_transition(&mut con, t, &current_state, &log_target).await;
                }
            }
            None => restore_state_from_snapshot(&mut con, &last_known_state, &log_target).await,
        }
    }
}