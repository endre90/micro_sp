use crate::{State, StateManager};
use crate::SPConnection;
use redis::AsyncCommands;

// PERF (worst offender on the Redis side): `KEYS *` is O(total keyspace) and,
// because Redis is single-threaded, it *blocks the whole server* for the
// duration - no other client makes progress meanwhile. This is called by
// `sop_runner` (100 ms), `auto_operation_runner` (200 ms) and
// `planned_operation_runner` (200 ms), i.e. roughly 20 full keyspace scans per
// second, and the keyspace also holds every transform and every logger blob.
// That is very likely a large part of the CPU you see when a SOP is running,
// and it directly delays every other command - which is why state changes lag.
// Suggested fixes, best first:
//   1. Do not read the full state at all. Every one of these runners can
//      compute its key set up front from `get_all_var_keys()` (the planner,
//      goal, tf and timer runners already do exactly that) and use
//      `get_state_for_keys`. The keys for dynamically created operations are
//      known the moment the operation is instantiated, so they can be appended
//      to the set instead of being discovered by scanning.
//   2. If a whole-state read is genuinely needed, store the state in one Redis
//      HASH and use `HGETALL` - one command, one round trip, no keyspace scan.
//   3. As a stop-gap, replace `KEYS` with `SCAN` (non-blocking, cursored) or
//      maintain the key list in a Redis SET updated on add/remove and read with
//      `SMEMBERS`; and cache the key list in the runner, refreshing it only
//      when variables are actually added or removed.
// PERF: `MGET` on a large key list returns every value even if nothing changed,
// and `build_state` then `serde_json`-parses all of them. Suggested: keep the
// previous tick's raw strings and only re-parse entries whose string differs -
// a byte compare is far cheaper than a JSON parse, and in steady state almost
// nothing changes between ticks.
pub(super) async fn get_full_state(con: &mut SPConnection) -> Option<State> {
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

#[cfg(test)]
mod tests {
    use crate::*;
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

    #[tokio::test]
    #[serial]
    async fn test_get_full_state_on_empty_db() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;

        let state = StateManager::get_full_state(&mut con)
            .await
            .expect("get_full_state should not fail on an empty DB");

        assert!(state.state.is_empty(), "State map should be empty");
    }

    #[tokio::test]
    #[serial]
    async fn test_get_full_state_with_populated_db() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;
        let initial_state = dummy_state();

        StateManager::set_state(&mut con, &initial_state).await;

        let retrieved_state = StateManager::get_full_state(&mut con)
            .await
            .expect("Failed to get full state");

        assert_eq!(
            initial_state, retrieved_state,
            "Retrieved state should match the initial state"
        );
        assert_eq!(retrieved_state.state.len(), 3);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_full_state_with_malformed_data() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;

        let key1 = "valid_key_1";
        let value1 = 123.to_spvalue();
        StateManager::set_sp_value(&mut con, key1, &value1).await;

        let key2 = "valid_key_2";
        let value2 = false.to_spvalue();
        StateManager::set_sp_value(&mut con, key2, &value2).await;

        let malformed_key = "malformed_key";
        let _: () = redis::cmd("SET")
            .arg(malformed_key)
            .arg("this is not a valid spvalue json")
            .query_async(&mut con)
            .await
            .unwrap();

        let state = StateManager::get_full_state(&mut con)
            .await
            .expect("get_full_state should not fail with malformed data");

        assert_eq!(state.state.len(), 2, "State should contain 2 valid items");
        assert_eq!(state.get_value(key1, "t"), Some(value1));
        assert_eq!(state.get_value(key2, "t"), Some(value2));

        let result = std::panic::catch_unwind(|| state.get_value(malformed_key, "t"));
        assert!(result.is_err(), "Was expected to panic, but it did not.");
    }
}