use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::*;
use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub enum RunnerState {
    Idle,
    Running,
    Stopped,
    Paused,
    UNKNOWN,
}

impl Default for RunnerState {
    fn default() -> Self {
        RunnerState::UNKNOWN
    }
}

impl RunnerState {
    pub fn from_str(x: &str) -> RunnerState {
        match x {
            "idle" => RunnerState::Idle,
            "running" => RunnerState::Running,
            "paused" => RunnerState::Paused,
            "stopped" => RunnerState::Stopped,
            _ => RunnerState::UNKNOWN,
        }
    }
    pub fn to_spvalue(self) -> SPValue {
        match self {
            RunnerState::Running => "running".to_spvalue(),
            RunnerState::Paused => "paused".to_spvalue(),
            RunnerState::Stopped => "stopped".to_spvalue(),
            RunnerState::Idle => "idle".to_spvalue(),
            RunnerState::UNKNOWN => "UNKNOWN".to_spvalue(),
        }
    }
}

impl fmt::Display for RunnerState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RunnerState::UNKNOWN => write!(f, "UNKNOWN"),
            RunnerState::Running => write!(f, "running"),
            RunnerState::Paused => write!(f, "paused"),
            RunnerState::Stopped => write!(f, "stopped"),
            RunnerState::Idle => write!(f, "idle"),
        }
    }
}

// Run everything and provide a model
pub async fn main_runner(sp_id: &String, model: Model, tx: mpsc::Sender<StateManagement>,) {
    // Logs from extern crates to stdout
    // initialize_env_logger();

    // // Enable coverability tracking:
    // let coverability_tracking = false;

    // // Add the variables that keep track of the runner state
    // let runner_vars = generate_runner_state_variables(&sp_id);
    // let state = state.extend(runner_vars, true);

    // let op_vars = generate_operation_state_variables(&model, coverability_tracking);
    // let state = state.extend(op_vars, true);

    log::info!(target: &format!("{sp_id}_micro_sp"), "Spawning planner.");
    let model_clone = model.clone();
    let tx_clone = tx.clone();
    let sp_id_clone = sp_id.clone();
    tokio::task::spawn(async move {
        planner_ticker(&sp_id_clone, &model_clone, tx_clone)
            .await
            .unwrap()
    });

    // log::info!(target:  &format!("{sp_id}_micro_sp"), "Spawning plan runner.");
    // let model_clone = model.clone();
    // let tx_clone = tx.clone();
    // let sp_id_clone = sp_id.clone();
    // tokio::task::spawn(async move { plan_runner(&sp_id_clone, &model_clone, tx_clone).await.unwrap() });

    // log::info!(target:  &format!("{sp_id}_micro_sp"), "Spawning SOP runner.");
    // let model_clone = model.clone();
    // let tx_clone = tx.clone();
    // let sp_id_clone = sp_id.clone();
    // tokio::task::spawn(async move { sop_runner(&sp_id_clone, &model_clone, tx_clone).await.unwrap() });

    log::info!(target:  &format!("{sp_id}_micro_sp"), "Spawning combined operation runner.");
    let model_clone = model.clone();
    let tx_clone = tx.clone();
    // let sp_id_clone = sp_id.clone();
    tokio::task::spawn(async move { planned_operation_runner(&model_clone, tx_clone).await.unwrap() });

    log::info!(target: &format!("{sp_id}_micro_sp"), "Spawning auto transition runner");
    let model_clone = model.clone();
    let tx_clone = tx.clone();
    tokio::task::spawn(async move {
        auto_transition_runner(&model_clone.name, &model_clone, tx_clone)
            .await
            .unwrap()
    });

    log::info!(target: &format!("{sp_id}_micro_sp"), "Spawning goal runner.");
    let model_clone = model.clone();
    let tx_clone = tx.clone();
    let sp_id_clone = sp_id.clone();
    tokio::task::spawn(async move { goal_runner(&sp_id_clone, &model_clone, tx_clone).await.unwrap() });

    log::info!(target: &format!("{sp_id}_micro_sp"), "Spawning goal scheduler.");
    let tx_clone = tx.clone();
    let sp_id_clone = sp_id.clone();
    tokio::task::spawn(async move { goal_scheduler(&sp_id_clone, tx_clone).await.unwrap() });
}

// pub async fn high_level_runner(
//     sp_id: &str,
//     model: &Model,
//     command_sender: mpsc::Sender<StateManagement>,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let mut interval = interval(Duration::from_millis(100));

//     log::info!(target: &&format!("{}_high_level_runner", sp_id), "Online.");

//     // For nicer logging
//     let mut plan_current_step_old = 0;
//     let mut planner_information_old = "".to_string();
//     let mut operation_state_old = "".to_string();
//     let mut operation_information_old = "".to_string();
//     let mut current_goal_state_old = "".to_string();
//     let mut plan_old: Vec<String> = vec![];

//     loop {
//         let (response_tx, response_rx) = oneshot::channel();
//         command_sender
//             .send(StateManagement::GetState(response_tx))
//             .await?;
//         let state = response_rx.await?;

//         let mut high_level_runner_state = state.get_string_or_default_to_unknown(
//             &format!("{}_high_level_runner", sp_id),
//             &format!("{}_high_level_runner_state", sp_id),
//         );

//         match RunnerState::from_str(&high_level_runner_state) {
//             RunnerState::Idle => {
//                 log::info!(target: &&format!("{sp_id}_high_level_runner"), "Micro SP is: Idle.");
//             }
//             RunnerState::Running => {
//                 log::info!(target: &&format!("{sp_id}_high_level_runner"), "Micro SP is: Running.");

//             }

//         }

//         interval.tick().await;

//     }

// }
