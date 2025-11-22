use crate::{running::process_operation::OperationProcessingType, *};
use std::sync::Arc;
use tokio::{
    sync::mpsc,
    time::{Duration, interval},
};

static TICK_INTERVAL: u64 = 100; // millis

pub async fn sop_runner(
    sp_id: &str,
    model: &Model,
    diagnostics_tx: mpsc::Sender<LogMsg>,
    sop_op_diagnostics_tx: mpsc::Sender<LogMsg>,
    connection_manager: &Arc<ConnectionManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    initialize_env_logger();
    let mut interval = interval(Duration::from_millis(TICK_INTERVAL));
    let log_target = &format!("{}_sop_runner", sp_id);

    log::info!(target: log_target, "Online.");

    let mut old_sop_id = String::new();

    loop {
        interval.tick().await;
        if let Err(_) = connection_manager.check_redis_health(&log_target).await {
            continue;
        }
        let mut con = connection_manager.get_connection().await;
        let state = match StateManager::get_full_state(&mut con).await {
            Some(s) => s,
            None => continue,
        };

        let current_sop_id =
            state.get_string_or_default_to_unknown(&format!("{}_sop_id", sp_id), &log_target);

        if old_sop_id != current_sop_id && !current_sop_id.is_empty() {
            if let Some(root_sop) = model.sops.iter().find(|s| s.id == current_sop_id) {
                log::info!(target: &log_target, "Now executing new SOP '{}':", current_sop_id);
                log::info!(target: &log_target, "{:?}", visualize_sop(&root_sop.sop));
            }
            old_sop_id = current_sop_id;
        }

        let con_clone = con.clone();
        let new_state = process_sop_tick(
            sp_id,
            model,
            &state,
            con_clone,
            diagnostics_tx.clone(),
            sop_op_diagnostics_tx.clone(),
            &log_target,
        )
        .await?;
        let modified_state = state.get_diff_partial_state(&new_state);

        if !modified_state.state.is_empty() {
            StateManager::set_state(&mut con, &modified_state).await;
        }
    }
}

async fn process_sop_tick(
    sp_id: &str,
    model: &Model,
    state: &State,
    con: redis::aio::MultiplexedConnection,
    diagnostics_tx: mpsc::Sender<LogMsg>,
    sop_op_diagnostics_tx: mpsc::Sender<LogMsg>,
    log_target: &str,
) -> Result<State, Box<dyn std::error::Error>> {
    let mut new_state = state.clone();
    let mut sop_overall_state =
        state.get_string_or_default_to_unknown(&format!("{}_sop_state", sp_id), &log_target);

    match SOPState::from_str(&sop_overall_state) {
        SOPState::Initial => {
            handle_sop_initial(
                sp_id,
                state,
                &mut new_state,
                &mut sop_overall_state,
                &log_target,
            )?;
        }
        SOPState::Executing => {
            handle_sop_executing(
                sp_id,
                model,
                state,
                &mut new_state,
                &mut sop_overall_state,
                con,
                diagnostics_tx,
                sop_op_diagnostics_tx,
                &log_target,
            )
            .await;
        }
        SOPState::Completed | SOPState::Failed => {}
        SOPState::UNKNOWN => {
            // log::warn!(target: &log_target, "SOP in UNKNOWN state. Resetting.");
            sop_overall_state = SOPState::Initial.to_string();
        }
    }

    new_state = new_state.update(
        &format!("{}_sop_state", sp_id),
        sop_overall_state.to_spvalue(),
    );
    Ok(new_state)
}

fn handle_sop_initial(
    sp_id: &str,
    state: &State,
    new_state: &mut State,
    sop_overall_state: &mut String,
    log_target: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if state.get_bool_or_default_to_false(&format!("{}_sop_enabled", sp_id), &log_target) {
        log::info!(target: &log_target, "SOP enabled. Transitioning to Executing.");
        *new_state = new_state.update(&format!("{}_sop_enabled", sp_id), false.to_spvalue());
        *sop_overall_state = SOPState::Executing.to_string();
    }
    Ok(())
}

