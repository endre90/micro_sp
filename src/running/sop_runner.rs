use crate::*;
use log::Level;
use crate::SPConnection;
use std::sync::Arc;
use tokio::{
    sync::mpsc,
    time::{Duration, interval},
};

static TICK_INTERVAL: u64 = 100; // millis

// PERF: this is the loop you feel when a SOP is running. Per 100 ms tick it
// does: an `MGET` of its key set, a full `state.clone()`, a recursive walk of
// the SOP tree that clones the tree several times and evaluates every operation
// guard, a full-state diff, and an MSET. Concretely, the things worth changing
// here:
//
// 1. DONE: `StateManager::get_full_state` -> `get_state_for_keys`. The key set
//    is `SOP::get_all_var_keys()` unioned with the per-operation bookkeeping
//    keys (`{op}`, `{op}_information`, `{op}_elapsed_*`, `{op}_*_retry_counter`)
//    and the handful of `{sp_id}_sop_*` keys - see `running::runner_keys`. It
//    is recomputed only when a SOP is activated or torn down, not per tick, so
//    the blocking `KEYS *` is gone.
// 2. `active_sop_container.clone().unwrap()` appears three times in the
//    `Executing` arm alone and deep-copies the whole SOP tree (every
//    `Operation`, every `Transition`, every `Predicate`) each time. Use
//    `as_ref()`/`if let Some(sop) = &active_sop_container` - the functions only
//    need `&SOP`. Same for `sop_template.sop.clone()` in the activation path.
// 3. `let mut new_state = state.clone()` clones the entire state map every tick
//    just so the diff at the bottom has something to compare against. With
//    `update_mut` + a dirty-key list (see `State`) you can drop both the clone
//    and the diff scan.
// 4. The `format!` calls building `new_sop_info` run every tick even when the
//    text is identical to `old_sop_information` and is then discarded; the
//    `visualize_sop(..)` in the `Initial` arm renders the whole tree to a
//    string. Suggested: compare cheap discriminants first and only format when
//    something actually changed.
// 5. `model.sops.iter().find(|s| s.id == sop_id)` is a linear scan of the model
//    every tick; a `HashMap<String, &SOPStruct>` built once at startup is O(1).
// 6. Tick rate vs. latency: at 100 ms, and with each operation step having to
//    round-trip through Redis, a SOP of N sequential operations takes at least
//    ~2-3 ticks per operation. Driving this loop off change notifications (see
//    `ConnectionManager`) rather than a timer is what actually fixes the
//    "state change doesn't occur as quickly as I want" symptom; lowering
//    `TICK_INTERVAL` only trades latency for more `KEYS *` storms.
// 7. Note that `process_operation` charges elapsed time using
//    `OPERAION_RUNNER_TICK_INTERVAL_MS` (200 ms) while this runner ticks at
//    100 ms, so SOP operations accumulate elapsed time at 2x real speed and
//    time out early. Measure real elapsed time with `Instant` instead of
//    assuming a tick constant.
pub async fn sop_runner(
    sp_id: &str,
    model: &Model,
    // logging_tx: mpsc::Sender<LogMsg>,
    connection_manager: &Arc<ConnectionManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    initialize_env_logger();
    let mut interval = interval(Duration::from_millis(TICK_INTERVAL));
    let log_target = &format!("{}_sop_runner", sp_id);

    log::info!(target: log_target, "Online.");

    let mut active_unique_sop_id: Option<String> = None;
    let mut active_unique_sop_state: SOPState = SOPState::Initial;
    let mut active_sop_container: Option<SOP> = None;
    // let mut terminated_operations: Vec<String> = vec!();

    // The variables read every tick no matter what is running, and the set
    // actually requested from Redis. The latter grows with the bookkeeping
    // variables of a SOP's operations while that SOP is active - their names
    // only exist once `uniquify_sop_operations` has run, which is why the set
    // is rebuilt there rather than computed once here.
    let static_keys = sop_runner_static_keys(sp_id, model);
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
        let mut sop_state =
            state.get_string_or_default_to_unknown(&format!("{}_sop_state", sp_id), &log_target);

        let sop_enabled =
            state.get_bool_or_default_to_false(&format!("{}_sop_enabled", sp_id), &log_target);

        let sop_id =
            state.get_string_or_default_to_unknown(&format!("{}_sop_id", sp_id), &log_target);

        let Some(sop_template) = model.sops.iter().find(|s| s.id == sop_id) else {
            log::debug!(target: &log_target, "SOP with id '{}' not found in model. Skipping evaluation.", sop_id);
            continue;
        };

        let old_sop_information = new_state
            .get_string_or_default_to_unknown(&format!("{}_sop_information", sop_id), &log_target);

        let mut new_sop_info: String; // = old_sop_information.clone();
        let mut sop_info_level: Level = log::Level::Info;

        // let terminated_operations_sp_value = state.get_array_or_default_to_empty(
        //     &format!("{}_terminated_operations", sp_id),
        //     &log_target,
        // );

        // let terminated_operations: Vec<String> = terminated_operations_sp_value
        //     .iter()
        //     .filter(|val| val.is_string())
        //     .map(|y| y.to_string())
        //     .collect();

        // Check first if there is an active unique SOP already running
        match active_unique_sop_id {
            None => {
                if sop_enabled {
                    let unique_sop_id = nanoid::nanoid!(10, &NANOID_ALPHABET);
                    active_unique_sop_id = Some(format!("{}_{}", sop_template.id, unique_sop_id));

                    let unique_sop = uniquify_sop_operations(sop_template.sop.clone());
                    active_sop_container = Some(unique_sop.clone());
                    let ops_in_sop = get_all_operations_from_sop(&unique_sop);
                    let op_names: Vec<String> =
                        ops_in_sop.iter().map(|x| x.name.clone()).collect();

                    // The bookkeeping variables of these operations are created
                    // in `new_state` below and written out by the diff at the
                    // end of this tick; from the next tick on they have to be
                    // read back, so the key set grows with them here.
                    keys = keys_with_active_operations(&static_keys, &op_names);

                    new_state = add_operation_meta_tracking_variables(
                        &op_names,
                        &new_state,
                        false,
                        &log_target,
                    );
                    new_state =
                        add_operation_state_tracking_variable(&op_names, &new_state, &log_target);
                    new_sop_info = format!("SOP '{sop_id}' is enabled, starting execution.");
                    sop_info_level = log::Level::Info;
                    new_state =
                        new_state.update(&format!("{}_sop_enabled", sp_id), false.to_spvalue());
                } else {
                    continue;
                }
            }
            Some(ref active_sop) => match active_unique_sop_state {
                SOPState::Initial => {
                    active_unique_sop_state = SOPState::Executing;
                    new_sop_info = format!(
                        "Initializing a new SOP '{}':\n{}",
                        active_sop,
                        visualize_sop(&active_sop_container.clone().unwrap())
                    );
                }
                SOPState::Executing => {
                    // Inform the operation that the sop is executing
                    sop_state = SOPState::Executing.to_string();
                    let con_clone = con.clone();
                    new_sop_info = format!("Executing SOP '{active_sop}'.");
                    sop_info_level = log::Level::Info;
                    new_state = process_sop_node_tick(
                        sp_id,
                        new_state.clone(),
                        &active_sop_container.clone().unwrap(),
                        con_clone,
                        // logging_tx.clone(),
                        &log_target,
                    )
                    .await;

                    let calculated_root_state = active_sop_container
                        .clone()
                        .unwrap()
                        .get_state(&new_state, &log_target);

                    if calculated_root_state != SOPState::Executing {
                        new_sop_info = format!("Completing SOP '{active_sop}'.");
                        sop_info_level = log::Level::Info;

                        active_unique_sop_state = calculated_root_state;
                    }
                }
                SOPState::Fatal => {
                    new_sop_info = format!("Fataled SOP '{active_sop}'.");
                    sop_info_level = log::Level::Error;
                    active_unique_sop_state = SOPState::Initial;
                    // Inform the operation that the sop has failed:
                    sop_state = SOPState::Fatal.to_string();

                    if let Some(unique_sop) = active_sop_container {
                        let con_clone = con.clone();
                        remove_operations_from_state(active_sop, &unique_sop, con_clone).await;
                    }

                    active_sop_container = None;
                    active_unique_sop_id = None;
                    // Those operation variables have just been deleted from
                    // Redis, so stop asking for them.
                    keys = static_keys.clone();
                }
                SOPState::Completed => {
                    new_sop_info = format!("Completed SOP '{active_sop}'.");
                    sop_info_level = log::Level::Info;
                    active_unique_sop_state = SOPState::Initial;
                    // Inform the operation that the sop has completed:
                    sop_state = SOPState::Completed.to_string();

                    if let Some(unique_sop) = active_sop_container {
                        let con_clone = con.clone();
                        remove_operations_from_state(active_sop, &unique_sop, con_clone).await;
                    }

                    active_sop_container = None;
                    active_unique_sop_id = None;
                    // Those operation variables have just been deleted from
                    // Redis, so stop asking for them.
                    keys = static_keys.clone();
                }
                SOPState::Cancelled => {
                    new_sop_info = format!("Cancelled SOP '{active_sop}'.");
                    sop_info_level = log::Level::Warn;
                    active_unique_sop_state = SOPState::Initial;
                    // Inform the operation that the sop has ben cancelled:
                    sop_state = SOPState::Cancelled.to_string();

                    if let Some(unique_sop) = active_sop_container {
                        let con_clone = con.clone();
                        remove_operations_from_state(active_sop, &unique_sop, con_clone).await;
                    }

                    active_sop_container = None;
                    active_unique_sop_id = None;
                    // Those operation variables have just been deleted from
                    // Redis, so stop asking for them.
                    keys = static_keys.clone();
                }
                SOPState::UNKNOWN => {
                    new_sop_info = format!("SOP '{active_sop}' state id UNKNOWN.");
                    sop_info_level = log::Level::Info;
                    active_unique_sop_state = SOPState::Initial;
                    active_sop_container = None;
                    active_unique_sop_id = None;
                    keys = static_keys.clone();
                }
            },
        }

        if new_sop_info != old_sop_information {
            match sop_info_level {
                log::Level::Info => log::info!(target: &log_target, "{}", new_sop_info),
                log::Level::Warn => log::warn!(target: &log_target, "{}", new_sop_info),
                log::Level::Error => log::error!(target: &log_target, "{}", new_sop_info),
                _ => (),
            }
        }

        new_state = new_state
            .update(
                &format!("{}_sop_information", sop_id),
                new_sop_info.to_spvalue(),
            )
            .update(&format!("{}_sop_state", sp_id), sop_state.to_spvalue());

        let modified_state = state.get_diff_partial_state_and_add_missing(&new_state);

        if !modified_state.state.is_empty() {
            StateManager::set_state(&mut con, &modified_state).await;
        }

    }
}

