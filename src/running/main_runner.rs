use tokio::sync::mpsc;

use crate::{running::goal_runner::goal_runner, transforms::interface::tf_interface, *};
use std::sync::Arc;

// Run everything and provide a model
//
// PERF: `model.clone()` is done five times here (and each spawned runner then
// clones it *again* internally), so the process holds ~10 deep copies of every
// operation, transition, predicate and action for its whole lifetime. Suggested:
// wrap it once - `let model = Arc::new(model);` - and hand each task an
// `Arc::clone`. Same for `sp_id`, which is cloned per task.
//
// PERF (the big architectural one): seven independent tasks each poll Redis on
// their own timer - 50 ms, 100 ms x4, 200 ms x2, 250 ms, 500 ms. Together they
// issue roughly 40-60 PINGs, ~20 blocking `KEYS *` scans and a similar number of
// MGET/MSET pairs per second, whether or not anything is happening. That is the
// idle CPU floor. And because a single logical step (auto operation fires ->
// SOP runner observes -> plan runner reacts) has to travel through Redis
// between each pair of runners, the observed reaction time is the *sum* of the
// intervening tick intervals, not the fastest one. Two directions worth taking:
//   1. Event-driven: have each runner subscribe to changes on its own key set
//      (Redis keyspace notifications or an explicit PUBLISH on write) and use
//      `tokio::select!` between that stream and a slow watchdog tick. Latency
//      then tracks the actual Redis round trip (sub-millisecond locally)
//      instead of the tick period, and idle CPU drops to near zero.
//   2. Co-locate the tightly coupled runners: `sop_runner`,
//      `auto_operation_runner` and `planned_operation_runner` all read the same
//      full state and all write operation variables. Running them as three
//      phases of one loop over one shared in-memory `State` would remove two of
//      the three full-state reads per cycle *and* the Redis hop between them,
//      leaving Redis as the external interface/persistence layer rather than
//      the inter-runner message bus.
// A middle ground that needs no restructuring: keep one authoritative in-memory
// `State` per process behind an `Arc<RwLock<State>>`, have exactly one task
// sync it with Redis, and let the runners read from memory and publish deltas.
pub async fn main_runner(
    sp_id: &String,
    model: Model,
    number_of_timers: u64,
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

    // let (op_log_tx, op_log_rx) = mpsc::channel::<LogMsg>(100);
    // log::info!(target: &format!("{sp_id}_micro_sp"), "Spawning operation logging receiver.");
    // let con_clone = connection_manager.clone();
    // let sp_id_clone = sp_id.clone();
    // tokio::task::spawn(async move {
    //     operation_log_receiver_task(op_log_rx, &con_clone, &sp_id_clone).await
    // });

    log::info!(target:  &format!("{sp_id}_micro_sp"), "Spawning SOP runner.");
    let model_clone = model.clone();
    let con_clone = connection_manager.clone();
    let sp_id_clone = sp_id.clone();
    // let op_log_tx_clone = op_log_tx.clone();
    // let sop_log_tx_clone = sop_op_log_tx.clone();
    tokio::task::spawn(async move {
        sop_runner(&sp_id_clone, &model_clone, &con_clone)
            .await
            .unwrap()
    });

    log::info!(target:  &format!("{sp_id}_micro_sp"), "Spawning operation runner.");
    let model_clone = model.clone();
    let con_clone = connection_manager.clone();
    // let op_log_tx_clone = op_log_tx.clone();
    // let sop_log_tx_clone = sop_op_log_tx.clone();
    tokio::task::spawn(async move {
        planned_operation_runner(&model_clone, &con_clone)
            .await
            .unwrap()
    });

    log::info!(target: &format!("{sp_id}_micro_sp"), "Spawning auto transition runner");
    let model_clone = model.clone();
    let con_clone = connection_manager.clone();
    // let op_log_tx_clone = op_log_tx.clone();
    tokio::task::spawn(async move {
        auto_transition_runner(&model_clone.name, &model_clone, &con_clone)
            .await
            .unwrap()
    });

    log::info!(target: &format!("{sp_id}_micro_sp"), "Spawning auto operation runner");
    let model_clone = model.clone();
    let con_clone = connection_manager.clone();
    // let op_log_tx_clone = op_log_tx.clone();
    // let sop_log_tx_clone = sop_op_log_tx.clone();
    tokio::task::spawn(async move {
        auto_operation_runner(
            &model_clone.name,
            &model_clone,
            // op_log_tx_clone,
            // sop_log_tx_clone,
            &con_clone,
        )
        .await
        .unwrap()
    });

    // log::info!(target: &format!("{sp_id}_micro_sp"), "Spawning mutexed auto operation runner");
    // let model_clone = model.clone();
    // let con_clone = connection_manager.clone();
    // let op_log_tx_clone = op_log_tx.clone();
    // // let sop_log_tx_clone = sop_op_log_tx.clone();
    // tokio::task::spawn(async move {
    //     mutexed_auto_operation_runner(
    //         &model_clone.name,
    //         &model_clone,
    //         op_log_tx_clone,
    //         // sop_log_tx_clone,
    //         &con_clone,
    //     )
    //     .await
    //     .unwrap()
    // });

    log::info!(target: &format!("{sp_id}_micro_sp"), "Spawning time runner");
    let con_clone = connection_manager.clone();
    let sp_id_clone = sp_id.clone();
    tokio::task::spawn(async move {
        time_interface_runner(&sp_id_clone, &con_clone, number_of_timers)
            .await
            .unwrap()
    });

    log::info!(target: &format!("{sp_id}_micro_sp"), "Spawning time runner");
    let con_clone = connection_manager.clone();
    let sp_id_clone = sp_id.clone();
    tokio::task::spawn(async move { goal_runner(&sp_id_clone, &con_clone).await.unwrap() });

    log::info!(target: &format!("{sp_id}_micro_sp"), "Spawning TF interface");
    let con_clone = connection_manager.clone();
    let sp_id_clone = sp_id.clone();
    tokio::task::spawn(async move { tf_interface(&sp_id_clone, &con_clone).await.unwrap() });
}
