//! Replanning: the plan was right, the world was not.
//!
//! A plan is only as good as the state it was planned from, and some of that
//! state is a guess. This model has a robot that may be holding a tool, but
//! nothing has looked yet - `robot_mounted_estimated` is `UNKNOWN`. The
//! operation that looks, `robot_check_for_gripper_tool_mounted`, therefore has
//! *two* postconditions:
//!
//!   * the measured tool is the one we hoped for - carry on;
//!   * it is something else - record what is really there and set
//!     `{sp_id}_replan_for_same_goal`.
//!
//! The planner searches over the first postcondition, so the plan it produces
//! assumes the optimistic answer. When the second one fires instead, the goal
//! runner sees the flag, re-triggers the planner *for the same goal*, and the
//! plan runner picks up a new plan from the state that is actually true.
//!
//! The scenario: ask for the gripper tool to be mounted, while the robot is in
//! fact holding the suction tool.
//!
//! ```text
//! plan 1  check_for_gripper_tool_mounted            <- one step; assumes we
//!                                                      already have it
//!         ... measurement says "suction_tool" ...
//!
//! plan 2  gantry_unlock -> gantry_calibrate -> gantry_lock
//!         -> robot_move_to_suction_tool_rack -> robot_unmount_suction_tool
//!         -> robot_move_to_gripper_tool_rack -> robot_mount_gripper_tool
//! ```
//!
//! Nobody wrote either sequence down. The second one exists because the model
//! says a robot can only move when the gantry is locked and calibrated, and
//! can only calibrate while unlocked - constraints, not a script.
//!
//! Run with:
//!
//! ```text
//! docker compose up -d
//! cargo run --example replanning
//! ```

mod common;

use common::{DONT_EMULATE_FAILURE, SP_ID, TARGET};
use micro_sp::running::goal_runner::{GoalPriority, goal_string_to_sp_value};
use micro_sp::*;

/// The tool the robot is actually holding, unknown to the planner.
const ACTUAL_TOOL: &str = "suction_tool";
/// The tool we ask for.
const WANTED_TOOL: &str = "gripper_tool";

