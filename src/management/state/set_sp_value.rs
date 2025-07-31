use crate::SPValue;
use redis::{AsyncCommands, aio::MultiplexedConnection};

pub(super) async fn set_sp_value(con: &mut MultiplexedConnection, key: &str, value: &SPValue) {
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

#[cfg(test)]
mod tests_for_get_state_for_keys {
    use super::set_sp_value;
    use crate::*;
    use redis::AsyncCommands;
    use serial_test::serial;
    use testcontainers::{ImageExt, core::ContainerPort, runners::AsyncRunner};
    use testcontainers_modules::redis::Redis;

    #[tokio::test]
    #[serial]
    async fn test_set_sp_value_int_success() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;
        let key = "my_int_key";
        let sp_value = SPValue::Int64(IntOrUnknown::Int64(42));

        set_sp_value(&mut con, key, &sp_value).await;

        let result: String = con.get(key).await.unwrap();
        let expected_json = serde_json::to_string(&sp_value).unwrap();

        assert_eq!(result, expected_json);
    }

    #[tokio::test]
    #[serial]
    async fn test_set_sp_value_string_success() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;
        let key = "my_string_key";
        let sp_value = SPValue::String(StringOrUnknown::String("test_value".to_string()));

        set_sp_value(&mut con, key, &sp_value).await;

        let result: String = con.get(key).await.unwrap();
        let expected_json = serde_json::to_string(&sp_value).unwrap();

        assert_eq!(result, expected_json);
    }

    #[tokio::test]
    #[serial]
    async fn test_set_sp_value_bool_success() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;
        let key = "my_bool_key";
        let sp_value = SPValue::Bool(BoolOrUnknown::Bool(false));

        set_sp_value(&mut con, key, &sp_value).await;

        let result: String = con.get(key).await.unwrap();
        let expected_json = serde_json::to_string(&sp_value).unwrap();

        assert_eq!(result, expected_json);
    }

    #[tokio::test]
    #[serial]
    async fn test_set_sp_value_overwrite() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;
        let key = "overwrite_key";
        let initial_value = SPValue::Int64(IntOrUnknown::Int64(1));
        let new_value = SPValue::Int64(IntOrUnknown::Int64(99));

        set_sp_value(&mut con, key, &initial_value).await;
        set_sp_value(&mut con, key, &new_value).await;

        let result: String = con.get(key).await.unwrap();
        let expected_json = serde_json::to_string(&new_value).unwrap();

        assert_eq!(result, expected_json);
    }
}
