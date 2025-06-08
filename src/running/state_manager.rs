// use r2r_transform::*;
use redis::aio::MultiplexedConnection;
use std::collections::HashMap;

use std::env;

use crate::*;
use redis::{AsyncCommands, Client, Value};
use tokio::sync::{mpsc, oneshot};
use tokio::time::{Duration, interval};

/// Available commands that the async tasks can ask from the state manager.
pub enum StateManagement {
    GetState(oneshot::Sender<State>),
    Get((String, oneshot::Sender<Option<SPValue>>)), // maybe respond here with the old value instead of option
    SetPartialState(State),
    Set((String, SPValue)),
    InsertTransform((String, SPTransformStamped)),
    MoveTransform(String, SPTransform),
    LoadTransformScenario(String), // overlay?
    GetAllTransforms(oneshot::Sender<HashMap<String, SPTransformStamped>>),

    /// ew parent, child
    ReparentTransform((String, String, oneshot::Sender<bool>)),

    /// Parent -> Child
    LookupTransform((String, String, oneshot::Sender<Option<SPTransformStamped>>)), // Try to remove the transform prefix
                                                                                    // MoveTransform((String, SPTransform)), // move to a new position specified by SPTransform
}

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

// Martin: If you have more than one command for redis to do, use a pipeline to group commands together

