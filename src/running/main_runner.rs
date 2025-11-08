use tokio::sync::mpsc;

use crate::{transforms::interface::tf_interface, *};
use std::sync::Arc;

// Run everything and provide a model
pub async fn main_runner(
    sp_id: &String,
    model: Model,
    connection_manager: &Arc<ConnectionManager>,
) {
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
    let con_clone = connection_manager.clone();
    let sp_id_clone = sp_id.clone();
    tokio::task::spawn(async move {
        planner_ticker(&sp_id_clone, &model_clone, &con_clone)
            .await
            .unwrap()
    });

    // log::info!(target:  &format!("{sp_id}_micro_sp"), "Spawning plan runner.");
    // let model_clone = model.clone();
    // let tx_clone = tx.clone();
    // let sp_id_clone = sp_id.clone();
    // tokio::task::spawn(async move { plan_runner(&sp_id_clone, &model_clone, tx_clone).await.unwrap() });

    let (op_diag_tx, op_diag_rx) = mpsc::channel::<OperationMsg>(100);
    log::info!(target: &format!("{sp_id}_micro_sp"), "Spawning operation diagnostics receiver.");
    let con_clone = connection_manager.clone();
    let sp_id_clone = sp_id.clone();
    tokio::task::spawn(async move {
        operation_diagnostics_receiver_task(op_diag_rx, &con_clone, &sp_id_clone)
            .await
    });

    log::info!(target:  &format!("{sp_id}_micro_sp"), "Spawning SOP runner.");
    let model_clone = model.clone();
    let con_clone = connection_manager.clone();
    let sp_id_clone = sp_id.clone();
    let op_diag_tx_clone = op_diag_tx.clone();
    tokio::task::spawn(async move {
        sop_runner(&sp_id_clone, &model_clone, op_diag_tx_clone, &con_clone)
            .await
            .unwrap()
    });

    log::info!(target:  &format!("{sp_id}_micro_sp"), "Spawning operation runner.");
    let model_clone = model.clone();
    let con_clone = connection_manager.clone();
    let op_diag_tx_clone = op_diag_tx.clone();
    tokio::task::spawn(async move {
        planned_operation_runner(&model_clone, op_diag_tx_clone, &con_clone)
            .await
            .unwrap()
    });

    log::info!(target: &format!("{sp_id}_micro_sp"), "Spawning auto transition runner");
    let model_clone = model.clone();
    let con_clone = connection_manager.clone();
    tokio::task::spawn(async move {
        auto_transition_runner(&model_clone.name, &model_clone, &con_clone)
            .await
            .unwrap()
    });

    log::info!(target: &format!("{sp_id}_micro_sp"), "Spawning auto operation runner");
    let model_clone = model.clone();
    let con_clone = connection_manager.clone();
    let op_diag_tx_clone = op_diag_tx.clone();
    tokio::task::spawn(async move {
        auto_operation_runner(&model_clone.name, &model_clone, op_diag_tx_clone, &con_clone)
            .await
            .unwrap()
    });

    log::info!(target: &format!("{sp_id}_micro_sp"), "Spawning time runner");
    let con_clone = connection_manager.clone();
    let sp_id_clone = sp_id.clone();
    tokio::task::spawn(async move {
        time_interface_runner(&sp_id_clone, &con_clone)
            .await
            .unwrap()
    });

    log::info!(target: &format!("{sp_id}_micro_sp"), "Spawning TF interface");
    let con_clone = connection_manager.clone();
    let sp_id_clone = sp_id.clone();
    tokio::task::spawn(async move { tf_interface(&sp_id_clone, &con_clone).await.unwrap() });

    // log::info!(target: &format!("{sp_id}_micro_sp"), "Spawning goal runner.");
    // let model_clone = model.clone();
    // let tx_clone = tx.clone();
    // let sp_id_clone = sp_id.clone();
    // tokio::task::spawn(async move { goal_runner(&sp_id_clone, &model_clone, tx_clone).await.unwrap() });

    // log::info!(target: &format!("{sp_id}_micro_sp"), "Spawning goal scheduler.");
    // let tx_clone = tx.clone();
    // let sp_id_clone = sp_id.clone();
    // tokio::task::spawn(async move { goal_scheduler(&sp_id_clone, tx_clone).await.unwrap() });
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
