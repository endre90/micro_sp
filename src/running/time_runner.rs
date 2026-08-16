use std::sync::Arc;

use crate::*;


/// Also the resolution of every sleep timer. Override with
/// `MICRO_SP_TIMER_TICK_MS`. See `running::tick`.
static TICK_INTERVAL_MS: u64 = 1;

// DONE: PERF: the `set_state` call used to be *inside* the
// `for timer_id in 1..=number_of_timers` loop, so a diff and a Redis round trip
// went out per timer per tick. Measured with three timers running: 153 MSETs
// per 5 seconds against 50 MGETs - three writes on every tick. All timers now
// accumulate into one `new_state`, which is diffed once and written once, so a
// tick costs at most one MSET no matter how many timers there are (and none at
// all when nothing changed).
// Note the ordering hazard this had to fix: each iteration used to build
// `new_state` from `state`, the tick's original snapshot, so simply hoisting
// the write would have kept only the last timer's changes. A single
// `new_state` is threaded through the loop instead.
// DONE: PERF: `request_state == ActionRequestState::Executing.to_string()`
// allocated a fresh `String` for the comparison on every timer on every tick.
//
// DONE (correctness): `elapsed_ms += TICK_INTERVAL_MS` charged a compile-time
// constant per tick, so a sleep timer only kept real time while the loop
// happened to run at exactly that period. It made the tick period and the
// timer's notion of a millisecond the same number - which is fine until the
// period changes. With the period now configurable (`MICRO_SP_TIMER_TICK_MS`)
// this was actively dangerous: at a 1 ms tick every tick still charged 100 ms,
// so a 60 second sleep finished in 600 milliseconds. The loop measures the real
// time its tick took and advances by that instead.
//
// PERF (still open): the counter is written for every executing timer on every
// tick. Deriving it from a stored start `SystemTime` would remove that write,
// at the cost of `_timer_N_elapsed_ms` no longer being readable as live
// progress - left alone deliberately.
pub async fn time_interface_runner(
    sp_id: &str,
    connection_manager: &Arc<ConnectionManager>,
    number_of_timers: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = runner_interval("MICRO_SP_TIMER_TICK_MS", TICK_INTERVAL_MS);
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

    // Real time between ticks. Timers count in milliseconds of wall clock, not
    // in ticks.
    let mut tick_clock = TickClock::new();

    loop {
        interval.tick().await;
        let tick_elapsed_ms = tick_clock.elapsed_ms();
        let state = match StateManager::get_state_for_keys(&mut con, &keys, &log_target).await {
            Some(s) => s,
            None => continue,
        };

        // One accumulator for every timer, diffed and written once below.
        let mut new_state = state.clone();

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
                if matches!(ActionRequestState::from_str(&request_state), ActionRequestState::Initial) {
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

            if matches!(ActionRequestState::from_str(&request_state), ActionRequestState::Executing) {
                elapsed_ms += tick_elapsed_ms;

                if elapsed_ms >= duration_ms {
                    elapsed_ms = duration_ms;
                    request_state = ActionRequestState::Succeeded.to_string();
                    log::info!(target: &log_target, "Sleep timer {} finished.", timer_id);
                }
            }

            new_state.update_mut(
                &format!("{}_timer_{}_request_trigger", sp_id, timer_id),
                request_trigger.to_spvalue(),
            );
            new_state.update_mut(
                &format!("{}_timer_{}_request_state", sp_id, timer_id),
                request_state.to_spvalue(),
            );
            new_state.update_mut(
                &format!("{}_timer_{}_elapsed_ms", sp_id, timer_id),
                elapsed_ms.to_spvalue(),
            );
        }

        let modified_state = state.get_diff_partial_state(&new_state);
        if !modified_state.state.is_empty() {
            StateManager::set_state(&mut con, &modified_state).await;
        }
    }
}
