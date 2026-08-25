//! All of the runners in one loop, on one snapshot, with one writer.
//!
//! [`crate::sequential_runner`] does what the eight separate runner tasks do, but one
//! after another against a single `MGET` and publishing a single diff. That is
//! not a performance trick - it is what makes the read-modify-write of a tick
//! atomic with respect to the other runners.
//!
//! Spawned as eight tasks, each runner reads its own snapshot, decides from it,
//! and writes its own diff back. Nothing in Redis records that a write was
//! computed from a particular read, so two runners whose ticks overlap both
//! decide from the same stale values and the later write wins - silently, with
//! no error anywhere. `get_diff_partial_state` limits the damage to keys two
//! runners both write, but that set is the whole model domain: the three
//! operation runners all read and write `model_variable_keys`, and the
//! `_plan*` / `_planner_state` / `_current_goal_state` family is driven by three
//! runners together.
//!
//! One loop removes the overlap by construction. There is no second reader to
//! be stale and no second writer to lose to, so no compare-and-set, no version
//! numbers and no retry policy are needed for any of it.
//!
//! What it does *not* fix is the writers this loop does not contain: a device
//! driver in its own process, a dashboard, another `micro_sp`, or anything else
//! holding a connection to the same Redis. Those still race, and the handoffs
//! that cross that boundary are made atomic individually - see
//! [`StateManager::take_sp_value`](crate::StateManager::take_sp_value).
//!
//! # Ordering
//!
//! The bodies run in data-flow order, so a request crosses the whole system in
//! one tick rather than in eight: timers, transforms, reactive transitions,
//! automatic operations, plan execution, SOPs, goal admission, planning. Running
//! them in a fixed order also makes the emergent timing deterministic, which the
//! eight-task arrangement never was.
//!
//! # The planner
//!
//! `bfs_operation_planner` runs on a blocking thread with a five second
//! deadline, so it cannot run inline. It does not need to: it works from a
//! snapshot clone and reads nothing afterwards. The loop starts it, keeps the
//! handle in [`crate::PlannerCtx`], and folds the result in on whichever later tick it
//! is ready - while the other seven bodies keep ticking at the normal period.
//! Because nothing is published until the search returns, the request stays
//! pending in the state exactly as it does while the standalone planner task is
//! parked inside the search.

use crate::{
    running::goal_runner::{goal_runner_keys, goal_tick},
    transforms::interface::{tf_interface_keys, tf_tick},
    *,
};
use std::sync::Arc;