fn model(sp_id: &str, state: &State) -> (Model, State) {
    let state = state.clone();
    let mut operations = vec![];

    // ------------------------------------------------------------ the gantry
    // Calibration needs the gantry unlocked; the robot needs it locked. That
    // pair of constraints is the whole reason the recovery plan is seven steps
    // and not three - and the planner is the only thing that has to know it.
    operations.push(gantry_command(
        "gantry_unlock",
        "unlock",
        "var:gantry_request_state == initial && var:gantry_request_trigger == false",
        "var:gantry_locked_estimated <- false",
        &state,
    ));
    operations.push(gantry_command(
        "gantry_lock",
        "lock",
        "var:gantry_request_state == initial && var:gantry_request_trigger == false",
        "var:gantry_locked_estimated <- true",
        &state,
    ));
    operations.push(gantry_command(
        "gantry_calibrate",
        "calibrate",
        "var:gantry_locked_estimated == false \
         && var:gantry_request_state == initial \
         && var:gantry_request_trigger == false",
        "var:gantry_calibrated_estimated <- true",
        &state,
    ));

    // ------------------------------------------------------------- the robot
    for pos in ["gripper_tool_rack", "suction_tool_rack", "pipe_blue_box"] {
        operations.push(Operation::new(
            &format!("robot_move_to_{pos}"),
            None,
            None,
            None,
            None,
            false,
            vec![Transition::parse(
                &format!("start_robot_move_to_{pos}"),
                "var:robot_request_state == initial \
                 && var:robot_request_trigger == false \
                 && var:gantry_locked_estimated == true \
                 && var:gantry_calibrated_estimated == true",
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

    for tool in ["gripper_tool", "suction_tool"] {
        operations.push(check_for_tool(sp_id, tool, &state));
        operations.push(mount_tool(tool, &state));
        operations.push(unmount_tool(tool, &state));
    }

    let model = Model::new(sp_id, vec![], vec![], vec![], vec![], operations);

    (model, state)
}

/// A one-shot gantry command: raise the trigger, wait for `succeeded`, record
/// the effect.
fn gantry_command(
    name: &str,
    command: &str,
    guard: &str,
    effect: &str,
    state: &State,
) -> Operation {
    Operation::new(
        name,
        None,
        None,
        None,
        None,
        false,
        vec![Transition::parse(
            &format!("start_{name}"),
            guard,
            "true",
            vec![
                &format!("var:gantry_command_command <- {command}"),
                "var:gantry_request_trigger <- true",
            ],
            Vec::<&str>::new(),
            state,
        )],
        vec![Transition::parse(
            &format!("complete_{name}"),
            "true",
            "var:gantry_request_state == succeeded",
            vec![
                "var:gantry_request_trigger <- false",
                "var:gantry_request_state <- initial",
                effect,
            ],
            Vec::<&str>::new(),
            state,
        )],
        vec![],
        vec![],
        vec![],
        vec![],
    )
}

/// The operation that turns a guess into a fact - and asks for a replan when
/// the fact disagrees with the guess.
///
/// Both postconditions have guard `true` and differ only in their *runner*
/// guard, which is what makes this work: the planner sees a step that always
/// establishes `robot_mounted_estimated == {tool}`, while at runtime whichever
/// branch matches the measurement is the one that fires.
fn check_for_tool(sp_id: &str, tool: &str, state: &State) -> Operation {
    Operation::new(
        &format!("robot_check_for_{tool}_mounted"),
        None,
        None,
        None,
        None,
        false,
        vec![Transition::parse(
            &format!("start_robot_check_for_{tool}_mounted"),
            "(var:robot_mounted_checked == false || var:robot_mounted_checked == UNKNOWN_bool) \
             && var:robot_request_state == initial \
             && var:robot_request_trigger == false \
             && var:robot_mounted_estimated == UNKNOWN_string",
            "true",
            vec![
                "var:robot_command_command <- check_mounted_tool",
                "var:robot_request_trigger <- true",
            ],
            Vec::<&str>::new(),
            state,
        )],
        vec![
            // The optimistic branch - and the only one the planner models.
            Transition::parse(
                &format!("complete_robot_check_for_{tool}_mounted"),
                "true",
                &format!(
                    "var:robot_request_state == succeeded \
                     && var:robot_mounted_one_time_measured == {tool}"
                ),
                vec![
                    "var:robot_request_trigger <- false",
                    "var:robot_request_state <- initial",
                    "var:robot_mounted_checked <- true",
                    &format!("var:robot_mounted_estimated <- {tool}"),
                ],
                Vec::<&str>::new(),
                state,
            ),
            // The surprise. Record what is really mounted and ask the goal
            // runner to plan again from there, keeping the same goal.
            Transition::parse(
                &format!("complete_robot_check_for_{tool}_mounted_2"),
                "true",
                &format!(
                    "var:robot_request_state == succeeded \
                     && var:robot_mounted_one_time_measured != {tool}"
                ),
                vec![
                    "var:robot_request_trigger <- false",
                    "var:robot_request_state <- initial",
                    "var:robot_mounted_checked <- true",
                    "var:robot_mounted_estimated <- var:robot_mounted_one_time_measured",
                    &format!("var:{sp_id}_replan_for_same_goal <- true"),
                ],
                Vec::<&str>::new(),
                state,
            ),
        ],
        vec![],
        vec![],
        vec![],
        vec![],
    )
}

fn mount_tool(tool: &str, state: &State) -> Operation {
    Operation::new(
        &format!("robot_mount_{tool}"),
        None,
        None,
        None,
        None,
        false,
        vec![Transition::parse(
            &format!("start_robot_mount_{tool}"),
            &format!(
                "var:robot_request_state == initial \
                 && var:robot_request_trigger == false \
                 && var:robot_position_estimated == {tool}_rack \
                 && var:robot_mounted_estimated == none \
                 && var:gantry_locked_estimated == true"
            ),
            "true",
            vec![
                "var:robot_command_command <- mount",
                "var:robot_request_trigger <- true",
            ],
            Vec::<&str>::new(),
            state,
        )],
        vec![Transition::parse(
            &format!("complete_robot_mount_{tool}"),
            "true",
            "var:robot_request_state == succeeded",
            vec![
                "var:robot_request_trigger <- false",
                "var:robot_request_state <- initial",
                &format!("var:robot_mounted_estimated <- {tool}"),
            ],
            Vec::<&str>::new(),
            state,
        )],
        vec![],
        vec![],
        vec![],
        vec![],
    )
}

fn unmount_tool(tool: &str, state: &State) -> Operation {
    Operation::new(
        &format!("robot_unmount_{tool}"),
        None,
        None,
        None,
        None,
        false,
        vec![Transition::parse(
            &format!("start_robot_unmount_{tool}"),
            &format!(
                "var:robot_request_state == initial \
                 && var:robot_request_trigger == false \
                 && var:robot_position_estimated == {tool}_rack \
                 && var:robot_mounted_estimated == {tool} \
                 && var:gantry_locked_estimated == true"
            ),
            "true",
            vec![
                "var:robot_command_command <- unmount",
                "var:robot_request_trigger <- true",
            ],
            Vec::<&str>::new(),
            state,
        )],
        vec![Transition::parse(
            &format!("complete_robot_unmount_{tool}"),
            "true",
            "var:robot_request_state == succeeded",
            vec![
                "var:robot_request_trigger <- false",
                "var:robot_request_state <- initial",
                "var:robot_mounted_estimated <- none",
            ],
            Vec::<&str>::new(),
            state,
        )],
        vec![],
        vec![],
        vec![],
        vec![],
    )
}

#[tokio::main]
async fn main() {
    let (model, domain) = model(SP_ID, &common::base_state(SP_ID, 1));
    let connection_manager = common::boot(SP_ID, &model, domain, 1).await;

    common::spawn_robot(&connection_manager);
    common::spawn_gantry(&connection_manager);
    common::configure_emulator(&connection_manager, "robot", Some(200), DONT_EMULATE_FAILURE).await;
    common::configure_emulator(&connection_manager, "gantry", Some(200), DONT_EMULATE_FAILURE)
        .await;

    // Tell the emulator what is really on the robot. The *model* has no idea -
    // `robot_mounted_estimated` stays UNKNOWN until something measures it.
    let mut con = connection_manager.get_connection().await;
    StateManager::set_sp_value(&mut con, "robot_emulate_mounted_tool", &true.to_spvalue()).await;
    StateManager::set_sp_value(
        &mut con,
        "robot_emulated_mounted_tool",
        &ACTUAL_TOOL.to_spvalue(),
    )
    .await;

    println!("Asking for the {WANTED_TOOL}; the robot is really holding the {ACTUAL_TOOL}.");
    main_runner(&SP_ID.to_string(), model, 1, &connection_manager).await;

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    StateManager::set_sp_value(
        &mut con,
        &format!("{SP_ID}_incoming_goals"),
        &vec![goal_string_to_sp_value(
            "",
            &format!("var:robot_mounted_estimated == {WANTED_TOOL}"),
            GoalPriority::Normal,
        )]
        .to_spvalue(),
    )
    .await;

    let reached = common::wait_until(
        &connection_manager,
        &[
            "robot_mounted_estimated",
            &format!("{SP_ID}_plan_state"),
        ],
        90_000,
        |state| {
            state.get_value("robot_mounted_estimated", TARGET) == Some(WANTED_TOOL.to_spvalue())
                && state.get_value(&format!("{SP_ID}_plan_state"), TARGET)
                    == Some("completed".to_spvalue())
        },
    )
    .await;

    // `sp_plan_counter` above 1 is the evidence: the goal was planned for more
    // than once, without ever being re-posted.
    common::print_state(
        &connection_manager,
        "Final state",
        &[
            "robot_mounted_estimated",
            "robot_position_estimated",
            "gantry_locked_estimated",
            "gantry_calibrated_estimated",
            &format!("{SP_ID}_plan_counter"),
        ],
    )
    .await;

    common::finish(
        reached,
        "the planner recovered from a wrong assumption and reached the goal",
    );
}
