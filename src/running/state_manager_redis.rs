use std::collections::HashMap;

use crate::*;
use redis::{AsyncCommands, Client, Value};
use tokio::sync::{mpsc, oneshot};
use tokio::time::{interval, Duration};

/// Available commands that the async tasks can ask from the state manager.
pub enum StateManagement {
    GetState(oneshot::Sender<State>),
    Get((String, oneshot::Sender<SPValue>)),
    SetPartialState(State),
    Set((String, SPValue)),
}

pub async fn redis_state_manager(mut receiver: mpsc::Receiver<StateManagement>, state: State) {
    let mut interval = interval(Duration::from_millis(100));
    let mut con = 'connect: loop {
        match Client::open("redis://127.0.0.1/") {
            Ok(redis_client) => match redis_client.get_multiplexed_async_connection().await {
                Ok(redis_connection) => break 'connect redis_connection,
                Err(e) => {
                    log::error!(target: &&format!("redis_state_manager"), "Cannot connect to Redis with error: {}.", e)
                }
            },
            Err(e) => {
                log::error!(target: &&format!("redis_state_manager"), "Cannot connect to Redis with error: {}.", e)
            }
        }
        interval.tick().await;
    };

    for (var, assignment) in state.state.clone() {
        if let Err(e) = con
            .set::<_, String, String>(&var, serde_json::to_string(&assignment.val).unwrap())
            .await
        {
            log::error!(target: &&format!("redis_state_manager"), "Failed to set initial value of {} with error {}.", var, e)
        }
    }

    let mut old_state = state.clone();
    while let Some(command) = receiver.recv().await {
        match command {
            StateManagement::GetState(response_sender) => match con
                .keys::<&str, Vec<String>>("*")
                .await
            {
                Ok(keys) => {
                    let values: Vec<Option<String>> = con
                        .mget(&keys)
                        .await
                        .expect("Failed to get values for all keys.");

                    let mut map: HashMap<String, SPAssignment> = HashMap::new();
                    for (key, maybe_value) in keys.into_iter().zip(values.into_iter()) {
                        if state.contains(&key) {
                            // Only get state that is locally tracked
                            if let Some(value) = maybe_value {
                                let var = state.get_assignment(&key).var;
                                let new_assignment =
                                    SPAssignment::new(var, serde_json::from_str(&value).unwrap());
                                map.insert(key, new_assignment);
                            }
                        }
                    }

                    // we want to keep updating a copy of a state so that we can maintain it if
                    // the connection to Redis gets disrupted
                    let new_state = State { state: map };
                    old_state = new_state;
                    let _ = response_sender.send(old_state.clone());
                }
                Err(e) => {
                    log::error!(target: &&format!("redis_state_manager"), "Failed to get keys with: '{e}'.");
                    let _ = response_sender.send(old_state.clone());
                }
            },

            StateManagement::Get((var, response_sender)) => {
                match con.get::<_, Option<String>>(&var).await {
                    Ok(val) => match val {
                        Some(redis_value) => {
                            old_state =
                                old_state.update(&var, serde_json::from_str(&redis_value).unwrap());
                            let _ =
                                response_sender.send(serde_json::from_str(&redis_value).unwrap());
                        }
                        None => {
                            log::error!(target: &&format!("redis_state_manager"), "Variable doesn't exist in Redis.");
                            let _ = response_sender.send(old_state.get_value(&var));
                        }
                    },
                    Err(e) => {
                        log::error!(target: &&format!("redis_state_manager"), "Failed to get variable {} with error: {}.", var, e);
                        let _ = response_sender.send(old_state.get_value(&var));
                    }
                }
            }

            StateManagement::SetPartialState(partial_state) => {
                for (var, assignment) in partial_state.state {
                    // state = state.update(&var, assignment.val.clone());
                    if let Err(e) = con
                        .set::<_, String, Value>(
                            &var,
                            serde_json::to_string(&assignment.val).unwrap(),
                        )
                        .await
                    {
                        log::error!(target: &&format!("redis_state_manager"), "Failed to set variable {} with error: {}.", var, e);
                    }
                }
            }

            StateManagement::Set((var, val)) => {
                if let Err(e) = con
                    .set::<_, String, Value>(&var, serde_json::to_string(&val).unwrap())
                    .await
                {
                    log::error!(target: &&format!("redis_state_manager"), "Failed to set variable {} with error: {}.", var, e);
                }
            }
        }
    }
}
