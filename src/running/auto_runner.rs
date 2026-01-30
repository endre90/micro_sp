use crate::{
    running::process_operation::{OperationProcessingType, process_operation},
    *,
};
use chrono::Utc;
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
    let log_target = format!("{}_auto_trans_runner", name);
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

pub async fn auto_operation_runner(
    name: &str,
    model: &Model,
    logging_tx: mpsc::Sender<LogMsg>,
    connection_manager: &Arc<ConnectionManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = interval(Duration::from_millis(OPERAION_RUNNER_TICK_INTERVAL_MS));
    let log_target = format!("{}_auto_op_runner", name);

    // Currently running operation instances (template + uuid)
    let mut active_instances: Vec<Operation> = Vec::new();

    // Keys for triggers (Templates) AND running operations (Instances)
    let mut keys: Vec<String> = Vec::new();

    keys.extend(vec![format!("{}_dashboard_command", name)]);

    // Keys from Templates to check if we should start new ones
    for op in &model.auto_operations {
        keys.extend(op.get_all_var_keys());
        keys.push(op.name.clone());
    }

    println!("{:?}", keys);

    loop {
        interval.tick().await;
        if let Err(_) = connection_manager.check_redis_health(&log_target).await {
            continue;
        }
        let mut con = connection_manager.get_connection().await;

        // Keys from Active Instances to run them
        for op in &active_instances {
            keys.extend(op.get_all_var_keys());
        }

        let state = match StateManager::get_state_for_keys(&mut con, &keys, &log_target).await {
            Some(s) => s,
            None => continue,
        };
        let mut new_state = state.clone();

        for template in &model.auto_operations {
            // Check if this template should spawn a new instance
            // "Trigger is True" AND "No instance of this type is currently running"
            let is_already_running = active_instances
                .iter()
                .any(|i| i.name.starts_with(&template.name));

            if !is_already_running && template.eval(&state, &log_target) {
                let unique_id = nanoid::nanoid!(10, &NANOID_ALPHABET);
                let old_name = &template.name;
                let new_name = format!("{}_{}", old_name, unique_id);
                let mut new_instance = template.clone();
                new_instance.name = new_name.clone();

                let initial_state_key = v!(&&format!("{}", new_instance.name));
                new_state = new_state.add(assign!(
                    initial_state_key,
                    SPValue::String(StringOrUnknown::String(OperationState::Initial.to_string()))
                ));
                new_state = add_operation_tracking_variables(
                    &vec![new_instance.clone()],
                    &new_state,
                    false,
                );

                log::info!(target: &log_target, "Spawning unique operation {}.", new_instance.name);
                active_instances.push(new_instance);
            }
        }

        // We use a retain pattern to run and clean up in one pass
        let mut keep_indices = Vec::new();

        for (i, instance) in active_instances.iter().enumerate() {
            new_state = process_operation(
                &name,
                new_state,
                instance,
                OperationProcessingType::Automatic,
                None,
                None,
                logging_tx.clone(),
                &log_target,
            )
            .await;

            // Check if finished
            let op_state_str =
                new_state.get_string_or_default_to_unknown(&instance.name, &log_target);
            let op_state = OperationState::from_str(&op_state_str);

            match op_state {
                OperationState::Completed
                | OperationState::Failed
                | OperationState::Timedout
                | OperationState::Disabled => {
                    log::info!(target: &log_target, "Finished {}", instance.name);
                }
                _ => {
                    keep_indices.push(i);
                }
            }
        }

        // Retain only active operations
        active_instances = keep_indices
            .iter()
            .map(|&i| active_instances[i].clone())
            .collect();

        let modified_state = state.get_diff_partial_state(&new_state);
        if !modified_state.state.is_empty() {
            StateManager::set_state(&mut con, &modified_state).await;
        }
    }
}

// pub async fn auto_operation_runner_old(
//     name: &str,
//     model: &Model,
//     logging_tx: mpsc::Sender<LogMsg>,
//     connection_manager: &Arc<ConnectionManager>,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     initialize_env_logger();
//     let mut interval = interval(Duration::from_millis(OPERAION_RUNNER_TICK_INTERVAL_MS));
//     let model = model.clone();
//     let log_target = format!("{}_auto_op_runner", name);

//     let mut keys: Vec<String> = model
//         .auto_operations
//         .iter()
//         .flat_map(|o| o.get_all_var_keys())
//         .collect();

//     keys.extend(vec![format!("{}_dashboard_command", name)]);

//     keys.extend(
//         model
//             .auto_operations
//             .iter()
//             .flat_map(|op| {
//                 vec![
//                     format!("{}", op.name),
//                     format!("{}_information", op.name),
//                     format!("{}_failure_retry_counter", op.name),
//                     format!("{}_timeout_retry_counter", op.name),
//                     format!("{}_elapsed_executing_ms", op.name),
//                     format!("{}_elapsed_disabled_ms", op.name),
//                 ]
//             })
//             .collect::<Vec<String>>(),
//     );

//     loop {
//         interval.tick().await;
//         if let Err(_) = connection_manager.check_redis_health(&log_target).await {
//             continue;
//         }
//         let mut con = connection_manager.get_connection().await;
//         let state =
//             match StateManager::get_state_for_keys(&mut con.clone(), &keys, &log_target).await {
//                 Some(s) => s,
//                 None => continue,
//             };

//         let mut enabled_operations = vec![];
//         for o in &model.auto_operations {
//             if o.eval(&state, &log_target) {
//                 enabled_operations.push(o);
//             }
//         }

//         let mut new_state = state.clone();

//         let mut active_operations = Vec::new();
//         let mut pending_operations = Vec::new();
//         let mut terminated_operations = Vec::new();

//         for operation in &model.auto_operations {
//             let operation_state_str = new_state
//                 .get_string_or_default_to_unknown(&format!("{}", operation.name), &log_target);
//             match OperationState::from_str(&operation_state_str) {
//                 OperationState::Initial | OperationState::UNKNOWN => {
//                     pending_operations.push(operation);
//                 }
//                 OperationState::Executing
//                 | OperationState::Failed
//                 | OperationState::Timedout
//                 | OperationState::Disabled
//                 | OperationState::Completed => {
//                     active_operations.push(operation);
//                 }
//                 _ => {
//                     terminated_operations.push(operation);
//                 }
//             }
//         }

//         for operation in &active_operations {
//             new_state = process_operation(
//                 &name,
//                 new_state,
//                 operation,
//                 OperationProcessingType::Automatic,
//                 None,
//                 None,
//                 logging_tx.clone(),
//                 &log_target,
//             )
//             .await;
//         }

//         if active_operations.len() == 0 {
//             let mut enabled_pending_ops = Vec::new();
//             for op in pending_operations {
//                 if op.eval(&new_state, &log_target) {
//                     enabled_pending_ops.push(op);
//                 }
//             }

//             let maybe_random_op = {
//                 let mut rng = rand::rng();
//                 enabled_pending_ops.choose(&mut rng).cloned()
//             };

//             if let Some(random_operation) = maybe_random_op {
//                 new_state = process_operation(
//                     &name,
//                     new_state,
//                     random_operation,
//                     OperationProcessingType::Automatic,
//                     None,
//                     None,
//                     logging_tx.clone(),
//                     &log_target,
//                 )
//                 .await;
//             }
//         }

//         let modified_state = state.get_diff_partial_state(&new_state);
//         StateManager::set_state(&mut con, &modified_state).await;
//     }
// }