/// Runs every runner sequentially in one loop until the process ends.
///
/// Reads one snapshot of the union of every runner's keys, threads it through
/// each tick body in turn so each sees what the ones before it did, and
/// publishes one diff. `number_of_timers` matches
/// [`time_interface_runner`]; log output goes to
/// the `{sp_id}_sequential_runner` target, and each body still logs its own
/// activity under its own target.
///
/// A panic in any body takes the process down rather than being swallowed. That
/// is deliberate: as eight tasks, a panicking runner dies on its own and nothing
/// notices, leaving the system running in a degraded state no one detects.
pub async fn sequential_runner(
    sp_id: &str,
    model: &Model,
    connection_manager: &Arc<ConnectionManager>,
    number_of_timers: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    initialize_env_logger();
    activity_log::init_from_env();

    let mut interval = runner_interval();
    let log_target = format!("{}_sequential_runner", sp_id);
    log::info!(target: &log_target, "Online.");

    let mut con = connection_manager.get_connection().await;

    let mut plan_ctx = PlanRunnerCtx::new(sp_id, model);
    let mut sop_ctx = SopRunnerCtx::new(sp_id, model);
    let mut auto_ctx = AutoOperationCtx::new(sp_id, model);
    let mut planner_ctx = PlannerCtx::new(sp_id, model);
    let mut goal_info_old = String::new();

    // The key sets that never change. The three that do - the plan runner's,
    // the SOP runner's and the automatic operation runner's - live in their
    // contexts and are folded in below whenever they are rebuilt.
    let fixed_keys: Vec<String> = [
        planner_ctx.keys.clone(),
        goal_runner_keys(sp_id),
        time_runner_keys(sp_id, number_of_timers),
        tf_interface_keys(sp_id),
        auto_transition_runner_keys(model),
    ]
    .concat();

    let mut keys = union_keys(&fixed_keys, &plan_ctx, &sop_ctx, &auto_ctx);

    let read_full_state = read_full_state_enabled();
    if read_full_state {
        log::warn!(target: &log_target, "MICRO_SP_READ_FULL_STATE is set: reading the whole keyspace every tick.");
    }

    let tf_trigger_key = format!("{}_tf_request_trigger", sp_id);

    // Real time between ticks. Every body that ages a counter is handed the same
    // measurement, so one slow tick moves them all by the same amount.
    let mut tick_clock = TickClock::new();

    loop {
        interval.tick().await;
        let tick_elapsed_ms = tick_clock.elapsed_ms();

        let read = match read_full_state {
            true => StateManager::get_full_state(&mut con).await,
            false => StateManager::get_state_for_keys(&mut con, &keys, &log_target).await,
        };
        let state = match read {
            Some(s) => s,
            None => continue,
        };

        // Attributing each change to the runner that made it costs a clone per
        // body, so it is only done when the activity log is actually on.
        let attribute = activity_log::is_enabled();
        let mut working = state.clone();

        // 1. Timers, so everything downstream sees the current elapsed values.
        let before = attribute.then(|| working.clone());
        working = time_tick(
            sp_id,
            &working,
            number_of_timers,
            tick_elapsed_ms,
            &format!("{}_timer_interface", sp_id),
        );
        log_body_diff(attribute, before, &working, &format!("{}_timer_interface", sp_id));

        // 2. Transforms. `tf_tick` answers directly into the transform keyspace,
        //    so it publishes its own reply rather than returning one; the
        //    snapshot decides whether there is a request at all.
        if matches!(
            working.get_value(&tf_trigger_key, &log_target),
            Some(SPValue::Bool(BoolOrUnknown::Bool(true)))
        ) {
            tf_tick(
                sp_id,
                &mut con,
                &working,
                &format!("{}_tf_interface", sp_id),
            )
            .await;
        }

        // 3. Reactive transitions, before anything that reads what they rewrite.
        let target = format!("{}_auto_transition_runner", sp_id);
        let before = attribute.then(|| working.clone());
        working = auto_transition_tick(model, &working, &target);
        log_body_diff(attribute, before, &working, &target);

        // 4. Automatic operations.
        let target = format!("{}_operation_runner", sp_id);
        let before = attribute.then(|| working.clone());
        let (next, deletes) = auto_operation_tick(
            sp_id,
            model,
            &working,
            &mut auto_ctx,
            tick_elapsed_ms,
            &target,
        )
        .await;
        working = next;
        log_body_diff(attribute, before, &working, &target);

        // 5. The current plan.
        let target = format!("{}_op_runner", sp_id);
        let before = attribute.then(|| working.clone());
        // `plan_tick` tops its input up in place when a new plan brings
        // bookkeeping variables the snapshot does not have, and carries them
        // into what it returns - so the input has to outlive the call rather
        // than be a temporary.
        let mut plan_input = working;
        working = plan_tick(
            sp_id,
            &mut con,
            model,
            &mut plan_input,
            &mut plan_ctx,
            read_full_state,
            tick_elapsed_ms,
            &target,
        )
        .await;
        log_body_diff(attribute, before, &working, &target);

        // 6. SOPs.
        let target = format!("{}_sop_runner", sp_id);
        let before = attribute.then(|| working.clone());
        working = sop_tick(
            sp_id,
            &mut con,
            model,
            &working,
            &mut sop_ctx,
            tick_elapsed_ms,
            &target,
        )
        .await;
        log_body_diff(attribute, before, &working, &target);

        // 7. Goal admission, which is what raises a replan request...
        let target = format!("{}_goal_runner", sp_id);
        let before = attribute.then(|| working.clone());
        working = goal_tick(sp_id, &mut con, &working, &mut goal_info_old, &target).await;
        log_body_diff(attribute, before, &working, &target);

        // 8. ...and the planner, which serves it in the same tick.
        let target = format!("{}_planner", sp_id);
        let before = attribute.then(|| working.clone());
        working = planner_tick(sp_id, &working, &mut planner_ctx, &target).await;
        log_body_diff(attribute, before, &working, &target);

        // One publish for the whole tick. `_and_add_missing` rather than the
        // plain diff because the operation runners and the planner create
        // bookkeeping variables that are not in the snapshot at all, and the
        // plain diff drops those.
        let modified_state = state.get_diff_partial_state_and_add_missing(&working);
        StateManager::apply(&mut con, &modified_state, &[&deletes]).await;

        // A body whose active set changed needs different keys from the next
        // tick on, so the union follows it.
        if plan_ctx.keys_changed || sop_ctx.keys_changed || auto_ctx.keys_changed {
            keys = union_keys(&fixed_keys, &plan_ctx, &sop_ctx, &auto_ctx);
        }
    }
}

/// Every key any body needs this tick, deduplicated for the `MGET`.
fn union_keys(
    fixed: &[String],
    plan_ctx: &PlanRunnerCtx,
    sop_ctx: &SopRunnerCtx,
    auto_ctx: &AutoOperationCtx,
) -> Vec<String> {
    let mut keys = fixed.to_vec();
    keys.extend_from_slice(&plan_ctx.keys);
    keys.extend_from_slice(&sop_ctx.keys);
    keys.extend_from_slice(&auto_ctx.keys);
    normalize_keys(keys)
}

/// Record what one body changed, under that body's own log target, so the
/// activity log reads the same as it does with the runners spawned separately.
fn log_body_diff(attribute: bool, before: Option<State>, after: &State, log_target: &str) {
    if !attribute {
        return;
    }
    let Some(before) = before else {
        return;
    };
    let diff = before.get_diff_partial_state_and_add_missing(after);
    if !diff.state.is_empty() {
        activity_log::log_state_diff(log_target, &before, &diff);
    }
}
