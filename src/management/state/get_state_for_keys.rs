use crate::SPConnection;
use redis::AsyncCommands;

use crate::State;

// PERF: this is the good path - one `MGET` for a fixed key set, no keyspace
// scan - and every runner now uses it.
// DONE: PERF: `keys.clone()` on the way into `build_state` copied every key
// `String` on every tick, for every runner. It calls the borrowing form now.
// PERF: for very large key sets, `MGET` sends the whole key list on every tick.
// With a HASH layout this becomes `HMGET` on one key, and with keyspace
// notifications you only need to fetch the keys that actually changed.
// NOTE on test coverage: the `Err` arm of the `MGET` below is not exercised.
// `MGET` returns `nil` for a key of the wrong type rather than erroring (see
// the same note in `get_full_state.rs`), so hitting this branch would require
// a genuine network/IO failure rather than a settable Redis state - not
// practical to test without mocking the connection.
pub(super) async fn get_state_for_keys(
    con: &mut SPConnection,
    keys: &Vec<String>,
    log_target: &str
) -> Option<State> {
    if keys.is_empty() {
        return Some(State::new());
    }

    let values: Vec<Option<String>> = match con.mget(keys).await {
        Ok(v) => v,
        Err(e) => {
            log::error!(target: &log_target, "Failed to get values from Redis: {e}");
            return None;
        }
    };

    Some(super::build_state::build_state_from_slice(keys, values))
}

#[cfg(test)]
mod tests_for_get_state_for_keys {
    use super::get_state_for_keys;
    use crate::*;
    use redis::AsyncCommands;
    use serial_test::serial;
    use std::collections::HashMap;
    use testcontainers::{ImageExt, core::ContainerPort, runners::AsyncRunner};
    use testcontainers_modules::redis::Redis;

    #[tokio::test]
    #[serial]
    async fn test_get_state_for_all_existing_keys() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;

        let var1 = SPVariable {
            name: "x".to_string(),
            value_type: SPValueType::Int64,
        };
        let val1 = SPValue::Int64(IntOrUnknown::Int64(1));
        let assignment1 = SPAssignment::new(var1.clone(), val1.clone());

        let var2 = SPVariable {
            name: "y".to_string(),
            value_type: SPValueType::String,
        };
        let val2 = SPValue::String(StringOrUnknown::String("hello".to_string()));
        let assignment2 = SPAssignment::new(var2.clone(), val2.clone());

        let _: () = con
            .mset(&[
                ("x", serde_json::to_string(&val1).unwrap()),
                ("y", serde_json::to_string(&val2).unwrap()),
            ])
            .await
            .unwrap();

        let keys_to_get = vec!["x".to_string(), "y".to_string()];
        let result_state = get_state_for_keys(&mut con, &keys_to_get, "test").await.unwrap();

        let mut expected_state_map = HashMap::new();
        expected_state_map.insert("x".to_string(), assignment1);
        expected_state_map.insert("y".to_string(), assignment2);

        assert_eq!(result_state.state, expected_state_map);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_state_for_partial_keys() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;

        let var1 = SPVariable {
            name: "x".to_string(),
            value_type: SPValueType::Int64,
        };
        let val1 = SPValue::Int64(IntOrUnknown::Int64(100));
        let assignment1 = SPAssignment::new(var1.clone(), val1.clone());

        let _: () = con
            .set("x", serde_json::to_string(&val1).unwrap())
            .await
            .unwrap();

        let keys_to_get = vec!["x".to_string(), "z".to_string()];
        let result_state = get_state_for_keys(&mut con, &keys_to_get, "test").await.unwrap();

        let mut expected_state_map = HashMap::new();
        expected_state_map.insert("x".to_string(), assignment1);

        assert_eq!(result_state.state, expected_state_map);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_state_for_no_existing_keys() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;

        let keys_to_get = vec!["a".to_string(), "b".to_string()];
        let result_state = get_state_for_keys(&mut con, &keys_to_get, "test").await.unwrap();

        assert!(result_state.state.is_empty());
    }

    #[tokio::test]
    #[serial]
    async fn test_get_state_for_empty_keys_vector() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;

        let keys_to_get = vec![];
        let result_state = get_state_for_keys(&mut con, &keys_to_get, "test").await.unwrap();

        assert_eq!(result_state, State::new());
    }

    #[tokio::test]
    #[serial]
    async fn test_get_state_with_invalid_data_in_redis() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;

        let var_good = SPVariable {
            name: "good_key".to_string(),
            value_type: SPValueType::Bool,
        };
        let val_good = SPValue::Bool(BoolOrUnknown::Bool(true));
        let assignment_good = SPAssignment::new(var_good, val_good.clone());

        let _: () = con
            .mset(&[
                ("good_key", serde_json::to_string(&val_good).unwrap()),
                ("bad_key", "this is not valid json".to_string()),
            ])
            .await
            .unwrap();

        let keys_to_get = vec!["good_key".to_string(), "bad_key".to_string()];
        let result_state = get_state_for_keys(&mut con, &keys_to_get, "test").await.unwrap();

        let mut expected_state_map = HashMap::new();
        expected_state_map.insert("good_key".to_string(), assignment_good);

        assert_eq!(result_state.state, expected_state_map);
    }
}
