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
//! # Which runner drives this
//!
//! [`main_runner`] spawns [`sop_multi_runner`], so the wrapper operation below
//! talks to that runner's per-SOP channel:
//!
//! ```text
//! var:sop_move_robot_and_gantry_sop_enabled <- true      // start it
//! var:sop_move_robot_and_gantry_sop_state   == completed // wait for it
//! ```
//!
//! The old single-SOP [`sop_runner`] channel is kept alongside it, commented
//! out, so the difference is visible in one place. It went through three keys
//! shared by every procedure in the model - `{sp_id}_sop_id` named the SOP to
//! run, `{sp_id}_sop_enabled` started it, `{sp_id}_sop_state` reported back -
//! which is exactly why only one could be in flight at a time. The multi runner
//! drops the `{sp_id}_sop_id` indirection and namespaces the other two by the
//! SOP's own id instead; see `sop_multi` for two procedures at once.
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

const SOP_ID: &str = "sop_move_robot_and_gantry";

fn model(sp_id: &str, state: &State) -> (Model, State) {
    let mut state = state.clone();

    let done = bv!("done");
    state.add_mut(assign!(done, false.to_spvalue()), TARGET);

    let sops = vec![SOPStruct {
        id: SOP_ID.to_string(),
        sop: SOP::Parallel(vec![
            robot_move_to_pos("a", &state),
            gantry_move_to_pos("b", &state),
        ]),
    }];

    // `{sop_id}_sop_enabled` and `{sop_id}_sop_state` are not part of
    // `generate_runner_state_variables`, and they have to exist in the state the
    // wrapper operation is parsed against - `Transition::parse` resolves `var:`
    // names at parse time and panics on one it cannot find. So this goes between
    // building the SOPs and building the operations.
    //
    // The single-SOP runner needed none of this: its three keys are per `sp_id`
    // and `generate_runner_state_variables` already creates them.
    state.extend_mut(generate_multi_sop_state_variables(&sops, TARGET), true);

    let auto_operations = vec![enable_sop(&state)];
    // let auto_operations = vec![enable_sop_single(sp_id, &state)];

    let model = Model::new(sp_id, vec![], auto_operations, vec![], sops, vec![]);

    (model, state)
}

/// Enables the SOP and completes when the whole tree reports `completed`.
///
/// The operation's "hardware" is the SOP runner: the start action raises this
/// SOP's own enable flag, the postcondition waits on this SOP's own status key.
/// Nothing here is keyed by `sp_id`, which is what would let a second wrapper
/// for a second procedure run beside this one.
fn enable_sop(state: &State) -> Operation {
    Operation::new(
        SOP_ID,
        None,
        None,
        None,
        None,
        false,
        vec![Transition::parse(
            &format!("start_{SOP_ID}"),
            "var:done == false",
            "true",
            vec![
                &format!("var:{SOP_ID}_sop_state <- initial"),
                &format!("var:{SOP_ID}_sop_enabled <- true"),
            ],
            Vec::<&str>::new(),
            state,
        )],
        vec![Transition::parse(
            &format!("complete_{SOP_ID}"),
            "true",
            &format!("var:{SOP_ID}_sop_state == completed"),
            vec![
                "var:done <- true",
                &format!("var:{SOP_ID}_sop_state <- initial"),
                // No `_sop_enabled <- false` here: the multi runner consumes the
                // request flag itself when it activates the SOP.
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

// The same wrapper written against the single-SOP `sop_runner`, for comparison.
// To run the example this way, uncomment this function and the
// `enable_sop_single` line in `model`, and re-spawn `sop_runner` (it is
// commented out in `main_runner`, which now spawns `sop_multi_runner`).
//
// Note what the extra key buys and costs: `{sp_id}_sop_id` has to be written on
// start and holds the runner for the whole run, so a second procedure could only
// be started by overwriting it - and `{sp_id}_sop_enabled` has to be lowered by
// hand on completion, because it is a standing flag rather than a consumed
// request.
//
// fn enable_sop_single(sp_id: &str, state: &State) -> Operation {
//     Operation::new(
//         SOP_ID,
//         None,
//         None,
//         None,
//         None,
//         false,
//         vec![Transition::parse(
//             &format!("start_{SOP_ID}"),
//             "var:done == false",
//             "true",
//             vec![
//                 &format!("var:{sp_id}_sop_id <- {SOP_ID}"),
//                 &format!("var:{sp_id}_sop_state <- initial"),
//                 &format!("var:{sp_id}_sop_enabled <- true"),
//             ],
//             Vec::<&str>::new(),
//             state,
//         )],
//         vec![Transition::parse(
//             &format!("complete_{SOP_ID}"),
//             "true",
//             &format!("var:{sp_id}_sop_state == completed"),
//             vec![
//                 "var:done <- true",
//                 &format!("var:{sp_id}_sop_state <- initial"),
//                 &format!("var:{sp_id}_sop_enabled <- false"),
//             ],
//             Vec::<&str>::new(),
//             state,
//         )],
//         vec![],
//         vec![],
//         vec![],
//         vec![],
//     )
// }

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

    // Spawns `sop_multi_runner`; nothing extra to start here.
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
            &format!("{SOP_ID}_sop_state"),
            &format!("{SP_ID}_sop_runner_information"),
            // The single-SOP runner's status key, for comparison: untouched,
            // because nothing ever named a SOP in `{sp_id}_sop_id`.
            // &format!("{SP_ID}_sop_state"),
        ],
    )
    .await;

    common::finish(done, "both branches completed and the parallel node closed");
}