// PERF: three sequential Redis round trips (two `remove_sp_values` plus one
// `remove_sp_value`) where one `DEL` of the concatenated key list would do.
// Also the `println!` on the next line writes to stdout unconditionally and
// bypasses the `log` filter - it should be `log::debug!` so it can be turned
// off, since stdout writes are synchronous and block the tokio worker thread.
async fn remove_operations_from_state(sop_id: &str, unique_sop: &SOP, mut con: SPConnection) {
    let ops_in_sop = get_all_operations_from_sop(&unique_sop);
    let mut op_ids_meta = vec![];
    let sop_id = format!("op_{}", sop_id);
    println!("REMOVING: {}", sop_id);
    let mut op_ids = ops_in_sop
        .iter()
        .map(|x| x.name.to_string())
        .collect::<Vec<String>>();
    op_ids.push(sop_id.clone());
    for op in &op_ids {
        op_ids_meta.push(format!("{}_information", op));
        op_ids_meta.push(format!("{}_failure_retry_counter", op));
        op_ids_meta.push(format!("{}_timeout_retry_counter", op));
        op_ids_meta.push(format!("{}_elapsed_executing_ms", op));
        op_ids_meta.push(format!("{}_elapsed_disabled_ms", op));
    }

    StateManager::remove_sp_values(&mut con, &op_ids).await;
    StateManager::remove_sp_values(&mut con, &op_ids_meta).await;
    StateManager::remove_sp_value(&mut con, &sop_id).await;
}

