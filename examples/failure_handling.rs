//! What an operation does when it goes wrong: retry, bypass, fatal, timeout.
//!
//! Everything above the [`Operation`] - plans, SOPs - assumes its steps either
//! finish or stop. This is the layer where that gets decided, and it is
//! configured entirely by six fields on the operation itself:
//!
//! ```text
//!                           executing
//!                               │
//!             ┌─────────────────┴─────────────────┐
//!             │ a failure_transition              │ past
//!             │ holds                             │ timeout_executing_ms
//!             v                                   v
//!          failed                             timedout
//!             │                                   │
//!             └─────────────────┬─────────────────┘
//!                               │
//!     ┌─────────────────────────┼─────────────────────────┐
//!     │ a retry is left         │ out of retries,         │ out of retries,
//!     │ (failure_retries /      │ can_be_bypassed         │ otherwise
//!     │  timeout_retries)       │                         │
//!     v                         v                         v
//!  initial                  bypassed                    fatal
//!                               ┆                         ┆
//!                               v                         v
//!                      terminated_bypassed        terminated_fatal
//! ```
//!
//! The two dotted edges are the intent, not yet the behaviour:
//! `Operation::terminate` only implements `TerminationReason::Completed`, so
//! today a bypassed or fatal operation stays in `bypassed` / `fatal` instead of
//! reaching `terminated_*`. The three scenarios below never notice - each one
//! checks a flag its own bypass or timeout transition set, not the lifecycle
//! variable. A bypassable operation inside a SOP does notice: `SOP::get_state`
//! counts only `terminated_bypassed` as a finished step, so that branch would
//! report `executing` forever.
//!
//! Two precedences decide which arrow is taken when more than one could be. A
//! `stop` command beats everything, and a `failure_transition` beats a deadline
//! that came due on the same tick - so a modelled failure is never silently
//! turned into a timeout, and neither is ever silently completed.
//!
//! Three scenarios, run one after another against the same model. Each is an
//! automatic operation gated on a `scenario` variable, so `main` can step
//! through them without a planner in the way.
//!
//! 1. **Retry, then bypass.** `gantry_unlock` has `failure_retries: Some(2)`
//!    and `can_be_bypassed: true`, and the gantry emulator is told to fail
//!    every time. It fails, retries twice, runs out of retries - and because
//!    it may be bypassed, it is waved through instead of killing anything.
//!
//! 2. **Fail, then fatal.** `gantry_calibrate` is the same shape with no
//!    retries and `can_be_bypassed: false`. One failure and it is
//!    unrecoverable.
//!
//! 3. **Timeout, then fatal.** `slow_operation` asks timer 1 for a 3-second
//!    sleep but gives itself `timeout_executing_ms: Some(500)`. It cannot
//!    possibly finish in time, so it times out, spends its one
//!    `timeout_retries`, times out again and goes fatal.
//!
//!    Note *when* that second timeout lands. A retry puts the operation back to
//!    `initial` but does not reset `{name}_elapsed_executing_ms`, and that
//!    counter is already past 500 ms from the first attempt - so the retry
//!    times out on the tick after it restarts, not 500 ms later. The observed
//!    sequence is `executing -> timedout -> initial -> executing -> timedout ->
//!    fatal`. `timeout_retries` buys extra attempts, not extra time.
//!
//! The sharp edge worth reading twice: **a retry only ever happens if there is
//! a transition to fire.** `failure_retries` counts failures, and a failure is
//! a `failure_transitions` entry whose guard held. An operation with retries
//! and no failure transition never fails and so never retries - it just runs
//! until its timeout.
//!
//! Run with:
//!
//! ```text
//! docker compose up -d
//! cargo run --example failure_handling
//! ```

mod common;

use common::{DONT_EMULATE_FAILURE, EMULATE_FAILURE_ALWAYS, SP_ID, TARGET};
use micro_sp::*;

fn model(sp_id: &str, state: &State) -> (Model, State) {
    let state = state.clone();

    // Which scenario is armed. Every operation's precondition checks it, so
    // only one of them can be enabled at a time.
    let scenario = v!("scenario");
    let state = state.add(assign!(scenario, "none".to_spvalue()), TARGET);

    // Outcome flags, set by the off-nominal transitions themselves.
    let state = state.add(assign!(bv!("was_bypassed"), false.to_spvalue()), TARGET);
    let state = state.add(assign!(bv!("was_fatal"), false.to_spvalue()), TARGET);
    let state = state.add(assign!(bv!("was_timedout"), false.to_spvalue()), TARGET);
    let state = state.add(assign!(iv!("failure_count"), 0.to_spvalue()), TARGET);
    let state = state.add(assign!(iv!("timeout_count"), 0.to_spvalue()), TARGET);

    let auto_operations = vec![
        retry_then_bypass(&state),
        fail_then_fatal(&state),
        timeout_then_fatal(sp_id, &state),
    ];

    let model = Model::new(sp_id, vec![], auto_operations, vec![], vec![], vec![]);

    (model, state)
}

