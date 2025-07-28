use rayon::prelude::*;
use redis::{aio::MultiplexedConnection, pipe};
use std::collections::HashMap;

use std::env;

use crate::*;
use redis::{AsyncCommands, Client, Value};
use tokio::time::{Duration, interval};

pub struct StateManager {}

const TRANSFORM_INDEX_KEY: &str = "transforms_index";

impl StateManager {
    pub async fn get_redis_mpx_connection() -> MultiplexedConnection {
        let redis_host = env::var("REDIS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let redis_port = env::var("REDIS_PORT").unwrap_or_else(|_| "6379".to_string());
        let redis_addr = format!("redis://{}:{}", redis_host, redis_port);

        let mut interval = interval(Duration::from_millis(500));
        let mut last_error: Option<String> = None;

        log::warn!(
            target: "redis_state_manager",
            "Attempting to connect to Redis at {}. Retrying on failure...",
            redis_addr
        );

        loop {
            let connection_result = async {
                let client = Client::open(redis_addr.clone())?;
                client.get_multiplexed_async_connection().await
            }
            .await;

            match connection_result {
                Ok(connection) => {
                    log::info!(target: "redis_state_manager", "Redis connection established.");
                    return connection;
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    if last_error.as_ref() != Some(&error_msg) {
                        log::error!(target: "redis_state_manager", "Connection failed: {}. Retrying...", error_msg);
                        last_error = Some(error_msg);
                    }
                }
            }

            interval.tick().await;
        }
    }

    pub async fn get_full_state(con: &mut MultiplexedConnection) -> Option<State> {
        let keys: Vec<String> = match con.keys("*").await {
            Ok(k) => k,
            Err(e) => {
                log::error!("Failed to get keys from Redis: {e}");
                return None;
            }
        };

        if keys.is_empty() {
            return Some(State::new());
        }

        let values: Vec<Option<String>> = match con.mget(&keys).await {
            Ok(v) => v,
            Err(e) => {
                log::error!("Failed to get values from Redis: {e}");
                return None;
            }
        };

        Some(StateManager::build_state(keys, values))
    }

    pub async fn get_state_for_keys(
        con: &mut MultiplexedConnection,
        keys: &Vec<String>,
    ) -> Option<State> {
        if keys.is_empty() {
            return Some(State::new());
        }

        let values: Vec<Option<String>> = match con.mget(keys).await {
            Ok(v) => v,
            Err(e) => {
                log::error!("Failed to get values from Redis: {e}");
                return None;
            }
        };

        Some(StateManager::build_state(keys.clone(), values))
    }

    pub fn build_state(keys: Vec<String>, values: Vec<Option<String>>) -> State {
        let mut state_map = HashMap::new();

        for (key, maybe_value) in keys.into_iter().zip(values.into_iter()) {
            let Some(value_str) = maybe_value else {
                continue;
            };

            if key == "heartbeat".to_string() {
                continue;
            }

            if let Ok(sp_value) = serde_json::from_str::<SPValue>(&value_str) {
                let assignment = create_assignment(&key, sp_value);
                state_map.insert(key, assignment);
            } else {
                log::warn!("Failed to deserialize value for key '{}'.", key);
            }
        }

        State { state: state_map }
    }

    pub async fn get_sp_value(con: &mut MultiplexedConnection, var: &str) -> Option<SPValue> {
        let redis_result: Option<String> = match con.get(var).await {
            Ok(value) => value,
            Err(e) => {
                log::error!("Failed to get '{var}' from Redis: {e}");
                return None;
            }
        };

        let Some(value_str) = redis_result else {
            return None;
        };

        match serde_json::from_str(&value_str) {
            Ok(deserialized_value) => Some(deserialized_value),
            Err(e) => {
                log::error!("Deserializing value for '{var}' failed: {e}");
                None
            }
        }
    }

    pub async fn set_state(con: &mut MultiplexedConnection, state: State) {
        let items_to_set: Vec<(String, String)> = state
            .state
            .into_iter()
            .filter_map(
                |(key, assignment)| match serde_json::to_string(&assignment.val) {
                    Ok(value_str) => Some((key, value_str)),
                    Err(e) => {
                        log::error!("Failed to serialize value for key '{key}': {e}");
                        None
                    }
                },
            )
            .collect();

        if !items_to_set.is_empty() {
            match con.mset::<_, String, Value>(&items_to_set).await {
                Ok(_) => {}
                Err(e) => log::error!("Redis MSET command failed: {e}"),
            }
        }
    }

    pub async fn set_sp_value(con: &mut MultiplexedConnection, key: &str, value: &SPValue) {
        let value_str = match serde_json::to_string(value) {
            Ok(s) => s,
            Err(e) => {
                log::error!("Failed to serialize value for key '{key}': {e}");
                return;
            }
        };

        match con.set::<_, _, ()>(key, value_str).await {
            Ok(_) => {}
            Err(e) => {
                log::error!("Redis SET command for key '{key}' failed: {e}");
            }
        }
    }

    pub async fn insert_transform(
        con: &mut MultiplexedConnection,
        key: &str,
        transform: SPTransformStamped,
    ) {
        let sp_value = SPValue::Transform(TransformOrUnknown::Transform(transform));
        let value_str = match serde_json::to_string(&sp_value) {
            Ok(s) => s,
            Err(e) => {
                log::error!("Failed to serialize transform for key '{key}': {e}");
                return;
            }
        };

        let result: redis::RedisResult<()> = pipe()
            .atomic()
            .set(key, value_str)
            .sadd(TRANSFORM_INDEX_KEY, key)
            .query_async(con)
            .await;

        if let Err(e) = result {
            log::error!("Failed to add transform for key '{key}': {e}");
        }
    }

    pub async fn insert_transforms(
        con: &mut MultiplexedConnection,
        transforms: HashMap<String, SPTransformStamped>,
    ) {
        if transforms.is_empty() {
            return;
        }

        let keys: Vec<String> = transforms.keys().cloned().collect();

        let mset_values: Vec<(String, String)> = match transforms
            .into_iter()
            .map(|(key, transform)| {
                let sp_value = SPValue::Transform(TransformOrUnknown::Transform(transform));
                serde_json::to_string(&sp_value).map(|json_val| (key, json_val))
            })
            .collect()
        {
            Ok(vals) => vals,
            Err(e) => {
                log::error!("Failed to serialize one or more transforms: {e}");
                return;
            }
        };

        let result: redis::RedisResult<()> = pipe()
            .atomic()
            .mset(&mset_values)
            .sadd(TRANSFORM_INDEX_KEY, &keys)
            .query_async(con)
            .await;

        if let Err(e) = result {
            log::error!("Failed to add multiple transforms: {e}");
        }
    }

    pub async fn remove_transform(con: &mut MultiplexedConnection, key: &str) {
        let result: redis::RedisResult<()> = pipe()
            .atomic()
            .del(key)
            .srem(TRANSFORM_INDEX_KEY, key)
            .query_async(con)
            .await;

        if let Err(e) = result {
            log::error!("Failed to remove transform for key '{key}': {e}");
        }
    }

    pub async fn get_all_transforms(
        con: &mut MultiplexedConnection,
    ) -> HashMap<String, SPTransformStamped> {
        let keys: Vec<String> = match con.smembers(TRANSFORM_INDEX_KEY).await {
            Ok(k) => k,
            Err(e) => {
                log::error!("Failed to get transform keys: {e}");
                return HashMap::new();
            }
        };

        if keys.is_empty() {
            return HashMap::new();
        }

        let values: Vec<String> = match con.mget(keys.clone()).await {
            Ok(v) => v,
            Err(e) => {
                log::error!("Failed to MGET transform values: {e}");
                return HashMap::new();
            }
        };

        let key_value_pairs: Vec<(String, String)> =
            keys.into_iter().zip(values.into_iter()).collect();

        // Use Rayon to process the data in parallel
        key_value_pairs
            .into_par_iter()
            .filter_map(
                |(key, value_str)| match serde_json::from_str::<SPValue>(&value_str) {
                    Ok(SPValue::Transform(TransformOrUnknown::Transform(transf))) => {
                        Some((key, transf))
                    }
                    Ok(_) => None,
                    Err(e) => {
                        log::error!("Deserialization failed for key '{key}': {e}");
                        None
                    }
                },
            )
            .collect()
    }

    pub async fn move_transform(
        con: &mut MultiplexedConnection,
        name: &str,
        new_transform: SPTransform,
    ) {
        let redis_value: String = match con.get(name).await {
            Ok(Some(val)) => val,
            Ok(None) => {
                log::warn!("Transform '{name}' not found in Redis, cannot move.");
                return;
            }
            Err(e) => {
                log::error!("Failed to GET transform '{name}': {e}");
                return;
            }
        };

        let mut sp_tf_stamped: SPTransformStamped =
            match serde_json::from_str::<SPValue>(&redis_value) {
                Ok(SPValue::Transform(TransformOrUnknown::Transform(val))) => val,
                _ => {
                    log::error!("Value for '{name}' is not a valid transform, cannot move.");
                    return;
                }
            };

        sp_tf_stamped.transform = new_transform;

        let updated_value_json = match serde_json::to_string(&sp_tf_stamped.to_spvalue()) {
            Ok(s) => s,
            Err(e) => {
                log::error!("Failed to serialize updated transform '{name}': {e}");
                return;
            }
        };

        if let Err(e) = con.set::<_, _, ()>(name, updated_value_json).await {
            log::error!("Failed to SET updated transform '{name}': {e}");
        }
    }

    pub async fn reparent_transform(
        con: &mut MultiplexedConnection,
        new_parent_frame_id: &str,
        child_frame_id: &str,
    ) -> bool {
        let buffer = StateManager::get_all_transforms(con).await;

        let Some(original_transform) = buffer.get(child_frame_id) else {
            log::error!(
                "Can't reparent non-existent transform '{}'.",
                child_frame_id
            );
            return false;
        };

        let mut updated_transform = original_transform.clone();
        updated_transform.parent_frame_id = new_parent_frame_id.to_string();

        if check_would_produce_cycle(&updated_transform, &buffer) {
            log::error!(
                "Reparenting '{}' to '{}' would create a cycle. Aborting.",
                child_frame_id,
                new_parent_frame_id
            );
            return false;
        }

        let Some(lookup_tf) =
            lookup_transform_with_root(new_parent_frame_id, child_frame_id, "world", &buffer)
        else {
            log::error!(
                "Failed to calculate the new transform from '{}' to '{}'.",
                new_parent_frame_id,
                child_frame_id
            );
            return false;
        };

        updated_transform.transform = lookup_tf.transform;
        let updated_value_json = match serde_json::to_string(&updated_transform.to_spvalue()) {
            Ok(s) => s,
            Err(e) => {
                log::error!(
                    "Failed to serialize reparented transform '{}': {e}",
                    child_frame_id
                );
                return false;
            }
        };

        if let Err(e) = con
            .set::<_, _, ()>(child_frame_id, updated_value_json)
            .await
        {
            log::error!(
                "Failed to SET reparented transform '{}': {e}",
                child_frame_id
            );
            return false;
        }

        log::info!(
            "Successfully reparented transform '{}' to new parent '{}'.",
            child_frame_id,
            new_parent_frame_id
        );
        true
    }

    pub async fn lookup_transform(
        con: &mut MultiplexedConnection,
        parent_frame_id: &str,
        child_frame_id: &str,
    ) -> Option<SPTransformStamped> {
        let buffer = StateManager::get_all_transforms(con).await;

        let root = get_tree_root(&buffer).unwrap_or_else(|| "world".to_string());

        let result = lookup_transform_with_root(parent_frame_id, child_frame_id, &root, &buffer);

        if result.is_none() {
            log::error!(
                "Couldn't lookup transform from parent '{}' to child '{}'.",
                parent_frame_id,
                child_frame_id
            );
        }

        result
    }

    pub async fn load_transform_scenario(con: &mut MultiplexedConnection, path: &str) {
        match list_frames_in_dir(&path) {
            Ok(list) => {
                let frames = load_new_scenario(&list);
                // if overlay { ??
                StateManager::insert_transforms(con, frames).await;
            }
            Err(_e) => (),
        }
    }
}

fn create_assignment(key: &str, value: SPValue) -> SPAssignment {
    let variable = match &value {
        SPValue::Bool(_) => bv!(key),
        SPValue::Float64(_) => fv!(key),
        SPValue::Int64(_) => iv!(key),
        SPValue::String(_) => v!(key),
        SPValue::Time(_) => tv!(key),
        SPValue::Array(_) => av!(key),
        SPValue::Map(_) => mv!(key),
        SPValue::Transform(_) => tfv!(key),
    };
    SPAssignment::new(variable, value)
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use super::*;
    use serial_test::serial;
    use testcontainers::{ImageExt, core::ContainerPort, runners::AsyncRunner};
    use testcontainers_modules::redis::Redis;

    fn dummy_state() -> State {
        let mut state = State::new();
        state
            .state
            .insert("x".to_string(), assign!(iv!("x"), 1.to_spvalue()));
        state
            .state
            .insert("y".to_string(), assign!(iv!("y"), 2.to_spvalue()));
        state
            .state
            .insert("z".to_string(), assign!(iv!("z"), 3.to_spvalue()));
        state
    }

    fn create_dummy_transform(parent: &str, child: &str) -> SPTransformStamped {
        SPTransformStamped {
            active_transform: true,
            enable_transform: true,
            time_stamp: SystemTime::now(),
            parent_frame_id: parent.to_string(),
            child_frame_id: child.to_string(),
            transform: SPTransform::default(),
            metadata: MapOrUnknown::UNKNOWN,
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_set_and_get_single_value() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = StateManager::get_redis_mpx_connection().await;
        let key = "x";
        let value = 123.to_spvalue();

        StateManager::set_sp_value(&mut con, key, &value).await;

        let retrieved = StateManager::get_sp_value(&mut con, key)
            .await
            .expect("Value should exist");

        assert_eq!(value, retrieved);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_non_existent_value() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = StateManager::get_redis_mpx_connection().await;

        let retrieved = StateManager::get_sp_value(&mut con, "key-does-not-exist").await;
        assert!(
            retrieved.is_none(),
            "Getting a non-existent key should return None"
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_get_state_on_empty_db() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = StateManager::get_redis_mpx_connection().await;

        let state = StateManager::get_full_state(&mut con)
            .await
            .expect("redis_get_state should not fail on an empty DB");

        println!("{:?}", state);

        assert!(
            state.state.iter().len() == 1,
            "State map should only have heartbeat"
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_set_partial_and_get_full_state() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = StateManager::get_redis_mpx_connection().await;
        let mut initial_state = dummy_state();

        StateManager::set_state(&mut con, initial_state.clone()).await;

        let retrieved_state = StateManager::get_full_state(&mut con)
            .await
            .expect("Failed to get state");

        initial_state.state.remove("heartbeat").unwrap();
        assert_eq!(initial_state, retrieved_state);
    }

    #[tokio::test]
    #[serial]
    async fn test_overwrite_and_add_values() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = StateManager::get_redis_mpx_connection().await;

        StateManager::set_state(&mut con, dummy_state()).await;

        let mut partial_update = State::new();
        partial_update
            .state
            .insert("x".to_string(), assign!(iv!("x"), 99.to_spvalue()));
        partial_update
            .state
            .insert("j".to_string(), assign!(iv!("j"), 100.to_spvalue()));

        StateManager::set_state(&mut con, partial_update).await;

        let final_state = StateManager::get_full_state(&mut con).await.unwrap();

        let get_val = |s: &State, k: &str| s.state.get(k).unwrap().val.clone();

        assert_eq!(get_val(&final_state, "x"), 99.to_spvalue()); // Overwritten
        assert_eq!(get_val(&final_state, "y"), 2.to_spvalue()); // Unchanged
        assert_eq!(get_val(&final_state, "z"), 3.to_spvalue()); // Unchanged
        assert_eq!(get_val(&final_state, "j"), 100.to_spvalue()); // Added
    }

    #[tokio::test]
    #[serial]
    async fn test_add_and_get_all_transforms() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = StateManager::get_redis_mpx_connection().await;
        let mut transforms_to_add = HashMap::new();
        transforms_to_add.insert(
            "floor".to_string(),
            create_dummy_transform("world", "floor"),
        );
        transforms_to_add.insert(
            "table".to_string(),
            create_dummy_transform("world", "table"),
        );

        StateManager::insert_transforms(&mut con, transforms_to_add).await;
        let fetched_transforms = StateManager::get_all_transforms(&mut con).await;

        assert_eq!(2, fetched_transforms.len());
        assert!(fetched_transforms.contains_key("floor"));
        assert!(fetched_transforms.contains_key("table"));
        assert_eq!(
            fetched_transforms.get("table").unwrap().parent_frame_id,
            "world"
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_move_transform() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = StateManager::get_redis_mpx_connection().await;
        let mut initial_data = HashMap::new();
        initial_data.insert(
            "floor".to_string(),
            create_dummy_transform("world", "floor"),
        );
        StateManager::insert_transforms(&mut con, initial_data).await;

        let new_transform_data = SPTransform::default();

        StateManager::move_transform(&mut con, "floor", new_transform_data.clone()).await;

        let final_state = StateManager::get_all_transforms(&mut con).await;
        let moved_transform = final_state.get("floor").unwrap();
        assert_eq!(moved_transform.transform, new_transform_data);
    }

    #[tokio::test]
    #[serial]
    async fn test_lookup_transform() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = StateManager::get_redis_mpx_connection().await;
        let mut initial_data = HashMap::new();
        initial_data.insert(
            "floor".to_string(),
            create_dummy_transform("world", "floor"),
        );
        initial_data.insert(
            "table".to_string(),
            create_dummy_transform("floor", "table"),
        );
        initial_data.insert("cup".to_string(), create_dummy_transform("table", "cup"));
        StateManager::insert_transforms(&mut con, initial_data).await;

        let result = StateManager::lookup_transform(&mut con, "world", "cup").await;

        assert!(result.is_some());
        assert_eq!(result.unwrap().parent_frame_id, "world");
    }

    #[tokio::test]
    #[serial]
    async fn test_reparent_transform() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = StateManager::get_redis_mpx_connection().await;
        let mut initial_data = HashMap::new();

        initial_data.insert(
            "floor".to_string(),
            create_dummy_transform("world", "floor"),
        );
        initial_data.insert(
            "robot".to_string(),
            create_dummy_transform("world", "robot"),
        );
        StateManager::insert_transforms(&mut con, initial_data).await;
        let success = StateManager::reparent_transform(&mut con, "robot", "floor").await;

        assert_eq!(true, success);

        let final_state = StateManager::get_all_transforms(&mut con).await;
        let reparented_transform = final_state.get("floor").unwrap();
        assert_eq!(reparented_transform.parent_frame_id, "robot");
    }
}
