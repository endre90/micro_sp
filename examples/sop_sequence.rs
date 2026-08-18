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
//! **A SOP does not start itself.** The SOP runner executes whichever SOP
//! `{sp_id}_sop_id` names, once `{sp_id}_sop_enabled` is set. Those are
//! ordinary state variables, so anything can set them - a dashboard, another
//! process, or (as here) an automatic operation whose only job is to enable the
//! SOP and wait for `{sp_id}_sop_state == completed`. That wrapper is what
//! turns a procedure into something with a lifecycle of its own.
//!
//! **The same operation appears five times.** The SOP runner uniquifies each
//! occurrence at runtime, so `robot_move_to_a` in step 1 and step 3 get
//! separate lifecycle variables and cannot be confused with each other.
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

fn model(sp_id: &str, state: &State) -> (Model, State) {
    let state = state.clone();

    let done = bv!("done");
    let state = state.add(assign!(done, false.to_spvalue()), TARGET);

    let sops = vec![SOPStruct {
        id: "sop_robot_move_ababa".to_string(),
        sop: SOP::Sequence(vec![
            robot_move_to_pos("a", &state),
            robot_move_to_pos("b", &state),
            robot_move_to_pos("a", &state),
            robot_move_to_pos("b", &state),
            robot_move_to_pos("a", &state),
        ]),
    }];

    // The auto operation that enables the SOP and waits for it. Without
    // something like this, the SOP sits in the model and never runs.
    let auto_operations = vec![enable_sop(sp_id, &state)];

    let model = Model::new(sp_id, vec![], auto_operations, vec![], sops, vec![]);

    (model, state)
}

/// An operation whose "hardware" is the SOP runner: it raises the enable flag
/// and completes when the procedure reports `completed`.
fn enable_sop(sp_id: &str, state: &State) -> Operation {
    Operation::new(
        "sop_robot_move_ababa",
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
                &format!("var:{sp_id}_sop_id <- sop_robot_move_ababa"),
                &format!("var:{sp_id}_sop_state <- initial"),
                &format!("var:{sp_id}_sop_enabled <- true"),
            ],
            Vec::<&str>::new(),
            state,
        )],
        vec![Transition::parse(
            "complete_robot_move_ababa",
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
            &format!("{SP_ID}_sop_state"),
            &format!("{SP_ID}_sop_current_step"),
            &format!("{SP_ID}_sop_enabled"),
        ],
    )
    .await;

    common::finish(done, "the five-step sequence ran to completion");
}
