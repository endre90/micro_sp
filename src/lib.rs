// ============================================================================
// PERFORMANCE REVIEW - summary of the `PERF:` comments spread through the crate
// ----------------------------------------------------------------------------
// Nothing below has been changed in code; each item is written up in detail in
// a comment above the relevant definition. Grep for `PERF:` to find them.
//
// Ranked by expected impact on "high CPU while SOPs run" and "state changes
// are slower than I want":
//
//  1. `State::get_value` / `get_assignment` / `contains` / `add` do
//     `self.state.clone()` before looking up a single key - i.e. they deep-copy
//     the whole system state on every variable read. This sits under every
//     predicate evaluation, so it multiplies by (variables per guard) x
//     (operations) x (ticks). Dropping four `.clone()` calls is the single
//     cheapest large win.                                 -> core/sp_state.rs
//
//  2. DONE. `get_full_state` uses `KEYS *`, which is O(keyspace) and blocks the
//     whole Redis server. `sop_runner`, `auto_operation_runner` and
//     `planned_operation_runner` called it at 100-200 ms, so ~20 blocking scans
//     per second over a keyspace that also holds transforms and log blobs, each
//     delaying every other command. All three now use `get_state_for_keys`.
//     Their key sets live in `running/runner_keys.rs`: a static part built from
//     the model once before the loop, plus the six bookkeeping variables of the
//     operations that are currently active, rebuilt when the active set changes
//     (for the plan runner, when `{sp_id}_plan` changes - it does not create its
//     own operations). Measured on a scripted SOP + auto-operation + plan run:
//     16 `KEYS` calls before, 0 after, with the same number of MGETs.
//     Two holes in the key derivation were found and fixed on the way, both of
//     which were invisible while the runners read the whole database:
//     `Operation::get_all_var_keys` skipped `bypass_transitions` entirely, and
//     `Transition::get_all_var_keys` collected only an action's *target*
//     variable - so `var:a <- var:b` never contributed `b`, which is how a
//     consuming package's own variables usually enter a model. Every runner now
//     also reads the union of *all* model variables rather than only those of
//     the operations it drives, since guards routinely reference variables
//     another operation group writes.
//     Escape hatch: `MICRO_SP_READ_FULL_STATE=1` puts all three runners back on
//     `get_full_state` with no code change, for the case where something still
//     goes missing in the field.
//     `StateManager::get_full_state` itself is unchanged and still available -
//     if a whole-state read is ever needed again, store the state as one Redis
//     HASH and use `HGETALL` rather than reinstating `KEYS *`.
//                        -> running/runner_keys.rs, management/state/get_full_state.rs
//
//  3. `operation_log_receiver_task` does GET + JSON parse + JSON serialise +
//     SET of an unbounded, ever-growing log blob per log message - O(N^2) work
//     over a run, on the shared connection. Replace with `RPUSH`/`XADD` plus a
//     length cap, or buffer in memory and flush on a timer. -> utils/op_logger.rs
//
//  4. DONE. `Predicate::eval` and `Transition::eval`/`take` took `self` by
//     value, so every caller cloned the entire guard/action tree first - in
//     loops, per operation, per tick, and per node in the planner. The
//     signatures moved to `&self` (with `take_mut`/`take_planning_mut` as the
//     in-place forms), and the call sites have now been migrated too: the
//     ~30 dead `.clone()`/`.to_owned()` calls in `Operation`'s eight guard
//     methods, in `process_operation` (twelve `operation.clone().x(..)` on
//     `&self` methods), in `process_transition` and in both BFS planners are
//     gone. `Operation::start`/`complete`/`fail`/`bypass`/`timeout` and
//     `take_planning` additionally dropped one full `State` copy each by using
//     `take_mut` + `assign_mut` instead of chaining `take` and `assign`.
//                                    -> modelling/predicate.rs, transition.rs
//
//  5. DONE. `check_redis_health` fired a PING round trip before every tick of
//     every runner (~40-60 extra RTTs/s) that added directly to state-change
//     latency. The connection layer now uses `redis::aio::ConnectionManager`
//     (alias `SPConnection`), which reconnects itself with exponential backoff
//     and jitter, so no pre-flight probe is needed: the pings are gone from all
//     eight runner loops and from the log receiver, the `Arc<RwLock<..>>` around
//     the connection is gone, and each runner now takes one long-lived handle
//     before its loop instead of re-fetching one per tick. `check_redis_health`
//     survives as a diagnostic, plus an opt-in
//     `ConnectionManager::spawn_health_monitor` that pings once per process
//     every few seconds.                            -> management/connection.rs
//
//  6. Polling architecture: seven timer-driven tasks, each reading and writing
//     Redis on its own interval. A logical step that crosses runners costs the
//     *sum* of the intervening tick periods, and the polling is the idle CPU
//     floor. Drive the runners off change notifications (keyspace
//     notifications / an explicit PUBLISH on write / a Redis Stream) with a
//     slow watchdog tick, and/or co-locate the three operation runners that
//     already read the same state.                     -> running/main_runner.rs
//
//  7. DONE (option A - in-place mutation). `State` is used as a persistent
//     value but is backed by a plain `HashMap` with no structural sharing, so
//     every `update`/`add`/`remove`/`extend` copied the whole map. There are now
//     `update_mut` / `add_mut` / `remove_mut` / `extend_mut` that mutate in
//     place; the owned methods are unchanged in behaviour and signature but are
//     thin wrappers that clone exactly once, so even unmigrated call sites got
//     cheaper (`add` and `extend` were cloning two and three times). The hot
//     paths were migrated: `Action::assign_mut` and `Transition::take_mut` /
//     `take_planning_mut` mean a transition costs one clone instead of
//     one-per-action; `process_operation` writes its per-tick bookkeeping in
//     place; `state_init` builds the initial and per-operation variable sets
//     with `add_mut` (76 call sites) instead of a full-state copy each.
//     Option B - swapping in a persistent map (`im::HashMap`) or `Arc`-wrapping
//     values - is still open and would make the remaining owned-style call
//     sites cheap too.                                       -> core/sp_state.rs
//
//  8. Redis round trips are never pipelined: a single tick can issue PING,
//     KEYS, MGET, MSET, DEL, DEL sequentially. `redis::pipe()` collapses the
//     writes; HGETALL collapses the reads.              -> management/state.rs
//
//  9. DONE. Unconditional per-tick writes. Measured against a real Redis
//     rather than assumed, which corrected the item: eight runners idling for
//     five seconds already issued *zero* writes, so the "MSET ~10x/s on a fully
//     idle system" claim was wrong. Two real write storms did show up:
//       - `goal_runner` re-generated every scheduled goal's id with `nanoid!`
//         on every tick, so `_scheduled_goals` serialised differently every
//         time and an MSET went out on every tick for as long as anything sat
//         in the queue - 50 MSETs per 5 s, now 0. Ids are assigned once, on
//         admission (`admit_goals`), and a goal now keeps the id it was
//         announced with. Its `.update(..)` chains are `update_mut` too.
//       - `time_interface_runner` called `set_state` *inside* its per-timer
//         loop: 153 MSETs per 5 s with three timers running, now 51 (one per
//         tick, regardless of timer count). All timers accumulate into one
//         `new_state` threaded through the loop.
//     `plan_runner` got its terminated-operations block guarded and its
//     six-`update` tail converted to `update_mut`.
//     `process_operation`'s `_elapsed_*_ms` write is the one that remains, and
//     it is legitimate: it only fires for an operation that is actually
//     running, where the counter really changed. Deriving elapsed time from a
//     stored start `SystemTime` would remove it *and* fix the tick-constant bug
//     (`process_operation` charges 200 ms per tick while `sop_runner` ticks at
//     100 ms), but that is a timeout-semantics change, still open.
//                       -> running/goal_runner.rs, time_runner.rs, plan_runner.rs
//
// 10. DONE. `bfs_operation_planner` ran synchronously inside an async task for
//     up to its 5 s deadline, blocking a tokio worker and therefore the other
//     runners scheduled on it; internally it used `Vec::insert(0, ..)` (O(n^2)
//     over the search), cloned the full state into `visited` per node, hashed
//     states by allocating and sorting every key, cloned the plan prefix per
//     successor, and took the state and the whole operation model by value.
//     It now runs on `spawn_blocking` from `planner_ticker` (with the
//     operations behind an `Arc` built once before the loop), and internally
//     uses a `VecDeque`, a visited key over only the model's variables, a
//     parent-link arena for the plan, and `&State` / `&[Operation]`.
//     Measured: a 12-operation problem over a 200-variable state went from
//     ~3.0 s to ~1.25 s; and during a 1.3 s search a heartbeat task on the same
//     runtime went from 1 tick (stalled) to 60 (unaffected) - that stall is the
//     "state changes stop happening while planning" symptom.
//     API note: `bfs_operation_planner` now takes `&State`, `&Predicate` and
//     `&[Operation]` instead of owned values.
//     Still open: the frontier holds a full `State` per node, so `take_planning`
//     copies every variable including the ones planning never touches. Running
//     the search over a projection of the state (the model's variables plus the
//     goal's) would cut that, at the cost of relying on the key derivation
//     being complete - see the note on `Transition::get_all_var_keys`.
//     `bfs_transition_planner` still has the original shape; it is only used by
//     tests.                    -> planning/operation.rs, running/planner_ticker.rs
//
// 11. DONE. `Arc<Model>`. `main_runner` cloned the model five times and two of
//     the spawned runners cloned it again internally, so seven deep copies of
//     every operation, transition, predicate and action were live for the whole
//     process. It is one `Arc<Model>` now, with each task holding a refcount.
//     Runner signatures are unchanged (`&Model`), so nothing downstream breaks.
//     Measured with a counting allocator: one deep copy of a 30-operation model
//     is ~271 KB and of a 100-operation model ~878 KB, so this gives back
//     ~1.6 MB and ~5.3 MB respectively, plus the startup CPU of six deep copies.
//                          -> running/main_runner.rs, auto_runner.rs
//
// Smaller but cheap: hoist the per-tick `format!("{}_...", name)` key building
// into cached key strings; dedup the `keys` vectors before `MGET`; only build
// log/info strings when they actually changed (the `Disabled` arm of
// `process_operation` renders full predicate trees every tick).
//
// Note: `src/management/snapshot.rs` and `src/utils/op_logger.rs` were removed
// from the module tree (their `pub use`/`pub mod` lines below and in
// `utils/mod.rs` are commented out), so the op-logger item (3) above is
// currently dead code - fix it before re-enabling that module.
// ============================================================================

