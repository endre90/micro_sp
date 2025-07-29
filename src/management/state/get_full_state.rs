use crate::{State, StateManager};
use redis::{AsyncCommands, aio::MultiplexedConnection};

pub(super) async fn get_full_state(con: &mut MultiplexedConnection) -> Option<State> {
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
    use testcontainers::{core::ContainerPort, runners::AsyncRunner, ImageExt};
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

        assert_eq!(initial_state, retrieved_state, "Retrieved state should match the initial state");
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
        assert_eq!(state.get_value(key1), Some(value1));
        assert_eq!(state.get_value(key2), Some(value2));

        assert_eq!(state.get_value(malformed_key), None);

    }
}