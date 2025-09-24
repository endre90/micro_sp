use redis::cmd;
use redis::aio::MultiplexedConnection;

pub(super) async fn flush_state(con: &mut MultiplexedConnection) {
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