pub static MAX_ALLOWED_OPERATION_DURATION_MS: i64 = 600000; // milliseconds
pub static MAX_REPLAN_RETRIES: i64 = 3;
pub static MAX_RECURSION_DEPTH: u64 = 1000;

pub const NANOID_ALPHABET: [char; 62] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
    'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B',
    'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U',
    'V', 'W', 'X', 'Y', 'Z',
];

pub mod core;
pub use crate::core::sp_assignment::*;
pub use crate::core::sp_state::*;
pub use crate::core::sp_value::*;
pub use crate::core::sp_variable::*;
pub use crate::core::sp_wrapped::*;

pub mod modelling;
pub use crate::modelling::action::*;
pub use crate::modelling::model::*;
pub use crate::modelling::operation::*;
pub use crate::modelling::parser::*;
pub use crate::modelling::predicate::*;
pub use crate::modelling::sops::*;
pub use crate::modelling::transition::*;

pub mod planning;
pub use crate::planning::operation::*;
pub use crate::planning::transition::*;

pub mod running;
pub use crate::running::auto_runner::*;
// pub use crate::running::goal_runner::*;
// tests
// pub use crate::running::goal_scheduler::*;
pub use crate::running::main_runner::*;
pub use crate::running::plan_runner::*;
pub use crate::running::planner_ticker::*;
pub use crate::running::runner_keys::*;
pub use crate::running::runner_states::*;
pub use crate::running::sop_runner::*;
pub use crate::running::state_init::*;
pub use crate::running::time_runner::*;

pub mod management;
pub use crate::management::connection::*;
// pub use crate::management::snapshot::*;
pub use crate::management::state::*;
pub use crate::management::transforms::*;

pub mod transforms;
pub use crate::transforms::cycles::*;
pub use crate::transforms::loading::*;
pub use crate::transforms::lookup::*;
pub use crate::transforms::treeviz::*;

pub mod utils;
pub use crate::utils::info_logger::*;
pub use crate::utils::metadata::*;
// pub use crate::utils::op_logger::*;

pub mod macros;
#[allow(unused_imports)]
pub use crate::macros::action::*;
#[allow(unused_imports)]
pub use crate::macros::predicate::*;
#[allow(unused_imports)]
pub use crate::macros::sp_assignment::*;
#[allow(unused_imports)]
pub use crate::macros::sp_variable::*;
#[allow(unused_imports)]
pub use crate::macros::transition::*;
