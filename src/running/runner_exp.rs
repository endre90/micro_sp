use crate::*;
use tokio::time::{interval, Duration};
use std::sync::{Arc, Mutex};

pub async fn simple_operation_runner(
    name: &str,
    model: &Model,
    shared_state: &Arc<Mutex<State>>,
    coverability_tracking: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = interval(Duration::from_millis(100));
    let model = model.clone();

    // Add the variables that keep track of the runner state
    let planner_state_vars = generate_runner_state_variables(&model, name, coverability_tracking);
    let shared_state_local = shared_state.lock().unwrap().clone();
    let updated_state = shared_state_local.extend(planner_state_vars, true);
    *shared_state.lock().unwrap() = updated_state.clone();

    loop {
        let mut shared_state_local = shared_state.lock().unwrap().clone();
        let runner_state =
            match shared_state_local.get_value(&&format!("{}_runner_state", name)) {
                SPValue::String(value) => value,
                _ => {
                    log::error!(target: &&format!("{}_runner", name), 
                "Couldn't get '{}_runner_state' from the shared state.", name);
                    "unknown".to_string()
                }
            };
        let plan_state =
            match shared_state_local.get_value(&&format!("{}_plan_state", name)) {
                SPValue::String(value) => value,
                _ => {
                    log::error!(target: &&format!("{}_runner", name), 
                "Couldn't get '{}_plan_state' from the shared state.", name);
                    "unknown".to_string()
                }
            };
        let goal_exists =
            match shared_state_local.get_value(&&format!("{}_goal_exists", name)) {
                SPValue::Bool(value) => value,
                _ => {
                    log::error!(target: &&format!("{}_runner", name), 
                "Couldn't get '{}_goal_exists' from the shared state.", name);
                    false
                }
            };
        match RunnerState::from_str(&runner_state) {
            RunnerState::Idle => {
                log::info!(target: &&format!("{}_runner", name), "Current state of the runner: Idle.");
                if goal_exists {
                    shared_state_local = shared_state_local.update(name, val)
                }
            },
            RunnerState::Running => {
                log::info!(target: &&format!("{}_runner", name), "Current state of the runner: Running.");
            },
            RunnerState::Paused => {
                log::info!(target: &&format!("{}_runner", name), "Current state of the runner: Paused.");
            },
            RunnerState::Stopped => {
                log::info!(target: &&format!("{}_runner", name), "Current state of the runner: Stopped.");
            },
            RunnerState::UNKNOWN => {
                log::info!(target: &&format!("{}_runner", name), "Current state of the runner: UNKNOWN.");
            },
        }

        interval.tick().await;
    }
}