async fn handle_sop_executing(
    sp_id: &str,
    model: &Model,
    state: &State,
    new_state: &mut State,
    sop_overall_state: &mut String,
    con: redis::aio::MultiplexedConnection,
    diagnostics_tx: mpsc::Sender<LogMsg>,
    sop_op_diagnostics_tx: mpsc::Sender<LogMsg>,
    log_target: &str,
) {
    let sop_id = state.get_string_or_default_to_unknown(&format!("{}_sop_id", sp_id), &log_target);

    let Some(root_sop_container) = model.sops.iter().find(|s| s.id == sop_id) else {
        log::error!(target: &log_target, "SOP with id '{}' not found in model. Failing.", sop_id);
        *sop_overall_state = SOPState::Failed.to_string();
        return;
    };

    let updated_state = process_sop_node_tick(
        sp_id,
        state.clone(),
        &root_sop_container.sop,
        con,
        diagnostics_tx.clone(),
        sop_op_diagnostics_tx.clone(),
        &log_target,
    )
    .await;
    *new_state = updated_state;

    if is_sop_completed(sp_id, &root_sop_container.sop, new_state, &log_target) {
        log::info!(target: &log_target, "SOP root is complete. SOP Completed.");
        *sop_overall_state = SOPState::Completed.to_string();
    } else if is_sop_failed(sp_id, &root_sop_container.sop, new_state, &log_target) {
        log::error!(target: &log_target, "Fatal error detected in SOP. SOP Failed.");
        *sop_overall_state = SOPState::Failed.to_string();
    }
}

async fn process_sop_node_tick(
    sp_id: &str,
    mut state: State,
    sop: &SOP,
    con: redis::aio::MultiplexedConnection,
    diagnostics_tx: mpsc::Sender<LogMsg>,
    sop_op_diagnostics_tx: mpsc::Sender<LogMsg>,
    log_target: &str,
) -> State {
    if is_sop_completed(sp_id, sop, &state, log_target)
        || is_sop_failed(sp_id, sop, &state, log_target)
    {
        return state;
    }

    match sop {
        SOP::Operation(operation) => {
            state = running::process_operation::process_operation(
                &sp_id,
                state,
                operation,
                OperationProcessingType::SOP,
                None,
                None,
                diagnostics_tx,
                sop_op_diagnostics_tx,
                log_target,
            )
            .await;
        }

        SOP::Sequence(sops) => {
            // Find the first child that is not yet completed and process it
            if let Some(active_child) = sops
                .iter()
                .find(|child| !is_sop_completed(sp_id, child, &state, log_target))
            {
                state = Box::pin(process_sop_node_tick(
                    sp_id,
                    state,
                    active_child,
                    con,
                    diagnostics_tx,
                    sop_op_diagnostics_tx,
                    log_target,
                ))
                .await;
            }
        }

        SOP::Parallel(sops) => {
            // Process ALL children that are not yet completed
            for child in sops {
                // The state is threaded through each call, so updates from one branch
                // are visible to the next within the same tick
                state = Box::pin(process_sop_node_tick(
                    sp_id,
                    state,
                    child,
                    con.clone(),
                    diagnostics_tx.clone(),
                    sop_op_diagnostics_tx.clone(),
                    log_target,
                ))
                .await;
            }
        }

        SOP::Alternative(sops) => {
            // Check if a path is already active (i.e., not initial and not completed)
            let active_path = sops.iter().find(|child| {
                !is_sop_in_initial_state(sp_id, child, &state, log_target)
                    && !is_sop_completed(sp_id, child, &state, log_target)
            });

            if let Some(path) = active_path {
                // If a path is active, keep processing it
                state = Box::pin(process_sop_node_tick(
                    sp_id,
                    state,
                    path,
                    con,
                    diagnostics_tx,
                    sop_op_diagnostics_tx,
                    log_target,
                ))
                .await;
            } else {
                // If no path is active, find the first one that can start
                if let Some(path_to_start) = sops
                    .iter()
                    .find(|child| can_sop_start(sp_id, child, &state, log_target))
                {
                    log::info!(target: log_target, "Found valid alternative path to start.");
                    state = Box::pin(process_sop_node_tick(
                        sp_id,
                        state,
                        path_to_start,
                        con,
                        diagnostics_tx,
                        sop_op_diagnostics_tx,
                        log_target,
                    ))
                    .await;
                }
            }
        }
    }

    state
}

