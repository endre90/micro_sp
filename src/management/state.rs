use crate::*;
use crate::SPConnection;

mod apply;
mod build_state;
mod get_full_state;
mod get_sp_value;
mod get_state_for_keys;
mod set_sp_value;
mod set_state;
mod remove_sp_value;
mod remove_sp_values;
mod flush_state;

// PERF (storage layout): the state is currently one Redis top-level key per
// variable, with the value as a JSON string. That forces `KEYS *` + `MGET` to
// read the state and makes every read/write touch the global keyspace.
// Suggested: store the whole state as a single Redis HASH, e.g.
// `sp:{sp_id}:state`, with one field per variable. That gives you:
//   - `HGETALL` instead of `KEYS *` + `MGET` (one O(n) command, no O(keyspace)
//     scan, one round trip instead of two);
//   - `HMGET key f1 f2 ..` for the partial reads the runners already do;
//   - `HSET key f v f v ..` / `HDEL` for writes, all atomic per command;
//   - clean namespacing, so transforms (`TF_PREFIX`) and logger blobs stop
//     sharing a keyspace with state variables and `FLUSHDB` is no longer the
//     only way to reset one runner's state.
//
// PERF (round trips): a single tick of e.g. `auto_operation_runner` currently
// issues PING, KEYS, MGET, MSET, DEL, DEL - six sequential round trips, each of
// which the tokio task awaits before doing anything else. Suggested: batch them
// with `redis::pipe()` (`.atomic()` if you want MULTI/EXEC semantics). Reads
// and writes cannot always be merged, but PING can go entirely, KEYS+MGET
// collapse into HGETALL, and MSET+DEL+DEL collapse into one pipelined write -
// taking six RTTs down to two.
//
// CORRECTNESS (open - needs a design decision, not a bug fix): read-modify-write
// across runners is not atomic. Each runner reads a snapshot, computes a diff
// against it, and writes the diff; two runners whose ticks overlap can both
// decide what to write from the same stale read, and the later MSET wins. It
// shows up as "the state change did not take" plus a tick of latency while it
// is redone.
//
// Since each runner only writes keys whose value *changed* in its own tick,
// this only bites where two runners genuinely write the same key. That set is
// small and known:
//
//   {sp_id}_plan              planner_ticker, plan_runner, goal_runner
//   {sp_id}_plan_state        plan_runner, goal_runner
//   {sp_id}_planner_state     planner_ticker, plan_runner, goal_runner
//   {sp_id}_plan_current_step plan_runner, goal_runner
//   {sp_id}_current_goal_state plan_runner, goal_runner
//   {sp_id}_replan_trigger    planner_ticker, goal_runner
//   {sp_id}_replanned         planner_ticker, goal_runner
//
// Two ways out, both bigger than a fix:
//
//   1. Exclusive ownership per key. The natural split is planner_ticker owning
//      `_planner_state`/`_plan`/`_plan_id`/`_plan_counter`, plan_runner owning
//      `_plan_state`/`_plan_current_step`/`_terminated_operations`, and
//      goal_runner owning the `_current_goal_*`/`_scheduled_goals`/`_replan_*`
//      family. That does not work as a straight edit, because goal_runner
//      currently *resets* the plan fields when it admits a new goal - the
//      cross-writes are how the handover is implemented today. Doing this
//      properly means reworking that handover so each runner resets its own
//      fields when it observes a new `_current_goal_id`.
//   2. Make the read-compute-write atomic with WATCH/MULTI/EXEC or a Lua
//      script, and retry on conflict. Cheaper to implement, but it turns every
//      tick into a transaction and needs a retry policy.
//
// Note `StateManager::apply` deliberately does *not* use `.atomic()`: batching
// a runner's own writes into one MULTI/EXEC does nothing for this, because the
// race is between the read and the write, not among the writes.
//
// PERF (serialisation): every value is `serde_json` encoded/decoded on every
// hop. For `SPTransformStamped` and array/map values that is the dominant cost
// of a tick. Suggested: skip re-deserialising values that did not change (see
// the note in `get_full_state`), and consider a compact binary codec
// (`rmp-serde`, `bincode`) for the transform/array-heavy keys - typically
// 2-5x faster and smaller on the wire.
pub struct StateManager {}

impl StateManager {
    pub async fn get_full_state(con: &mut SPConnection) -> Option<State> {
        get_full_state::get_full_state(con).await
    }

    pub async fn get_state_for_keys(
        con: &mut SPConnection,
        keys: &Vec<String>,
        log_target: &str
    ) -> Option<State> {
        get_state_for_keys::get_state_for_keys(con, keys, &log_target).await
    }

    pub async fn get_sp_value(con: &mut SPConnection, var: &str) -> Option<SPValue> {
        get_sp_value::get_sp_value(con, var).await
    }

    pub async fn set_state(con: &mut SPConnection, state: &State) {
        set_state::set_state(con, state).await
    }

    pub async fn set_sp_value(con: &mut SPConnection, key: &str, value: &SPValue) {
        set_sp_value::set_sp_value(con, key, value).await
    }

    pub async fn remove_sp_value(con: &mut SPConnection, key: &str) {
        remove_sp_value::remove_sp_value(con, key).await
    }

    pub async fn remove_sp_values(con: &mut SPConnection, keys: &[String]) {
        remove_sp_values::remove_sp_values(con, keys).await
    }

    /// Write a state delta and delete a set of keys in a single round trip.
    ///
    /// DONE: PERF: the runners' tail was `set_state` followed by one or two
    /// `remove_sp_values` calls - three sequential round trips, each awaited
    /// before the next could start, for what is one logical "publish this
    /// tick's changes" step. `redis::pipe()` sends them together.
    ///
    /// Not `.atomic()` (no MULTI/EXEC): the previous code was three separate
    /// commands with no atomicity either, so wrapping them in a transaction
    /// here would be a semantic change smuggled in under a performance fix.
    /// See the atomicity note on this module for the real fix.
    pub async fn apply(con: &mut SPConnection, state: &State, deletes: &[&[String]]) {
        apply::apply(con, state, deletes).await
    }

    pub fn build_state(keys: Vec<String>, values: Vec<Option<String>>) -> State {
        build_state::build_state(keys, values)
    }

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
    /// the keyspace - which is the reason the module note calls it "the only way
    /// to reset one runner's state".
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
