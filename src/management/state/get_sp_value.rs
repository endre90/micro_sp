use crate::SPValue;
use crate::SPConnection;
use redis::AsyncCommands;

pub(super) async fn get_sp_value(con: &mut SPConnection, var: &str) -> Option<SPValue> {
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

#[cfg(test)]
mod tests {
    use crate::{ConnectionManager, SPValue, ToSPValue};

    use super::get_sp_value;

    use redis::{AsyncCommands, Client};
    use serial_test::serial;
    use testcontainers::{ImageExt, core::ContainerPort, runners::AsyncRunner};
    use testcontainers_modules::redis::Redis;

    #[tokio::test]
    #[serial]
    async fn test_get_sp_value_success() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;

        let key = "test_key_success";
        let expected_value = SPValue::Map(crate::MapOrUnknown::Map(vec![
            ("field1".to_spvalue(), "hello".to_spvalue()),
            ("field2".to_spvalue(), 123.to_spvalue()),
        ]));

        let json_string = serde_json::to_string(&expected_value).unwrap();

        let _: () = con.set(key, &json_string).await.unwrap();

        let result = get_sp_value(&mut con, key).await;
        assert_eq!(result, Some(expected_value));
    }

    #[tokio::test]
    #[serial]
    async fn test_get_sp_value_key_not_found() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;

        let key = "test_key_not_found";
        let result = get_sp_value(&mut con, key).await;
        assert_eq!(result, None);
    }

    /// `GET` (unlike `MGET`) genuinely errors with WRONGTYPE when the key
    /// holds a non-string value, so this is a real Redis error, not a mocked
    /// one. It must be logged and treated the same as "not found".
    #[tokio::test]
    #[serial]
    async fn test_get_sp_value_wrong_redis_type() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;

        let key = "test_key_wrong_type";
        let _: () = con.rpush(key, "an_element").await.unwrap();

        let result = get_sp_value(&mut con, key).await;

        assert_eq!(
            result, None,
            "GET against a list key must fail with WRONGTYPE and surface as None"
        );

        // The list itself must be untouched by the failed read.
        let list_len: usize = con.llen(key).await.unwrap();
        assert_eq!(list_len, 1);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_sp_value_deserialization_error() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;

        let key = "test_key_bad_json";
        let malformed_json = "{ \"key\": \"value\", }"; // Extra comma is invalid JSON

        let _: () = con.set(key, malformed_json).await.unwrap();

        let result = get_sp_value(&mut con, key).await;

        assert_eq!(result, None);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_sp_value_empty_string_is_invalid_json() {
        let container = Redis::default().start().await.unwrap();
        let host_port = container.get_host_port_ipv4(6379).await.unwrap();
        let url = format!("redis://127.0.0.1:{host_port}");
        let client = Client::open(url).unwrap();
        let mut con = redis::aio::ConnectionManager::new(client).await.unwrap();

        let key = "test_key_empty_string";
        let empty_string = "";

        let _: () = con.set(key, empty_string).await.unwrap();

        let result = get_sp_value(&mut con, key).await;

        assert_eq!(result, None);
    }
}