// put this in another process that we can trigger from outside to reconnect if dsconnected
pub async fn redis_state_manager(
    mut receiver: mpsc::Receiver<StateManagement>,
    state: State,
) -> Result<(), Box<dyn std::error::Error>> {
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
                        error = format!("Cannot connect to Redis with error: {e}.");
                    }
                },
                Err(e) => {
                    error_tracker = 3;
                    error = format!("Cannot connect to Redis with error: {e}.");
                }
            }

            if error_value != error_tracker {
                error_value = error_tracker;
                match error_value {
                    0 => {
                        log::warn!(target: &&format!("redis_state_manager"), "Waiting for a Redis connection.");
                        log::warn!(target: &&format!("redis_state_manager"), "Have you started the redis container?")
                    }
                    _ => {
                        log::error!(target: &&format!("redis_state_manager"), "{}", error);
                        log::warn!(target: &&format!("redis_state_manager"), "Have you started the redis container?")
                    }
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
            log::error!(target: &&format!("redis_state_manager"), "Failed to set initial value of {var} with error {e}.")
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
                                if let Some(value) = maybe_value {
                                    match serde_json::from_str::<SPValue>(&value) {
                                        Ok(deser_value) => {
                                            let new_assignment = match deser_value {
                                                SPValue::Bool(_) => {
                                                    SPAssignment::new(bv!(&&key), deser_value)
                                                }
                                                SPValue::Float64(_) => {
                                                    SPAssignment::new(fv!(&&key), deser_value)
                                                }
                                                SPValue::Int64(_) => {
                                                    SPAssignment::new(iv!(&&key), deser_value)
                                                }
                                                SPValue::String(_) => {
                                                    SPAssignment::new(v!(&&key), deser_value)
                                                }
                                                SPValue::Time(_) => {
                                                    SPAssignment::new(tv!(&&key), deser_value)
                                                }
                                                SPValue::Array(_) => {
                                                    SPAssignment::new(av!(&&key), deser_value)
                                                }
                                                SPValue::Map(_) => {
                                                    SPAssignment::new(mv!(&&key), deser_value)
                                                }
                                                SPValue::Transform(_) => {
                                                    SPAssignment::new(tfv!(&&key), deser_value)
                                                }
                                            };
                                            map.insert(key, new_assignment);
                                        }
                                        Err(e) => {
                                            log::error!(target: &&format!("redis_state_manager"), 
                                            "Failed to deserialize '{key}' with error '{e}'.")
                                        }
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
                            if let Err(_) = response_sender
                                .send(match serde_json::from_str(&redis_value) {
                                    Ok(deser_value) => Some(deser_value),
                                    Err(e) => {
                                        log::error!(target: &&format!("redis_state_manager"), "Deserializing '{var}' failed with: {e}.");
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
                            log::error!(target: &&format!("redis_state_manager"), "Failed to get value '{var}' with: {e}.");
                            let _ = response_sender.send(None);
                        }
                    }
            }

            StateManagement::GetAllTransforms(response_sender) => {
                let transforms = get_all_transforms(con.clone()).await;
                let _ = response_sender.send(transforms);
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

            StateManagement::InsertTransform((name, transform)) => {
                (error_tracker, error) = insert_transform(name, transform, con.clone()).await;
            }

            StateManagement::MoveTransform(name, new_transform) => {
                match con.get::<_, Option<String>>(&name).await {
                    Ok(val) => match val {
                        Some(redis_value) => match serde_json::from_str::<SPValue>(&redis_value) {
                            Ok(deser_value) => match deser_value {
                                SPValue::Transform(tf_or_unknown) => match tf_or_unknown {
                                    TransformOrUnknown::Transform(sp_tf_stamped) => {
                                        let updated_sp_tf_stamped = SPTransformStamped {
                                            active_transform: sp_tf_stamped.active_transform,
                                            enable_transform: sp_tf_stamped.enable_transform,
                                            time_stamp: sp_tf_stamped.time_stamp,
                                            parent_frame_id: sp_tf_stamped.parent_frame_id,
                                            child_frame_id: sp_tf_stamped.child_frame_id,
                                            transform: new_transform,
                                            metadata: sp_tf_stamped.metadata,
                                        };
                                        if let Err(e) = con
                                            .set::<_, String, Value>(
                                                &format!("{name}"),
                                                serde_json::to_string(
                                                    &updated_sp_tf_stamped.to_spvalue(),
                                                )
                                                .unwrap(),
                                            )
                                            .await
                                        {
                                            error_tracker = 16;
                                            error = format!(
                                                "Failed to move transform {} with error: {}.",
                                                name, e
                                            );
                                        }
                                    }
                                    TransformOrUnknown::UNKNOWN => {}
                                },
                                _ => {}
                            },
                            Err(e) => {
                                log::error!(target: &&format!("redis_state_manager"), "Deserializing '{name}' failed with: {e}.");
                            }
                        },
                        None => {}
                    },
                    Err(e) => {
                        log::error!(target: &&format!("redis_state_manager"), "Failed to get value '{name}' with: {e}.");
                    }
                }
            }

            StateManagement::ReparentTransform((
                // old_parent_frame_id,
                new_parent_frame_id,
                child_frame_id,
                response_sender,
            )) => {
                let buffer = get_all_transforms(con.clone()).await;
                if let Some(transform) = buffer.get(&child_frame_id) {
                    let mut temp = transform.clone();
                    temp.parent_frame_id = new_parent_frame_id.clone();
                    if check_would_produce_cycle(&temp, &buffer) {
                        log::error!(
                            "Transform '{}' would produce cycle if reparented, no action taken.",
                            child_frame_id
                        );
                    } else {
                        match lookup_transform_with_root(
                            &new_parent_frame_id,
                            &child_frame_id,
                            "world",
                            &buffer,
                        ) {
                            Some(lookup_tf) => {
                                temp.transform = lookup_tf.transform;
                                if let Err(e) = con
                                    .set::<_, String, Value>(
                                        &child_frame_id,
                                        serde_json::to_string(&temp.to_spvalue()).unwrap(),
                                    )
                                    .await
                                {
                                    error_tracker = 26;
                                    error = format!(
                                        "Failed to reparent transform {} with error: {}.",
                                        child_frame_id, e
                                    );
                                } else {
                                    log::info!(
                                        "Reparented transform '{}' from to '{}'.",
                                        child_frame_id,
                                        new_parent_frame_id
                                    );
                                    let _ = response_sender.send(true);
                                }
                            }
                            None => {
                                log::error!("Failed to lookup during reparenting.");
                                let _ = response_sender.send(false);
                            }
                        };
                    }
                } else {
                    log::error!(
                        "Can't reparent transform '{}' because it doesn't exist.",
                        child_frame_id
                    );
                }
            }

            StateManagement::LoadTransformScenario(path) => {
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
                    }
                    Err(_e) => (),
                }
            }

            StateManagement::LookupTransform((
                parent_frame_id,
                child_frame_id,
                response_sender,
            )) => {
                let buffer = get_all_transforms(con.clone()).await;

                match get_tree_root(&buffer) {
                    Some(root) => {
                        match lookup_transform_with_root(
                            &parent_frame_id,
                            &child_frame_id,
                            &root,
                            &buffer,
                        ) {
                            Some(transform) => {
                                let _ = response_sender.send(Some(transform));
                            }
                            None => {
                                error_tracker = 9;
                                error = "Couldn't lookup transform".to_string();
                                let _ = response_sender.send(None);
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
                                let _ = response_sender.send(Some(transform));
                            }
                            None => {
                                error_tracker = 10;
                                error = "Couldn't lookup transform".to_string();
                                let _ = response_sender.send(None);
                            }
                        }
                    }
                }
            }
        }

        if error_value != error_tracker {
            error_value = error_tracker;
            log::error!(target: &&format!("redis_state_manager"), "{}", error)
        }
    }

    Ok(())
}

