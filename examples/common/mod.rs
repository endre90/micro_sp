//! Shared support code for the `micro_sp` examples.
//!
//! Every example needs the same four things: a Redis connection, a seeded
//! state, emulated hardware to act on, and a way to wait for the system to
//! settle before printing what happened. They live here so each example file
//! is just its model plus its scenario.
//!
//! Nothing in this module is part of the `micro_sp` API - it is example
//! scaffolding, and a real deployment would replace all of it.

// Each example uses a different slice of this module; the rest is dead code
// from that example's point of view.
#![allow(dead_code)]

pub mod emulators;
pub mod state;

use micro_sp::*;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Tick period of the hardware emulators, in milliseconds.
///
/// Deliberately much slower than the runners' 5 ms tick: emulated hardware
/// should be the slow part, the way real hardware is.
pub const EMULATOR_TICK_INTERVAL: u64 = 100;

// How long an emulated action takes. Written to `{resource}_emulate_execution_time`.
/// The action completes on the emulator's next tick.
pub const DONT_EMULATE_EXECUTION_TIME: i64 = 0;
/// The action always takes `{resource}_emulated_execution_time` milliseconds.
pub const EMULATE_EXACT_EXECUTION_TIME: i64 = 1;
/// The action takes a random time between 0 and `{resource}_emulated_execution_time`.
pub const EMULATE_RANDOM_EXECUTION_TIME: i64 = 2;

// Whether an emulated action fails. Written to `{resource}_emulate_failure_rate`.
/// The action always succeeds.
pub const DONT_EMULATE_FAILURE: i64 = 0;
/// The action always fails.
pub const EMULATE_FAILURE_ALWAYS: i64 = 1;
/// The action fails with a `{resource}_emulated_failure_rate` percent chance.
pub const EMULATE_RANDOM_FAILURE_RATE: i64 = 2;

// Why a failing action failed. Written to `{resource}_emulate_failure_cause`.
/// Fails with a generic cause.
pub const DONT_EMULATE_FAILURE_CAUSE: i64 = 0;
/// Fails with the first cause in `{resource}_emulated_failure_cause`.
pub const EMULATE_EXACT_FAILURE_CAUSE: i64 = 1;
/// Fails with a random cause from `{resource}_emulated_failure_cause`.
pub const EMULATE_RANDOM_FAILURE_CAUSE: i64 = 2;

/// The namespace every runner key is prefixed with, in all examples.
pub const SP_ID: &str = "sp";

/// Log target for the example scaffolding itself.
pub const TARGET: &str = "example";

/// The state a model is built *against*: the shared domain plus the runners'
/// own variables.
///
/// [`Transition::parse`] resolves every `var:` name against the state it is
/// given and panics on one that is not there. A model that drives a SOP writes
/// `var:{sp_id}_sop_enabled`, and a model that posts its own goals writes
/// `var:{sp_id}_incoming_goals` - runner variables - so those have to exist
/// before `model()` is called, not merely before `main_runner` is.
pub fn base_state(sp_id: &str, number_of_timers: u64) -> State {
    let mut state = state::state();
    state.extend_mut(
        generate_runner_state_variables(sp_id, number_of_timers, TARGET),
        true,
    );
    state
}

/// Connect to Redis, seed everything the runners need, and hand back the pool.
///
/// The runners coordinate purely through named keys and *panic* on reading a
/// key that does not exist, so the whole key set has to be in Redis before
/// `main_runner` is called. That is what this does:
///
/// 1. connect (see [`connect`]);
/// 2. flush, so repeat runs of an example start from the same place;
/// 3. [`generate_runner_state_variables`] - the runners' own bookkeeping;
/// 4. [`generate_operation_state_variables`] - one lifecycle variable per
///    operation, including the ones nested inside SOPs;
/// 5. the model's own domain variables, as returned by its `model()`;
/// 6. the handful of keys the runners expect to be *initialised* rather than
///    merely present.
pub async fn boot(
    sp_id: &str,
    model: &Model,
    domain: State,
    number_of_timers: u64,
) -> Arc<ConnectionManager> {
    let connection_manager = connect().await;
    let mut con = connection_manager.get_connection().await;

    // A previous run's state would otherwise be read as this run's starting
    // point - including any operation left mid-lifecycle.
    StateManager::flush_state(&mut con).await;

    let mut state = generate_runner_state_variables(sp_id, number_of_timers, TARGET);
    state.extend_mut(
        generate_operation_state_variables(model, false, TARGET),
        true,
    );
    state.extend_mut(domain, true);

    state.add_mut(
        SPAssignment::new(
            SPVariable::new(&format!("{sp_id}_dashboard_command"), SPValueType::String),
            "none".to_spvalue(),
        ),
        TARGET,
    );

    // Present is not enough for these four - the runners compare them against
    // concrete values on the very first tick.
    state = state.update(
        &format!("{sp_id}_terminated_operations"),
        Vec::<SPValue>::new().to_spvalue(),
    );
    state = state.update(&format!("{sp_id}_current_goal_state"), "initial".to_spvalue());
    state = state.update(&format!("{sp_id}_plan_state"), "initial".to_spvalue());
    state = state.update(&format!("{sp_id}_planner_state"), "ready".to_spvalue());

    StateManager::set_state(&mut con, &state).await;

    connection_manager
}

