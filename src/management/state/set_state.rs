use crate::State;
use crate::SPConnection;
use redis::{AsyncCommands, Value};

pub(super) async fn set_state(con: &mut SPConnection, state: &State) {
    let items_to_set: Vec<(String, String)> = state
        .state
        .clone()
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

#[cfg(test)]
mod tests_for_set_state {
    use super::set_state;
    use crate::*;
    use redis::AsyncCommands;
    use serial_test::serial;
    use std::collections::HashMap;
    use testcontainers::{ImageExt, core::ContainerPort, runners::AsyncRunner};
    use testcontainers_modules::redis::Redis;

    #[tokio::test]
    #[serial]
    async fn test_set_state_success() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;

        let key1 = "x".to_string();
        let val1 = SPValue::Int64(IntOrUnknown::Int64(1));
        let assignment1 = SPAssignment::new(iv!(&&key1), val1.clone());

        let key2 = "y".to_string();
        let val2 = SPValue::String(StringOrUnknown::String("hello".to_string()));
        let assignment2 = SPAssignment::new(v!(&&key2), val2.clone());

        let mut state_map = HashMap::new();
        state_map.insert(key1.clone(), assignment1);
        state_map.insert(key2.clone(), assignment2);
        let state_to_set = State { state: state_map };

        set_state(&mut con, &state_to_set).await;

        let result: Vec<String> = con.mget(&[key1, key2]).await.unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], serde_json::to_string(&val1).unwrap());
        assert_eq!(result[1], serde_json::to_string(&val2).unwrap());
    }

    #[tokio::test]
    #[serial]
    async fn test_set_state_empty() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;

        let empty_state = State::new();
        set_state(&mut con, &empty_state).await;

        let keys: Vec<String> = con.keys("*").await.unwrap();
        assert!(keys.is_empty());
    }

    #[tokio::test]
    #[serial]
    async fn test_set_state_overwrite_existing_keys() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;

        let key1 = "x".to_string();
        let key2 = "y".to_string();

        let _: () = con
            .mset(&[(&key1, "initial_x"), (&key2, "initial_y")])
            .await
            .unwrap();

        let new_val1 = SPValue::Int64(IntOrUnknown::Int64(999));
        let assignment1 = SPAssignment::new(iv!(&&key1), new_val1.clone());

        let new_val2 = SPValue::String(StringOrUnknown::String("new_world".to_string()));
        let assignment2 = SPAssignment::new(v!(&&key2), new_val2.clone());

        let mut state_map = HashMap::new();
        state_map.insert(key1.clone(), assignment1);
        state_map.insert(key2.clone(), assignment2);
        let state_to_set = State { state: state_map };

        set_state(&mut con, &state_to_set).await;

        let result: Vec<String> = con.mget(&[key1, key2]).await.unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], serde_json::to_string(&new_val1).unwrap());
        assert_eq!(result[1], serde_json::to_string(&new_val2).unwrap());
    }

    #[tokio::test]
    #[serial]
    async fn test_set_state_with_only_some_keys() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;

        let _: () = con
            .mset(&[("a", "1"), ("b", "2"), ("c", "3")])
            .await
            .unwrap();

        let key_b = "b".to_string();
        let val_b = SPValue::Int64(IntOrUnknown::Int64(222));
        let assignment_b = SPAssignment::new(iv!(&&key_b), val_b.clone());

        let key_d = "d".to_string();
        let val_d = SPValue::String(StringOrUnknown::String("new_d".to_string()));
        let assignment_d = SPAssignment::new(v!(&&key_d), val_d.clone());

        let mut state_map = HashMap::new();
        state_map.insert(key_b.clone(), assignment_b);
        state_map.insert(key_d.clone(), assignment_d);
        let state_to_set = State { state: state_map };

        set_state(&mut con, &state_to_set).await;

        let result: Vec<Option<String>> = con.mget(&["a", "b", "c", "d"]).await.unwrap();
        assert_eq!(result[0], Some("1".to_string()));
        assert_eq!(result[1], Some(serde_json::to_string(&val_b).unwrap()));
        assert_eq!(result[2], Some("3".to_string()));
        assert_eq!(result[3], Some(serde_json::to_string(&val_d).unwrap()));
    }

    /// One unserialisable value (serde refuses a `SystemTime` before the UNIX
    /// epoch) must not cost the rest of the state: the bad key is dropped with
    /// a log and every other key in the same write still lands.
    #[tokio::test]
    #[serial]
    async fn an_unserializable_value_is_dropped_but_the_rest_of_the_state_is_written() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;

        let good_val = SPValue::Int64(IntOrUnknown::Int64(7));
        let bad_val = SPValue::Time(TimeOrUnknown::Time(
            std::time::SystemTime::UNIX_EPOCH - std::time::Duration::from_secs(1),
        ));
        assert!(
            serde_json::to_string(&bad_val).is_err(),
            "test premise: a pre-epoch SystemTime must be unserializable"
        );

        let mut state_map = HashMap::new();
        state_map.insert(
            "good".to_string(),
            SPAssignment::new(iv!("good"), good_val.clone()),
        );
        state_map.insert(
            "bad".to_string(),
            SPAssignment::new(tv!("bad"), bad_val),
        );

        set_state(&mut con, &State { state: state_map }).await;

        let result: Vec<Option<String>> = con.mget(&["good", "bad"]).await.unwrap();
        assert_eq!(
            result[0],
            Some(serde_json::to_string(&good_val).unwrap()),
            "the serialisable half of the state must still be written"
        );
        assert_eq!(result[1], None, "the unserialisable key must be skipped");
    }

    /// A refused `MSET` (a real ACL permission error) is logged rather than
    /// panicking, and leaves the keyspace exactly as it was.
    #[tokio::test]
    #[serial]
    async fn a_refused_mset_is_logged_and_changes_nothing() {
        // ACL SETUSER needs Redis 6+; the crate's default test image is 5.0.
        let _container = Redis::default()
            .with_tag("7.2")
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;
        let _: () = con.set("untouched", "original").await.unwrap();

        let _: () = redis::cmd("ACL")
            .arg("SETUSER")
            .arg("default")
            .arg("-mset")
            .query_async(&mut con)
            .await
            .unwrap();

        let mut state_map = HashMap::new();
        state_map.insert(
            "untouched".to_string(),
            SPAssignment::new(v!("untouched"), "overwritten".to_spvalue()),
        );
        set_state(&mut con, &State { state: state_map }).await;

        // Restore before asserting so the shared Redis is left clean.
        let _: () = redis::cmd("ACL")
            .arg("SETUSER")
            .arg("default")
            .arg("+mset")
            .query_async(&mut con)
            .await
            .unwrap();

        let stored: Option<String> = con.get("untouched").await.unwrap();
        assert_eq!(
            stored,
            Some("original".to_string()),
            "a denied MSET must not have modified anything"
        );
    }
}