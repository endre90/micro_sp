use crate::{SPConnection, State};

/// Publish a tick's changes - a state delta plus any keys to delete - in one
/// round trip.
///
/// The runners used to do this as `set_state(..).await` followed by one or two
/// `remove_sp_values(..).await`, each waiting for the previous reply before it
/// could even be sent. That is three sequential round trips per tick of the
/// operation runners whenever an operation terminates.
///
/// Deliberately *not* `.atomic()`. The three separate commands it replaces had
/// no atomicity either, so adding MULTI/EXEC here would change behaviour under
/// the cover of a performance fix - see the atomicity note in `state.rs` for
/// what actually needs doing there.
pub(super) async fn apply(con: &mut SPConnection, state: &State, deletes: &[&[String]]) {
    let items_to_set: Vec<(&str, String)> = state
        .state
        .iter()
        .filter_map(|(key, assignment)| match serde_json::to_string(&assignment.val) {
            Ok(value_str) => Some((key.as_str(), value_str)),
            Err(e) => {
                log::error!("Failed to serialize value for key '{key}': {e}");
                None
            }
        })
        .collect();

    let delete_count: usize = deletes.iter().map(|keys| keys.len()).sum();
    if items_to_set.is_empty() && delete_count == 0 {
        return;
    }

    let mut pipe = redis::pipe();
    if !items_to_set.is_empty() {
        pipe.mset(&items_to_set).ignore();
    }
    for keys in deletes {
        if !keys.is_empty() {
            pipe.del(*keys).ignore();
        }
    }

    match pipe.query_async::<()>(con).await {
        Ok(()) => {}
        Err(e) => log::error!("Redis pipelined write failed: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use redis::AsyncCommands;
    use serial_test::serial;
    use testcontainers::{ImageExt, core::ContainerPort, runners::AsyncRunner};
    use testcontainers_modules::redis::Redis;

    fn delta() -> State {
        let mut state = State::new();
        state.state.insert(
            "written".to_string(),
            assign!(iv!("written"), 7.to_spvalue()),
        );
        state
    }

    #[tokio::test]
    #[serial]
    async fn writes_and_deletes_in_one_pipeline() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;

        let _: () = con.set("doomed_a", "x").await.unwrap();
        let _: () = con.set("doomed_b", "y").await.unwrap();

        let a = vec!["doomed_a".to_string()];
        let b = vec!["doomed_b".to_string()];
        StateManager::apply(&mut con, &delta(), &[&a, &b]).await;

        let written: Option<String> = con.get("written").await.unwrap();
        assert_eq!(
            written,
            Some(serde_json::to_string(&7.to_spvalue()).unwrap()),
            "the delta should have been written"
        );
        let remaining: usize = con.exists(&["doomed_a", "doomed_b"]).await.unwrap();
        assert_eq!(remaining, 0, "both delete lists should have been applied");
    }

    #[tokio::test]
    #[serial]
    async fn an_empty_delta_with_deletes_still_deletes() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;
        let _: () = con.set("doomed", "x").await.unwrap();

        let doomed = vec!["doomed".to_string()];
        StateManager::apply(&mut con, &State::new(), &[&doomed]).await;

        let exists: bool = con.exists("doomed").await.unwrap();
        assert!(!exists);
    }

    #[tokio::test]
    #[serial]
    async fn nothing_to_do_issues_no_command() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;
        let _: () = redis::cmd("CONFIG")
            .arg("RESETSTAT")
            .query_async(&mut con)
            .await
            .unwrap();

        StateManager::apply(&mut con, &State::new(), &[&[], &[]]).await;

        let stats: String = redis::cmd("INFO")
            .arg("commandstats")
            .query_async(&mut con)
            .await
            .unwrap();
        assert!(
            !stats.contains("cmdstat_mset") && !stats.contains("cmdstat_del"),
            "an empty apply should not touch Redis at all:\n{stats}"
        );
    }
}