/// Scenario 1: two retries, then bypassed rather than fatal.
fn retry_then_bypass(state: &State) -> Operation {
    Operation::new(
        "gantry_unlock",
        Some(10_000), // timeout_executing_ms
        Some(10_000), // timeout_disabled_ms
        Some(2),      // failure_retries: two more goes after the first failure
        None,         // timeout_retries: none
        true,         // can_be_bypassed: exhausted retries are waved through
        vec![Transition::parse(
            "start_gantry_unlock",
            "var:scenario == retry_then_bypass \
             && var:gantry_request_state == initial \
             && var:gantry_request_trigger == false",
            "true",
            vec![
                "var:gantry_command_command <- unlock",
                "var:gantry_request_trigger <- true",
            ],
            Vec::<&str>::new(),
            state,
        )],
        vec![Transition::parse(
            "complete_gantry_unlock",
            "true",
            "var:gantry_request_state == succeeded",
            vec![
                "var:gantry_request_trigger <- false",
                "var:gantry_request_state <- initial",
                "var:gantry_locked_estimated <- false",
            ],
            Vec::<&str>::new(),
            state,
        )],
        // The failure transition. Its job is to put the *hardware* back where a
        // retry can use it - clearing the handshake - not to give up.
        vec![Transition::parse(
            "failed_gantry_unlock",
            "true",
            "var:gantry_request_state == failed",
            vec![
                "var:gantry_request_trigger <- false",
                "var:gantry_request_state <- initial",
                "var:failure_count += 1",
            ],
            Vec::<&str>::new(),
            state,
        )],
        vec![], // timeout_transitions: empty, so a timeout is unconditional
        // Bypass transition. Declared here, so the bypass is *conditional* on
        // its guard - an empty vec would make bypass unconditional instead.
        vec![Transition::parse(
            "bypass_gantry_unlock",
            "true",
            "true",
            vec![
                "var:gantry_request_trigger <- false",
                "var:gantry_request_state <- initial",
                "var:was_bypassed <- true",
            ],
            Vec::<&str>::new(),
            state,
        )],
        vec![], // cancel_transitions
    )
}

/// Scenario 2: no retries, no bypass - one failure is the end of it.
fn fail_then_fatal(state: &State) -> Operation {
    Operation::new(
        "gantry_calibrate",
        Some(10_000),
        Some(10_000),
        None,  // failure_retries: none
        None,  // timeout_retries: none
        false, // can_be_bypassed: so an exhausted operation goes fatal
        vec![Transition::parse(
            "start_gantry_calibrate",
            "var:scenario == fail_then_fatal \
             && var:gantry_request_state == initial \
             && var:gantry_request_trigger == false",
            "true",
            vec![
                "var:gantry_command_command <- calibrate",
                "var:gantry_request_trigger <- true",
            ],
            Vec::<&str>::new(),
            state,
        )],
        vec![Transition::parse(
            "complete_gantry_calibrate",
            "true",
            "var:gantry_request_state == succeeded",
            vec![
                "var:gantry_request_trigger <- false",
                "var:gantry_request_state <- initial",
                "var:gantry_calibrated_estimated <- true",
            ],
            Vec::<&str>::new(),
            state,
        )],
        vec![Transition::parse(
            "failed_gantry_calibrate",
            "true",
            "var:gantry_request_state == failed",
            vec![
                "var:gantry_request_trigger <- false",
                "var:gantry_request_state <- initial",
                "var:was_fatal <- true",
            ],
            Vec::<&str>::new(),
            state,
        )],
        vec![],
        vec![],
        vec![],
    )
}

