//! Automatic transitions: rules, not tasks.
//!
//! An automatic transition is the simplest thing a model can contain - a guard
//! and a set of assignments, taken by `auto_transition_runner` the moment the
//! guard holds. There is no lifecycle, no timeout, no failure branch and
//! nothing to schedule it: it is a rule the system obeys continuously.
//!
//! This model blinks a light three times. `turn_lights_on` fires while the
//! counter is below three and the light is off; `turn_lights_off` fires
//! whenever the light is on. They chase each other every tick until the
//! counter runs out, and then the system goes quiet on its own.
//!
//! Note which half of the transition does the work. `guard` is the model
//! condition; `runner_guard` is the runtime one, and `runner_actions` are the
//! assignments that only a runner makes - so a *planner* searching this model
//! never sees the light change, but the running system does.
//!
//! Run with:
//!
//! ```text
//! docker compose up -d
//! cargo run --example auto_transitions
//! ```

mod common;

use common::{SP_ID, TARGET};
use micro_sp::*;

/// How many times the light should blink before the model settles.
const BLINKS: i64 = 3;

fn model(sp_id: &str, state: &State) -> (Model, State) {
    let state = state.clone();
    let mut auto_transitions = vec![];

    let counter = iv!("counter");
    let state = state.add(assign!(counter, 0.to_spvalue()), TARGET);

    let lights_on = bv!("lights_on");
    let state = state.add(assign!(lights_on, false.to_spvalue()), TARGET);

    auto_transitions.push(Transition::parse(
        "turn_lights_on",
        &format!("var:counter < {BLINKS}"), // guard: the model's own condition
        "var:lights_on == false",           // runner_guard: only true at runtime
        Vec::<&str>::new(),                 // actions: none the planner would see
        vec!["var:lights_on <- true", "var:counter += 1"], // runner_actions
        &state,
    ));

    auto_transitions.push(Transition::parse(
        "turn_lights_off",
        "true",
        "var:lights_on == true",
        Vec::<&str>::new(),
        vec!["var:lights_on <- false"],
        &state,
    ));

    let model = Model::new(sp_id, auto_transitions, vec![], vec![], vec![], vec![]);

    (model, state)
}

#[tokio::main]
async fn main() {
    let (model, domain) = model(SP_ID, &State::new());

    // No emulators: this model touches nothing outside its own two variables.
    let connection_manager = common::boot(SP_ID, &model, domain, 1).await;

    println!("Blinking the light {BLINKS} times, with nothing driving it.");
    main_runner(&SP_ID.to_string(), model, 1, &connection_manager).await;

    // The counter stops at BLINKS and the light ends up off - both halves
    // matter, since a counter at BLINKS with the light still on means the
    // second transition has not caught up yet.
    let settled = common::wait_until(
        &connection_manager,
        &["counter", "lights_on"],
        10_000,
        |state| {
            state.get_value("counter", TARGET) == Some(BLINKS.to_spvalue())
                && state.get_value("lights_on", TARGET) == Some(false.to_spvalue())
        },
    )
    .await;

    common::print_state(
        &connection_manager,
        "Final state",
        &["counter", "lights_on"],
    )
    .await;

    common::finish(settled, "the light blinked three times and the model went quiet");
}