async fn get_all_transforms(mut con: MultiplexedConnection) -> HashMap<String, SPTransformStamped> {
    match con.keys::<&str, Vec<String>>("*").await {
        Ok(keys) => {
            match con
                .mget::<&Vec<std::string::String>, Vec<Option<String>>>(&keys)
                .await
            {
                Ok(values) => {
                    let mut buffer: HashMap<String, SPTransformStamped> = HashMap::new();
                    for (key, maybe_value) in keys.into_iter().zip(values.into_iter()) {
                        if let Some(value) = maybe_value {
                            match serde_json::from_str::<SPValue>(&value) {
                                Ok(val) => match val {
                                    SPValue::Transform(TransformOrUnknown::Transform(transf)) => {
                                        buffer.insert(key, transf);
                                    }
                                    _ => (),
                                },
                                Err(e) => log::error!(target: &&format!("redis_state_manager"),
                                    "Transform '{key}' failed deserialization with: {e}."
                                ),
                            }
                        }
                    }
                    buffer
                }
                Err(e) => {
                    log::error!(target: &&format!("redis_state_manager"), "Failed to get values with: {e}.");
                    HashMap::new()
                }
            }
        }
        Err(e) => {
            log::error!(target: &&format!("redis_state_manager"), "Failed to get values with: {e}.");
            HashMap::new()
        }
    }
}

async fn insert_transform(
    name: String,
    transform: SPTransformStamped,
    mut con: MultiplexedConnection,
) -> (i32, String) {
    let mut error_tracker = 0;
    let mut error: String = "".to_string();
    match con.keys::<&str, Vec<String>>("*").await {
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
                                match serde_json::from_str::<SPValue>(&value) {
                                    Ok(val) => match val {
                                        SPValue::Transform(TransformOrUnknown::Transform(
                                            transf,
                                        )) => {
                                            buffer.insert(key, transf);
                                        }
                                        _ => (),
                                    },
                                    Err(e) => log::error!(target: &&format!("redis_state_manager"),
                                        "Transform '{key}' failed deserialization with: {e}."
                                    ),
                                }
                            }
                        }

                        if name != transform.child_frame_id {
                            log::info!(target: &&format!("redis_state_manager"), 
                                "Transform name '{name}' in buffer doesn't match the child_frame_id {}, 
                                they should be the same. Not added.", transform.child_frame_id);
                        } else if let Some(_) = buffer.get(&name) {
                            log::info!(target: &&format!("redis_state_manager"),
                                "Transform '{}' already exists, not added.", name);
                        } else {
                            let transform = transform.clone();
                            if check_would_produce_cycle(&transform, &buffer) {
                                log::error!(target: &&format!("redis_state_manager"),
                                    "Transform '{}' would produce cycle, not added.", name);
                            } else {
                                if let Err(e) = con
                                    .set::<_, String, Value>(
                                        &format!("{name}"),
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
                                    log::info!(target: &&format!("redis_state_manager"),
                                        "Inserted transform '{name}'.");
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
                        &format!("{name}"),
                        serde_json::to_string(&transform.to_spvalue()).unwrap(),
                    )
                    .await
                {
                    error_tracker = 6;
                    error = format!("Failed to insert transform {} with error: {}.", name, e);
                } else {
                    log::info!(target: &&format!("redis_state_manager"),
                        "Inserted transform '{name}'.");
                }
            }
        }
        Err(e) => {
            error_tracker = 8;
            error = e.to_string();
        }
    }
    (error_tracker, error)
}

