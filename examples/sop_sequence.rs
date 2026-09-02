//! SOPs: a procedure you wrote down, executed step by step.
//!
//! A SOP - Standard Operating Procedure - is a *tree* of operations with the
//! order fixed at modelling time. [`SOP::Sequence`] runs its children one after
//! another; this one walks the robot a -> b -> a -> b -> a, five steps that
//! were never planned and could not have been: the planner would collapse
//! "get to a" into a single move, because reaching `a` is all a goal predicate
//! can express. When the *route* is the requirement, a SOP is the tool.
//!
//! Two things are worth noticing in the model below.
//!
//! **A SOP does not start itself.** The SOP runner executes whichever SOP has
//! its `{sop_id}_sop_enabled` flag set. Those are ordinary state variables, so
//! anything can set them - a dashboard, another process, or (as here) an
//! automatic operation whose only job is to enable the SOP and wait for
//! `{sop_id}_sop_state == completed`. That wrapper is what turns a procedure
//! into something with a lifecycle of its own.
//!
//! **The same operation appears five times.** The SOP runner uniquifies each
//! occurrence at runtime, so `robot_move_to_a` in step 1 and step 3 get
//! separate lifecycle variables and cannot be confused with each other.
//!
//! # Which runner drives this
//!
//! [`main_runner`] spawns [`sop_multi_runner`], so the wrapper operation below
//! talks to that runner's per-SOP channel:
//!
//! ```text
//! var:sop_robot_move_ababa_sop_enabled <- true      // start it
//! var:sop_robot_move_ababa_sop_state   == completed // wait for it
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
//! cargo run --example sop_sequence
//! ```

mod common;

use common::{DONT_EMULATE_FAILURE, SP_ID, TARGET};
use micro_sp::*;

const SOP_ID: &str = "sop_robot_move_ababa";

fn model(sp_id: &str, state: &State) -> (Model, State) {
    let mut state = state.clone();

    let done = bv!("done");
    state.add_mut(assign!(done, false.to_spvalue()), TARGET);

    let sops = vec![SOPStruct {
        id: SOP_ID.to_string(),
        sop: SOP::Sequence(vec![
            robot_move_to_pos("a", &state),
            robot_move_to_pos("b", &state),
            robot_move_to_pos("a", &state),
            robot_move_to_pos("b", &state),
            robot_move_to_pos("a", &state),
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

    // The auto operation that enables the SOP and waits for it. Without
    // something like this, the SOP sits in the model and never runs.
    let auto_operations = vec![enable_sop(&state)];
    // let auto_operations = vec![enable_sop_single(sp_id, &state)];

    let model = Model::new(sp_id, vec![], auto_operations, vec![], sops, vec![]);

    (model, state)
}

/// An operation whose "hardware" is the SOP runner: it raises this SOP's own
/// enable flag and completes when this SOP's own status key reports `completed`.
///
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
            "start_robot_move_ababa",
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
            "complete_robot_move_ababa",
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
//             "start_robot_move_ababa",
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
//             "complete_robot_move_ababa",
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

/// One leaf of the tree: an ordinary [`Operation`], wrapped in
/// [`SOP::Operation`]. Nothing about it is SOP-specific.
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

#[tokio::main]
async fn main() {
    let (model, domain) = model(SP_ID, &common::base_state(SP_ID, 1));
    let connection_manager = common::boot(SP_ID, &model, domain, 1).await;

    common::spawn_robot(&connection_manager);
    common::configure_emulator(&connection_manager, "robot", Some(300), DONT_EMULATE_FAILURE).await;

    println!("Running the a -> b -> a -> b -> a procedure, in the order it was written.");

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
            &format!("{SOP_ID}_sop_state"),
            // The multi runner reports progress per SOP here. It has no
            // equivalent of the single runner's `{sp_id}_sop_current_step`:
            // with several procedures in flight there is no single step index.
            &format!("{SOP_ID}_sop_information"),
            &format!("{SOP_ID}_sop_enabled"),
            &format!("{SP_ID}_sop_runner_information"),
            // The single-SOP runner's status keys, for comparison: untouched,
            // because nothing ever named a SOP in `{sp_id}_sop_id`.
            // &format!("{SP_ID}_sop_state"),
            // &format!("{SP_ID}_sop_current_step"),
            // &format!("{SP_ID}_sop_enabled"),
        ],
    )
    .await;

    common::finish(done, "the five-step sequence ran to completion");
}
