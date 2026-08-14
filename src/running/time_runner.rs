use std::sync::Arc;

use crate::*;
use tokio::time::{Duration, interval};

static TICK_INTERVAL_MS: u64 = 100;

// PERF: the `set_state` call is *inside* the `for timer_id in 1..=number_of_timers`
// loop, so a system with 10 timers performs 10 separate diffs and 10 separate
// Redis round trips every 100 ms - 100 MSETs per second, almost all of them
// writing values that did not change. Suggested: accumulate all timers into one
// `new_state`, diff once, and write once outside the loop (and skip the write
// when the diff is empty).
// PERF/correctness: each iteration builds `new_state` from `state` (the tick's
// original snapshot) rather than from the previous iteration's result, so with
// the write moved out of the loop only the last timer's changes would survive -
// thread a single `new_state` through the loop instead.
// PERF: `elapsed_ms += TICK_INTERVAL_MS` assumes the tick never slips; since
// every tick also waits on a PING and an MGET, timers drift long under load.
// Storing the start `SystemTime` and computing `elapsed` from the wall clock is
// both more accurate and removes a state write per timer per tick.
// PERF: `request_state == ActionRequestState::Executing.to_string()` allocates
// a fresh `String` for the comparison on every timer on every tick; compare
// against a `&'static str` instead.
pub async fn time_interface_runner(
    sp_id: &str,
    connection_manager: &Arc<ConnectionManager>,
    number_of_timers: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = interval(Duration::from_millis(TICK_INTERVAL_MS));
    let log_target = format!("{}_timer_interface", sp_id);

    log::info!(target: &log_target,  "Online.");

    let mut keys: Vec<String> = vec![];
    for timer_id in 1..=number_of_timers {
        keys.push(format!("{}_timer_{}_request_trigger", sp_id, timer_id));
        keys.push(format!("{}_timer_{}_request_state", sp_id, timer_id));
        keys.push(format!("{}_timer_{}_command", sp_id, timer_id));
        keys.push(format!("{}_timer_{}_duration_ms", sp_id, timer_id));
        keys.push(format!("{}_timer_{}_elapsed_ms", sp_id, timer_id));
    }

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

        let mut new_state: State;

        for timer_id in 1..=number_of_timers {
            let mut request_trigger = state.get_bool_or_default_to_false(
                &format!("{}_timer_{}_request_trigger", sp_id, timer_id),
                &log_target,
            );

            let mut request_state = state.get_string_or_default_to_unknown(
                &format!("{}_timer_{}_request_state", sp_id, timer_id),
                &log_target,
            );

            let command = state.get_string_or_default_to_unknown(
                &format!("{}_timer_{}_command", sp_id, timer_id),
                &log_target,
            );

            let duration_ms = state.get_int_or_default_to_zero(
                &format!("{}_timer_{}_duration_ms", sp_id, timer_id),
                &log_target,
            );

            let mut elapsed_ms = state.get_int_or_default_to_zero(
                &format!("{}_timer_{}_elapsed_ms", sp_id, timer_id),
                &log_target,
            );

            if request_trigger {
                request_trigger = false;
                if request_state == ActionRequestState::Initial.to_string() {
                    match command.as_str() {
                        "sleep" => {
                            if duration_ms > 0 {
                                log::info!(target: &log_target, "Starting sleep timer {} for {} ms.", timer_id, duration_ms);
                                request_state = ActionRequestState::Executing.to_string();
                                elapsed_ms = 0;
                            } else {
                                log::error!(target: &log_target, "Invalid sleep duration: {}. Must be > 0.", duration_ms);
                                request_state = ActionRequestState::Failed.to_string();
                            }
                        }
                        _ => {
                            log::error!(target: &log_target, "Timer interface command '{}' is invalid.", command);
                            request_state = ActionRequestState::Failed.to_string();
                        }
                    }
                }
            }

            if request_state == ActionRequestState::Executing.to_string() {
                elapsed_ms += TICK_INTERVAL_MS as i64;

                if elapsed_ms >= duration_ms {
                    elapsed_ms = duration_ms;
                    request_state = ActionRequestState::Succeeded.to_string();
                    log::info!(target: &log_target, "Sleep timer {} finished.", timer_id);
                }
            }

            new_state = state
                .update(
                    &format!("{}_timer_{}_request_trigger", sp_id, timer_id),
                    request_trigger.to_spvalue(),
                )
                .update(
                    &format!("{}_timer_{}_request_state", sp_id, timer_id),
                    request_state.to_spvalue(),
                )
                .update(
                    &format!("{}_timer_{}_elapsed_ms", sp_id, timer_id),
                    elapsed_ms.to_spvalue(),
                );
            let modified_state = state.get_diff_partial_state(&new_state);
            StateManager::set_state(&mut con, &modified_state).await;
        }
    }
}
