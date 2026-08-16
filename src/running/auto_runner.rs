use crate::{
    running::process_operation::{OperationProcessingType, process_operation},
    *,
};
use chrono::Utc;
// use rand::seq::IndexedRandom;
use crate::SPConnection;
use std::{sync::Arc, time::Duration};
use tokio::{sync::mpsc, time::interval};

// Add automatic operations here as well that finish immediatelly, god for setting some values, triggering robot moves etc.
pub static TRANSITION_RUNNER_TICK_INTERVAL_MS: u64 = 50;

// PERF: issues its own `set_state` (a full Redis round trip) per fired
// transition, inside the caller's `for t in &model.auto_transitions` loop. If
// three auto transitions fire on the same tick that is three sequential MSETs.
// Suggested: return the modified `State` (or accumulate into a shared
// `new_state`) and have `auto_transition_runner` write once per tick.
// PERF/correctness: each transition is evaluated against the same `state`
// snapshot but the effects are written straight to Redis, so a transition whose
// guard depends on a variable an earlier transition just wrote will not see it
// until the next tick - an avoidable extra 50 ms of latency per chained
// transition. Threading a single mutable `State` through the loop fixes both
// the latency and the round trips.
// DONE: PERF: `transition.to_owned().eval(..)` and the two `transition.clone()`
// calls used to deep-copy the whole transition (guard predicate tree, runner
// guard, both action vectors) three times per evaluation - and the first copy
// happened even for transitions whose guard is false, which is the
// overwhelmingly common case. `Predicate::eval`/`Transition::eval` take `&self`
// and the take path uses `take_mut`, so the common no-fire path now allocates
// nothing at all.
// PERF: `nanoid!` and the `format!` for the unique name are computed before the
// guard result is used elsewhere; they are only needed when the transition
// actually fires, which is already the case here - but the `TransitionMsg` with
// its `format!("Executed.")` and `Utc::now()` is built even when nobody
// consumes the log. Consider gating on `log::log_enabled!`.
async fn process_transition(
    con: &mut SPConnection,
    transition: &Transition,
    state: &State,
    // logging_tx: mpsc::Sender<LogMsg>,
    log_target: &str,
) {
    if !transition.eval(state, &log_target) {
        return;
    }

    // DONE: PERF: this used to clone the whole transition twice - once to rename
    // it for the log line, once more for the by-value `take` - even though the
    // rename only ever feeds a `format!`. The name is now built as a plain
    // `String` and the actions are applied in place on one state copy.
    let unique_id = nanoid::nanoid!(10, &NANOID_ALPHABET);
    let unique_name = format!("{}_{}", transition.name, unique_id);

    let mut new_state = state.clone();
    transition.take_mut(&mut new_state, &log_target);
    log::info!(target: &log_target, "Executed auto transition: '{}'.", unique_name);

    // let transition_msg = TransitionMsg {
    //     transition_name: transition.name.clone(),
    //     timestamp: Utc::now(),
    //     severity: log::Level::Info,
    //     log: format!("Executed."),
    // };
    // let log_msg = LogMsg::TransitionMsg(transition_msg);
    // match logging_tx.send(log_msg).await {
    //     Ok(()) => (),
    //     Err(e) => log::error!(target: &log_target, "Failed to send logging with: {e}."),
    // }

    let modified_state = state.get_diff_partial_state(&new_state);
    StateManager::set_state(con, &modified_state).await;
}

