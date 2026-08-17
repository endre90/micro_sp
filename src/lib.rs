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
// 12. DONE. The `Disabled` arm of `process_operation` rebuilt its information
//     message on every tick for every disabled operation - cloning every
//     precondition's guard and runner guard, wrapping them in two
//     `Predicate::OR` trees and rendering both through `Display` - only for the
//     `!=` check below to throw the identical string away. A disabled operation
//     is exactly the one that sits there for minutes. Both steady-state
//     messages (this and the waiting branch of `Executing`) are now built once,
//     when the operation first reports that state, and skipped afterwards via
//     an allocation-free prefix test against the message already in the state.
//     Measured per disabled operation per tick: 1.2 us (1 precondition, 2
//     conjuncts) / 5.9 us (3 x 4) / 17.3 us (5 x 8) of rebuild work, replaced
//     by a ~2 ns check.                       -> running/process_operation.rs
//
// 13. DONE (mostly). `sop_runner` per-tick costs. `active_sop_container
//     .clone().unwrap()` deep-copied the whole SOP tree - every `Operation`,
//     `Transition` and `Predicate` - twice on every tick of a running SOP, and
//     the walk was additionally handed `new_state.clone()` whose result is
//     assigned straight back over it. All are borrows now. `SOP::get_state`
//     allocated a `format!` copy of the operation name per leaf and collected a
//     `Vec<SOPState>` per branch to run five `any`/`all` passes over; it is one
//     allocation-free pass. Measured on a 30-operation nested SOP: one tree
//     clone is 74 KB, so ~148 KB per tick (~1.5 MB/s at 100 ms) stops being
//     allocated, and `get_state` went from 5350 ns / 76 allocations to
//     1534 ns / 30 - at 60 operations, 9164 ns / 151 to 2645 ns / 60.
//     Left open deliberately, both noted in place: the `Box::pin` per visited
//     node (removing it means making `process_operation` synchronous, which is
//     exactly what has to be undone to re-enable the logging channel), and
//     precomputing every node's state once per tick (the walk threads `State`
//     through, so a `Parallel` branch sees what the branch before it just did -
//     precomputing would turn that into a one-tick delay, a behaviour change).
//                          -> running/sop_runner.rs, modelling/sops.rs
//
// 14. The low-priority batch. Everything below was measured before and after,
//     because several of these turned out not to be worth their complexity.
//
//     DONE:
//       - `tf_interface` polled a 7-key `MGET` every 250 ms purely to read one
//         boolean; its whole body sits inside `if request_trigger`. One `GET`
//         now, with the rest fetched only when a request is actually pending.
//       - `planner_ticker` polled the whole planning key set every 500 ms to
//         discover that no replan was requested. It reads the two flags that
//         decide that and fetches the rest only when there is work.
//         Idle Redis server time across five runners: 1285 us -> 887 us per
//         5 s window. Note the round-trip *count* is unchanged - these are
//         still polls, just much smaller ones. Removing the polls needs the
//         event-driven change (item 6).
//       - `keys` deduplicated in `auto_transition_runner` and `planner_ticker`
//         (`normalize_keys`); operations share most of their variables, so the
//         `MGET` was sending the same key many times over.
//       - `build_state`: `HashMap::with_capacity`, and a borrowing form so
//         `get_state_for_keys` stops cloning its whole key list every tick for
//         every runner. `StateManager::build_state` keeps its owned signature.
//       - `StateManager::apply`: the runners' tail was `set_state` then one or
//         two `remove_sp_values`, three sequential round trips each awaited
//         before the next could be sent. One `redis::pipe()` now. Deliberately
//         not `.atomic()` - the code it replaces had no atomicity either, and
//         adding MULTI/EXEC under a perf fix would be a silent semantic change.
//       - `OperationState::as_str()` plus an allocation-free `value_is`: the
//         guard path compared with `value == OperationState::X.to_spvalue()`,
//         which allocates a `String` per comparison - five of them per
//         operation per tick in `can_be_cancelled` alone. 79 ns -> ~0.
//         Careful: `to_spvalue()` collapses "UNKNOWN" to the UNKNOWN *variant*,
//         so the replacement has to mirror that; a test pins every pairing.
//       - TF `get_all_transforms`: `scan_match` used the Redis default COUNT of
//         10, so a lookup over a 3000-key keyspace took ~294 sequential SCAN
//         round trips. COUNT 1000 makes it 4. Measured per lookup: 9.33 ms ->
//         1.21 ms. Dropping `into_par_iter()` for a plain iterator took it to
//         855 us - rayon's hand-off cost more than the handful of small JSON
//         parses, as suspected. That was the crate's last rayon use.
//
//     NOT done, with the measurement that decided it:
//       - Single Redis HASH + `HGETALL`/`HMGET` instead of key-per-variable.
//         This is a change to the on-disk layout: anything reading these keys
//         directly (dashboards, other processes) breaks, and it needs a
//         migration. Wants a deliberate decision, not a drive-by.
//       - Key-string caching (an `OperationKeys` side table). Measured: the
//         nine `format!("{}_...", name)` calls cost 580 ns per operation per
//         tick - about 58 us/s with ten active operations. Not worth threading
//         a cache through all three runners. The `to_spvalue()` half of that
//         item, which was on the far hotter guard path, is done.
//       - `build_state` skipping re-parses of unchanged values, and a prebuilt
//         variable table. Measured: 82 us per call over 300 keys (274 ns per
//         variable), so roughly 0.25% of a core across the runners. It needs a
//         raw-string cache threaded through every runner's read path and a new
//         public API to carry it; the ratio does not justify that yet.
//       - Caching the TF buffer with a version counter. Correct only if *every*
//         writer goes through `TransformsManager`; a process writing
//         `TF_PREFIX` keys directly would leave the cache serving stale
//         transforms - a correctness bug traded for a slow path. The SCAN fix
//         above already took the dominant cost out.
//
// 15. Correctness fixes (these were bugs, not slow paths):
//
//     DONE:
//       - Elapsed-time accounting. `process_operation` charged a compile-time
//         constant of 200 ms per call, but `sop_runner` calls it at 100 ms - so
//         every SOP operation aged at *twice* real speed and timed out at half
//         its configured deadline. Any runner whose tick slipped because Redis
//         was slow under-counted in the other direction. Each runner now
//         measures its real tick period with `Instant` and passes it in.
//                            -> running/process_operation.rs and its three callers
//       - Plan step resolution. `op_name.starts_with(&op.name)` matched any
//         operation whose name is a prefix of the step, so a model with both
//         `op_move` and `op_move_to_b` could drive the wrong operation's
//         transitions depending only on model order. Steps are
//         `{name}_{nanoid}`, so they now resolve exactly, with a longest-match
//         prefix fallback for step names that carry no suffix.
//                                            -> running/plan_runner.rs
//       - Auto transition chaining. Every transition was evaluated against the
//         same snapshot and wrote its own effects straight to Redis, so a chain
//         of N advanced one link per 50 ms tick, and two transitions writing
//         the same variable both decided from the same stale read. A single
//         `State` is threaded through the loop and written once. Measured: a
//         four-link chain settles in 13 ms instead of >= 200 ms.
//                                            -> running/auto_runner.rs
//       - `Operation::can_be_cancelled` guard was a tautology: three of its
//         five clauses were `!=` where `==` was meant, so it was true for every
//         operation state. Since `Operation::cancel` does not check the current
//         state, pressing stop drove *finished* operations to `Cancelled` too.
//         It now lists the states where cancelling means something.
//                                            -> modelling/operation.rs
//
//     OPEN (deliberately - see the long note in management/state.rs):
//       - Read-modify-write across runners is not atomic. Seven `{sp_id}_*`
//         keys are written by two or three runners each, and the handover
//         between goal_runner, planner_ticker and plan_runner is currently
//         *implemented* through those cross-writes. Fixing it means either
//         reworking that handover to give each runner exclusive ownership, or
//         wrapping each tick in WATCH/MULTI/EXEC with a retry policy. Both are
//         design changes rather than bug fixes, so they need your call.
//
// Smaller but cheap: hoist the per-tick `format!("{}_...", name)` key building
// into cached key strings (see the measurement above before bothering).
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
pub use crate::running::tick::*;
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

