use crate::{
    running::process_operation::{OperationProcessingType, process_operation},
    *,
};
use chrono::Utc;
// use rand::seq::IndexedRandom;
use redis::aio::MultiplexedConnection;
use std::{sync::Arc, time::Duration};
use tokio::{sync::mpsc, time::interval};

// Add automatic operations here as well that finish immediatelly, god for setting some values, triggering robot moves etc.
pub static TRANSITION_RUNNER_TICK_INTERVAL_MS: u64 = 50;

async fn process_transition(
    con: &mut MultiplexedConnection,
    transition: &Transition,
    state: &State,
    logging_tx: mpsc::Sender<LogMsg>,
    log_target: &str,
) {
    if !transition.to_owned().eval(state, &log_target) {
        return;
    }

    let unique_id = nanoid::nanoid!(10, &NANOID_ALPHABET);
    let mut transition = transition.clone();
    transition.name = format!("{}_{}", transition.name, unique_id);

    let new_state = transition.to_owned().take(state, &log_target);
    log::info!(target: &log_target, "Executed auto transition: '{}'.", transition.name);

    let transition_msg = TransitionMsg {
        transition_name: transition.name.clone(),
        timestamp: Utc::now(),
        severity: log::Level::Info,
        log: format!("Executed."),
    };
    let log_msg = LogMsg::TransitionMsg(transition_msg);
    match logging_tx.send(log_msg).await {
        Ok(()) => (),
        Err(e) => log::error!(target: &log_target, "Failed to send logging with: {e}."),
    }

    let modified_state = state.get_diff_partial_state(&new_state);
    StateManager::set_state(con, &modified_state).await;
}

pub async fn auto_transition_runner(
    name: &str,
    model: &Model,
    connection_manager: &Arc<ConnectionManager>,
    logging_tx: mpsc::Sender<LogMsg>,
) -> Result<(), Box<dyn std::error::Error>> {
    initialize_env_logger();
    let mut interval = interval(Duration::from_millis(TRANSITION_RUNNER_TICK_INTERVAL_MS));
    let model = model.clone();
    let log_target = format!("{}_auto_transition_runner", name);
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
            process_transition(&mut con, t, &state, logging_tx.clone(), &log_target).await;
        }
    }
}

// pub async fn auto_operation_runner(
//     sp_id: &str,
//     model: &Model,
//     logging_tx: mpsc::Sender<LogMsg>,
//     connection_manager: &Arc<ConnectionManager>,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     initialize_env_logger();
//     let mut interval = interval(Duration::from_millis(OPERAION_RUNNER_TICK_INTERVAL_MS));
//     let model = model.clone();
//     let log_target = format!("{}_auto_operation_runner", sp_id);

//     let mut active_op: Option<Operation> = None;
//     let mut terminated_operations: Vec<String> = vec!();

//     loop {
//         interval.tick().await;
//         if let Err(_) = connection_manager.check_redis_health(&log_target).await {
//             continue;
//         }
//         let mut con = connection_manager.get_connection().await;
//         let state = match StateManager::get_full_state(&mut con).await {
//             Some(s) => s,
//             None => continue,
//         };

//         let mut enabled_operations = vec![];
//         for o in &model.auto_operations {
//             if o.eval(&state, &log_target) {
//                 enabled_operations.push(o);
//             }
//         }

//         match active_op.clone() {
//             None => {
//                 let maybe_random_op = {
//                     let mut rng = rand::rng();
//                     enabled_operations.choose(&mut rng).cloned()
//                 };
//                 if let Some(op) = maybe_random_op {
//                     let mut new_state = state.clone();
//                     let unique_id = nanoid::nanoid!(10, &NANOID_ALPHABET);
//                     let unique_op_id = format!("{}_{}", op.name.clone(), unique_id);
//                     let mut op_mut = op.clone();
//                     op_mut.name = unique_op_id.clone();
//                     active_op = Some(op_mut.clone());

