//! Reading and writing state variables in Redis.
//!
//! One Redis top-level key per variable, holding the variable's [`SPValue`] as
//! a JSON string. [`StateManager`] is the only door onto that layout; every
//! runner reads and writes its slice of the state through it.

use crate::*;
use crate::SPConnection;

mod apply;
mod build_state;
mod get_full_state;
mod get_sp_value;
mod get_state_for_keys;
mod set_sp_value;
mod take_sp_value;
mod set_state;
mod remove_sp_value;
mod remove_sp_values;
mod flush_state;
/// Reads and writes [`State`] through a Redis connection.
///
/// A stateless namespace of associated functions: every call takes the
/// connection (`con`) to work on, so a runner can hold one long-lived
/// [`SPConnection`] and call these from anywhere. Reads return `None` on
/// failure, writes log and continue - nothing here panics or propagates a
/// Redis error, because a runner must survive a blip rather than die on it.
///
/// # Caveats
///
/// Read-modify-write through this type is **not** atomic, and nothing here makes
/// it so. A caller reads a snapshot, decides from it and writes back; nothing in
/// Redis records that the write was computed from that read, so a write landing
/// in between is simply overwritten - silently, and with the other writer's own
/// write having succeeded, so nothing retries.
///
/// Inside the runtime this no longer bites, because
/// [`sequential_runner`] drives every runner in one
/// loop off one snapshot and publishes one diff. There is no second reader to be
/// stale and no second writer to lose to. (`MICRO_SP_SEQUENTIAL=0` goes back to
/// eight concurrent tasks and brings the race back with it - see
/// [`sequential_runner_enabled`].)
///
/// What remains is every writer outside that loop: a device driver in its own
/// process, a dashboard, another `micro_sp`, or any consumer of this API. Two
/// rules cover it:
///
/// - A handoff that crosses the process boundary must be consumed atomically.
///   [`StateManager::take_sp_value`] is one `GETSET`, and is how
///   `{sp_id}_incoming_goals` is drained.
/// - A `*_request_trigger` is not such a handoff, despite the shape. It is a
///   request flag *and* a busy flag, cleared together with the `request_state`
///   that answers it - see [`StateManager::take_sp_value`] for why taking one
///   early re-opens the model's start guard mid-request.
///
/// A consumer doing its own read-modify-write on a key a runner also writes is
/// on its own; there is no locking here to borrow.
///
/// Note that [`StateManager::apply`] is deliberately non-atomic, but for an
/// unrelated reason: the race was always between the read and the write, never
/// among the writes of one publish.
///
/// ```no_run
/// use micro_sp::*;
///
/// # async fn example() {
/// let connection_manager = ConnectionManager::new().await;
/// let mut con = connection_manager.get_connection().await;
///
/// let mut state = State::new();
/// state.add_mut(
///     SPAssignment::new(SPVariable::new("pos", SPValueType::String), "a".to_spvalue()),
///     "docs",
/// );
/// StateManager::set_state(&mut con, &state).await;
///
/// let read = StateManager::get_state_for_keys(&mut con, &vec!["pos".to_string()], "docs")
///     .await
///     .expect("the read failed");
/// assert_eq!(read.get_value("pos", "docs"), Some("a".to_spvalue()));
/// # }
/// ```
pub struct StateManager {}

impl StateManager {
    /// Read every key in the Redis database into a [`State`].
    ///
    /// A `KEYS *` followed by an `MGET`, so it scales with the whole keyspace -
    /// prefer [`StateManager::get_state_for_keys`] on a hot path. An empty
    /// database yields an empty `State`; only a failed command yields `None`.
    /// Keys whose value does not deserialise are skipped with a warning.
    pub async fn get_full_state(con: &mut SPConnection) -> Option<State> {
        get_full_state::get_full_state(con).await
    }

    /// Read just `keys` into a [`State`], in one `MGET`.
    ///
    /// Keys that do not exist, or whose value fails to deserialise, simply
    /// contribute no variable - so the result may be smaller than `keys`, and an
    /// empty `keys` never reaches Redis. Returns `None` only if the command
    /// itself failed; `log_target` is the `log` target errors are reported under.
    pub async fn get_state_for_keys(
        con: &mut SPConnection,
        keys: &Vec<String>,
        log_target: &str
    ) -> Option<State> {
        get_state_for_keys::get_state_for_keys(con, keys, &log_target).await
    }

    /// Read a single variable's value.
    ///
    /// `None` covers all three of "the key is not set", "the command failed" and
    /// "the stored value did not deserialise"; the latter two are logged.
    pub async fn get_sp_value(con: &mut SPConnection, var: &str) -> Option<SPValue> {
        get_sp_value::get_sp_value(con, var).await
    }

