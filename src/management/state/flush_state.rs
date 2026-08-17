use redis::cmd;
use crate::SPConnection;

pub(super) async fn flush_state(con: &mut SPConnection) {
    match cmd("FLUSHDB").query_async::<()>(con).await {
        Ok(_) => {}
        Err(e) => {
            log::error!("Redis FLUSHDB command failed: {}", e);
        }
    }
}

#[cfg(test)]
mod tests_for_flush_state {
    use super::flush_state;
    use crate::*;
    use redis::{cmd, AsyncCommands};
    use serial_test::serial;
    use testcontainers::{core::ContainerPort, runners::AsyncRunner, ImageExt};
    use testcontainers_modules::redis::Redis;

    #[tokio::test]
    #[serial]
    async fn test_flush_populated_database() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;

        let _: () = con.set("key1", "value1").await.unwrap();
        let _: () = con.set("key2", "value2").await.unwrap();
        let _: () = con.set("key3", "value3").await.unwrap();

        let size_before: usize = cmd("DBSIZE").query_async(&mut con).await.unwrap();
        assert_eq!(size_before, 3, "Test setup failed: keys were not set.");

        flush_state(&mut con).await;

        let size_after: usize = cmd("DBSIZE").query_async(&mut con).await.unwrap();
        assert_eq!(size_after, 0, "The database should be empty after flush.");
    }

    /// A FLUSHDB that Redis refuses (here: the `default` user has been denied
    /// the command via ACL - a real permission error, not a mock) must be
    /// logged rather than panicking, and - crucially - must leave the existing
    /// data untouched rather than silently losing it.
    #[tokio::test]
    #[serial]
    async fn test_flush_logs_and_does_not_panic_when_denied() {
        // ACL SETUSER requires Redis 6+; the crate's default test image is 5.0.
        let _container = Redis::default()
            .with_tag("7.2")
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;
        let _: () = con.set("survivor", "value").await.unwrap();

        let _: () = cmd("ACL")
            .arg("SETUSER")
            .arg("default")
            .arg("-flushdb")
            .query_async(&mut con)
            .await
            .unwrap();

        flush_state(&mut con).await;

        let _: () = cmd("ACL")
            .arg("SETUSER")
            .arg("default")
            .arg("+flushdb")
            .query_async(&mut con)
            .await
            .unwrap();

        let survivor: String = con.get("survivor").await.unwrap();
        assert_eq!(
            survivor, "value",
            "a denied FLUSHDB must not have removed existing data"
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_flush_empty_database() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;

        let size_before: usize = cmd("DBSIZE").query_async(&mut con).await.unwrap();
        assert_eq!(size_before, 0, "Test setup failed: DB should be empty.");

        flush_state(&mut con).await;

        let size_after: usize = cmd("DBSIZE").query_async(&mut con).await.unwrap();
        assert_eq!(
            size_after, 0,
            "Flushing an empty database should result in an empty database."
        );
    }
}