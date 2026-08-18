//! [`SOP::Parallel`]: branches that run at the same time.
//!
//! A `Parallel` node starts all of its children and completes only when every
//! one of them has. Here the robot moves to `a` while the gantry moves to `b`,
//! against two independent emulators, and the node finishes when both are
//! there.
//!
//! This is the case a plan cannot express. The planner produces a *sequence* -
//! `{sp_id}_plan` is an ordered list and the plan runner drives one step at a
//! time - so two operations that could overlap will still be executed one
//! after the other. Concurrency has to be written into the model.
//!
//! The emulators are configured with different execution times (400 ms for the
//! robot, 800 ms for the gantry) so the overlap is visible in the log: the
//! robot reports success while the gantry is still moving, and the node waits.
//!
//! Run with:
//!
//! ```text
//! docker compose up -d
//! cargo run --example sop_parallel
//! ```

mod common;

use common::{DONT_EMULATE_FAILURE, SP_ID, TARGET};
use micro_sp::*;

fn model(sp_id: &str, state: &State) -> (Model, State) {
    let state = state.clone();

    let done = bv!("done");
    let state = state.add(assign!(done, false.to_spvalue()), TARGET);

    let sops = vec![SOPStruct {
        id: "sop_move_robot_and_gantry".to_string(),
        sop: SOP::Parallel(vec![
            robot_move_to_pos("a", &state),
            gantry_move_to_pos("b", &state),
        ]),
    }];

    let auto_operations = vec![enable_sop(sp_id, &state)];

    let model = Model::new(sp_id, vec![], auto_operations, vec![], sops, vec![]);

    (model, state)
}

/// Enables the SOP and completes when the whole tree reports `completed`.
fn enable_sop(sp_id: &str, state: &State) -> Operation {
    Operation::new(
        "sop_move_robot_and_gantry",
        None,
        None,
        None,
        None,
        false,
        vec![Transition::parse(
            "start_sop_move_robot_and_gantry",
            "var:done == false",
            "true",
            vec![
                &format!("var:{sp_id}_sop_id <- sop_move_robot_and_gantry"),
                &format!("var:{sp_id}_sop_state <- initial"),
                &format!("var:{sp_id}_sop_enabled <- true"),
            ],
            Vec::<&str>::new(),
            state,
        )],
        vec![Transition::parse(
            "complete_sop_move_robot_and_gantry",
            "true",
            &format!("var:{sp_id}_sop_state == completed"),
            vec![
                "var:done <- true",
                &format!("var:{sp_id}_sop_state <- initial"),
                &format!("var:{sp_id}_sop_enabled <- false"),
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

fn robot_move_to_pos(pos: &str, state: &State) -> SOP {
    SOP::Operation(Box::new(Operation::new(
        &format!("robot_move_to_{pos}"),
        None,
        None,
        None,
        None,
        false,
        vec![Transition::parse(
            &format!("start_robot_move_to_{pos}"),
            "var:robot_request_state == initial && var:robot_request_trigger == false",
            "true",
            vec![
                "var:robot_command_command <- move",
                &format!("var:robot_position_command <- {pos}"),
                "var:robot_speed_command <- 0.5",
                "var:robot_request_trigger <- true",
            ],
            Vec::<&str>::new(),
            state,
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
            state,
        )],
        vec![],
        vec![],
        vec![],
        vec![],
    )))
}

/// The gantry branch. It touches only `gantry_*` variables, which is *why* the
/// two branches can overlap - two operations contending for the same resource
/// would serialise on its `request_trigger` handshake no matter what the tree
/// says.
fn gantry_move_to_pos(pos: &str, state: &State) -> SOP {
    SOP::Operation(Box::new(Operation::new(
        &format!("gantry_move_to_{pos}"),
        None,
        None,
        None,
        None,
        false,
        vec![Transition::parse(
            &format!("start_gantry_move_to_{pos}"),
            "var:gantry_request_state == initial && var:gantry_request_trigger == false",
            "true",
            vec![
                "var:gantry_command_command <- move",
                &format!("var:gantry_position_command <- {pos}"),
                "var:gantry_speed_command <- 0.5",
                "var:gantry_request_trigger <- true",
            ],
            Vec::<&str>::new(),
            state,
        )],
        vec![Transition::parse(
            &format!("complete_gantry_move_to_{pos}"),
            "true",
            "var:gantry_request_state == succeeded",
            vec![
                "var:gantry_request_trigger <- false",
                "var:gantry_request_state <- initial",
                &format!("var:gantry_position_estimated <- {pos}"),
            ],
            Vec::<&str>::new(),
            state,
        )],
        vec![],
        vec![],
        vec![],
        vec![],
    )))
}

#[tokio::main]
async fn main() {
    let (model, domain) = model(SP_ID, &common::base_state(SP_ID, 1));
    let connection_manager = common::boot(SP_ID, &model, domain, 1).await;

    common::spawn_robot(&connection_manager);
    common::spawn_gantry(&connection_manager);

    // Different durations, so the overlap shows up in the log rather than
    // having to be taken on trust.
    common::configure_emulator(&connection_manager, "robot", Some(400), DONT_EMULATE_FAILURE).await;
    common::configure_emulator(&connection_manager, "gantry", Some(800), DONT_EMULATE_FAILURE)
        .await;

    println!("Robot to a and gantry to b, at the same time; the node waits for both.");
    main_runner(&SP_ID.to_string(), model, 1, &connection_manager).await;

    let done = common::wait_until(&connection_manager, &["done"], 40_000, |state| {
        state.get_value("done", TARGET) == Some(true.to_spvalue())
    })
    .await;

    common::print_state(
        &connection_manager,
        "Final state",
        &[
            "done",
            "robot_position_estimated",
            "gantry_position_estimated",
            &format!("{SP_ID}_sop_state"),
        ],
    )
    .await;

    common::finish(done, "both branches completed and the parallel node closed");
}