#[cfg(test)]
mod tests {

    use std::time::SystemTime;

    use crate::*;
    use ordered_float::OrderedFloat;
    use serial_test::serial;
    use tokio::sync::{mpsc, oneshot};

    use testcontainers::{ImageExt, core::ContainerPort, runners::AsyncRunner};

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

        tokio::task::spawn(async move {
            match redis_state_manager(rx, state).await {
                Ok(()) => (),
                Err(e) => log::error!(target: &&format!("redis_state_manager"), "{}", e),
            };
        });

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

        tokio::task::spawn(async move {
            match redis_state_manager(rx, state).await {
                Ok(()) => (),
                Err(e) => log::error!(target: &&format!("redis_state_manager"), "{}", e),
            };
        });

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

        tokio::task::spawn(async move {
            match redis_state_manager(rx, state).await {
                Ok(()) => (),
                Err(e) => log::error!(target: &&format!("redis_state_manager"), "{}", e),
            };
        });

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
    async fn test_set_get() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let state = dummy_state();

        let (tx, rx) = mpsc::channel(32);

        tokio::task::spawn(async move {
            match redis_state_manager(rx, state).await {
                Ok(()) => (),
                Err(e) => log::error!(target: &&format!("redis_state_manager"), "{}", e),
            };
        });

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

    #[tokio::test]
    #[serial]
    async fn test_set_get_state() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let state = dummy_state();

        let (tx, rx) = mpsc::channel(32);

        tokio::task::spawn(async move {
            match redis_state_manager(rx, state).await {
                Ok(()) => (),
                Err(e) => log::error!(target: &&format!("redis_state_manager"), "{}", e),
            };
        });

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
        tx.send(StateManagement::GetState(response_tx))
            .await
            .expect("failed");
        let recv_state = response_rx.await.expect("failed");

        let x = recv_state.get_int_or_default_to_zero(&format!("test_case"), &format!("x"));
        let y = recv_state.get_int_or_default_to_zero(&format!("test_case"), &format!("y"));
        let z = recv_state.get_int_or_default_to_zero(&format!("test_case"), &format!("z"));
        let j = recv_state.get_int_or_default_to_zero(&format!("test_case"), &format!("j"));