    /// Write every assignment in `state` in one `MSET`.
    ///
    /// This is a merge, not a replace: keys absent from `state` are left alone.
    /// Values that fail to serialise are dropped with an error, and a failed
    /// `MSET` is logged rather than returned.
    pub async fn set_state(con: &mut SPConnection, state: &State) {
        set_state::set_state(con, state).await
    }

    /// Write a single variable's value. Serialisation and command failures are
    /// logged and swallowed.
    pub async fn set_sp_value(con: &mut SPConnection, key: &str, value: &SPValue) {
        set_sp_value::set_sp_value(con, key, value).await
    }

    /// Read a variable and overwrite it in the same, atomic step, returning what
    /// was there before.
    ///
    /// One `GETSET`. This is how a runner drains a queue another *process*
    /// fills, such as `{sp_id}_incoming_goals`. Reading the key and later
    /// writing the emptied value back is two commands with a gap between them,
    /// and anything the producer writes inside that gap is erased: the
    /// producer's own write succeeded, so nothing ever retries and the request
    /// is simply lost. Taking the value *is* emptying it, so a write that lands
    /// afterwards is still there on the next tick.
    ///
    /// Not for the `*_request_trigger` keys, despite the shape looking the
    /// same. A trigger is a request flag *and* a busy flag: the models guard
    /// their start transitions on `request_state == initial && request_trigger
    /// == false`, so clearing it before the response is written declares the
    /// resource free while the request is still in flight, and the operation
    /// starts again. A trigger has to be cleared together with the
    /// `request_state` that answers it, which is a write, not a take.
    ///
    /// `replacement` is the cleared value, not a deletion - a key that stops
    /// existing would be missing from a later
    /// [`StateManager::get_state_for_keys`], which callers treat as fatal - so a
    /// key that was absent is created holding it and `None` comes back.
    ///
    /// `None` also covers "the command failed" and "the stored value did not
    /// deserialise"; both are logged, and in the latter case the replacement has
    /// still landed, so a poisoned key cannot wedge a runner permanently.
    ///
    /// Note this consumes the value at *read* time: a runner that dies
    /// mid-tick has already taken the request, where the old read-then-clear
    /// would have left it set.
    pub async fn take_sp_value(
        con: &mut SPConnection,
        key: &str,
        replacement: &SPValue,
    ) -> Option<SPValue> {
        take_sp_value::take_sp_value(con, key, replacement).await
    }

    /// Delete a single key. Idempotent - deleting a key that was never there is
    /// not an error - and a failed `DEL` is logged rather than returned.
    pub async fn remove_sp_value(con: &mut SPConnection, key: &str) {
        remove_sp_value::remove_sp_value(con, key).await
    }

    /// Delete `keys` in one `DEL`. An empty list is a no-op that never reaches
    /// Redis; failures are logged rather than returned.
    pub async fn remove_sp_values(con: &mut SPConnection, keys: &[String]) {
        remove_sp_values::remove_sp_values(con, keys).await
    }

    /// Write a state delta and delete several key sets in a single round trip.
    ///
    /// The "publish this tick" call: pipelines the `MSET` and the `DEL`s
    /// together, and issues nothing at all when there is nothing to do.
    /// Deliberately not `.atomic()` (no MULTI/EXEC) - see the caveat on
    /// [`StateManager`]. Failures are logged rather than returned.
    pub async fn apply(con: &mut SPConnection, state: &State, deletes: &[&[String]]) {
        apply::apply(con, state, deletes).await
    }

    /// Pair `keys` with the raw JSON `values` an `MGET` returned into a [`State`].
    ///
    /// Positional: `values[i]` belongs to `keys[i]`. A `None` value, or one that
    /// fails to deserialise, contributes no variable. The variable's type is
    /// inferred from the value it decoded to.
    pub fn build_state(keys: Vec<String>, values: Vec<Option<String>>) -> State {
        build_state::build_state(keys, values)
    }

    /// `FLUSHDB` - erase the entire Redis database.
    ///
    /// State variables *and* transform keys share one keyspace, so this takes
    /// the transform tree with it. Failures are logged rather than returned.
    pub async fn flush_state(con: &mut SPConnection) {
        flush_state::flush_state(con).await
    }
}
/// The `StateManager` facade, end to end.
///
/// Each submodule tests its own function, but several of the facade methods and
/// several of the *edge* cases - an empty keyspace, an empty key list, deleting
/// nothing, a value that fails to deserialise - are only reachable from here.
/// They matter because they are precisely the states a freshly started or
/// freshly flushed system is in.
#[cfg(test)]
mod facade_tests {
    use crate::*;
    use serial_test::serial;
    use std::sync::Arc;
    use testcontainers::{ContainerAsync, ImageExt, core::ContainerPort, runners::AsyncRunner};
    use testcontainers_modules::redis::Redis;

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