fn is_sop_failed(sp_id: &str, sop: &SOP, state: &State, log_target: &str) -> bool {
    match sop {
        SOP::Operation(operation) => {
            let op_state_str = state.get_string_or_default_to_unknown(&operation.name, &log_target);
            OperationState::from_str(&op_state_str) == OperationState::Fatal
        }
        SOP::Sequence(sops) | SOP::Parallel(sops) | SOP::Alternative(sops) => sops
            .iter()
            .any(|child_sop| is_sop_failed(sp_id, child_sop, state, &log_target)),
    }
}

fn is_sop_completed(sp_id: &str, sop: &SOP, state: &State, log_target: &str) -> bool {
    match sop {
        SOP::Operation(operation) => {
            let operation_state =
                state.get_string_or_default_to_unknown(&format!("{}", operation.name), &log_target);
            OperationState::from_str(&operation_state) == OperationState::Completed
        }
        SOP::Sequence(sops) | SOP::Parallel(sops) => sops
            .iter()
            .all(|child_sop| is_sop_completed(sp_id, child_sop, state, &log_target)),
        SOP::Alternative(sops) => sops
            .iter()
            .any(|child_sop| is_sop_completed(sp_id, child_sop, state, &log_target)),
    }
}

fn is_sop_in_initial_state(sp_id: &str, sop: &SOP, state: &State, log_target: &str) -> bool {
    match sop {
        SOP::Operation(operation) => {
            let operation_state =
                state.get_string_or_default_to_unknown(&format!("{}", operation.name), &log_target);
            let op_state = OperationState::from_str(&operation_state);
            op_state == OperationState::Initial || op_state == OperationState::UNKNOWN
        }
        SOP::Sequence(sops) | SOP::Parallel(sops) | SOP::Alternative(sops) => sops
            .iter()
            .all(|child_sop| is_sop_in_initial_state(sp_id, child_sop, state, &log_target)),
    }
}

fn can_sop_start(sp_id: &str, sop: &SOP, state: &State, log_target: &str) -> bool {
    match sop {
        SOP::Operation(operation) => {
            let operation_state =
                state.get_string_or_default_to_unknown(&format!("{}", operation.name), &log_target);
            (OperationState::from_str(&operation_state) == OperationState::Initial)
                && operation.eval(state, &log_target)
        }
        SOP::Sequence(sops) => sops.first().map_or(false, |first_sop| {
            can_sop_start(sp_id, first_sop, state, &log_target)
        }),
        SOP::Parallel(sops) => sops
            .iter()
            .all(|child_sop| can_sop_start(sp_id, child_sop, state, &log_target)),
        SOP::Alternative(sops) => sops
            .iter()
            .any(|child_sop| can_sop_start(sp_id, child_sop, state, &log_target)),
    }
}

pub fn uniquify_sop_operations(sop: SOP) -> SOP {
    match sop {
        SOP::Operation(op) => {
            let unique_id = nanoid::nanoid!(6);
            let new_name = format!("{}_{}", op.name, unique_id);
            SOP::Operation(Box::new(Operation {
                name: new_name,
                ..*op
            }))
        }
        SOP::Sequence(sops) => {
            SOP::Sequence(sops.into_iter().map(uniquify_sop_operations).collect())
        }
        SOP::Parallel(sops) => {
            SOP::Parallel(sops.into_iter().map(uniquify_sop_operations).collect())
        }
        SOP::Alternative(sops) => {
            SOP::Alternative(sops.into_iter().map(uniquify_sop_operations).collect())
        }
    }
}
