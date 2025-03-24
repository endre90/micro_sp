use std::collections::HashMap;

use crate::*;
use redis::{AsyncCommands, Client, Value};
use tokio::sync::{mpsc, oneshot};

/// Available commands that the async tasks can ask from the state manager.
pub enum StateManagement {
    GetState(oneshot::Sender<State>),
    Get((String, oneshot::Sender<SPValue>)),
    SetPartialState(State),
    Set((String, SPValue)),
}

pub async fn redis_state_manager(mut receiver: mpsc::Receiver<StateManagement>, state: State) {
    let redis_client =
        Client::open("redis://127.0.0.1/").expect("Failed to instantiate redis client.");
    let mut con = redis_client
        .get_multiplexed_async_connection()
        .await
        .expect("Failed to establish Redis connection.");

    // First populate the redis DB with the state.
    for (var, assignment) in state.state.clone() {
        if let Err(e) = con
            .set::<_, String, String>(&var, serde_json::to_string(&assignment.val).unwrap())
            .await
        {
            eprintln!("Failed to set {}: {:?}", var, e);
        }
    }

    while let Some(command) = receiver.recv().await {
        match command {
            StateManagement::GetState(response_sender) => {
                let keys: Vec<String> = con.keys("*").await.expect("Failed to get all keys.");

                let values: Vec<Option<String>> = con
                    .mget(&keys)
                    .await
                    .expect("Failed to get values for all keys.");

                let mut map: HashMap<String, SPAssignment> = HashMap::new();
                for (key, maybe_value) in keys.into_iter().zip(values.into_iter()) {
                    if let Some(value) = maybe_value {
                        let var = state.get_assignment(&key).var;
                        let new_assignment =
                            SPAssignment::new(var, serde_json::from_str(&value).unwrap());
                        map.insert(key, new_assignment);
                    }
                }

                let _ = response_sender.send(State { state: map });
            }

            StateManagement::Get((var, response_sender)) => {
                match con.get::<_, Option<String>>(&var).await {
                    Ok(val) => {
                        match val {
                            Some(redis_value) => {
                                let _ = response_sender
                                    .send(serde_json::from_str(&redis_value).unwrap());
                            }
                            None => panic!("Var doesn't exist!"),
                        }
                        panic!("Var doesn't exist!")
                    }
                    Err(e) => {
                        eprintln!("Failed to get {}: {:?}", var, e);
                        panic!("Var doesn't exist!")
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
                        eprintln!("Failed to set {}: {:?}", var, e);
                        panic!("!")
                    }
                }
            }

            StateManagement::Set((var, val)) => {
                if let Err(e) = con
                    .set::<_, String, Value>(&var, serde_json::to_string(&val).unwrap())
                    .await
                {
                    eprintln!("Failed to set {}: {:?}", var, e);
                    panic!("!")
                }
            }
        }
    }
}