    fn state_with(pairs: &[(&str, SPValue)]) -> State {
        let mut state = State::new();
        for (name, value) in pairs {
            let variable = match value {
                SPValue::Bool(_) => SPVariable::new(name, SPValueType::Bool),
                SPValue::Int64(_) => SPVariable::new(name, SPValueType::Int64),
                SPValue::Float64(_) => SPVariable::new(name, SPValueType::Float64),
                SPValue::Array(_) => SPVariable::new(name, SPValueType::Array),
                SPValue::Map(_) => SPVariable::new(name, SPValueType::Map),
                _ => SPVariable::new(name, SPValueType::String),
            };
            state.add_mut(SPAssignment::new(variable, value.clone()), TARGET);
        }
        state
    }

    /// The state a process is in before anything has written: `get_full_state`
    /// has to return an empty `State`, not `None`. `None` means "the read
    /// failed, skip this tick", and a runner that cannot distinguish the two
    /// would spin forever on a fresh database.
    #[tokio::test]
    #[serial]
    async fn an_empty_keyspace_reads_as_an_empty_state_not_a_failure() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        let full = StateManager::get_full_state(&mut con).await;
        assert_eq!(full, Some(State::new()), "an empty database is not an error");

        // Same for a key set none of whose keys exist.
        let missing = StateManager::get_state_for_keys(
            &mut con,
            &vec!["nope".to_string(), "also_nope".to_string()],
            TARGET,
        )
        .await;
        assert_eq!(missing, Some(State::new()));

