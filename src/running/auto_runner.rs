use crate::{
    ConnectionManager, Model, State, Transition, handle_redis_error, redis_get_state_for_keys,
    redis_set_state,
};
use redis::{AsyncCommands, aio::MultiplexedConnection};
use std::{sync::Arc, time::Duration};
use tokio::time::interval;

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

    // let last_known_state: Arc<RwLock<Option<State>>> = Arc::new(RwLock::new(None));

    loop {
        interval.tick().await;
        let mut con = connection_manager.get_connection().await;

        if let Err(e) = con.set::<_, _, ()>("heartbeat", "alive").await {
            handle_redis_error(&e, &log_target, connection_manager).await;
            continue;
        }
        let state = match redis_get_state_for_keys(&mut con, &keys).await {
            Some(s) => s,
            None => continue,
        };

        for t in &model.auto_transitions {
            process_single_transition(&mut con, t, &state, &log_target).await;
        }
    }
}

// Letest experiment
// pub async fn auto_transition_runner(
//     name: &str,
//     model: &Model,
//     connection_manager: &Arc<ConnectionManager>,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let mut interval = interval(Duration::from_millis(100));
//     let model = model.clone();
//     let log_target = format!("{}_auto_runner", name);

//     log::info!(target: &log_target, "Online.");
//     let keys: Vec<String> = model
//         .auto_transitions
//         .iter()
//         .flat_map(|t| t.get_all_var_keys())
//         .collect();

//     loop {
//         interval.tick().await;
//         let mut con = connection_manager.get_connection().await;

//         // Attempt the Redis command
//         let result: RedisResult<()> = con.set("heartbeat", "alive").await;

//         if let Err(e) = result {
//             if e.is_io_error() {
//                 log::error!(target: &log_target, "Redis command failed. Triggering reconnect.");
//                 connection_manager.reconnect().await;
//             } else {
//                 log::error!(target: &log_target, "An unexpected Redis error occurred: {}", e);
//             }
//         } else {
//             if let Some(state) = redis_get_state_for_keys(&mut con, &keys).await {
//                 for t in &model.auto_transitions {
//                     if !t.to_owned().eval_running(&state) {
//                         continue;
//                     }

//                     let new_state = t.to_owned().take_running(&state);
//                     log::info!(
//                         target: &log_target,
//                         "Executed auto transition: '{}'.", t.name
//                     );

//                     let modified_state = state.get_diff_partial_state(&new_state);
//                     redis_set_state(&mut con, modified_state).await;
//                 }
//             }
//         }
//     }
// }

// Automatic transitions should be taken as soon as their guard becomes true.
// OLD
// pub async fn auto_transition_runner(
//     name: &str,
//     model: &Model,
//     command_sender: mpsc::Sender<StateManagement>,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let mut interval = interval(Duration::from_millis(100));
//     let model = model.clone();

//     log::info!(target: &&format!("{}_auto_runner", name), "Online.");

//     loop {
//         let (response_tx, response_rx) = oneshot::channel();
//         command_sender.send(StateManagement::GetState(response_tx)).await?;
//         let state = response_rx.await?;

//         for t in &model.auto_transitions {
//             if t.clone().eval_running(&state) {
//                 let new_state = t.clone().take_running(&state);
//                 log::info!(target: &&format!("{}_auto_runner", name), "Executed auto transition: '{}'.", t.name);

//                 let modified_state = state.get_diff_partial_state(&new_state);
//                 command_sender
//                     .send(StateManagement::SetPartialState(modified_state))
//                     .await?;
//             }
//         }
//         interval.tick().await;
//     }
// }

// NEW
// pub async fn auto_transition_runner(
//     name: &str,
//     model: &Model,
//     con: &mut MultiplexedConnection
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let mut interval = interval(Duration::from_millis(100));
//     let model = model.clone();

//     log::info!(target: &&format!("{}_auto_runner", name), "Online.");

//     loop {
//         match redis_get_state(con).await {
//             Some(state) => {for t in &model.auto_transitions {
//                 if t.clone().eval_running(&state) {
//                     let new_state = t.clone().take_running(&state);
//                     log::info!(target: &&format!("{}_auto_runner", name), "Executed auto transition: '{}'.", t.name);

//                     let modified_state = state.get_diff_partial_state(&new_state);
//                     redis_set_state(con, modified_state).await
//                 }
//             }}
//             None => ()
//         }
//         interval.tick().await;
//     }
// }

// Run operations automatically without a planner.
// Taken as soon as the guard becomes true.
// pub async fn auto_operation_runner(
//     name: &str,
//     model: &Model,
//     command_sender: mpsc::Sender<StateManagement>,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let mut interval = interval(Duration::from_millis(100));
//     let model = model.clone();
//     loop {
//         let (response_tx, response_rx) = oneshot::channel();
//         command_sender
//             .send(StateManagement::GetState(response_tx))
//             .await?;
//         let state = response_rx.await?;

//         for o in &model.operations {
//             if o.eval_running(&state) {
//                 let new_state = o.start_running(&state);
//                 log::info!(target: &&format!("{}_auto_runner", name), "Started auto operation: '{}'.", o.name);

//                 let modified_state = state.get_diff_partial_state(&new_state);
//                 command_sender
//                     .send(StateManagement::SetPartialState(modified_state))
//                     .await?;
//             } else if o.can_be_completed(&state) {
//                 let new_state = o.complete_running(&state);
//                 log::info!(target: &&format!("{}_auto_runner", name), "Completed auto operation: '{}'.", o.name);
//                 let modified_state = state.get_diff_partial_state(&new_state);
//                 command_sender
//                     .send(StateManagement::SetPartialState(modified_state))
//                     .await?;
//             }
//         }
//         interval.tick().await;
//     }
// }