/// The on-disk activity log. Deliberately re-exported as a *module* rather than
/// glob-imported: its API is `init`, `flush`, `is_enabled`, `log_operation`,
/// ... - names far too generic to put in the crate root, where they would
/// collide with anything a consuming package defines. Call it as
/// `micro_sp::activity_log::init_from_env()`.
pub use crate::utils::activity_log;
pub use crate::utils::activity_log::{
    ActivityKind, ActivityLogConfig, ActivityRecord, ActivityWriter,
};

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

/// `NANOID_ALPHABET` backs every id generated with `nanoid::nanoid!(10,
/// &NANOID_ALPHABET)` across the runners (`auto_runner`, `goal_runner`,
/// `sop_runner`, `planner_ticker`, transform loading, ...), and
/// `running/plan_runner.rs` separately relies on
/// `NANOID_ALPHABET.contains(&c)` to recognise which suffix characters of a
/// step name are a generated id versus part of the operation name. Both uses
/// silently break the same way if the alphabet ever gained a duplicate
/// character or shrank: nanoid's collision-resistance and plan_runner's
/// suffix detection both assume every character in it is distinct.
#[cfg(test)]
mod lib_tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn nanoid_alphabet_is_62_distinct_alphanumeric_chars() {
        assert_eq!(NANOID_ALPHABET.len(), 62);

        let unique: HashSet<char> = NANOID_ALPHABET.iter().copied().collect();
        assert_eq!(
            unique.len(),
            NANOID_ALPHABET.len(),
            "every character in the alphabet must be distinct, or nanoid's \
             collision-resistance and plan_runner's suffix detection both weaken"
        );

        assert!(
            NANOID_ALPHABET.iter().all(|c| c.is_ascii_alphanumeric()),
            "plan_runner strips id suffixes assuming this alphabet is plain alphanumeric"
        );
    }

    /// These are read by `Operation::start`/friends as *defaults* that only
    /// apply when an operation/config does not override them - pin the
    /// documented values so a change here is a deliberate policy change, not
    /// an accidental one.
    #[test]
    fn tunable_limits_hold_their_documented_defaults() {
        assert_eq!(MAX_ALLOWED_OPERATION_DURATION_MS, 600_000);
        assert_eq!(MAX_REPLAN_RETRIES, 3);
        assert_eq!(MAX_RECURSION_DEPTH, 1000);
    }
}
