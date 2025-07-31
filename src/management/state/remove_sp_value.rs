use redis::AsyncCommands;
use redis::aio::MultiplexedConnection;

pub(super) async fn remove_sp_value(con: &mut MultiplexedConnection, key: &str) {
    match con.del::<_, ()>(&key).await {
        Ok(_) => {}
        Err(e) => {
            log::error!("Redis DEL command for key '{}' failed: {}", key, e);
        }
    }
}

#[cfg(test)]
mod tests_for_remove_sp_value {
    use super::remove_sp_value;
    use crate::*;
    use redis::AsyncCommands;
    use serial_test::serial;
    use testcontainers::{ImageExt, core::ContainerPort, runners::AsyncRunner};
    use testcontainers_modules::redis::Redis;

    #[tokio::test]
    #[serial]
    async fn test_remove_existing_key() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;
        let key = "key_to_delete";

        let _: () = con.set(key, "some_value").await.unwrap();
        let exists_before: bool = con.exists(key).await.unwrap();
        assert!(exists_before, "Test setup failed: key was not set.");

        remove_sp_value(&mut con, key).await;

        let exists_after: bool = con.exists(key).await.unwrap();
        assert!(!exists_after, "The key should have been deleted.");
    }

    #[tokio::test]
    #[serial]
    async fn test_remove_non_existing_key() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;
        let key = "key_that_does_not_exist";

        let exists_before: bool = con.exists(key).await.unwrap();
        assert!(!exists_before, "Test setup failed: key should not exist.");

        remove_sp_value(&mut con, key).await;

        let exists_after: bool = con.exists(key).await.unwrap();
        assert!(
            !exists_after,
            "A non-existent key should still not exist after DEL."
        );
    }
}