// PERF: the tree walk threads `State` by value, so every recursion level moves
// (and every `process_operation` call rebuilds) the whole state map. Taking
// `&mut State` instead would let each node mutate in place with no copying.
// PERF: `Box::pin(..)` on each recursive call heap-allocates a future per node
// per tick. Since none of this recursion actually awaits anything except
// `process_operation` (which only awaits the logging channel), the walk could
// be a synchronous function that returns the collected log messages for the
// caller to send - removing every `Box::pin` and every `.clone()` of `con` and
// `logging_tx` along the way.
// PERF: `SOP::Sequence`/`Alternative` call `child.get_state(&state, ..)` for
// each child, and `get_state` recurses over that child's entire subtree doing a
// state lookup and a `format!` per operation. Finding the active child of a
// sequence is therefore O(subtree) per level, i.e. O(n^2) over the tick.
// Suggested: compute every node's state once per tick in a single bottom-up
// pass and cache it in a `Vec<SOPState>` indexed by node, then have the walk
// read from that.
async fn process_sop_node_tick(
    sp_id: &str,
    mut state: State,
    sop: &SOP,
    con: crate::SPConnection,
    // logging_tx: mpsc::Sender<LogMsg>,
    log_target: &str,
) -> State {
    match sop {
        SOP::Operation(operation) => {
            state = running::process_operation::process_operation(
                &sp_id,
                state,
                operation,
                running::process_operation::OperationProcessingType::SOP,
                None,
                None,
                // logging_tx,
                log_target,
                // &mut terminated_operations
            )
            .await;
        }

        SOP::Sequence(sops) => {
            let active_child = sops
                .iter()
                .find(|child| child.get_state(&state, &log_target) != SOPState::Completed);

            if let Some(child) = active_child {
                state = Box::pin(process_sop_node_tick(
                    sp_id, state, child, con, log_target,
                ))
                .await;
            }
        }

        SOP::Parallel(sops) => {
            for child in sops {
                state = Box::pin(process_sop_node_tick(
                    sp_id,
                    state,
                    child,
                    con.clone(),
                    // logging_tx.clone(),
                    log_target,
                ))
                .await;
            }
        }

        SOP::Alternative(sops) => {
            let active_child = sops.iter().find(|child| {
                let child_state = child.get_state(&state, &log_target);
                child_state != SOPState::Initial && child_state != SOPState::Completed
            });

            // If a path is active, keep processing it
            if let Some(child) = active_child {
                state = Box::pin(process_sop_node_tick(
                    sp_id, state, child, con, log_target,
                ))
                .await;
            } else {
                // If no path is active, find the first one that can start
                if let Some(path_to_start) = sops
                    .iter()
                    .find(|child| can_sop_start(sp_id, child, &state, log_target))
                {
                    log::info!(target: log_target, "Found valid alternative path to start.");
                    state = Box::pin(process_sop_node_tick(
                        sp_id,
                        state,
                        path_to_start,
                        con,
                        // logging_tx,
                        log_target,
                    ))
                    .await;
                }
            }
        }
    }

    state
}

