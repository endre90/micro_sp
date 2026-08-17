//! The transform tree as a request/response service.
//!
//! A consumer writes a command and its arguments into the `{sp_id}_tf_*` state
//! keys, sets the trigger, and reads the request state back - the same protocol
//! every other service in the runtime uses, so transforms can be manipulated
//! from a model without any direct Redis access.

use std::sync::Arc;

use crate::*;

/// The transform service runner: serves transform requests posted into the state.
///
/// Polls `{sp_id}_tf_request_trigger` and, when it is set, reads the rest of the
/// `{sp_id}_tf_*` request keys and runs the named command - `lookup`,
/// `reparent`, `snap_to_parent` or `insert` - against [`TransformsManager`]. It
/// then clears the trigger and writes the outcome back as
/// `{sp_id}_tf_request_state`, so a request is served exactly once. Loops
/// forever; a failed read logs and skips the tick.
///
/// PERF (open): a Redis keyspace notification on the trigger key would remove
/// the poll entirely and cut request latency from one tick to one round trip.
pub async fn tf_interface(
    sp_id: &str,
    connection_manager: &Arc<ConnectionManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = runner_interval();
    let log_target = format!("{}_tf_interface", sp_id);

    log::info!(target: &log_target,  "Online.");

    let keys: Vec<String> = vec![
        format!("{}_tf_request_trigger", sp_id),
        format!("{}_tf_request_state", sp_id),
        format!("{}_tf_command", sp_id),
        format!("{}_tf_parent", sp_id),
        format!("{}_tf_child", sp_id),
        format!("{}_tf_lookup_result", sp_id),
        format!("{}_tf_insert_transforms", sp_id),
    ];

    // PERF: one long-lived connection handle for the whole runner instead of
    // re-fetching one every tick, and no pre-flight PING before the real work.
    // `SPConnection` is cheap to clone, multiplexed and self-healing, so this
    // handle stays valid across reconnects; a dropped socket now surfaces as an
    // error on the command itself, which the callee already logs and skips.
    let mut con = connection_manager.get_connection().await;

    let trigger_key = format!("{}_tf_request_trigger", sp_id);

    loop {
        interval.tick().await;

        // Nothing below happens unless the trigger is set, so an idle tick
        // costs one `GET` instead of an `MGET` of the whole request key set.
        // Anything that is not exactly `true` means "no request", which is what
        // `get_bool_or_default_to_false` used to decide after fetching all of it.
        match StateManager::get_sp_value(&mut con, &trigger_key).await {
            Some(SPValue::Bool(BoolOrUnknown::Bool(true))) => (),
            _ => continue,
        }

        let state = match StateManager::get_state_for_keys(&mut con, &keys, &log_target).await {
            Some(s) => s,
            None => continue,
        };

        let mut request_trigger = state
            .get_bool_or_default_to_false(&format!("{}_tf_request_trigger", sp_id), &log_target);

        let mut request_state = state
            .get_string_or_default_to_unknown(&format!("{}_tf_request_state", sp_id), &log_target);

        if request_trigger {
            request_trigger = false;
            if request_state == ServiceRequestState::Initial.to_string() {
                let command = state.get_string_or_default_to_unknown(
                    &format!("{}_tf_command", sp_id),
                    &log_target,
                );

                let parent = state
                    .get_string_or_default_to_unknown(&format!("{}_tf_parent", sp_id), &log_target);

                let child = state
                    .get_string_or_default_to_unknown(&format!("{}_tf_child", sp_id), &log_target);

                let mut tf_lookup_result = state.get_transform_or_default_to_default(
                    &format!("{}_tf_lookup_result", sp_id),
                    &log_target,
                );

                let tf_insert_transforms_sp_values = state.get_array_or_default_to_empty(
                    &format!("{}_tf_insert_transforms", sp_id),
                    &log_target,
                );

                let mut tf_insert_transforms = vec![];
                tf_insert_transforms_sp_values.iter().for_each(|x| match x {
                    SPValue::Transform(TransformOrUnknown::Transform(transform)) => {
                        tf_insert_transforms.push(transform.to_owned())
                    }
                    _ => (),
                });

                match command.as_str() {
                    "lookup" => {
                        match TransformsManager::lookup_transform(&mut con, &parent, &child).await {
                            Ok(tf) => {
                                tf_lookup_result = tf;
                                request_state = ServiceRequestState::Succeeded.to_string();
                            }
                            Err(e) => {
                                log::error!(target: &log_target,
                                    "Failed to lookup {} to {}.", parent, child);
                                log::error!(target: &log_target, "{e}");
                                request_state = ServiceRequestState::Failed.to_string();
                            }
                        }
                    }
                    "reparent" => {
                        match TransformsManager::reparent_transform(&mut con, &parent, &child).await
                        {
                            Ok(()) => request_state = ServiceRequestState::Succeeded.to_string(),
                            Err(e) => {
                                log::error!(target:  &log_target,
                                    "Failed to reparent {} to {}.", child, parent);
                                log::error!(target:  &log_target, "{e}");
                                request_state = ServiceRequestState::Failed.to_string();
                            }
                        }
                    }
                    "snap_to_parent" => {
                        match TransformsManager::snap_to_parent_transform(&mut con, &parent, &child)
                            .await
                        {
                            Ok(()) => request_state = ServiceRequestState::Succeeded.to_string(),
                            Err(e) => {
                                log::error!(target:  &log_target,
                                    "Failed to snap {} to parent {}.", child, parent);
                                log::error!(target:  &log_target, "{e}");
                                request_state = ServiceRequestState::Failed.to_string();
                            }
                        }
                    }
                    "insert" => {
                        match TransformsManager::insert_transforms(&mut con, &tf_insert_transforms)
                            .await
                        {
                            Ok(()) => request_state = ServiceRequestState::Succeeded.to_string(),
                            Err(e) => {
                                log::error!(target:  &log_target,
                                    "Failed to insert transforms {:?}.", tf_insert_transforms);
                                log::error!(target:  &log_target, "{e}");
                                request_state = ServiceRequestState::Failed.to_string();
                            }
                        }
                    }
                    _ => {
                        log::error!(target:  &log_target,
                            "TF interface command {} is invalid.", command);
                        request_state = ServiceRequestState::Failed.to_string()
                    }
                }

                let new_state = state
                    .update(
                        &format!("{}_tf_request_trigger", sp_id),
                        request_trigger.to_spvalue(),
                    )
                    .update(
                        &format!("{}_tf_request_state", sp_id),
                        request_state.to_spvalue(),
                    )
                    .update(
                        &format!("{}_tf_lookup_result", sp_id),
                        tf_lookup_result.to_spvalue(),
                    );

                let modified_state = state.get_diff_partial_state(&new_state);
                activity_log::log_state_diff(&log_target, &state, &modified_state);
                StateManager::set_state(&mut con, &modified_state).await;
            }
        }
    }
}

