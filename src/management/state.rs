use crate::*;
use crate::SPConnection;

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
// PERF (atomicity): read-modify-write here is not atomic. Two runners that tick
// at the same time both read the full state, both compute a diff against their
// own snapshot, and both write - so the later MSET can resurrect a value the
// other runner just changed. Besides the correctness risk this shows up as
// "the state change did not take" and an extra tick of latency when the write
// is lost and has to be redone. Suggested: either give each runner exclusive
// ownership of the keys it writes (it mostly already has that - it is the
// full-state *reads* that create the overlap), or move the read-compute-write
// into a Lua script / WATCH-MULTI-EXEC so it is atomic.
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

    pub fn build_state(keys: Vec<String>, values: Vec<Option<String>>) -> State {
        build_state::build_state(keys, values)
    }

    pub async fn flush_state(con: &mut SPConnection) {
        flush_state::flush_state(con).await
    }
}