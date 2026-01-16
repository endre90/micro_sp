use tokio::sync::mpsc;

use crate::{
    running::goal_runner::goal_runner,
    transforms::interface::tf_interface,
    *,
};
use std::sync::Arc;

// Run everything and provide a model
pub async fn main_runner(
    sp_id: &String,
    model: Model,
    goal_runner_enabled: bool,
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

    let (op_log_tx, op_log_rx) = mpsc::channel::<LogMsg>(100);
    log::info!(target: &format!("{sp_id}_micro_sp"), "Spawning operation logging receiver.");
    let con_clone = connection_manager.clone();
    let sp_id_clone = sp_id.clone();
    tokio::task::spawn(async move {
        operation_log_receiver_task(op_log_rx, &con_clone, &sp_id_clone).await
    });

    log::info!(target:  &format!("{sp_id}_micro_sp"), "Spawning SOP runner.");
    let model_clone = model.clone();
    let con_clone = connection_manager.clone();
    let sp_id_clone = sp_id.clone();
    let op_log_tx_clone = op_log_tx.clone();
    // let sop_log_tx_clone = sop_op_log_tx.clone();
    tokio::task::spawn(async move {
        sop_runner(&sp_id_clone, &model_clone, op_log_tx_clone, &con_clone)
            .await
            .unwrap()
    });

    log::info!(target:  &format!("{sp_id}_micro_sp"), "Spawning operation runner.");
    let model_clone = model.clone();
    let con_clone = connection_manager.clone();
    let op_log_tx_clone = op_log_tx.clone();
    // let sop_log_tx_clone = sop_op_log_tx.clone();
    tokio::task::spawn(async move {
        planned_operation_runner(&model_clone, op_log_tx_clone, &con_clone)
            .await
            .unwrap()
    });

    log::info!(target: &format!("{sp_id}_micro_sp"), "Spawning auto transition runner");
    let model_clone = model.clone();
    let con_clone = connection_manager.clone();
    let op_log_tx_clone = op_log_tx.clone();
    tokio::task::spawn(async move {
        auto_transition_runner(&model_clone.name, &model_clone, &con_clone, op_log_tx_clone)
            .await
            .unwrap()
    });

    log::info!(target: &format!("{sp_id}_micro_sp"), "Spawning auto operation runner");
    let model_clone = model.clone();
    let con_clone = connection_manager.clone();
    let op_log_tx_clone = op_log_tx.clone();
    // let sop_log_tx_clone = sop_op_log_tx.clone();
    tokio::task::spawn(async move {
        auto_operation_runner(
            &model_clone.name,
            &model_clone,
            op_log_tx_clone,
            // sop_log_tx_clone,
            &con_clone,
        )
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

    if goal_runner_enabled {
        log::info!(target: &format!("{sp_id}_micro_sp"), "Spawning time runner");
        let con_clone = connection_manager.clone();
        let sp_id_clone = sp_id.clone();
        tokio::task::spawn(async move { goal_runner(&sp_id_clone, &con_clone).await.unwrap() });
    }

    log::info!(target: &format!("{sp_id}_micro_sp"), "Spawning TF interface");
    let con_clone = connection_manager.clone();
    let sp_id_clone = sp_id.clone();
    tokio::task::spawn(async move { tf_interface(&sp_id_clone, &con_clone).await.unwrap() });
}