/// Scenario 3: a deadline the operation cannot possibly meet.
///
/// Asks timer 1 for a 3-second sleep while allowing itself 500 ms. Nothing
/// fails here - the hardware is working perfectly, it is just too slow, which
/// is exactly what `timeout_executing_ms` is for.
fn timeout_then_fatal(sp_id: &str, state: &State) -> Operation {
    Operation::new(
        "slow_operation",
        Some(500),    // timeout_executing_ms: far less than the sleep takes
        Some(10_000), // timeout_disabled_ms
        None,         // failure_retries
        Some(1),      // timeout_retries: one more go, which also times out
        false,        // can_be_bypassed: so the second timeout is fatal
        vec![Transition::parse(
            "start_slow_operation",
            &format!(
                "var:scenario == timeout_then_fatal \
                 && var:{sp_id}_timer_1_request_state == initial \
                 && var:{sp_id}_timer_1_request_trigger == false"
            ),
            "true",
            vec![
                &format!("var:{sp_id}_timer_1_command <- sleep"),
                &format!("var:{sp_id}_timer_1_duration_ms <- 3000"),
                &format!("var:{sp_id}_timer_1_request_trigger <- true"),
            ],
            Vec::<&str>::new(),
            state,
        )],
        vec![Transition::parse(
            "complete_slow_operation",
            "true",
            &format!("var:{sp_id}_timer_1_request_state == succeeded"),
            vec![
                &format!("var:{sp_id}_timer_1_request_trigger <- false"),
                &format!("var:{sp_id}_timer_1_request_state <- initial"),
            ],
            Vec::<&str>::new(),
            state,
        )],
        vec![], // failure_transitions: nothing here can fail
        // Timeout transition: cancel the outstanding request so a retry starts
        // from a clean timer.
        vec![Transition::parse(
            "timeout_slow_operation",
            "true",
            "true",
            vec![
                &format!("var:{sp_id}_timer_1_request_trigger <- false"),
                &format!("var:{sp_id}_timer_1_request_state <- initial"),
                "var:was_timedout <- true",
                "var:timeout_count += 1",
            ],
            Vec::<&str>::new(),
            state,
        )],
        vec![], // bypass_transitions
        vec![], // cancel_transitions
    )
}

#[tokio::main]
async fn main() {
    let (model, domain) = model(SP_ID, &common::base_state(SP_ID, 1));
    let connection_manager = common::boot(SP_ID, &model, domain, 1).await;

    common::spawn_gantry(&connection_manager);

    main_runner(&SP_ID.to_string(), model, 1, &connection_manager).await;
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let mut ok = true;

    // ---------------------------------------------------- 1. retry, then bypass
    println!("\n[1/3] Gantry always fails. Two retries, then bypassed.");
    common::configure_emulator(&connection_manager, "gantry", Some(100), EMULATE_FAILURE_ALWAYS)
        .await;
    ok &= run_scenario(
        &connection_manager,
        "retry_then_bypass",
        &["was_bypassed", "failure_count"],
        |state| state.get_value("was_bypassed", TARGET) == Some(true.to_spvalue()),
    )
    .await;

    // ------------------------------------------------------ 2. fail, then fatal
    println!("\n[2/3] Gantry always fails. No retries, no bypass, so: fatal.");
    ok &= run_scenario(
        &connection_manager,
        "fail_then_fatal",
        &["was_fatal"],
        |state| state.get_value("was_fatal", TARGET) == Some(true.to_spvalue()),
    )
    .await;

    // --------------------------------------------------- 3. timeout, then fatal
    println!("\n[3/3] A 3s sleep with a 500ms deadline. One timeout retry, then fatal.");
    common::configure_emulator(&connection_manager, "gantry", Some(100), DONT_EMULATE_FAILURE)
        .await;
    ok &= run_scenario(
        &connection_manager,
        "timeout_then_fatal",
        &["was_timedout", "timeout_count"],
        // Two timeouts: the first spends the retry, the second is fatal.
        |state| state.get_value("timeout_count", TARGET) == Some(2.to_spvalue()),
    )
    .await;

    common::print_state(
        &connection_manager,
        "Outcomes",
        &[
            "was_bypassed",
            "failure_count",
            "was_fatal",
            "was_timedout",
            "timeout_count",
        ],
    )
    .await;

    // Failure is the *subject* of this example, so reaching all three
    // off-nominal outcomes is what success means here.
    common::finish(ok, "retry, bypass, fatal and timeout paths were all taken");
}

/// Arm one scenario, wait for its outcome, then disarm it.
async fn run_scenario(
    connection_manager: &std::sync::Arc<ConnectionManager>,
    name: &str,
    watch: &[&str],
    condition: impl Fn(&State) -> bool,
) -> bool {
    let mut con = connection_manager.get_connection().await;
    StateManager::set_sp_value(&mut con, "scenario", &name.to_spvalue()).await;

    let mut keys: Vec<&str> = watch.to_vec();
    keys.push("scenario");
    let reached = common::wait_until(connection_manager, &keys, 30_000, condition).await;

    // Disarm, so the operation is not re-initialized and run again.
    StateManager::set_sp_value(&mut con, "scenario", &"none".to_spvalue()).await;
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    if reached {
        println!("      -> reached");
    } else {
        eprintln!("      -> TIMED OUT waiting for {name}");
    }
    reached
}
