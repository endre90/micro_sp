//! Automatic operations: tasks that start themselves.
//!
//! An automatic operation is a full [`Operation`] - preconditions,
//! postconditions, a lifecycle, timeouts, retries - that nobody schedules. The
//! `auto_operation_runner` starts it the moment a precondition holds, exactly
//! as the plan runner would, and nothing had to plan for it.
//!
//! That is the whole difference from an automatic *transition*: a transition
//! fires and is over in one tick, so it can never wait on hardware. This model
//! commands the robot and then waits for the emulator to answer, which takes
//! many ticks - and while it waits, the operation is `executing` and its
//! deadline is counting down. A transition has nowhere to put that state.
//!
//! Two operations, `robot_move_to_a` and `robot_move_to_b`, sit in
//! [`Model::auto_operations`]. Each is enabled only when the robot is *not*
//! already at its target, so they alternate: a, b, a, b, ... The shared
//! `counter` stops them after five moves.
//!
//! Run with:
//!
//! ```text
//! docker compose up -d
//! cargo run --example auto_operations
//! ```

mod common;

use common::{DONT_EMULATE_FAILURE, SP_ID, TARGET};
use micro_sp::*;

/// The model stops itself after this many moves.
const MOVES: i64 = 5;

fn model(sp_id: &str, state: &State) -> (Model, State) {
    let state = state.clone();
    let mut auto_operations = vec![];

    let counter = iv!("counter");
    let state = state.add(assign!(counter, 0.to_spvalue()), TARGET);

    for pos in ["a", "b"] {
        auto_operations.push(Operation::new(
            &format!("robot_move_to_{pos}"),
            None,  // timeout_executing_ms: default (10 min)
            None,  // timeout_disabled_ms:  default
            None,  // failure_retries:      none
            None,  // timeout_retries:      none
            false, // can_be_bypassed
            // Precondition. Note what it checks: the robot must be idle
            // (`request_state == initial`, `request_trigger == false`) *and*
            // not already where we are sending it. Without the last clause the
            // two operations would fight over an already-satisfied goal.
            vec![Transition::parse(
                &format!("start_robot_move_to_{pos}"),
                &format!(
                    "var:counter < {MOVES} \
                     && var:robot_request_state == initial \
                     && var:robot_request_trigger == false \
                     && var:robot_position_estimated != {pos}"
                ),
                "true",
                vec![
                    "var:robot_command_command <- move",
                    &format!("var:robot_position_command <- {pos}"),
                    "var:robot_speed_command <- 0.5",
                    "var:robot_request_trigger <- true",
                ],
                Vec::<&str>::new(),
                &state,
            )],
            // Postcondition. The guard is `true` and the *runner* guard waits
            // for the emulator's answer, so the planner treats this as a step
            // that always completes while the runtime waits for the hardware.
            vec![Transition::parse(
                &format!("complete_robot_move_to_{pos}"),
                "true",
                "var:robot_request_state == succeeded",
                vec![
                    "var:robot_request_trigger <- false",
                    "var:robot_request_state <- initial",
                    &format!("var:robot_position_estimated <- {pos}"),
                    "var:counter += 1",
                ],
                Vec::<&str>::new(),
                &state,
            )],
            vec![], // failure_transitions
            vec![], // timeout_transitions
            vec![], // bypass_transitions
            vec![], // cancel_transitions
        ));
    }

    let model = Model::new(sp_id, vec![], auto_operations, vec![], vec![], vec![]);

    (model, state)
}

#[tokio::main]
async fn main() {
    let (model, domain) = model(SP_ID, &common::state::state());
    let connection_manager = common::boot(SP_ID, &model, domain, 1).await;

    common::spawn_robot(&connection_manager);
    common::configure_emulator(&connection_manager, "robot", Some(300), DONT_EMULATE_FAILURE).await;

    println!("Two auto operations bouncing the robot between a and b, {MOVES} times.");
    main_runner(&SP_ID.to_string(), model, 1, &connection_manager).await;

    let done = common::wait_until(&connection_manager, &["counter"], 30_000, |state| {
        state.get_value("counter", TARGET) == Some(MOVES.to_spvalue())
    })
    .await;

    common::print_state(
        &connection_manager,
        "Final state",
        &[
            "counter",
            "robot_position_estimated",
            "robot_request_state",
            "op_robot_move_to_a",
            "op_robot_move_to_b",
        ],
    )
    .await;

    common::finish(done, "the robot made five moves with nothing planning them");
}
