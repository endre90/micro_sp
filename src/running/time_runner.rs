use std::sync::Arc;

use crate::*;


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
// period changes. With the period now configurable
// (`MICRO_SP_TICK_INTERVAL_MS`) this was actively dangerous: at a 1 ms tick
// every tick still charged 100 ms, so a 60 second sleep finished in 600
// milliseconds. The loop measures the real time its tick took and advances by
// that instead.
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
    let mut interval = runner_interval();
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
            activity_log::log_state_diff(&log_target, &state, &modified_state);
            StateManager::set_state(&mut con, &modified_state).await;
        }
    }
}

/// The timer interface, driven end to end against a real Redis.
///
/// Everything below the `interval.tick()` is a pure function of the state, but
/// the thing worth testing is the loop as a whole: a caller sets a trigger in
/// Redis, waits, and expects the request state to have moved. That is the
/// contract every consumer of this crate uses, and it is also where the
/// tick-period/elapsed-time coupling shows up - a sleep has to take the number
/// of milliseconds it was asked for, whatever the tick period happens to be.
#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::time::Duration;
    use testcontainers::{ContainerAsync, ImageExt, core::ContainerPort, runners::AsyncRunner};
    use testcontainers_modules::redis::Redis;

    const SP: &str = "sp";
    const TARGET: &str = "test";
    const TIMERS: u64 = 2;

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

    fn key(timer: u64, suffix: &str) -> String {
        format!("{SP}_timer_{timer}_{suffix}")
    }

    /// Every timer the runner was told about has to be seeded, not just the one
    /// a test drives. `State::get_value` *panics* on a key that is not in the
    /// state, and every `get_*_or_default_to_*` accessor funnels through it -
    /// the "or default" only covers a value of the wrong type, never a missing
    /// key. A runner told to drive N timers reads all N on every tick, so one
    /// un-seeded timer takes the whole runner task down on its first tick.
    async fn seed_all_timers(con: &mut SPConnection, command: &str, duration_ms: i64) {
        for timer in 1..=TIMERS {
            seed_timer(con, timer, command, duration_ms).await;
        }
    }

    /// The variables a timer needs to exist before the runner can drive it.
    async fn seed_timer(con: &mut SPConnection, timer: u64, command: &str, duration_ms: i64) {
        let mut state = State::new();
        state.add_mut(
            SPAssignment::new(
                SPVariable::new(&key(timer, "request_trigger"), SPValueType::Bool),
                false.to_spvalue(),
            ),
            TARGET,
        );
        state.add_mut(
            SPAssignment::new(
                SPVariable::new(&key(timer, "request_state"), SPValueType::String),
                ActionRequestState::Initial.to_string().to_spvalue(),
            ),
            TARGET,
        );
        state.add_mut(
            SPAssignment::new(
                SPVariable::new(&key(timer, "command"), SPValueType::String),
                command.to_spvalue(),
            ),
            TARGET,
        );
        state.add_mut(
            SPAssignment::new(
                SPVariable::new(&key(timer, "duration_ms"), SPValueType::Int64),
                duration_ms.to_spvalue(),
            ),
            TARGET,
        );
        state.add_mut(
            SPAssignment::new(
                SPVariable::new(&key(timer, "elapsed_ms"), SPValueType::Int64),
                0.to_spvalue(),
            ),
            TARGET,
        );
        StateManager::set_state(con, &state).await;
    }

    /// Poll a key until it holds `expected`, or give up. Returns what it saw.
    async fn wait_for(
        con: &mut SPConnection,
        key: &str,
        expected: &str,
        timeout_ms: u64,
    ) -> String {
        let deadline = std::time::Instant::now() + Duration::from_millis(timeout_ms);
        let mut last = String::new();
        while std::time::Instant::now() < deadline {
            last = match StateManager::get_sp_value(con, key).await {
                Some(SPValue::String(StringOrUnknown::String(s))) => s,
                other => format!("{other:?}"),
            };
            if last == expected {
                return last;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        last
    }

    fn spawn_runner(manager: &Arc<ConnectionManager>) -> tokio::task::JoinHandle<()> {
        let manager = Arc::clone(manager);
        tokio::spawn(async move {
            let _ = time_interface_runner(SP, &manager, TIMERS).await;
        })
    }

    /// The happy path: trigger a sleep, and it reports `succeeded` once the
    /// requested duration has actually passed - not before.
    #[tokio::test]
    #[serial]
    async fn a_sleep_timer_runs_for_the_duration_it_was_asked_for() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;
        seed_all_timers(&mut con, "sleep", 200).await;

        let runner = spawn_runner(&manager);

        let started = std::time::Instant::now();
        StateManager::set_sp_value(&mut con, &key(1, "request_trigger"), &true.to_spvalue()).await;

        let state = wait_for(&mut con, &key(1, "request_state"), "succeeded", 3000).await;
        let took = started.elapsed();
        runner.abort();

        assert_eq!(state, "succeeded");
        assert!(
            took >= Duration::from_millis(190),
            "a 200 ms sleep finished after only {took:?}"
        );
        assert!(
            took < Duration::from_millis(1500),
            "a 200 ms sleep took {took:?}"
        );

        // The elapsed counter is clamped to the duration rather than
        // overshooting it, so a consumer can compare the two for equality.
        let elapsed = StateManager::get_sp_value(&mut con, &key(1, "elapsed_ms")).await;
        assert_eq!(elapsed, Some(200.to_spvalue()));
    }

    /// The trigger is consumed by the runner. A caller that sets it and walks
    /// away must not see the timer restart on the next tick.
    #[tokio::test]
    #[serial]
    async fn the_trigger_is_cleared_when_the_request_is_taken() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;
        seed_all_timers(&mut con, "sleep", 100).await;

        let runner = spawn_runner(&manager);
        StateManager::set_sp_value(&mut con, &key(1, "request_trigger"), &true.to_spvalue()).await;

        let deadline = std::time::Instant::now() + Duration::from_millis(2000);
        let mut cleared = false;
        while std::time::Instant::now() < deadline {
            if StateManager::get_sp_value(&mut con, &key(1, "request_trigger")).await
                == Some(false.to_spvalue())
            {
                cleared = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        runner.abort();

        assert!(cleared, "the runner must clear the trigger it consumed");
    }

    /// A sleep of zero (or a negative duration) is a caller error, and the
    /// runner has to say so rather than succeed instantly or hang.
    #[tokio::test]
    #[serial]
    async fn a_non_positive_duration_fails_the_request() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;
        seed_timer(&mut con, 1, "sleep", 0).await;
        seed_timer(&mut con, 2, "sleep", -5).await;

        let runner = spawn_runner(&manager);
        StateManager::set_sp_value(&mut con, &key(1, "request_trigger"), &true.to_spvalue()).await;
        StateManager::set_sp_value(&mut con, &key(2, "request_trigger"), &true.to_spvalue()).await;

        let first = wait_for(&mut con, &key(1, "request_state"), "failed", 2000).await;
        let second = wait_for(&mut con, &key(2, "request_state"), "failed", 2000).await;
        runner.abort();

        assert_eq!(first, "failed", "a 0 ms sleep is not a valid request");
        assert_eq!(second, "failed", "a negative sleep is not a valid request");
    }

    #[tokio::test]
    #[serial]
    async fn an_unknown_command_fails_the_request() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;
        seed_all_timers(&mut con, "teleport", 100).await;

        let runner = spawn_runner(&manager);
        StateManager::set_sp_value(&mut con, &key(1, "request_trigger"), &true.to_spvalue()).await;

        let state = wait_for(&mut con, &key(1, "request_state"), "failed", 2000).await;
        runner.abort();

        assert_eq!(state, "failed");
    }

    /// A trigger arriving while the request is not `initial` - a second trigger
    /// on a timer that is already running - is swallowed: the trigger is
    /// cleared but the running sleep is not restarted.
    #[tokio::test]
    #[serial]
    async fn a_second_trigger_does_not_restart_a_running_sleep() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;
        seed_all_timers(&mut con, "sleep", 400).await;

        let runner = spawn_runner(&manager);
        let started = std::time::Instant::now();
        StateManager::set_sp_value(&mut con, &key(1, "request_trigger"), &true.to_spvalue()).await;
        assert_eq!(
            wait_for(&mut con, &key(1, "request_state"), "executing", 2000).await,
            "executing"
        );

        // Half way through, trigger it again.
        tokio::time::sleep(Duration::from_millis(200)).await;
        StateManager::set_sp_value(&mut con, &key(1, "request_trigger"), &true.to_spvalue()).await;

        let state = wait_for(&mut con, &key(1, "request_state"), "succeeded", 3000).await;
        let took = started.elapsed();
        runner.abort();

        assert_eq!(state, "succeeded");
        assert!(
            took < Duration::from_millis(1200),
            "the second trigger restarted the sleep - it took {took:?} instead of ~400 ms"
        );
    }

    /// Two timers run independently, and the runner writes both in one tick.
    #[tokio::test]
    #[serial]
    async fn timers_run_independently_of_each_other() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;
        seed_timer(&mut con, 1, "sleep", 100).await;
        seed_timer(&mut con, 2, "sleep", 600).await;


        let runner = spawn_runner(&manager);
        StateManager::set_sp_value(&mut con, &key(1, "request_trigger"), &true.to_spvalue()).await;
        StateManager::set_sp_value(&mut con, &key(2, "request_trigger"), &true.to_spvalue()).await;

        assert_eq!(
            wait_for(&mut con, &key(1, "request_state"), "succeeded", 2000).await,
            "succeeded"
        );

        // The long one must still be running when the short one is done.
        let long_state = StateManager::get_sp_value(&mut con, &key(2, "request_state")).await;
        assert_eq!(
            long_state,
            Some("executing".to_spvalue()),
            "the 600 ms timer should not be finished when the 100 ms one is"
        );

        assert_eq!(
            wait_for(&mut con, &key(2, "request_state"), "succeeded", 3000).await,
            "succeeded"
        );
        runner.abort();
    }

    /// An idle runner - no triggers, nothing executing - must not write on
    /// every tick. At a 5 ms period that would be 200 writes a second per
    /// process for a system doing nothing.
    #[tokio::test]
    #[serial]
    async fn an_idle_runner_writes_nothing() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;
        seed_all_timers(&mut con, "sleep", 100).await;

        let runner = spawn_runner(&manager);
        tokio::time::sleep(Duration::from_millis(100)).await;

        let before: i64 = redis::cmd("DBSIZE").query_async(&mut con).await.unwrap();
        let keys_before = StateManager::get_full_state(&mut con).await.unwrap();

        tokio::time::sleep(Duration::from_millis(300)).await;

        let after: i64 = redis::cmd("DBSIZE").query_async(&mut con).await.unwrap();
        let keys_after = StateManager::get_full_state(&mut con).await.unwrap();

        assert_eq!(before, after, "an idle runner must not add keys");
        assert!(
            keys_before.get_diff_partial_state(&keys_after).state.is_empty(),
            "an idle runner must not change any value"
        );

        // The runner has to still be *alive* for the above to mean anything -
        // a task that panicked on tick one would also change nothing.
        assert!(!runner.is_finished(), "the runner died instead of idling");
        StateManager::set_sp_value(&mut con, &key(1, "request_trigger"), &true.to_spvalue()).await;
        assert_eq!(
            wait_for(&mut con, &key(1, "request_state"), "succeeded", 2000).await,
            "succeeded",
            "the idle runner must still respond to a trigger"
        );
        runner.abort();
    }

    /// The sharp edge behind `seed_all_timers`, pinned on its own because it is
    /// a real deployment hazard rather than a test-setup detail: a variable the
    /// runner reads but that was never written to Redis is simply absent from
    /// the state `get_state_for_keys` builds, and reading it panics the runner
    /// task. It does *not* fall back to a default despite the accessor's name,
    /// and it does not surface as a logged error either - the task is just gone,
    /// silently, and that runner stops ticking for the life of the process.
    #[tokio::test]
    #[serial]
    async fn a_timer_that_was_never_initialised_kills_the_runner() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        // Timer 1 is seeded, timer 2 is not - and the runner drives both.
        seed_timer(&mut con, 1, "sleep", 100).await;

        let runner = spawn_runner(&manager);
        tokio::time::sleep(Duration::from_millis(200)).await;

        assert!(
            runner.is_finished(),
            "reading the un-seeded timer 2 should have taken the runner down"
        );
        let outcome = runner.await;
        assert!(
            outcome.is_err() && outcome.unwrap_err().is_panic(),
            "and it should have gone down by panicking, not by returning"
        );
    }

    /// Same accessor, isolated from Redis: this is the `State` behaviour every
    /// runner inherits.
    #[test]
    #[should_panic(expected = "not in state")]
    fn get_bool_or_default_to_false_panics_on_a_missing_key() {
        let state = State::new();
        let _ = state.get_bool_or_default_to_false("never_written", TARGET);
    }

    /// The "or default" really does apply to a value of the wrong *type*, which
    /// is the case the name was written for.
    #[test]
    fn get_bool_or_default_to_false_does_default_on_a_wrong_type() {
        let mut state = State::new();
        state.add_mut(
            SPAssignment::new(
                SPVariable::new("a_string", SPValueType::String),
                "not a bool".to_spvalue(),
            ),
            TARGET,
        );
        assert!(!state.get_bool_or_default_to_false("a_string", TARGET));
    }
}