// PERF: this runner is already doing the right thing on the read side - it
// precomputes `keys` once and uses `get_state_for_keys`. Two things left:
//   - `let model = model.clone()` deep-copies the entire model into the task.
//     `main_runner` already clones the model once per spawned runner; with five
//     runners that is five full copies of every operation, transition and
//     predicate held for the lifetime of the process. Pass `Arc<Model>` instead.
//   - the `keys` vector is not deduplicated, so transitions sharing variables
//     make the per-tick `MGET` send the same key several times. `sort_unstable()
//     + dedup()` once here shrinks every subsequent request.
// PERF: at 50 ms this is the fastest-ticking runner and therefore the biggest
// contributor to idle CPU. If the auto transitions only react to variables
// written by other runners, a keyspace-notification subscription on `keys`
// would let it sleep entirely when nothing changes.
pub async fn auto_transition_runner(
    name: &str,
    model: &Model,
    connection_manager: &Arc<ConnectionManager>,
    // logging_tx: mpsc::Sender<LogMsg>,
) -> Result<(), Box<dyn std::error::Error>> {
    initialize_env_logger();
    let mut interval = interval(Duration::from_millis(TRANSITION_RUNNER_TICK_INTERVAL_MS));
    let model = model.clone();
    let log_target = format!("{}_auto_transition_runner", name);
    let keys: Vec<String> = model
        .auto_transitions
        .iter()
        .flat_map(|t| t.get_all_var_keys())
        .collect();

    log::info!(target: &log_target, "Online.");

    // PERF: one long-lived connection handle for the whole runner instead of
    // re-fetching one every tick, and no pre-flight PING before the real work.
    // `SPConnection` is cheap to clone, multiplexed and self-healing, so this
    // handle stays valid across reconnects; a dropped socket now surfaces as an
    // error on the command itself, which the callee already logs and skips.
    let mut con = connection_manager.get_connection().await;

    loop {
        interval.tick().await;
        let state = match StateManager::get_state_for_keys(&mut con, &keys, &log_target).await {
            Some(s) => s,
            None => continue,
        };

        for t in &model.auto_transitions {
            process_transition(&mut con, t, &state, &log_target).await;
        }
    }
}

