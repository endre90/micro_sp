use crate::SPConnection;
use redis::AsyncCommands;

use crate::State;

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

    /// A failed `MGET` (a real ACL permission error) must surface as `None`
    /// rather than as an empty state: callers treat an empty read as "Redis was
    /// flushed" and would restore a snapshot over live data.
    #[tokio::test]
    #[serial]
    async fn a_failed_mget_yields_none_rather_than_an_empty_state() {
        // ACL SETUSER needs Redis 6+; the crate's default test image is 5.0.
        let _container = Redis::default()
            .with_tag("7.2")
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;
        let val = SPValue::Int64(IntOrUnknown::Int64(1));
        let _: () = con.set("x", serde_json::to_string(&val).unwrap()).await.unwrap();

        let _: () = redis::cmd("ACL")
            .arg("SETUSER")
            .arg("default")
            .arg("-mget")
            .query_async(&mut con)
            .await
            .unwrap();

        let keys = vec!["x".to_string()];
        let denied = get_state_for_keys(&mut con, &keys, "test").await;

        // Restore before asserting so the shared Redis is left clean.
        let _: () = redis::cmd("ACL")
            .arg("SETUSER")
            .arg("default")
            .arg("+mget")
            .query_async(&mut con)
            .await
            .unwrap();

        assert!(denied.is_none(), "a failed MGET must report failure");

        let recovered = get_state_for_keys(&mut con, &keys, "test")
            .await
            .expect("the read must work again once the permission is back");
        assert_eq!(recovered.get_value("x", "test"), Some(val));
    }
}
