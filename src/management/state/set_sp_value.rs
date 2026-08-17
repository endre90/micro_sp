use crate::SPValue;
use crate::SPConnection;
use redis::AsyncCommands;

pub(super) async fn set_sp_value(con: &mut SPConnection, key: &str, value: &SPValue) {
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

    /// A value that cannot be serialised (serde refuses a `SystemTime` before
    /// the UNIX epoch) must be dropped with a log rather than panicking, and
    /// crucially must not clobber whatever is already stored under that key.
    #[tokio::test]
    #[serial]
    async fn an_unserializable_value_is_skipped_and_leaves_the_key_alone() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;
        let key = "unserializable";

        let good = SPValue::Int64(IntOrUnknown::Int64(5));
        set_sp_value(&mut con, key, &good).await;

        let pre_epoch = SPValue::Time(TimeOrUnknown::Time(
            std::time::SystemTime::UNIX_EPOCH - std::time::Duration::from_secs(1),
        ));
        assert!(
            serde_json::to_string(&pre_epoch).is_err(),
            "test premise: a pre-epoch SystemTime must be unserializable"
        );

        set_sp_value(&mut con, key, &pre_epoch).await;

        let result: String = con.get(key).await.unwrap();
        assert_eq!(
            result,
            serde_json::to_string(&good).unwrap(),
            "the failed write must leave the previous value in place"
        );
    }

    /// A refused `SET` (here a real ACL permission error) is logged and
    /// swallowed - the caller does not panic - and nothing is written.
    #[tokio::test]
    #[serial]
    async fn a_refused_set_is_logged_and_writes_nothing() {
        // ACL SETUSER needs Redis 6+; the crate's default test image is 5.0.
        let _container = Redis::default()
            .with_tag("7.2")
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;
        let key = "denied_key";

        let _: () = redis::cmd("ACL")
            .arg("SETUSER")
            .arg("default")
            .arg("-set")
            .query_async(&mut con)
            .await
            .unwrap();

        set_sp_value(&mut con, key, &SPValue::Int64(IntOrUnknown::Int64(1))).await;

        // Restore before asserting so a failure cannot leave the shared,
        // fixed-port Redis unusable for later tests.
        let _: () = redis::cmd("ACL")
            .arg("SETUSER")
            .arg("default")
            .arg("+set")
            .query_async(&mut con)
            .await
            .unwrap();

        let stored: Option<String> = con.get(key).await.unwrap();
        assert_eq!(stored, None, "a denied SET must not write anything");

        // And the manager is still usable once the permission comes back.
        set_sp_value(&mut con, key, &SPValue::Int64(IntOrUnknown::Int64(2))).await;
        let stored: Option<String> = con.get(key).await.unwrap();
        assert_eq!(
            stored,
            Some(serde_json::to_string(&SPValue::Int64(IntOrUnknown::Int64(2))).unwrap())
        );
    }
}