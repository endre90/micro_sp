//! [`SOP::Alternative`]: branches that race, first one home wins.
//!
//! An `Alternative` node offers several ways of getting the same thing done and
//! completes as soon as *any one* of them has. It is how a procedure expresses
//! a fallback: try the preferred route, and if it cannot start, take another.
//!
//! Three branches here, all moving the robot. The first is deliberately
//! blocked - its precondition guard begins with a literal `FALSE`, so it can
//! never be enabled - which stands in for a route that is unavailable right
//! now: a fixture in use, a tool not mounted, a station in manual mode. The
//! other two are live, and the node closes when whichever of them wins the
//! robot finishes.
//!
//! Because all three branches want the *same* robot, they contend on its
//! `request_trigger` handshake: only one can hold it at a time. That is the
//! difference from `sop_parallel`, where each branch had a resource to itself.
//!
//! Run with:
//!
//! ```text
//! docker compose up -d
//! cargo run --example sop_alternative
//! ```

mod common;

use common::{DONT_EMULATE_FAILURE, SP_ID, TARGET};
use micro_sp::*;

fn model(sp_id: &str, state: &State) -> (Model, State) {
    let state = state.clone();

    let done = bv!("done");
    let state = state.add(assign!(done, false.to_spvalue()), TARGET);

    let sops = vec![SOPStruct {
        id: "sop_move_robot".to_string(),
        sop: SOP::Alternative(vec![
            // Blocked: can never start, so the node has to fall through to the
            // branches below it.
            robot_move_to_pos("a", &state, false),
            robot_move_to_pos("b", &state, true),
            robot_move_to_pos("c", &state, true),
        ]),
    }];

    let auto_operations = vec![enable_sop(sp_id, &state)];

    let model = Model::new(sp_id, vec![], auto_operations, vec![], sops, vec![]);

    (model, state)
}

/// Enables the SOP and completes when the tree reports `completed`.
fn enable_sop(sp_id: &str, state: &State) -> Operation {
    Operation::new(
        "sop_move_robot",
        None,
        None,
        None,
        None,
        false,
        vec![Transition::parse(
            "start_sop_move_robot",
            "var:done == false",
            "true",
            vec![
                &format!("var:{sp_id}_sop_id <- sop_move_robot"),
                &format!("var:{sp_id}_sop_state <- initial"),
                &format!("var:{sp_id}_sop_enabled <- true"),
            ],
            Vec::<&str>::new(),
            state,
        )],
        vec![Transition::parse(
            "complete_sop_move_robot",
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

/// One branch. With `available` false the guard is prefixed with `FALSE`, which
/// the DSL parses as a literal that never holds - the branch stays `disabled`
/// forever and the alternative has to be satisfied elsewhere.
fn robot_move_to_pos(pos: &str, state: &State, available: bool) -> SOP {
    let idle = "var:robot_request_state == initial && var:robot_request_trigger == false";
    let guard = if available {
        idle.to_string()
    } else {
        format!("FALSE && {idle}")
    };

    SOP::Operation(Box::new(Operation::new(
        &format!("robot_move_to_{pos}"),
        None,
        None,
        None,
        None,
        false,
        vec![Transition::parse(
            &format!("start_robot_move_to_{pos}"),
            &guard,
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

    println!("Three alternative routes; the first is blocked, so another has to close the node.");
    main_runner(&SP_ID.to_string(), model, 1, &connection_manager).await;

    let done = common::wait_until(&connection_manager, &["done"], 40_000, |state| {
        state.get_value("done", TARGET) == Some(true.to_spvalue())
    })
    .await;

    // `robot_position_estimated` says which branch won - never `a`, since that
    // branch could not start.
    common::print_state(
        &connection_manager,
        "Final state",
        &[
            "done",
            "robot_position_estimated",
            &format!("{SP_ID}_sop_state"),
        ],
    )
    .await;

    common::finish(done, "a live branch completed the alternative node");
}
