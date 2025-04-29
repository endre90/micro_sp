// use r2r_transform::*;
use redis::aio::MultiplexedConnection;
use std::collections::HashMap;

use std::env;

use crate::*;
use redis::{AsyncCommands, Client, Value};
use tokio::sync::{mpsc, oneshot};
use tokio::time::{interval, Duration};

/// Available commands that the async tasks can ask from the state manager.
pub enum StateManagement {
    GetState(oneshot::Sender<State>),
    Get((String, oneshot::Sender<Option<SPValue>>)),
    SetPartialState(State),
    Set((String, SPValue)),
    // Insert, need this to add new variables on the fly, and also transforms
    InsertTransform((String, SPTransformStamped)), // Use r2r_transforms why not!
    LoadTransformScenario(String),                 //((String, bool)), // path, overlay
    // GetAllTransforms(oneshot::Sender<State>),
    LookupTransform((String, String, oneshot::Sender<SPTransformStamped>)),
    // MoveTransform((String, SPTransform)), // move to a new position specified by SPTransform
}

// /// Available commands that the async tasks can ask from the transform manager.
// pub enum TransformManagement {
//     GetAll(oneshot::Sender<State>),
//     Get((String, oneshot::Sender<SPValue>)),
//     Lookup((String, String, oneshot::Sender<SPValue>)),
//     Add((String, SPValue)),
//     Move((String, SPValue)),
//     SetPartialState(State),
//     Set((String, SPValue))
// }

// /// Represents the type of update to perform on a transform.
// #[derive(Clone, Debug)]
// enum UpdateType {
//     Add,
//     Move,
//     Remove,
//     Rename,
//     Reparent,
//     Clone,
//     DeleteAll,
// }

// MArtin: If you have more than one command for redis to do, use a pipeline to group commands together

