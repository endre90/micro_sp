use std::collections::HashMap;
use std::env;

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



// MArtin: If you have more than one command for redis to do, use a pipeline to group commands together

// put this in another process that we can trigger from outside to reconnect if dsconnected
pub async fn redis_state_manager(mut receiver: mpsc::Receiver<StateManagement>, state: State) {
    let mut con = {
        let mut interval = interval(Duration::from_millis(100));
        let mut error_tracker;
        let mut error_value = 0;
        let mut error: String;
         // Read hostname and port from environment variables
        let redis_host = env::var("REDIS_HOST").unwrap_or_else(|_| "redis".to_string()); // Default to 'redis'
        let redis_port = env::var("REDIS_PORT").unwrap_or_else(|_| "6379".to_string());
        let redis_addr = format!("redis://{}:{}", redis_host, redis_port);
        'connect: loop {
            match Client::open(redis_addr.clone()) {
                Ok(redis_client) => match redis_client.get_multiplexed_async_connection().await {
                    Ok(redis_connection) => {
                        log::info!(target: &&format!("redis_state_manager"), "Redis connection established. ");
                        break 'connect redis_connection;
                    }
                    Err(e) => {
                        error_tracker = 2;
                        error = e.to_string();
                    }
                },
                Err(e) => {
                    error_tracker = 3;
                    error = e.to_string();
                }
            }
            if error_value != error_tracker {
                error_value = error_tracker;
                match error_value {
                    0 => {
                        log::warn!(target: &&format!("redis_state_manager"), "Waiting for a Redis connection.")
                    }
                    2 => {
                        log::error!(target: &&format!("redis_state_manager"), "Cannot connect to Redis with error: {}.", error)
                    }
                    3 => {
                        log::error!(target: &&format!("redis_state_manager"), "Cannot connect to Redis with error: {}.", error)
                    }
                    _ => unreachable!(),
                }
            }

            interval.tick().await;
        }
    };

    for (var, assignment) in state.state.clone() {
        if let Err(e) = con
            .set::<_, String, String>(&var, serde_json::to_string(&assignment.val).unwrap())
            .await
        {
            log::error!(target: &&format!("redis_state_manager"), "Failed to set initial value of {} with error {}.", var, e)
        }
    }

    log::info!(target: &&format!("redis_state_manager"), "Online.");

    let mut old_state = state.clone();
    let mut error_tracker = 0;
    let mut error_value = 0;
    let mut error = "".to_string();
    while let Some(command) = receiver.recv().await {
        match command {
            StateManagement::GetState(response_sender) => {
                match con.keys::<&str, Vec<String>>("*").await {
                    Ok(keys) => match con
                        .mget::<&Vec<std::string::String>, Vec<Option<String>>>(&keys)
                        .await
                    {
                        Ok(values) => {
                            let mut map: HashMap<String, SPAssignment> = HashMap::new();
                            for (key, maybe_value) in keys.into_iter().zip(values.into_iter()) {
                                if state.contains(&key) {
                                    // Only get state that is locally tracked
                                    if let Some(value) = maybe_value {
                                        let var = state.get_assignment(&key).var;
                                        let new_assignment = SPAssignment::new(
                                            var,
                                            serde_json::from_str(&value).unwrap(),
                                        );
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
                            error_tracker = 1;
                            error = e.to_string();
                            let _ = response_sender.send(old_state.clone());
                        }
                    },

                    Err(e) => {
                        error_tracker = 2;
                        error = e.to_string();
                        let _ = response_sender.send(old_state.clone());
                    }
                }
            }

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
                            error_tracker = 3;
                            let _ = response_sender.send(old_state.get_value(&var));
                        }
                    },
                    Err(e) => {
                        error_tracker = 4;
                        error = format!("Failed to get variable {} with error: {}.", var, e);
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
                        error_tracker = 5;
                        error = format!("Failed to set variable {} with error: {}.", var, e);
                    }
                }
            }

            StateManagement::Set((var, val)) => {
                if let Err(e) = con
                    .set::<_, String, Value>(&var, serde_json::to_string(&val).unwrap())
                    .await
                {
                    error_tracker = 6;
                    error = format!("Failed to set variable {} with error: {}.", var, e);
                }
            }
        }

        if error_value != error_tracker {
            error_value = error_tracker;
            match error_value {
                1 => {
                    log::error!(target: &&format!("redis_state_manager"), "Failed to get keys with error: {}'.", error)
                }
                2 => {
                    log::error!(target: &&format!("redis_state_manager"), "Failed to get keys with error: {}'.", error)
                }
                3 => {
                    log::error!(target: &&format!("redis_state_manager"), "Variable doesn't exist in Redis.")
                }
                4 => log::error!(target: &&format!("redis_state_manager"), "{}", error),
                5 => log::error!(target: &&format!("redis_state_manager"), "{}", error),
                6 => log::error!(target: &&format!("redis_state_manager"), "{}", error),
                _ => unreachable!(),
            }
        }
    }
}


/// Instead of sharing the state with Arc<Mutex<State>>, use a buffer of state read/write requests.
pub async fn state_manager_no_redis(mut receiver: mpsc::Receiver<StateManagement>, mut state: State) {
    while let Some(command) = receiver.recv().await {
        match command {
            StateManagement::GetState(response_sender) => {
                let _ = response_sender.send(state.clone());
            }
            StateManagement::Get((var, response_sender)) => {
                let _ = response_sender.send(state.get_value(&var));
            }
            StateManagement::SetPartialState(partial_state) => {
                for (var, assignment) in partial_state.state {
                    state = state.update(&var, assignment.val)
                }
            }
            StateManagement::Set((var, new_val)) => {
                state = state.update(&var, new_val);
            }
        }
    }
}
