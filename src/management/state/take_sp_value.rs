use crate::SPConnection;
use crate::SPValue;
use redis::AsyncCommands;

pub(super) async fn take_sp_value(
    con: &mut SPConnection,
    key: &str,
    replacement: &SPValue,
) -> Option<SPValue> {
    let replacement_str = match serde_json::to_string(replacement) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to serialize the replacement for key '{key}': {e}");
            return None;
        }
    };

    let previous: Option<String> = match con.getset(key, replacement_str).await {
        Ok(value) => value,
        Err(e) => {
            log::error!("Redis GETSET command for key '{key}' failed: {e}");
            return None;
        }
    };

    // The key did not exist. `GETSET` has still created it holding the
    // replacement, which is what the callers want - a trigger or a queue that
    // is always present, so a later read cannot miss it.
    let previous_str = previous?;

    match serde_json::from_str(&previous_str) {
        Ok(deserialized_value) => Some(deserialized_value),
        Err(e) => {
            // The old value is already gone at this point. That is no worse
            // than the blind overwrite this call replaces, but it is worth an
            // error rather than a silent `None`.
            log::error!("Deserializing the value taken from '{key}' failed: {e}");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::take_sp_value;
    use crate::*;
    use redis::AsyncCommands;
    use serial_test::serial;
    use testcontainers::{ImageExt, core::ContainerPort, runners::AsyncRunner};
    use testcontainers_modules::redis::Redis;

    #[tokio::test]
    #[serial]
    async fn takes_the_previous_value_and_leaves_the_replacement() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;
        StateManager::set_sp_value(&mut con, "trigger", &true.to_spvalue()).await;

        let taken = take_sp_value(&mut con, "trigger", &false.to_spvalue()).await;

        assert_eq!(taken, Some(true.to_spvalue()), "the old value is returned");
        let now: String = con.get("trigger").await.unwrap();
        assert_eq!(
            now,
            serde_json::to_string(&false.to_spvalue()).unwrap(),
            "the replacement is in place"
        );
    }

    /// The point of the whole exercise: a value written *after* the take is not
    /// erased by it. Under the read-then-blind-write it replaces, the producer's
    /// write would be overwritten by the consumer's stale `false`.
    #[tokio::test]
    #[serial]
    async fn a_write_landing_after_the_take_survives() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;
        StateManager::set_sp_value(&mut con, "trigger", &true.to_spvalue()).await;

        // The consumer takes the request and clears it in one command.
        let first = take_sp_value(&mut con, "trigger", &false.to_spvalue()).await;
        assert_eq!(first, Some(true.to_spvalue()));

        // A producer posts a new request while the consumer is still working.
        StateManager::set_sp_value(&mut con, "trigger", &true.to_spvalue()).await;

        // The consumer's next tick still sees it.
        let second = take_sp_value(&mut con, "trigger", &false.to_spvalue()).await;
        assert_eq!(
            second,
            Some(true.to_spvalue()),
            "the request posted mid-work must not be lost"
        );
    }

    #[tokio::test]
    #[serial]
    async fn a_missing_key_yields_none_but_is_created() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;
        let _: () = con.del("absent").await.unwrap();

        let taken = take_sp_value(&mut con, "absent", &false.to_spvalue()).await;

        assert_eq!(taken, None);
        let now: String = con.get("absent").await.unwrap();
        assert_eq!(
            now,
            serde_json::to_string(&false.to_spvalue()).unwrap(),
            "the key must exist afterwards, or a later state read would miss it"
        );
    }

    /// Draining a queue key: the whole array comes back and the key is left
    /// holding an empty one.
    #[tokio::test]
    #[serial]
    async fn drains_a_queue_key() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;
        let queued = vec!["a".to_spvalue(), "b".to_spvalue()].to_spvalue();
        StateManager::set_sp_value(&mut con, "queue", &queued).await;

        let taken = take_sp_value(&mut con, "queue", &Vec::<SPValue>::new().to_spvalue()).await;

        assert_eq!(taken, Some(queued));
        let now: String = con.get("queue").await.unwrap();
        assert_eq!(
            now,
            serde_json::to_string(&Vec::<SPValue>::new().to_spvalue()).unwrap(),
            "the queue is left empty, not deleted"
        );
    }

    /// A value that is not valid JSON is reported rather than returned - and the
    /// replacement still lands, so a poisoned key cannot wedge a runner forever.
    #[tokio::test]
    #[serial]
    async fn an_undeserializable_value_yields_none_and_is_still_replaced() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;
        let _: () = con.set("poisoned", "not json at all").await.unwrap();

        let taken = take_sp_value(&mut con, "poisoned", &false.to_spvalue()).await;

        assert_eq!(taken, None);
        let now: String = con.get("poisoned").await.unwrap();
        assert_eq!(
            now,
            serde_json::to_string(&false.to_spvalue()).unwrap(),
            "the replacement must land even when the old value was unreadable"
        );
    }
}
