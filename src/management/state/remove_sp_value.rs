use redis::AsyncCommands;
use crate::SPConnection;

pub(super) async fn remove_sp_value(con: &mut SPConnection, key: &str) {
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

    /// A refused `DEL` (a real ACL permission error) is logged and swallowed:
    /// the caller does not panic and, since nothing was deleted, the key is
    /// still there afterwards.
    #[tokio::test]
    #[serial]
    async fn a_refused_del_is_logged_and_leaves_the_key_in_place() {
        // ACL SETUSER needs Redis 6+; the crate's default test image is 5.0.
        let _container = Redis::default()
            .with_tag("7.2")
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;
        let key = "survivor";
        let _: () = con.set(key, "still_here").await.unwrap();

        let _: () = redis::cmd("ACL")
            .arg("SETUSER")
            .arg("default")
            .arg("-del")
            .query_async(&mut con)
            .await
            .unwrap();

        remove_sp_value(&mut con, key).await;

        // Restore before asserting so the shared Redis is left clean.
        let _: () = redis::cmd("ACL")
            .arg("SETUSER")
            .arg("default")
            .arg("+del")
            .query_async(&mut con)
            .await
            .unwrap();

        let value: Option<String> = con.get(key).await.unwrap();
        assert_eq!(
            value,
            Some("still_here".to_string()),
            "a denied DEL must leave the key untouched"
        );

        // And deleting works again once the permission is back.
        remove_sp_value(&mut con, key).await;
        let exists: bool = con.exists(key).await.unwrap();
        assert!(!exists);
    }
}
