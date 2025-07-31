use redis::AsyncCommands;
use redis::aio::MultiplexedConnection;
use std::error::Error;
use crate::tf_key;

pub(super) async fn remove_transform(
    con: &mut MultiplexedConnection,
    key: &str,
) -> Result<(), Box<dyn Error>> {
    let redis_key = tf_key(key);
    con.del::<_, ()>(&redis_key).await?;
    Ok(())
}

#[cfg(test)]
mod tests_for_remove_transform {
    use super::remove_transform;
    use crate::*;
    use redis::AsyncCommands;
    use serial_test::serial;
    use testcontainers::{ImageExt, core::ContainerPort, runners::AsyncRunner};
    use testcontainers_modules::redis::Redis;

    #[tokio::test]
    #[serial]
    async fn test_remove_transform_existing_key() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;
        let child_id = "key_to_delete";
        let redis_key = tf_key(child_id);

        let _: () = con.set(&redis_key, "some_value").await.unwrap();
        let exists_before: bool = con.exists(&redis_key).await.unwrap();
        assert!(exists_before, "Test setup failed: key was not set.");

        let _ = remove_transform(&mut con, child_id).await;

        let exists_after: bool = con.exists(&redis_key).await.unwrap();
        assert!(!exists_after, "The key should have been deleted.");
    }

    #[tokio::test]
    #[serial]
    async fn test_remove_transform_non_existing_key() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;
        let child_id = "key_that_does_not_exist";
        let redis_key = tf_key(child_id);

        let exists_before: bool = con.exists(&redis_key).await.unwrap();
        assert!(!exists_before, "Test setup failed: key should not exist.");

        let _ = remove_transform(&mut con, child_id).await;

        let exists_after: bool = con.exists(&redis_key).await.unwrap();
        assert!(
            !exists_after,
            "A non-existent key should still not exist after DEL."
        );
    }
}
