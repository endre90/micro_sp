//! Planning: say where you want to be, not how to get there.
//!
//! Nothing here decides an order. The model just lists seven operations - move
//! the robot to a, b, c, ... g - in [`Model::operations`], the field the
//! planner is allowed to search over. Then a goal *predicate* is posted to
//! `{sp_id}_incoming_goals` and the stack works the rest out:
//!
//! ```text
//! {sp_id}_incoming_goals   goal_runner admits it, sorts by priority
//!         v
//! {sp_id}_scheduled_goals  one goal at a time is promoted to current
//!         v
//! {sp_id}_replan_trigger   planner_ticker searches operations -> {sp_id}_plan
//!         v
//! {sp_id}_plan             planned_operation_runner walks it, step by step
//!         v
//! {sp_id}_plan_state       == completed, so goal_runner releases the goal
//! ```
//!
//! Six goals are queued at once. They are served one at a time, in priority
//! order, and each is planned against the state the previous one left behind -
//! which is why the same goal predicate can produce a different plan each time
//! it comes round.
//!
//! Contrast with `sop_sequence`, where *you* write the order down. Here the
//! order is a search result.
//!
//! Run with:
//!
//! ```text
//! docker compose up -d
//! cargo run --example planning_goals
//! ```

mod common;

use common::{DONT_EMULATE_FAILURE, SP_ID, TARGET};
use micro_sp::running::goal_runner::{GoalPriority, goal_string_to_sp_value};
use micro_sp::*;

/// The positions the robot can be planned to.
const POSITIONS: [&str; 7] = ["a", "b", "c", "d", "e", "f", "g"];

/// Safety stop, so a mis-specified goal cannot cycle the robot forever.
const MAX_MOVES: i64 = 10;

fn model(sp_id: &str, state: &State) -> (Model, State) {
    let state = state.clone();
    let mut operations = vec![];

    let counter = iv!("counter");
    let state = state.add(assign!(counter, 0.to_spvalue()), TARGET);

    for pos in POSITIONS {
        operations.push(Operation::new(
            &format!("robot_move_to_{pos}"),
            None,
            None,
            None,
            None,
            false,
            vec![Transition::parse(
                &format!("start_robot_move_to_{pos}"),
                &format!(
                    "var:counter < {MAX_MOVES} \
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
            vec![],
            vec![],
            vec![],
            vec![],
        ));
    }

    // Note the empty `auto_operations`: these are the *same* `Operation` type
    // as in the auto_operations example. Which field of the model they sit in
    // is the entire difference between "the planner may use this" and "this
    // runs on its own".
    let model = Model::new(sp_id, vec![], vec![], vec![], vec![], operations);

    (model, state)
}

#[tokio::main]
async fn main() {
    let (model, domain) = model(SP_ID, &common::state::state());
    let connection_manager = common::boot(SP_ID, &model, domain, 1).await;

    common::spawn_robot(&connection_manager);
    common::configure_emulator(&connection_manager, "robot", Some(200), DONT_EMULATE_FAILURE).await;

    // Six goals, posted in one write. Each is a predicate over the state - not
    // an operation name - so the planner is free to reach it any way it can.
    let goals: Vec<SPValue> = ["a", "b", "c", "a", "b", "c"]
        .iter()
        .map(|pos| {
            goal_string_to_sp_value(
                "", // empty id: goal_runner assigns one on admission
                &format!("var:robot_position_estimated == {pos}"),
                GoalPriority::Normal,
            )
        })
        .collect();

    println!("Posting {} goals; the planner decides how to reach each.", goals.len());
    main_runner(&SP_ID.to_string(), model, 1, &connection_manager).await;

    // Post after the runners are up, so the goal runner sees the write rather
    // than finding a queue already there on its first tick.
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    let mut con = connection_manager.get_connection().await;
    StateManager::set_sp_value(
        &mut con,
        &format!("{SP_ID}_incoming_goals"),
        &goals.to_spvalue(),
    )
    .await;

    // Every goal is served when the queue has drained, the current goal has
    // been released back to `initial`, and the robot is at the last one (c).
    let all_served = common::wait_until(
        &connection_manager,
        &[
            &format!("{SP_ID}_incoming_goals"),
            &format!("{SP_ID}_scheduled_goals"),
            &format!("{SP_ID}_current_goal_state"),
            "robot_position_estimated",
        ],
        60_000,
        |state| {
            let empty = |key: &str| {
                matches!(
                    state.get_value(key, TARGET),
                    Some(SPValue::Array(ArrayOrUnknown::Array(ref a))) if a.is_empty()
                )
            };
            empty(&format!("{SP_ID}_incoming_goals"))
                && empty(&format!("{SP_ID}_scheduled_goals"))
                && state.get_value(&format!("{SP_ID}_current_goal_state"), TARGET)
                    == Some("initial".to_spvalue())
                && state.get_value("robot_position_estimated", TARGET) == Some("c".to_spvalue())
        },
    )
    .await;

    common::print_state(
        &connection_manager,
        "Final state",
        &[
            "robot_position_estimated",
            "counter",
            &format!("{SP_ID}_scheduled_goals"),
            &format!("{SP_ID}_current_goal_state"),
            &format!("{SP_ID}_plan_state"),
            &format!("{SP_ID}_planner_state"),
            &format!("{SP_ID}_plan_counter"),
        ],
    )
    .await;

    common::finish(all_served, "all six goals were planned for and reached");
}