/// Connect to Redis, failing fast with a usable message instead of hanging.
///
/// [`ConnectionManager::new`] retries forever, which is right for a deployment
/// and wrong for an example - a user with no Redis running would just watch it
/// spin. So probe the socket first and say what to do about it.
pub async fn connect() -> Arc<ConnectionManager> {
    let host = std::env::var("REDIS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("REDIS_PORT").unwrap_or_else(|_| "6379".to_string());

    if tokio::net::TcpStream::connect(format!("{host}:{port}"))
        .await
        .is_err()
    {
        eprintln!("Cannot reach Redis at {host}:{port}.");
        eprintln!();
        eprintln!("Start one first:");
        eprintln!("    docker compose up -d");
        eprintln!("or:");
        eprintln!("    docker run --name my-redis -p 6379:6379 -d redis");
        std::process::exit(1);
    }

    Arc::new(ConnectionManager::new().await)
}

/// Poll `key` until it holds `expected`, or the deadline passes.
///
/// Returns the last value seen, so a caller that timed out can report what the
/// system was actually doing rather than only that it did not finish.
pub async fn wait_for(
    connection_manager: &Arc<ConnectionManager>,
    key: &str,
    expected: SPValue,
    timeout_ms: u64,
) -> SPValue {
    let mut con = connection_manager.get_connection().await;
    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    let mut last = SPValue::String(StringOrUnknown::UNKNOWN);

    while Instant::now() < deadline {
        if let Some(value) = StateManager::get_sp_value(&mut con, key).await {
            last = value;
            if last == expected {
                return last;
            }
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    last
}

/// Like [`wait_for`], but for a condition over the whole state rather than one
/// key. Returns `true` if the condition held before the deadline.
pub async fn wait_until(
    connection_manager: &Arc<ConnectionManager>,
    keys: &[&str],
    timeout_ms: u64,
    condition: impl Fn(&State) -> bool,
) -> bool {
    let mut con = connection_manager.get_connection().await;
    let owned: Vec<String> = keys.iter().map(|k| k.to_string()).collect();
    let deadline = Instant::now() + Duration::from_millis(timeout_ms);

    while Instant::now() < deadline {
        if let Some(state) = StateManager::get_state_for_keys(&mut con, &owned, TARGET).await {
            if condition(&state) {
                return true;
            }
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    false
}

/// Print the named variables as they currently stand in Redis.
pub async fn print_state(
    connection_manager: &Arc<ConnectionManager>,
    heading: &str,
    keys: &[&str],
) {
    let mut con = connection_manager.get_connection().await;
    let owned: Vec<String> = keys.iter().map(|k| k.to_string()).collect();

    println!();
    println!("{heading}");
    println!("{}", "-".repeat(heading.len()));

    match StateManager::get_state_for_keys(&mut con, &owned, TARGET).await {
        Some(state) => {
            let width = keys.iter().map(|k| k.len()).max().unwrap_or(0);
            for key in keys {
                let value = state
                    .get_value(key, TARGET)
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "<absent>".to_string());
                println!("  {key:<width$}  {value}");
            }
        }
        None => println!("  <could not read state>"),
    }
    println!();
}

/// Spawn the robot emulator as a detached task.
pub fn spawn_robot(connection_manager: &Arc<ConnectionManager>) -> tokio::task::JoinHandle<()> {
    let con = connection_manager.clone();
    tokio::task::spawn(async move {
        if let Err(e) = emulators::robot::robot_emulator(&con).await {
            log::error!(target: TARGET, "Robot emulator stopped: {e}");
        }
    })
}

/// Spawn the gantry emulator as a detached task.
pub fn spawn_gantry(connection_manager: &Arc<ConnectionManager>) -> tokio::task::JoinHandle<()> {
    let con = connection_manager.clone();
    tokio::task::spawn(async move {
        if let Err(e) = emulators::gantry::gantry_emulator(&con).await {
            log::error!(target: TARGET, "Gantry emulator stopped: {e}");
        }
    })
}

/// Set the emulation knobs for one resource in a single write.
///
/// `execution_time_ms` of `None` means "complete on the next tick";
/// `fail` selects one of [`DONT_EMULATE_FAILURE`], [`EMULATE_FAILURE_ALWAYS`]
/// or [`EMULATE_RANDOM_FAILURE_RATE`].
pub async fn configure_emulator(
    connection_manager: &Arc<ConnectionManager>,
    resource: &str,
    execution_time_ms: Option<i64>,
    fail: i64,
) {
    let mut con = connection_manager.get_connection().await;
    let (mode, ms) = match execution_time_ms {
        Some(ms) => (EMULATE_EXACT_EXECUTION_TIME, ms),
        None => (DONT_EMULATE_EXECUTION_TIME, 0),
    };

    for (key, value) in [
        (format!("{resource}_emulate_execution_time"), mode.to_spvalue()),
        (format!("{resource}_emulated_execution_time"), ms.to_spvalue()),
        (format!("{resource}_emulate_failure_rate"), fail.to_spvalue()),
    ] {
        StateManager::set_sp_value(&mut con, &key, &value).await;
    }
}

/// End an example: report success or failure and exit with the matching code.
///
/// Examples are meant to be runnable in a loop by CI as well as read, so a
/// scenario that did not reach its terminal condition has to be an exit code,
/// not just a line of text.
pub fn finish(reached: bool, what: &str) -> ! {
    if reached {
        println!("OK: {what}");
        std::process::exit(0)
    } else {
        eprintln!("TIMED OUT: {what}");
        std::process::exit(1)
    }
}