        assert_eq!(5, x);
        assert_eq!(6, y);
        assert_eq!(3, z);
        assert_eq!(9, j);
    }

    #[tokio::test]
    #[serial]
    async fn test_insert_transform() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let state = dummy_state();

        let transform = SPTransformStamped {
            active_transform: true,
            enable_transform: true,
            time_stamp: SystemTime::now(),
            parent_frame_id: "transform_a".to_string(),
            child_frame_id: "transform_b".to_string(),
            transform: SPTransform::default(),
            metadata: MapOrUnknown::UNKNOWN,
        };

        let (tx, rx) = mpsc::channel(32);

        tokio::task::spawn(async move {
            match redis_state_manager(rx, state).await {
                Ok(()) => (),
                Err(e) => log::error!(target: &&format!("redis_state_manager"), "{}", e),
            };
        });

        tx.send(StateManagement::InsertTransform((
            transform.child_frame_id.clone(),
            transform,
        )))
        .await
        .expect("failed");

        let (response_tx, response_rx) = oneshot::channel();
        tx.send(StateManagement::GetState(response_tx))
            .await
            .expect("failed");
        let recv_state = response_rx.await.expect("failed");

        let t = match recv_state
            .get_transform_or_unknown(&format!("test_case"), &format!("transform_b"))
        {
            TransformOrUnknown::Transform(t) => t,
            TransformOrUnknown::UNKNOWN => {
                panic!("failed")
            }
        };

        assert_eq!(true, t.active_transform);
        assert_eq!("transform_b", t.child_frame_id);
        assert_eq!("transform_a", t.parent_frame_id);
        assert_eq!(SPTransform::default(), t.transform);
    }

    #[tokio::test]
    #[serial]
    async fn test_load_transform_scenario() {
        // initialize_env_logger();
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let state = dummy_state();

        let path = "/home/endre/rust_crates/micro_sp/src/transforms/examples/data/";

        let (tx, rx) = mpsc::channel(32);

        tokio::task::spawn(async move {
            match redis_state_manager(rx, state).await {
                Ok(()) => (),
                Err(e) => log::error!(target: &&format!("redis_state_manager"), "{}", e),
            };
        });

        tx.send(StateManagement::LoadTransformScenario(path.to_string()))
            .await
            .expect("failed");

        let (response_tx, response_rx) = oneshot::channel();
        tx.send(StateManagement::GetState(response_tx))
            .await
            .expect("failed");
        let recv_state = response_rx.await.expect("failed");

        let t1 = match recv_state.get_transform_or_unknown(&format!("test_case"), &format!("floor"))
        {
            TransformOrUnknown::Transform(t) => t,
            TransformOrUnknown::UNKNOWN => {
                panic!("failed")
            }
        };

        let t2 = match recv_state.get_transform_or_unknown(&format!("test_case"), &format!("table"))
        {
            TransformOrUnknown::Transform(t) => t,
            TransformOrUnknown::UNKNOWN => {
                panic!("failed")
            }
        };

        assert_eq!(false, t1.active_transform);
        assert_eq!("floor", t1.child_frame_id);
        assert_eq!("world", t1.parent_frame_id);
        assert_eq!("table", t2.child_frame_id);
        assert_eq!("world", t2.parent_frame_id);
        assert_ne!(SPTransform::default(), t1.transform);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_all_transforms() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let state = dummy_state();

        let path = "/home/endre/rust_crates/micro_sp/src/transforms/examples/data/";

        let (tx, rx) = mpsc::channel(32);

        tokio::task::spawn(async move {
            match redis_state_manager(rx, state).await {
                Ok(()) => (),
                Err(e) => log::error!(target: &&format!("redis_state_manager"), "{}", e),
            };
        });

        tx.send(StateManagement::LoadTransformScenario(path.to_string()))
            .await
            .expect("failed");

        let (response_tx, response_rx) = oneshot::channel();
        tx.send(StateManagement::GetAllTransforms(response_tx))
            .await
            .expect("failed");
        let transforms = response_rx.await.expect("failed");

        assert_eq!(6, transforms.len());
    }

    #[tokio::test]
    #[serial]
    async fn test_lookup_transforms() {
        // initialize_env_logger();
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let state = dummy_state();

        let path = "/home/endre/rust_crates/micro_sp/src/transforms/examples/data/";

        let (tx, rx) = mpsc::channel(32);

        tokio::task::spawn(async move {
            match redis_state_manager(rx, state).await {
                Ok(()) => (),
                Err(e) => log::error!(target: &&format!("redis_state_manager"), "{}", e),
            };
        });

        tx.send(StateManagement::LoadTransformScenario(path.to_string()))
            .await
            .expect("failed");

        let (response_tx, response_rx) = oneshot::channel();
        tx.send(StateManagement::LookupTransform((
            "world".to_string(),
            "floor".to_string(),
            response_tx,
        )))
        .await
        .expect("failed");
        let lookup = response_rx.await.expect("failed");

        assert_eq!("floor", lookup.clone().unwrap().child_frame_id);
        assert_eq!("world", lookup.clone().unwrap().parent_frame_id);

        let assert_t = SPTransform {
            translation: SPTranslation {
                x: OrderedFloat(0.0),
                y: OrderedFloat(0.0),
                z: OrderedFloat(1.0),
            },
            rotation: SPRotation {
                x: OrderedFloat(0.0),
                y: OrderedFloat(0.0),
                z: OrderedFloat(0.0),
                w: OrderedFloat(1.0),
            },
        };

        assert_eq!(assert_t, lookup.unwrap().transform);

        let (response_tx, response_rx) = oneshot::channel();
        tx.send(StateManagement::LookupTransform((
            "floor".to_string(),
            "food".to_string(),
            response_tx,
        )))
        .await
        .expect("failed");
        let lookup = response_rx.await.expect("failed");

        assert_eq!("food", lookup.clone().unwrap().child_frame_id);
        assert_eq!("floor", lookup.clone().unwrap().parent_frame_id);

        let assert_t = SPTransform {
            translation: SPTranslation {
                x: OrderedFloat(1.0),
                y: OrderedFloat(5.0),
                z: OrderedFloat(0.0),
            },
            rotation: SPRotation {
                x: OrderedFloat(0.0),
                y: OrderedFloat(0.0),
                z: OrderedFloat(0.0),
                w: OrderedFloat(1.0),
            },
        };

        assert_eq!(assert_t, lookup.unwrap().transform);
    }

    #[tokio::test]
    #[serial]
    async fn test_move_transform() {
        // initialize_env_logger();
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let state = dummy_state();

        let path = "/home/endre/rust_crates/micro_sp/src/transforms/examples/data/";

        let (tx, rx) = mpsc::channel(32);

        tokio::task::spawn(async move {
            match redis_state_manager(rx, state).await {
                Ok(()) => (),
                Err(e) => log::error!(target: &&format!("redis_state_manager"), "{}", e),
            };
        });

        tx.send(StateManagement::LoadTransformScenario(path.to_string()))
            .await
            .expect("failed");

        // let (response_tx, response_rx) = oneshot::channel();
        tx.send(StateManagement::MoveTransform(
            "floor".to_string(),
            SPTransform {
                translation: SPTranslation {
                    x: OrderedFloat(1.0),
                    y: OrderedFloat(2.0),
                    z: OrderedFloat(3.0),
                },
                rotation: SPRotation {
                    x: OrderedFloat(1.0),
                    y: OrderedFloat(0.0),
                    z: OrderedFloat(0.0),
                    w: OrderedFloat(0.0),
                },
            },
        ))
        .await
        .expect("failed");

        let (response_tx, response_rx) = oneshot::channel();
        tx.send(StateManagement::GetState(response_tx))
            .await
            .expect("failed");
        let recv_state = response_rx.await.expect("failed");

        let floor_moved =
            match recv_state.get_transform_or_unknown(&format!("test_case"), &format!("floor")) {
                TransformOrUnknown::Transform(t) => t,
                TransformOrUnknown::UNKNOWN => {
                    panic!("failed")
                }
            };

        let assert_t = SPTransform {
            translation: SPTranslation {
                x: OrderedFloat(1.0),
                y: OrderedFloat(2.0),
                z: OrderedFloat(3.0),
            },
            rotation: SPRotation {
                x: OrderedFloat(1.0),
                y: OrderedFloat(0.0),
                z: OrderedFloat(0.0),
                w: OrderedFloat(0.0),
            },
        };

        assert_eq!(assert_t, floor_moved.transform);
    }
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