fn can_sop_start(sp_id: &str, sop: &SOP, state: &State, log_target: &str) -> bool {
    match sop {
        SOP::Operation(operation) => {
            let current_state = sop.get_state(&state, &log_target);
            current_state == SOPState::Initial && operation.eval(state, log_target)
        }
        SOP::Sequence(sops) => sops.first().map_or(false, |first| {
            can_sop_start(sp_id, first, state, log_target)
        }),
        SOP::Parallel(sops) => sops
            .iter()
            .all(|child| can_sop_start(sp_id, child, state, log_target)),
        SOP::Alternative(sops) => sops
            .iter()
            .any(|child| can_sop_start(sp_id, child, state, log_target)),
    }
}

// PERF: fine as-is (runs once per SOP activation, not per tick), but note it
// rebuilds every `Operation` - including cloning all its transition vectors -
// only to change the `name`. If `Operation` held its transitions behind an
// `Arc<[Transition]>`, uniquifying a large SOP would become almost free and
// would also make the per-tick `operation.clone()` calls in `process_operation`
// cheap.
pub fn uniquify_sop_operations(sop: SOP) -> SOP {
    match sop {
        SOP::Operation(op) => {
            let unique_id = nanoid::nanoid!(10, &NANOID_ALPHABET); // 64^10 unique ids
            let new_name = format!("op_{}_{}", op.name, unique_id);
            SOP::Operation(Box::new(Operation {
                name: new_name,
                ..*op
            }))
        }
        SOP::Sequence(sops) => {
            SOP::Sequence(sops.into_iter().map(uniquify_sop_operations).collect())
        }
        SOP::Parallel(sops) => {
            SOP::Parallel(sops.into_iter().map(uniquify_sop_operations).collect())
        }
        SOP::Alternative(sops) => {
            SOP::Alternative(sops.into_iter().map(uniquify_sop_operations).collect())
        }
    }
}