        // And for an empty key list, which never even reaches Redis.
        let none = StateManager::get_state_for_keys(&mut con, &vec![], TARGET).await;
        assert_eq!(none, Some(State::new()));
    }

    /// A round trip through Redis has to preserve every value type exactly -
    /// this is the serialisation boundary every runner crosses twice per tick.
    #[tokio::test]
    #[serial]
    async fn every_value_type_survives_a_round_trip() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        let written = state_with(&[
            ("b", true.to_spvalue()),
            ("i", (-7).to_spvalue()),
            ("f", 1.25.to_spvalue()),
            ("s", "text".to_spvalue()),
            ("arr", vec![1.to_spvalue(), "two".to_spvalue()].to_spvalue()),
            (
                "map",
                vec![("k".to_spvalue(), "v".to_spvalue())].to_spvalue(),
            ),
            ("b_unknown", SPValue::Bool(BoolOrUnknown::UNKNOWN)),
            ("s_unknown", SPValue::String(StringOrUnknown::UNKNOWN)),
        ]);

        StateManager::set_state(&mut con, &written).await;
        let read = StateManager::get_full_state(&mut con).await.unwrap();

        for (key, assignment) in &written.state {
            assert_eq!(
                read.get_value(key, TARGET),
                Some(assignment.val.clone()),
                "'{key}' did not survive the round trip"
            );
        }
    }

    /// A key holding something that is not a serialised `SPValue` - written by
    /// another tool, or left over from an older encoding - is skipped with a
    /// warning rather than taking the read down. A runner that panicked here
    /// could be killed by anything else writing to the same Redis.
    #[tokio::test]
    #[serial]
    async fn a_key_that_is_not_a_serialised_value_is_skipped() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        StateManager::set_state(&mut con, &state_with(&[("good", "value".to_spvalue())])).await;
        redis::cmd("SET")
            .arg("garbage")
            .arg("{not json at all")
            .query_async::<()>(&mut con)
            .await
            .unwrap();

        let read = StateManager::get_full_state(&mut con).await.unwrap();
        assert_eq!(read.get_value("good", TARGET), Some("value".to_spvalue()));
        assert!(!read.contains("garbage"), "the unparseable key must be skipped");
    }

    /// `apply` is the pipelined "publish this tick" call: a delta plus deletes
    /// in one round trip.
    #[tokio::test]
    #[serial]
    async fn apply_writes_the_delta_and_deletes_in_one_go() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        StateManager::set_state(
            &mut con,
            &state_with(&[
                ("keep", "old".to_spvalue()),
                ("drop_me", "x".to_spvalue()),
                ("drop_me_too", "y".to_spvalue()),
            ]),
        )
        .await;

        let deletes_a = vec!["drop_me".to_string()];
        let deletes_b = vec!["drop_me_too".to_string()];
        StateManager::apply(
            &mut con,
            &state_with(&[("keep", "new".to_spvalue()), ("added", "z".to_spvalue())]),
            &[&deletes_a, &deletes_b],
        )
        .await;

        assert_eq!(
            StateManager::get_sp_value(&mut con, "keep").await,
            Some("new".to_spvalue())
        );
        assert_eq!(
            StateManager::get_sp_value(&mut con, "added").await,
            Some("z".to_spvalue())
        );
        assert_eq!(StateManager::get_sp_value(&mut con, "drop_me").await, None);
        assert_eq!(StateManager::get_sp_value(&mut con, "drop_me_too").await, None);
    }

    /// `apply` with nothing to do must not issue a command at all - this is the
    /// early return that keeps an idle runner's tick free of Redis traffic.
    #[tokio::test]
    #[serial]
    async fn apply_with_nothing_to_do_changes_nothing() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;
        StateManager::set_state(&mut con, &state_with(&[("keep", "value".to_spvalue())])).await;

        let empty: Vec<String> = vec![];
        StateManager::apply(&mut con, &State::new(), &[]).await;
        StateManager::apply(&mut con, &State::new(), &[&empty, &empty]).await;

        assert_eq!(
            StateManager::get_sp_value(&mut con, "keep").await,
            Some("value".to_spvalue())
        );
        let dbsize: i64 = redis::cmd("DBSIZE").query_async(&mut con).await.unwrap();
        assert_eq!(dbsize, 1);
    }

    /// Removal, one key and many, including keys that were never there.
    #[tokio::test]
    #[serial]
    async fn removing_keys_is_idempotent() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        StateManager::set_state(
            &mut con,
            &state_with(&[("a", 1.to_spvalue()), ("b", 2.to_spvalue()), ("c", 3.to_spvalue())]),
        )
        .await;

        StateManager::remove_sp_value(&mut con, "a").await;
        assert_eq!(StateManager::get_sp_value(&mut con, "a").await, None);
        // Again, and on something that never existed.
        StateManager::remove_sp_value(&mut con, "a").await;
        StateManager::remove_sp_value(&mut con, "never_there").await;

        StateManager::remove_sp_values(&mut con, &["b".to_string(), "never_there".to_string()])
            .await;
        assert_eq!(StateManager::get_sp_value(&mut con, "b").await, None);
        assert_eq!(StateManager::get_sp_value(&mut con, "c").await, Some(3.to_spvalue()));

        // An empty removal list is a no-op rather than an error.
        StateManager::remove_sp_values(&mut con, &[]).await;
        assert_eq!(StateManager::get_sp_value(&mut con, "c").await, Some(3.to_spvalue()));
    }

    /// `flush_state` clears everything, including the transform keys that share
    /// the keyspace - so it is a whole-database reset, not a state-only one.
    #[tokio::test]
    #[serial]
    async fn flush_state_clears_the_whole_keyspace() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        StateManager::set_state(&mut con, &state_with(&[("a", 1.to_spvalue())])).await;
        TransformsManager::insert_transform(
            &mut con,
            &SPTransformStamped {
                active_transform: true,
                enable_transform: true,
                time_stamp: std::time::SystemTime::now(),
                parent_frame_id: "world".to_string(),
                child_frame_id: "frame".to_string(),
                transform: SPTransform::default(),
                metadata: MapOrUnknown::UNKNOWN,
            },
        )
        .await
        .unwrap();

        StateManager::flush_state(&mut con).await;

        let dbsize: i64 = redis::cmd("DBSIZE").query_async(&mut con).await.unwrap();
        assert_eq!(dbsize, 0, "flush takes the transforms with it, not just the state");
        assert_eq!(
            StateManager::get_full_state(&mut con).await,
            Some(State::new())
        );
    }

    /// `build_state` is the public, owned form of the parser the reads use.
    #[tokio::test]
    #[serial]
    async fn build_state_pairs_keys_with_values_and_skips_the_gaps() {
        let built = StateManager::build_state(
            vec!["a".to_string(), "missing".to_string(), "b".to_string()],
            vec![
                Some(serde_json::to_string(&1.to_spvalue()).unwrap()),
                None,
                Some(serde_json::to_string(&"two".to_spvalue()).unwrap()),
            ],
        );

        assert_eq!(built.get_value("a", TARGET), Some(1.to_spvalue()));
        assert_eq!(built.get_value("b", TARGET), Some("two".to_spvalue()));
        assert!(!built.contains("missing"), "a None value contributes no variable");
    }
}