/// The TF interface, driven end to end against a real Redis.
///
/// This runner is a request/response bridge: a consumer writes a command plus
/// its arguments into the state, sets a trigger, and reads a request state back.
/// Every command is a separate branch, each with its own success and failure
/// path, and none of them is reachable without a running Redis - which is why
/// the whole module read as untested.
///
/// The other reason to drive it end to end is the trigger protocol itself. The
/// runner is responsible for clearing the trigger it consumed; a request that
/// does not clear it is a request the runner re-reads on every tick forever.
#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::time::{Duration, SystemTime};
    use testcontainers::{ContainerAsync, ImageExt, core::ContainerPort, runners::AsyncRunner};
    use testcontainers_modules::redis::Redis;

    const SP: &str = "sp";
    const TARGET: &str = "test";

    async fn redis() -> (ContainerAsync<Redis>, Arc<ConnectionManager>) {
        let container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();
        let manager = Arc::new(ConnectionManager::new().await);
        let mut con = manager.get_connection().await;
        StateManager::flush_state(&mut con).await;
        (container, manager)
    }

    fn key(suffix: &str) -> String {
        format!("{SP}_tf_{suffix}")
    }

    fn transform_at(child: &str, parent: &str, x: f64) -> SPTransformStamped {
        SPTransformStamped {
            active_transform: true,
            enable_transform: true,
            time_stamp: SystemTime::now(),
            parent_frame_id: parent.to_string(),
            child_frame_id: child.to_string(),
            transform: SPTransform {
                translation: SPTranslation {
                    x: ordered_float::OrderedFloat(x),
                    y: ordered_float::OrderedFloat(0.0),
                    z: ordered_float::OrderedFloat(0.0),
                },
                rotation: SPTransform::default().rotation,
            },
            metadata: MapOrUnknown::Map(vec![]),
        }
    }

    /// Every key the runner reads has to exist first - `get_state_for_keys`
    /// skips absent keys and the accessors panic on them.
    async fn seed_request(con: &mut SPConnection, command: &str, parent: &str, child: &str) {
        let mut state = State::new();
        state.add_mut(
            SPAssignment::new(
                SPVariable::new(&key("request_trigger"), SPValueType::Bool),
                false.to_spvalue(),
            ),
            TARGET,
        );
        state.add_mut(
            SPAssignment::new(
                SPVariable::new(&key("request_state"), SPValueType::String),
                ServiceRequestState::Initial.to_string().to_spvalue(),
            ),
            TARGET,
        );
        for (suffix, value) in [
            ("command", command),
            ("parent", parent),
            ("child", child),
        ] {
            state.add_mut(
                SPAssignment::new(
                    SPVariable::new(&key(suffix), SPValueType::String),
                    value.to_spvalue(),
                ),
                TARGET,
            );
        }
        state.add_mut(
            SPAssignment::new(
                SPVariable::new(&key("lookup_result"), SPValueType::Transform),
                SPValue::Transform(TransformOrUnknown::Transform(transform_at(
                    "none", "none", 0.0,
                ))),
                ),
            TARGET,
        );
        state.add_mut(
            SPAssignment::new(
                SPVariable::new(&key("insert_transforms"), SPValueType::Array),
                Vec::<SPValue>::new().to_spvalue(),
            ),
            TARGET,
        );
        StateManager::set_state(con, &state).await;
    }

    async fn wait_for_state(con: &mut SPConnection, expected: &str, timeout_ms: u64) -> String {
        let deadline = std::time::Instant::now() + Duration::from_millis(timeout_ms);
        let mut last = String::new();
        while std::time::Instant::now() < deadline {
            last = match StateManager::get_sp_value(con, &key("request_state")).await {
                Some(SPValue::String(StringOrUnknown::String(s))) => s,
                other => format!("{other:?}"),
            };
            if last == expected {
                return last;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        last
    }

    fn spawn_runner(manager: &Arc<ConnectionManager>) -> tokio::task::JoinHandle<()> {
        let manager = Arc::clone(manager);
        tokio::spawn(async move {
            let _ = tf_interface(SP, &manager).await;
        })
    }

    async fn trigger(con: &mut SPConnection) {
        StateManager::set_sp_value(con, &key("request_trigger"), &true.to_spvalue()).await;
    }

    /// `lookup` resolves a chain of frames and writes the answer back into
    /// `{sp_id}_tf_lookup_result`.
    #[tokio::test]
    #[serial]
    async fn a_lookup_request_succeeds_and_writes_the_result() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        TransformsManager::insert_transform(&mut con, &transform_at("a", "world", 1.0))
            .await
            .unwrap();
        TransformsManager::insert_transform(&mut con, &transform_at("b", "a", 2.0))
            .await
            .unwrap();

        seed_request(&mut con, "lookup", "world", "b").await;
        let runner = spawn_runner(&manager);
        trigger(&mut con).await;

        let state = wait_for_state(&mut con, "succeeded", 3000).await;
        runner.abort();
        assert_eq!(state, "succeeded");

        let result = StateManager::get_sp_value(&mut con, &key("lookup_result")).await;
        let Some(SPValue::Transform(TransformOrUnknown::Transform(tf))) = result else {
            panic!("expected a transform, got {result:?}");
        };
        assert_eq!(
            tf.transform.translation.x.0, 3.0,
            "world -> a -> b should compose to x = 1 + 2"
        );
    }

    /// A lookup for a frame that does not exist has to report `failed` rather
    /// than leaving the caller waiting or writing a garbage transform.
    #[tokio::test]
    #[serial]
    async fn a_lookup_of_a_missing_frame_fails() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        seed_request(&mut con, "lookup", "world", "nowhere").await;
        let runner = spawn_runner(&manager);
        trigger(&mut con).await;

        let state = wait_for_state(&mut con, "failed", 3000).await;
        runner.abort();
        assert_eq!(state, "failed");
    }

    #[tokio::test]
    #[serial]
    async fn a_reparent_request_moves_the_frame_under_its_new_parent() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        TransformsManager::insert_transform(&mut con, &transform_at("a", "world", 1.0))
            .await
            .unwrap();
        TransformsManager::insert_transform(&mut con, &transform_at("b", "world", 5.0))
            .await
            .unwrap();

        seed_request(&mut con, "reparent", "a", "b").await;
        let runner = spawn_runner(&manager);
        trigger(&mut con).await;

        let state = wait_for_state(&mut con, "succeeded", 3000).await;
        runner.abort();
        assert_eq!(state, "succeeded");

        let moved = TransformsManager::get_transform(&mut con, "b").await.unwrap();
        assert_eq!(moved.parent_frame_id, "a");
    }

    /// Reparenting onto a frame that would close a loop must be refused - this
    /// is the guard that keeps the whole transform tree walkable.
    #[tokio::test]
    #[serial]
    async fn a_reparent_that_would_make_a_cycle_fails() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        TransformsManager::insert_transform(&mut con, &transform_at("a", "world", 1.0))
            .await
            .unwrap();
        TransformsManager::insert_transform(&mut con, &transform_at("b", "a", 1.0))
            .await
            .unwrap();

        // Make 'a' a child of its own child.
        seed_request(&mut con, "reparent", "b", "a").await;
        let runner = spawn_runner(&manager);
        trigger(&mut con).await;

        let state = wait_for_state(&mut con, "failed", 3000).await;
        runner.abort();
        assert_eq!(state, "failed");

        let unchanged = TransformsManager::get_transform(&mut con, "a").await.unwrap();
        assert_eq!(unchanged.parent_frame_id, "world", "the tree must be untouched");
    }

    #[tokio::test]
    #[serial]
    async fn a_snap_to_parent_request_succeeds() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        TransformsManager::insert_transform(&mut con, &transform_at("a", "world", 1.0))
            .await
            .unwrap();
        TransformsManager::insert_transform(&mut con, &transform_at("b", "world", 5.0))
            .await
            .unwrap();

        seed_request(&mut con, "snap_to_parent", "a", "b").await;
        let runner = spawn_runner(&manager);
        trigger(&mut con).await;

        let state = wait_for_state(&mut con, "succeeded", 3000).await;
        runner.abort();
        assert_eq!(state, "succeeded");

        let snapped = TransformsManager::get_transform(&mut con, "b").await.unwrap();
        assert_eq!(snapped.parent_frame_id, "a");
        assert_eq!(
            snapped.transform.translation.x.0, 0.0,
            "snapping puts the child at its new parent's origin"
        );
    }

    #[tokio::test]
    #[serial]
    async fn a_snap_of_a_missing_frame_fails() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        seed_request(&mut con, "snap_to_parent", "world", "nowhere").await;
        let runner = spawn_runner(&manager);
        trigger(&mut con).await;

        let state = wait_for_state(&mut con, "failed", 3000).await;
        runner.abort();
        assert_eq!(state, "failed");
    }

    /// `insert` takes its payload from an array of transform values in the
    /// state, and anything in that array that is not a transform is skipped.
    #[tokio::test]
    #[serial]
    async fn an_insert_request_writes_every_transform_in_the_payload() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        seed_request(&mut con, "insert", "", "").await;
        StateManager::set_sp_value(
            &mut con,
            &key("insert_transforms"),
            &vec![
                SPValue::Transform(TransformOrUnknown::Transform(transform_at("a", "world", 1.0))),
                // Not a transform - has to be skipped, not fail the request.
                "junk".to_spvalue(),
                SPValue::Transform(TransformOrUnknown::Transform(transform_at("b", "a", 2.0))),
            ]
            .to_spvalue(),
        )
        .await;

        let runner = spawn_runner(&manager);
        trigger(&mut con).await;

        let state = wait_for_state(&mut con, "succeeded", 3000).await;
        runner.abort();
        assert_eq!(state, "succeeded");

        let all = TransformsManager::get_all_transforms(&mut con).await.unwrap();
        assert!(all.contains_key("a") && all.contains_key("b"), "got {:?}", all.keys());
    }

    #[tokio::test]
    #[serial]
    async fn an_unknown_command_fails_the_request() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        seed_request(&mut con, "teleport", "world", "a").await;
        let runner = spawn_runner(&manager);
        trigger(&mut con).await;

        let state = wait_for_state(&mut con, "failed", 3000).await;
        runner.abort();
        assert_eq!(state, "failed");
    }

    /// The trigger the runner consumed has to be cleared, or the caller cannot
    /// tell a finished request from one that was never picked up.
    #[tokio::test]
    #[serial]
    async fn a_handled_request_clears_its_trigger() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        seed_request(&mut con, "teleport", "world", "a").await;
        let runner = spawn_runner(&manager);
        trigger(&mut con).await;

        assert_eq!(wait_for_state(&mut con, "failed", 3000).await, "failed");
        assert_eq!(
            StateManager::get_sp_value(&mut con, &key("request_trigger")).await,
            Some(false.to_spvalue())
        );
        runner.abort();
    }

    /// BUG: the whole write-back - including clearing the trigger - sits inside
    /// `if request_state == ServiceRequestState::Initial.to_string()`. A trigger
    /// set while the request state is anything else (a caller that forgot to
    /// reset it after the previous request, or set both in the wrong order) is
    /// therefore never cleared, and the runner re-reads it on every tick for
    /// the life of the process: an `MGET` of the request key set at the tick
    /// rate, forever, and a request that never completes from the caller's
    /// point of view.
    ///
    /// `time_interface_runner` gets this right - it clears the trigger before
    /// checking the request state - so the two interfaces disagree about the
    /// same protocol.
    #[tokio::test]
    #[serial]
    async fn a_trigger_arriving_in_the_wrong_request_state_is_never_cleared() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        seed_request(&mut con, "lookup", "world", "a").await;
        // Anything other than "initial".
        StateManager::set_sp_value(
            &mut con,
            &key("request_state"),
            &ServiceRequestState::Succeeded.to_string().to_spvalue(),
        )
        .await;

        let runner = spawn_runner(&manager);
        trigger(&mut con).await;
        tokio::time::sleep(Duration::from_millis(300)).await;

        assert!(!runner.is_finished(), "the runner should still be spinning");
        assert_eq!(
            StateManager::get_sp_value(&mut con, &key("request_trigger")).await,
            Some(true.to_spvalue()),
            "if this is now false the trigger-clearing bug is fixed"
        );
        runner.abort();
    }

    /// An idle runner - the overwhelmingly common case - must not write.
    #[tokio::test]
    #[serial]
    async fn an_idle_runner_writes_nothing() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        seed_request(&mut con, "lookup", "world", "a").await;
        let runner = spawn_runner(&manager);
        tokio::time::sleep(Duration::from_millis(100)).await;

        let before = StateManager::get_full_state(&mut con).await.unwrap();
        tokio::time::sleep(Duration::from_millis(300)).await;
        let after = StateManager::get_full_state(&mut con).await.unwrap();

        assert!(
            before.get_diff_partial_state(&after).state.is_empty(),
            "an untriggered tf_interface must not change anything"
        );
        assert!(!runner.is_finished(), "and it must still be alive");
        runner.abort();
    }
}
