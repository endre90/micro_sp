//! Several SOPs running at the same time.
//!
//! [`sop_runner`] executes one procedure at a time: the SOP to run is named in
//! the single `{sp_id}_sop_id` key and reports through the single
//! `{sp_id}_sop_state` key, so starting a second one means overwriting the
//! first. Putting the operations into `auto_operations` instead does not help -
//! automatic operations have no ordering, so the sequence is lost.
//!
//! [`sop_multi_runner`] drops that indirection and namespaces everything by the
//! SOP's own id, so every procedure has its own request and status variables and
//! any number of them can be in flight:
//!
//! ```text
//! var:sop_move_robot_sop_enabled  <- true      // start it
//! var:sop_move_robot_sop_state    == completed // wait for it
//! ```
//!
//! Here the robot walks `a -> b` while the gantry walks `b -> a`, as **two
//! separate SOPs** enabled at the same moment by two independent wrapper
//! operations. Compare with `sop_parallel`, which gets overlap by writing a
//! [`SOP::Parallel`] node *inside one* procedure: there the two branches finish
//! together because the node waits for both, and there is exactly one thing to
//! start and one thing to wait for. Here they are genuinely separate procedures
//! - the robot's is already `completed` and torn down while the gantry's is
//! still `executing` - which is what the emulators' different execution times
//! (400 ms against 800 ms) make visible in the log.
//!
//! Like every other SOP example, this one has to seed the per-SOP variables:
//! `{sop_id}_sop_enabled` and `{sop_id}_sop_state` are not part of
//! `generate_runner_state_variables`, so [`generate_multi_sop_state_variables`]
//! creates them, and it has to be called *before* the wrapper operations are
//! built, because `Transition::parse` resolves `var:` names at parse time and
//! panics on one it cannot find.
//!
//! What is unique here is the shape of the model rather than any extra setup:
//! *two* [`SOPStruct`]s and two independent wrapper operations, sharing no key.
//! `main_runner` spawns [`sop_multi_runner`] and drives both of them - there is
//! nothing extra to start.
//!
//! Run with:
//!
//! ```text
//! docker compose up -d
//! cargo run --example sop_multi
//! ```

mod common;

use common::{DONT_EMULATE_FAILURE, SP_ID, TARGET};
use micro_sp::*;

const ROBOT_SOP: &str = "sop_move_robot";
const GANTRY_SOP: &str = "sop_move_gantry";

fn model(sp_id: &str, state: &State) -> (Model, State) {
    let mut state = state.clone();

    state.add_mut(assign!(bv!("done_robot"), false.to_spvalue()), TARGET);
    state.add_mut(assign!(bv!("done_gantry"), false.to_spvalue()), TARGET);

    // Two procedures, not two branches of one. They touch disjoint resources,
    // which is what lets them overlap - two operations contending for the same
    // `request_trigger` handshake would serialise no matter how they are
    // arranged.
    let sops = vec![
        SOPStruct {
            id: ROBOT_SOP.to_string(),
            sop: SOP::Sequence(vec![
                robot_move_to_pos("a", &state),
                robot_move_to_pos("b", &state),
            ]),
        },
        SOPStruct {
            id: GANTRY_SOP.to_string(),
            sop: SOP::Sequence(vec![
                gantry_move_to_pos("b", &state),
                gantry_move_to_pos("a", &state),
            ]),
        },
    ];

    // `{sop_id}_sop_enabled` and `{sop_id}_sop_state` have to be in the state
    // the wrapper operations below are parsed against, so this goes here rather
    // than in `main`.
    state.extend_mut(generate_multi_sop_state_variables(&sops, TARGET), true);

    let auto_operations = vec![
        enable_sop(ROBOT_SOP, "done_robot", &state),
        enable_sop(GANTRY_SOP, "done_gantry", &state),
    ];

    let model = Model::new(sp_id, vec![], auto_operations, vec![], sops, vec![]);

    (model, state)
}

/// An operation whose "hardware" is the multi-SOP runner: it raises that SOP's
/// own enable flag and completes when that SOP's own status key says so.
///
/// One of these per procedure, sharing no key with the other - which is the
/// whole difference from `sop_sequence`, where the single wrapper has to own
/// `{sp_id}_sop_id` for the duration of the run.
fn enable_sop(sop_id: &str, done: &str, state: &State) -> Operation {
    Operation::new(
        &format!("enable_{sop_id}"),
        None,
        None,
        None,
        None,
        false,
        vec![Transition::parse(
            &format!("start_{sop_id}"),
            &format!("var:{done} == false"),
            "true",
            vec![
                &format!("var:{sop_id}_sop_state <- initial"),
                &format!("var:{sop_id}_sop_enabled <- true"),
            ],
            Vec::<&str>::new(),
            state,
        )],
        vec![Transition::parse(
            &format!("complete_{sop_id}"),
            "true",
            &format!("var:{sop_id}_sop_state == completed"),
            vec![
                &format!("var:{done} <- true"),
                &format!("var:{sop_id}_sop_state <- initial"),
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

    // Different durations, so the two SOPs finishing at different times shows up
    // in the log rather than having to be taken on trust.
    common::configure_emulator(&connection_manager, "robot", Some(400), DONT_EMULATE_FAILURE).await;
    common::configure_emulator(&connection_manager, "gantry", Some(800), DONT_EMULATE_FAILURE)
        .await;

    println!("Two SOPs at once: the robot walks a -> b while the gantry walks b -> a.");

    // Spawns `sop_multi_runner`; nothing extra to start here. Spawning a second
    // copy by hand would give each one its own instance of both SOPs, and the
    // robot and gantry would be driven twice.
    main_runner(&SP_ID.to_string(), model, 1, &connection_manager).await;

    let done = common::wait_until(
        &connection_manager,
        &["done_robot", "done_gantry"],
        40_000,
        |state| {
            state.get_value("done_robot", TARGET) == Some(true.to_spvalue())
                && state.get_value("done_gantry", TARGET) == Some(true.to_spvalue())
        },
    )
    .await;

    common::print_state(
        &connection_manager,
        "Final state",
        &[
            "done_robot",
            "done_gantry",
            "robot_position_estimated",
            "gantry_position_estimated",
            &format!("{ROBOT_SOP}_sop_state"),
            &format!("{GANTRY_SOP}_sop_state"),
            &format!("{SP_ID}_sop_runner_information"),
            // Untouched throughout: the single-SOP runner never got a job.
            &format!("{SP_ID}_sop_state"),
        ],
    )
    .await;

    common::finish(done, "both SOPs ran to completion side by side");
}
