// use r2r_transform::*;
use redis::aio::MultiplexedConnection;
use std::collections::HashMap;

use std::env;

use crate::*;
use redis::{AsyncCommands, Client, Value};
use tokio::time::{Duration, interval};

// New, untested
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

pub async fn redis_get_state(con: &mut MultiplexedConnection) -> Option<State> {
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

    Some(build_state_from_redis(keys, values))
}

pub fn build_state_from_redis(keys: Vec<String>, values: Vec<Option<String>>) -> State {
    let mut state_map = HashMap::new();

    for (key, maybe_value) in keys.into_iter().zip(values.into_iter()) {
        let Some(value_str) = maybe_value else {
            continue;
        };

        if let Ok(sp_value) = serde_json::from_str::<SPValue>(&value_str) {
            let assignment = create_assignment(&key, sp_value);
            state_map.insert(key, assignment);
        } else {
            log::warn!("Failed to deserialize value for key '{}'.", key);
        }
    }

    State { state: state_map }
}

pub async fn redis_get_sp_value(con: &mut MultiplexedConnection, var: &str) -> Option<SPValue> {
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

pub async fn redis_set_state(con: &mut MultiplexedConnection, state: State) {
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

pub async fn redis_set_sp_value(con: &mut MultiplexedConnection, key: &str, value: &SPValue) {
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
    use super::*;
    use serial_test::serial;
    use testcontainers::{ImageExt, core::ContainerPort, runners::AsyncRunner};
    use testcontainers_modules::redis::Redis;

    /// Helper to create a dummy state for testing.
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

    #[tokio::test]
    #[serial]
    async fn test_set_and_get_single_value() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = get_redis_mpx_connection().await;
        let key = "x";
        let value = 123.to_spvalue();

        redis_set_sp_value(&mut con, key, &value).await;

        let retrieved = redis_get_sp_value(&mut con, key)
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

        let mut con = get_redis_mpx_connection().await;

        let retrieved = redis_get_sp_value(&mut con, "key-does-not-exist").await;
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

        let mut con = get_redis_mpx_connection().await;

        let state = redis_get_state(&mut con)
            .await
            .expect("redis_get_state should not fail on an empty DB");

        println!("{:?}", state);

        assert!(state.state.is_empty(), "State map should be empty");
    }

    #[tokio::test]
    #[serial]
    async fn test_set_partial_and_get_full_state() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = get_redis_mpx_connection().await;
        let initial_state = dummy_state();

        redis_set_state(&mut con, initial_state.clone()).await;

        let retrieved_state = redis_get_state(&mut con)
            .await
            .expect("Failed to get state");

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

        let mut con = get_redis_mpx_connection().await;

        redis_set_state(&mut con, dummy_state()).await;

        let mut partial_update = State::new();
        partial_update
            .state
            .insert("x".to_string(), assign!(iv!("x"), 99.to_spvalue()));
        partial_update
            .state
            .insert("j".to_string(), assign!(iv!("j"), 100.to_spvalue()));

        redis_set_state(&mut con, partial_update).await;

        let final_state = redis_get_state(&mut con).await.unwrap();

        let get_val = |s: &State, k: &str| s.state.get(k).unwrap().val.clone();

        assert_eq!(get_val(&final_state, "x"), 99.to_spvalue()); // Overwritten
        assert_eq!(get_val(&final_state, "y"), 2.to_spvalue()); // Unchanged
        assert_eq!(get_val(&final_state, "z"), 3.to_spvalue()); // Unchanged
        assert_eq!(get_val(&final_state, "j"), 100.to_spvalue()); // Added
    }
}