// PERF: same shape as `sop_runner`. Specifically:
//   1. DONE: `get_full_state` (a blocking `KEYS *` + `MGET` of the whole
//      database every 200 ms) is replaced with `get_state_for_keys`. The key
//      set is `model.auto_operations`/`mutexed_auto_operations`
//      `get_all_var_keys()` plus their template name variables, unioned with
//      the bookkeeping keys of whatever is currently in
//      `active_auto_ops`/`active_mutexed_op` - see `running::runner_keys`. It
//      is rebuilt only when the active set changes.
//   2. `for op in &model.auto_operations { if op.eval(&state, ..) }` evaluates
//      every auto operation's preconditions every tick. `Operation::eval` first
//      checks `state.get_value(&self.name, ..)` and only then the guards - but
//      because `get_value` clones the whole map (see `State::get_value`), even
//      the cheap rejection path is expensive. Fixing `get_value` makes this
//      loop nearly free; beyond that, indexing operations by the variables
//      their guards read would let you skip operations whose inputs did not
//      change since the last tick.
//   3. `active_auto_ops` is consumed and rebuilt (`next_active_auto_ops`) every
//      tick, cloning each `Operation` in and out. `Vec::retain` with the
//      state check, or holding `Arc<Operation>`, avoids the copying.
//   4. `format!("{}", current_active_op.name)` is an allocating no-op copy of a
//      `String` that is already owned - pass `&current_active_op.name`.
//   5. The tail does `set_state` then `remove_sp_values` twice: three
//      sequential round trips that should be one pipeline.
//   6. `let model = model.clone()` - see the note on `auto_transition_runner`;
//      use `Arc<Model>`.
pub async fn auto_operation_runner(
    sp_id: &str,
    model: &Model,
    // logging_tx: mpsc::Sender<LogMsg>,
    connection_manager: &Arc<ConnectionManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    initialize_env_logger();
    let mut interval = interval(Duration::from_millis(OPERAION_RUNNER_TICK_INTERVAL_MS));
    let model = model.clone();
    let log_target = format!("{}_operation_runner", sp_id);

    let mut active_auto_ops: Vec<Operation> = vec![];
    let mut active_mutexed_op: Option<Operation> = None;
    let mut terminated_operations: Vec<String> = vec![];

    // See the note on `sop_runner`: the static part comes from the model, the
    // dynamic part is the bookkeeping variables of the operations currently in
    // `active_auto_ops` / `active_mutexed_op`, whose names only exist once they
    // have been activated with a `nanoid` suffix.
    let static_keys = auto_operation_runner_static_keys(sp_id, &model);
    let mut keys = static_keys.clone();

    // PERF: one long-lived connection handle for the whole runner instead of
    // re-fetching one every tick, and no pre-flight PING before the real work.
    // `SPConnection` is cheap to clone, multiplexed and self-healing, so this
    // handle stays valid across reconnects; a dropped socket now surfaces as an
    // error on the command itself, which the callee already logs and skips.
    let mut con = connection_manager.get_connection().await;

    loop {
        interval.tick().await;
        let state = match StateManager::get_state_for_keys(&mut con, &keys, &log_target).await {
            Some(s) => s,
            None => continue,
        };

        let mut new_state = state.clone();
        let mut new_op_ids = vec![];

        for op in &model.auto_operations {
            if op.eval(&state, &log_target) {
                let prefix = format!("{}_", op.name);
                if !active_auto_ops.iter().any(|a| a.name.starts_with(&prefix)) {
                    let unique_id = nanoid::nanoid!(10, &NANOID_ALPHABET);
                    let unique_op_id = format!("{}{}", prefix, unique_id);
                    let mut op_mut = op.clone();
                    op_mut.name = unique_op_id.clone();
                    active_auto_ops.push(op_mut);
                    new_op_ids.push(unique_op_id);
                }
            }
        }

        if active_mutexed_op.is_none() {
            for op in &model.mutexed_auto_operations {
                if op.eval(&state, &log_target) {
                    let prefix = format!("{}_", op.name);
                    let unique_id = nanoid::nanoid!(10, &NANOID_ALPHABET);
                    let unique_op_id = format!("{}{}", prefix, unique_id);
                    let mut op_mut = op.clone();
                    op_mut.name = unique_op_id.clone();
                    
                    active_mutexed_op = Some(op_mut);
                    new_op_ids.push(unique_op_id);
                    
                    break;
                }
            }
        }

        if !new_op_ids.is_empty() {
            new_state =
                add_operation_meta_tracking_variables(&new_op_ids, &new_state, false, &log_target);
            new_state = add_operation_state_tracking_variable(&new_op_ids, &new_state, &log_target);
        }

        let mut next_active_auto_ops = vec![];
        for current_active_op in active_auto_ops {
            new_state = process_operation(
                &sp_id,
                new_state,
                &current_active_op,
                OperationProcessingType::Automatic,
                None,
                None,
                // logging_tx.clone(),
                &log_target,
            )
            .await;

            let operation_state = new_state.get_string_or_default_to_unknown(
                &format!("{}", current_active_op.name),
                &log_target,
            );

            match OperationState::from_str(&operation_state) {
                OperationState::Terminated(_) => {
                    terminated_operations.push(current_active_op.name.clone());
                }
                _ => next_active_auto_ops.push(current_active_op),
            };
        }
        active_auto_ops = next_active_auto_ops;

        let mut next_active_mutexed_op = None;
        if let Some(current_active_op) = active_mutexed_op {
            new_state = process_operation(
                &sp_id,
                new_state,
                &current_active_op,
                OperationProcessingType::Automatic,
                None,
                None,
                // logging_tx.clone(),
                &log_target,
            )
            .await;

            let operation_state = new_state.get_string_or_default_to_unknown(
                &format!("{}", current_active_op.name),
                &log_target,
            );

            match OperationState::from_str(&operation_state) {
                OperationState::Terminated(_) => {
                    terminated_operations.push(current_active_op.name.clone());
                }
                _ => {
                    next_active_mutexed_op = Some(current_active_op);
                }
            };
        }
        active_mutexed_op = next_active_mutexed_op;

        let modified_state = state.get_diff_partial_state_and_add_missing(&new_state);
        StateManager::set_state(&mut con, &modified_state).await;

        let active_set_changed = !new_op_ids.is_empty() || !terminated_operations.is_empty();

        if !terminated_operations.is_empty() {
            let mut terminated_operations_meta = vec![];
            for op in &terminated_operations {
                terminated_operations_meta.push(format!("{}_information", op));
                terminated_operations_meta.push(format!("{}_failure_retry_counter", op));
                terminated_operations_meta.push(format!("{}_timeout_retry_counter", op));
                terminated_operations_meta.push(format!("{}_elapsed_executing_ms", op));
                terminated_operations_meta.push(format!("{}_elapsed_disabled_ms", op));
            }
            StateManager::remove_sp_values(&mut con, &terminated_operations).await;
            StateManager::remove_sp_values(&mut con, &terminated_operations_meta).await;

            terminated_operations.clear();
        }

        // Operations were activated and/or terminated this tick, so the set of
        // bookkeeping variables to read from the next tick on has changed.
        if active_set_changed {
            let active: Vec<String> = active_auto_ops
                .iter()
                .chain(active_mutexed_op.iter())
                .map(|op| op.name.clone())
                .collect();
            keys = keys_with_active_operations(&static_keys, &active);
        }
    }
}