//                     new_state = add_operation_meta_tracking_variables(
//                         &vec![unique_op_id.clone()],
//                         &new_state,
//                         false,
//                         &log_target
//                     );
//                     new_state =
//                         add_operation_state_tracking_variable(&vec![unique_op_id], &new_state, &log_target);
//                     let modified_state = state.get_diff_partial_state_and_add_missing(&new_state);
//                     StateManager::set_state(&mut con, &modified_state).await;
//                 }
//             }
//             Some(current_active_op) => {
//                 let mut new_state = state.clone();
//                 new_state = process_operation(
//                     &sp_id,
//                     new_state,
//                     &current_active_op,
//                     OperationProcessingType::Automatic,
//                     None,
//                     None,
//                     logging_tx.clone(),
//                     &log_target,
//                     // &mut terminated_operations
//                 )
//                 .await;
//                 let operation_state = new_state.get_string_or_default_to_unknown(
//                     &format!("{}", current_active_op.name),
//                     &log_target,
//                 );

//                 match OperationState::from_str(&operation_state) {
//                     OperationState::Terminated(_) => {
//                         terminated_operations.push(current_active_op.name.clone());
//                         active_op = None;
//                     }
//                     _ => (),
//                 };

//                 let modified_state = state.get_diff_partial_state_and_add_missing(&new_state);
//                 StateManager::set_state(&mut con, &modified_state).await;
//             }
//         }

//         let mut terminated_operations_meta = vec![];
//         for op in &terminated_operations {
//             terminated_operations_meta.push(format!("{}_information", op));
//             terminated_operations_meta.push(format!("{}_failure_retry_counter", op));
//             terminated_operations_meta.push(format!("{}_timeout_retry_counter", op));
//             terminated_operations_meta.push(format!("{}_elapsed_executing_ms", op));
//             terminated_operations_meta.push(format!("{}_elapsed_disabled_ms", op));
//         }
//         StateManager::remove_sp_values(&mut con, &terminated_operations).await;
//         StateManager::remove_sp_values(&mut con, &terminated_operations_meta).await;
//     }
// }


pub async fn auto_operation_runner(
    sp_id: &str,
    model: &Model,
    logging_tx: mpsc::Sender<LogMsg>,
    connection_manager: &Arc<ConnectionManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    initialize_env_logger();
    let mut interval = interval(Duration::from_millis(OPERAION_RUNNER_TICK_INTERVAL_MS));
    let model = model.clone();
    let log_target = format!("{}_auto_operation_runner", sp_id);

    let mut active_ops: Vec<Operation> = vec![];
    let mut terminated_operations: Vec<String> = vec!();

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

        let mut new_state = state.clone();
        let mut new_op_ids = vec![];

        println!("ACTIVE: {:?}", active_ops.iter().map(|x| x.name.clone()).collect::<Vec<String>>());
        for op in &model.auto_operations {
            println!("For in...{:?}", op.name);
            if op.eval(&state, &log_target) {
                println!("Eval passed: {:?}", op.name);
                let prefix = format!("{}_", op.name);
                if !active_ops.iter().any(|a| a.name.starts_with(&prefix)) {
                    println!("A: {}", op.name);
                    let unique_id = nanoid::nanoid!(10, &NANOID_ALPHABET);
                    let unique_op_id = format!("{}{}", prefix, unique_id);
                    let mut op_mut = op.clone();
                    op_mut.name = unique_op_id.clone();
                    active_ops.push(op_mut);
                    new_op_ids.push(unique_op_id);
                } else {
                    println!("B: {}", op.name);
                }
            }
        }

        if !new_op_ids.is_empty() {
            new_state = add_operation_meta_tracking_variables(
                &new_op_ids,
                &new_state,
                false,
                &log_target,
            );
            new_state = add_operation_state_tracking_variable(&new_op_ids, &new_state, &log_target);
        }

        let mut next_active_ops = vec![];
        for current_active_op in active_ops {
            new_state = process_operation(
                &sp_id,
                new_state,
                &current_active_op,
                OperationProcessingType::Automatic,
                None,
                None,
                logging_tx.clone(),
                &log_target,
            )
            .await;

            let operation_state = new_state.get_string_or_default_to_unknown(
                &format!("{}", current_active_op.name),
                &log_target,
            );

            match OperationState::from_str(&operation_state) {
                OperationState::Terminated(_) => {
                    terminated_operations.push(current_active_op.name.clone());
                }
                _ => next_active_ops.push(current_active_op),
            };
        }

        active_ops = next_active_ops;

        let modified_state = state.get_diff_partial_state_and_add_missing(&new_state);
        StateManager::set_state(&mut con, &modified_state).await;

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

        terminated_operations.clear();
    }
}