// put this in another process that we can trigger from outside to reconnect if dsconnected
pub async fn redis_state_manager(mut receiver: mpsc::Receiver<StateManagement>, state: State) {
    let mut con = {
        let mut interval = interval(Duration::from_millis(100));
        let mut error_tracker;
        let mut error_value = 0;
        let mut error: String;
        // Read hostname and port from environment variables
        // Default to '127.0.0.1' when the environment variable is not set,
        // as this is the address accessible from the host machine.
        let redis_host = env::var("REDIS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        // let redis_host = env::var("REDIS_HOST").unwrap_or_else(|_| "redis".to_string()); // Default to 'redis'
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

    // let space_tree_server = SpaceTreeServer::new("space_tree_server");

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
                        // Ok(values) => {
                        //     let mut map: HashMap<String, SPAssignment> = HashMap::new();
                        //     for (key, maybe_value) in keys.into_iter().zip(values.into_iter()) {
                        //         if let Some(value) = maybe_value {
                        //             match serde_json::from_str(&value) {
                        //                 Ok(value_deser) => {

                        //                 },
                        //                 Err(_) => ()
                        //             }
                        //             let var: SPVariable = serde_json::from_str(&key).unwrap();
                        //             let val: SPValue = serde_json::from_str(&value).unwrap();

                        //             let new_assignment = SPAssignment::new(
                        //                 var,
                        //                 val
                        //             );
                        //             map.insert(key, new_assignment);
                        //         }
                        //         // }
                        //     }
                        //     // we want to keep updating a copy of a state so that we can maintain it if
                        //     // the connection to Redis gets disrupted
                        //     let new_state = State { state: map };
                        //     old_state = new_state;
                        //     let _ = response_sender.send(old_state.clone());
                        // }
                        Ok(values) => {
                            let mut map: HashMap<String, SPAssignment> = HashMap::new();
                            for (key, maybe_value) in keys.into_iter().zip(values.into_iter()) {
                                // if state.contains(&key) { // test without this BUT MIGHT NEED IT!
                                // Only get state that is locally tracked
                                if let Some(value) = maybe_value {
                                    let var = state.get_assignment(&key).var; //cant' have this if I want to add new variables
                                    let new_assignment = SPAssignment::new(
                                        var,
                                        serde_json::from_str(&value).unwrap(),
                                    );
                                    map.insert(key, new_assignment);
                                }
                                // }
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

            StateManagement::LoadTransformScenario(path) => {
                //, overlay)) => {
                match list_frames_in_dir(&path) {
                    Ok(list) => {
                        let frames = load_new_scenario(&list);
                        // if overlay {
                        for frame in frames.values() {
                            insert_transform(
                                frame.child_frame_id.clone(),
                                frame.clone(),
                                con.clone(),
                            )
                            .await;
                        }
                        // } else {
                        //     let buffer = self.local_buffer.lock().unwrap();
                        //     frames
                        //         .values()
                        //         .filter(|frame| buffer.get(&frame.child_frame_id) == None)
                        //         .for_each(|frame| {
                        //             Self::insert_transform(
                        //                 &self,
                        //                 &frame.child_frame_id,
                        //                 frame.clone(),
                        //             )
                        //         });
                        // }
                    }
                    Err(_e) => (),
                }
            }

            // StateManagement::Get((var, response_sender)) => {
            //     match con.get::<_, Option<String>>(&var).await {
            //         Ok(val) => match val {
            //             Some(redis_value) => {
            //                 old_state =
            //                     old_state.update(&var, serde_json::from_str(&redis_value).unwrap());
            //                 let _ =
            //                     response_sender.send(serde_json::from_str(&redis_value).unwrap());
            //             }
            //             None => {
            //                 error_tracker = 3;
            //                 error = format!("Failed to get variable {}.", var);
            //                 let _ = response_sender.send(old_state.get_value(&var));
            //             }
            //         },
            //         Err(e) => {
            //             error_tracker = 4;
            //             error = format!("Failed to get variable {} with error: {}.", var, e);
            //             let _ = response_sender.send(old_state.get_value(&var));
            //         }
            //     }
            // }
            StateManagement::Get((var, response_sender)) => {
                match con.get::<_, Option<String>>(&var).await {
                    Ok(val) => match val {
                        Some(redis_value) => {
                            if let Err(_) = response_sender
                                .send(match serde_json::from_str(&redis_value) {
                                    Ok(deser_value) => Some(deser_value),
                                    Err(e) => {
                                        log::error!(target: &&format!("redis_state_manager"), "Deserialize failure with: {}", e);
                                        None
                                    }
                                }) {
                                    log::error!(target: &&format!("redis_state_manager"), "Receiver has dropped.")
                                }
                            },
                            None => {
                                let _ = response_sender.send(None);
                            }
                        },
                        Err(e) => {
                            log::error!(target: &&format!("redis_state_manager"), "Failed to get with: {}", e);
                            let _ = response_sender.send(None);
                        }
                    }
            }

            // StateManagement::GetAllTransforms(response_sender) => {
            //     // OR:
            //     // space_tree_server.get_all()

            //     match con.keys::<&str, Vec<String>>("transform_*").await {
            //         Ok(keys) => match con
            //             .mget::<&Vec<std::string::String>, Vec<Option<String>>>(&keys)
            //             .await
            //         {
            //             Ok(values) => {
            //                 let mut map: HashMap<String, SPAssignment> = HashMap::new();
            //                 for (key, maybe_value) in keys.into_iter().zip(values.into_iter()) {
            //                     if let Some(value) = maybe_value {
            //                         let var = state.get_assignment(&key).var;
            //                         let new_assignment = SPAssignment::new(
            //                             var,
            //                             serde_json::from_str(&value).unwrap(),
            //                         );
            //                         map.insert(key, new_assignment);
            //                     }
            //                 }

            //                 let _ = response_sender.send(State { state: map });
            //             }
            //             Err(e) => {
            //                 error_tracker = 1;
            //                 error = e.to_string();
            //                 let _ = response_sender.send(State {
            //                     state: HashMap::new(),
            //                 });
            //             }
            //         },

            //         Err(e) => {
            //             error_tracker = 2;
            //             error = e.to_string();
            //             let _ = response_sender.send(State {
            //                 state: HashMap::new(),
            //             });
            //         }
            //     }
            // }
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

            StateManagement::InsertTransform((name, transform)) => {
                (error_tracker, error) = insert_transform(name, transform, con.clone()).await;
                // match con.keys::<&str, Vec<String>>("transform_*").await {
                //     Ok(keys) => match con
                //         .mget::<&Vec<std::string::String>, Vec<Option<String>>>(&keys)
                //         .await
                //     {
                //         Ok(values) => {
                //             let mut buffer: HashMap<String, SPTransformStamped> = HashMap::new();
                //             for (key, maybe_value) in keys.into_iter().zip(values.into_iter()) {
                //                 if let Some(value) = maybe_value {
                //                     buffer.insert(key, serde_json::from_str(&value).unwrap());
                //                 }
                //             }

                //             if name != transform.child_frame_id {
                //                 log::info!("Transform name '{name}' in buffer doesn't match the child_frame_id {},
                //                         they should be the same. Not added.", transform.child_frame_id);
                //             } else if let Some(_) = buffer.get(&name) {
                //                 log::info!("Transform '{}' already exists, not added.", name);
                //             } else {
                //                 let transform = transform.clone();
                //                 if check_would_produce_cycle(&transform, &buffer) {
                //                     log::info!(
                //                         "Transform '{}' would produce cycle, not added.",
                //                         name
                //                     );
                //                 } else {
                //                     buffer.insert(name.to_string(), transform);
                //                     log::info!("Inserted transform '{name}'.");
                //                 }
                //             }
                //         }
                //         Err(e) => {
                //             error_tracker = 7;
                //             error = e.to_string();
                //         }
                //     },
                //     Err(e) => {
                //         error_tracker = 8;
                //         error = e.to_string();
                //     } //
                // }
            }

            StateManagement::LookupTransform((
                parent_frame_id,
                child_frame_id,
                response_sender,
            )) => {
                match con.keys::<&str, Vec<String>>("transform_*").await {
                    Ok(keys) => match con
                        .mget::<&Vec<std::string::String>, Vec<Option<String>>>(&keys)
                        .await
                    {
                        Ok(values) => {
                            let mut buffer: HashMap<String, SPTransformStamped> = HashMap::new();
                            for (key, maybe_value) in keys.into_iter().zip(values.into_iter()) {
                                if let Some(value) = maybe_value {
                                    buffer.insert(key, serde_json::from_str(&value).unwrap());
                                }
                            }

                            match get_tree_root(&buffer) {
                                Some(root) => {
                                    match lookup_transform_with_root(
                                        &parent_frame_id,
                                        &child_frame_id,
                                        &root,
                                        &buffer,
                                    ) {
                                        Some(transform) => {
                                            let _ = response_sender.send(transform);
                                        }
                                        None => {
                                            error_tracker = 9;
                                            error = "couldn't lookup transform".to_string()
                                        }
                                    }
                                }
                                None => {
                                    match lookup_transform_with_root(
                                        &parent_frame_id,
                                        &child_frame_id,
                                        "world",
                                        &buffer,
                                    ) {
                                        Some(transform) => {
                                            let _ = response_sender.send(transform);
                                        }
                                        None => {
                                            error_tracker = 10;
                                            error = "couldn't lookup transform".to_string()
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error_tracker = 11;
                            error = e.to_string();
                        }
                    },
                    Err(e) => {
                        error_tracker = 12;
                        error = e.to_string();
                    } //
                }
            }
        }

        if error_value != error_tracker {
            error_value = error_tracker;
            log::error!(target: &&format!("redis_state_manager"), "{}", error)
        }
    }
}

async fn insert_transform(
    name: String,
    transform: SPTransformStamped,
    mut con: MultiplexedConnection,
) -> (i32, String) {
    let mut error_tracker = 0;
    let mut error: String = "asdf".to_string();
    match con.keys::<&str, Vec<String>>("transform_*").await {
        Ok(keys) => {
            if !keys.is_empty() {
                match con
                    .mget::<&Vec<std::string::String>, Vec<Option<String>>>(&keys)
                    .await
                {
                    Ok(values) => {
                        let mut buffer: HashMap<String, SPTransformStamped> = HashMap::new();
                        for (key, maybe_value) in keys.into_iter().zip(values.into_iter()) {
                            if let Some(value) = maybe_value {
                                buffer.insert(key, serde_json::from_str(&value).unwrap());
                            }
                        }

                        if name != transform.child_frame_id {
                            log::info!("Transform name '{name}' in buffer doesn't match the child_frame_id {}, 
                            they should be the same. Not added.", transform.child_frame_id);
                        } else if let Some(_) = buffer.get(&name) {
                            log::info!("Transform '{}' already exists, not added.", name);
                        } else {
                            let transform = transform.clone();
                            if check_would_produce_cycle(&transform, &buffer) {
                                log::error!("Transform '{}' would produce cycle, not added.", name);
                            } else {
                                if let Err(e) = con
                                    .set::<_, String, Value>(
                                        &format!("transform_{name}"),
                                        serde_json::to_string(&transform.to_spvalue()).unwrap(),
                                    )
                                    .await
                                {
                                    error_tracker = 6;
                                    error = format!(
                                        "Failed to insert transform {} with error: {}.",
                                        name, e
                                    );
                                } else {
                                    log::info!("Inserted transform '{name}'.");
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error_tracker = 7;
                        error = e.to_string();
                    }
                }
            } else {
                if let Err(e) = con
                    .set::<_, String, Value>(
                        &format!("transform_{name}"),
                        serde_json::to_string(&transform.to_spvalue()).unwrap(),
                    )
                    .await
                {
                    error_tracker = 6;
                    error = format!("Failed to insert transform {} with error: {}.", name, e);
                } else {
                    log::info!("Inserted transform '{name}'.");
                }
            }
        }
        Err(e) => {
            error_tracker = 8;
            error = e.to_string();
        } //
    }
    (error_tracker, error)
}

#[cfg(test)]
mod tests {

    use std::time::SystemTime;

    use crate::*;
    use serial_test::serial;
    use tokio::sync::{mpsc, oneshot};

    use testcontainers::{core::ContainerPort, runners::AsyncRunner, ImageExt};

    use testcontainers_modules::redis::Redis;

    fn dummy_state() -> State {
        let state = State::new();
        let x = iv!("x");
        let y = iv!("y");
        let z = iv!("z");
        let state = state.add(assign!(x, 1.to_spvalue()));
        let state = state.add(assign!(y, 2.to_spvalue()));
        let state = state.add(assign!(z, 3.to_spvalue()));
        state
    }

    #[tokio::test]
    #[serial]
    async fn test_get_state() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let state = dummy_state();
        let (tx, rx) = mpsc::channel(32);

        tokio::task::spawn(async move { redis_state_manager(rx, state).await });

        let (response_tx, response_rx) = oneshot::channel();
        tx.send(StateManagement::GetState(response_tx))
            .await
            .expect("failed");
        let recv_state = response_rx.await.expect("failed");

        let x = recv_state.get_int_or_default_to_zero(&format!("test_case"), &format!("x"));
        let y = recv_state.get_int_or_default_to_zero(&format!("test_case"), &format!("y"));
        let z = recv_state.get_int_or_default_to_zero(&format!("test_case"), &format!("z"));

        assert_eq!(1, x);
        assert_eq!(2, y);
        assert_eq!(3, z);
    }

    #[tokio::test]
    #[serial]
    async fn test_get() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let state = dummy_state();
        let (tx, rx) = mpsc::channel(32);

        tokio::task::spawn(async move { redis_state_manager(rx, state).await });

        let (response_tx, response_rx) = oneshot::channel();
        tx.send(StateManagement::Get(("x".to_string(), response_tx)))
            .await
            .expect("failed");
        let recv_x = response_rx.await.expect("failed");

        let (response_tx, response_rx) = oneshot::channel();
        tx.send(StateManagement::Get(("y".to_string(), response_tx)))
            .await
            .expect("failed");
        let recv_y = response_rx.await.expect("failed");

        let (response_tx, response_rx) = oneshot::channel();
        tx.send(StateManagement::Get(("z".to_string(), response_tx)))
            .await
            .expect("failed");
        let recv_z = response_rx.await.expect("failed");

        let x = recv_x.unwrap().to_int_or_zero();
        let y = recv_y.unwrap().to_int_or_zero();
        let z = recv_z.unwrap().to_int_or_zero();

        assert_eq!(1, x);
        assert_eq!(2, y);
        assert_eq!(3, z);
    }

    #[tokio::test]
    #[serial]
    async fn test_set_partial_state() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let state = dummy_state();
        let mut new_state = state.clone();

        new_state = new_state
            .update(&format!("x"), 5.to_spvalue())
            .update(&format!("y"), 6.to_spvalue());

        let modified_state = state.get_diff_partial_state(&new_state);

        let (tx, rx) = mpsc::channel(32);

        tokio::task::spawn(async move { redis_state_manager(rx, state).await });

        tx.send(StateManagement::SetPartialState(modified_state))
            .await
            .expect("failed");

        let (response_tx, response_rx) = oneshot::channel();
        tx.send(StateManagement::GetState(response_tx))
            .await
            .expect("failed");
        let recv_state = response_rx.await.expect("failed");

        let x = recv_state.get_int_or_default_to_zero(&format!("test_case"), &format!("x"));
        let y = recv_state.get_int_or_default_to_zero(&format!("test_case"), &format!("y"));
        let z = recv_state.get_int_or_default_to_zero(&format!("test_case"), &format!("z"));

        assert_eq!(5, x);
        assert_eq!(6, y);
        assert_eq!(3, z);
    }

    #[tokio::test]
    #[serial]
    async fn test_set() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let state = dummy_state();

        let (tx, rx) = mpsc::channel(32);

        tokio::task::spawn(async move { redis_state_manager(rx, state).await });

        tx.send(StateManagement::Set(("x".to_string(), 5.to_spvalue())))
            .await
            .expect("failed");

        tx.send(StateManagement::Set(("y".to_string(), 6.to_spvalue())))
            .await
            .expect("failed");

        tx.send(StateManagement::Set(("j".to_string(), 9.to_spvalue())))
            .await
            .expect("failed");

        let (response_tx, response_rx) = oneshot::channel();
        tx.send(StateManagement::Get(("x".to_string(), response_tx)))
            .await
            .expect("failed");
        let recv_x = response_rx.await.expect("failed");

        let (response_tx, response_rx) = oneshot::channel();
        tx.send(StateManagement::Get(("y".to_string(), response_tx)))
            .await
            .expect("failed");
        let recv_y = response_rx.await.expect("failed");

        let (response_tx, response_rx) = oneshot::channel();
        tx.send(StateManagement::Get(("z".to_string(), response_tx)))
            .await
            .expect("failed");
        let recv_z = response_rx.await.expect("failed");

        let (response_tx, response_rx) = oneshot::channel();
        tx.send(StateManagement::Get(("j".to_string(), response_tx)))
            .await
            .expect("failed");
        let recv_j = response_rx.await.expect("failed");

        let x = recv_x.unwrap().to_int_or_zero();
        let y = recv_y.unwrap().to_int_or_zero();
        let z = recv_z.unwrap().to_int_or_zero();
        let j = recv_j.unwrap().to_int_or_zero();

        assert_eq!(5, x);
        assert_eq!(6, y);
        assert_eq!(3, z);
        assert_eq!(9, j);
    }

    // #[tokio::test]
    // #[serial]
    // async fn test_insert_transform() {
    //     let _container = Redis::default()
    //         .with_mapped_port(6379, ContainerPort::Tcp(6379))
    //         .start()
    //         .await
    //         .unwrap();

    //     let state = dummy_state();

    //     let transform = SPTransformStamped {
    //         active: true,
    //         time_stamp: SystemTime::now(),
    //         parent_frame_id: "a".to_string(),
    //         child_frame_id: "b".to_string(),
    //         transform: SPTransform::default(),
    //         metadata: MapOrUnknown::UNKNOWN,
    //     };

    //     let (tx, rx) = mpsc::channel(32);

    //     tokio::task::spawn(async move { redis_state_manager(rx, state).await });

    //     tx.send(StateManagement::InsertTransform((
    //         transform.child_frame_id.clone(),
    //         transform,
    //     )))
    //     .await
    //     .expect("failed");

    //     let (response_tx, response_rx) = oneshot::channel();
    //     tx.send(StateManagement::GetState(response_tx))
    //         .await
    //         .expect("failed");
    //     let recv_state = response_rx.await.expect("failed");

    //     // let t = match recv_state.get_transform_or_unknown(&format!("test_case"), &format!("b")) {
    //     //     TransformOrUnknown::Transform(t) => t,
    //     //     TransformOrUnknown::UNKNOWN => {
    //     //         panic!("failed")
    //     //     }
    //     // };

    //     // assert_eq!(true, t.active);
    //     // assert_eq!("b", t.child_frame_id);
    //     // assert_eq!("a", t.parent_frame_id);
    //     // assert_eq!(SPTransform::default(), t.transform);
    // }
}

// /// Instead of sharing the state with Arc<Mutex<State>>, use a buffer of state read/write requests.
// pub async fn state_manager_no_redis(
//     mut receiver: mpsc::Receiver<StateManagement>,
//     mut state: State,
// ) {
//     while let Some(command) = receiver.recv().await {
//         match command {
//             StateManagement::GetState(response_sender) => {
//                 let _ = response_sender.send(state.clone());
//             }
//             StateManagement::Get((var, response_sender)) => {
//                 let _ = response_sender.send(state.get_value(&var));
//             }
//             StateManagement::SetPartialState(partial_state) => {
//                 for (var, assignment) in partial_state.state {
//                     state = state.update(&var, assignment.val)
//                 }
//             }
//             StateManagement::Set((var, new_val)) => {
//                 state = state.update(&var, new_val);
//             }
//         }
//     }
// }
