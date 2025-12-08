use redis::AsyncCommands;
use redis::aio::MultiplexedConnection;

pub(super) async fn remove_sp_values(con: &mut MultiplexedConnection, keys: &[String]) {
    if keys.is_empty() {
        return;
    }

    match con.del::<_, ()>(keys).await {
        Ok(_) => {}
        Err(e) => {
            log::error!("Redis DEL command for {} keys failed: {}", keys.len(), e);
        }
    }
}

#[cfg(test)]
mod tests_for_remove_sp_values {
    use super::remove_sp_values;
    use crate::*;
    use redis::AsyncCommands;
    use serial_test::serial;
    use testcontainers::{ImageExt, core::ContainerPort, runners::AsyncRunner};
    use testcontainers_modules::redis::Redis;

    #[tokio::test]
    #[serial]
    async fn test_remove_multiple_existing_keys() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;
        
        let keys = vec!["key_1".to_string(), "key_2".to_string(), "key_3".to_string()];

        for key in &keys {
            let _: () = con.set(key, "some_value").await.unwrap();
        }

        let count: usize = con.exists(&keys).await.unwrap();
        assert_eq!(count, 3, "Test setup failed: not all keys were set.");

        remove_sp_values(&mut con, &keys).await;

        let count_after: usize = con.exists(&keys).await.unwrap();
        assert_eq!(count_after, 0, "All keys should have been deleted.");
    }

    #[tokio::test]
    #[serial]
    async fn test_remove_mixed_keys() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;

        let key_exists = "key_exists".to_string();
        let key_missing = "key_missing".to_string();
        let keys_to_delete = vec![key_exists.clone(), key_missing.clone()];

        let _: () = con.set(&key_exists, "value").await.unwrap();
        
        remove_sp_values(&mut con, &keys_to_delete).await;

        let exists_after: bool = con.exists(&key_exists).await.unwrap();
        assert!(!exists_after, "The existing key should have been deleted.");
    }

    #[tokio::test]
    #[serial]
    async fn test_remove_empty_list() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;
        
        let control_key = "control_key";
        let _: () = con.set(control_key, "should_stay").await.unwrap();

        let keys: Vec<String> = vec![];

        remove_sp_values(&mut con, &keys).await;

        let control_exists: bool = con.exists(control_key).await.unwrap();
        assert!(control_exists, "The control key should remain untouched.");
    }
}