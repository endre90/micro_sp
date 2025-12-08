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
    logging_tx: mpsc::Sender<LogMsg>,
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

        let mut new_state = state.clone();
        let sop_state =
            state.get_string_or_default_to_unknown(&format!("{}_sop_state", sp_id), &log_target);

        let sop_enabled =
            state.get_bool_or_default_to_false(&format!("{}_sop_enabled", sp_id), &log_target);

        let sop_id =
            state.get_string_or_default_to_unknown(&format!("{}_sop_id", sp_id), &log_target);

        let Some(root_sop_container) = model.sops.iter().find(|s| s.id == sop_id) else {
            log::debug!(target: &log_target, "SOP with id '{}' not found in model. Skipping evaluation.", sop_id);
            continue;
        };

        if old_sop_id != sop_id && !sop_id.is_empty() {
            if let Some(root_sop) = model.sops.iter().find(|s| s.id == sop_id) {
                log::info!(target: &log_target, "Now executing new SOP '{}':", sop_id);
                log::info!(target: &log_target, "{:?}", visualize_sop(&root_sop.sop));

                let terminated_triggers: Vec<&String> = state
                    .state
                    .iter()
                    .filter_map(|(key, value)| {
                        if let SPValue::String(StringOrUnknown::String(s)) = &value.val {
                            if s == "terminated_completed" {
                                return Some(key);
                            }
                        }
                        None
                    })
                    .collect();

                if !terminated_triggers.is_empty() {
                    let keys_to_remove: Vec<String> = state
                        .state
                        .keys()
                        .filter(|key| {
                            terminated_triggers
                                .iter()
                                .any(|trigger| key.contains(trigger.as_str()))
                        })
                        .cloned()
                        .collect();

                    if !keys_to_remove.is_empty() {
                        StateManager::remove_sp_values(&mut con, &keys_to_remove).await;
                    }
                }
            }
            old_sop_id = sop_id.clone();
        }

        match SOPState::from_str(&sop_state) {
            SOPState::Initial => {
                if sop_enabled {
                    log::info!(target: &log_target, "SOP {sop_id} enabled, starting execution.");
                    new_state = new_state
                        .update(&format!("{}_sop_enabled", sp_id), false.to_spvalue())
                        .update(
                            &format!("{}_sop_state", sp_id),
                            SOPState::Executing.to_string().to_spvalue(),
                        );
                }
            }
            SOPState::Executing => {
                // log::info!(target: &log_target, "SOP {sop_id} executing.");
                let con_clone = con.clone();
                new_state = process_sop_node_tick(
                    sp_id,
                    state.clone(),
                    &root_sop_container.sop,
                    con_clone,
                    logging_tx.clone(),
                    &log_target,
                )
                .await;
            }
            SOPState::Fatal => {
                log::info!(target: &log_target, "SOP {sop_id} Fatal.");
            }
            SOPState::Completed => {
                let con_clone = con.clone();
                new_state = process_sop_node_tick(
                    sp_id,
                    state.clone(),
                    &root_sop_container.sop,
                    con_clone,
                    logging_tx.clone(),
                    &log_target,
                )
                .await;
                log::info!(target: &log_target, "SOP {sop_id} Completed.");
            }
            SOPState::Cancelled => {
                log::info!(target: &log_target, "SOP {sop_id} Cancelled.");
            }
            SOPState::Terminated => {
                log::info!(target: &log_target, "SOP {sop_id} Terminated. This should never print.");
            }
            SOPState::UNKNOWN => {
                log::info!(target: &log_target, "SOP {sop_id} UNKNOWN.");
            }
        }

        let modified_state = state.get_diff_partial_state(&new_state);

        if !modified_state.state.is_empty() {
            StateManager::set_state(&mut con, &modified_state).await;
        }
    }
}

async fn process_sop_node_tick(
    sp_id: &str,
    mut state: State,
    sop: &SOP,
    con: redis::aio::MultiplexedConnection,
    logging_tx: mpsc::Sender<LogMsg>,

    log_target: &str,
) -> State {
    match sop {
        SOP::Operation(operation) => {
            // log::info!("ticking process operation");
            state = running::process_operation::process_operation(
                &sp_id,
                state,
                operation,
                OperationProcessingType::SOP,
                None,
                None,
                logging_tx,
                log_target,
            )
            .await;
        }

        SOP::Sequence(sops) => {
            // log::info!("ticking process sequence");
            let active_child = sops
                .iter()
                .find(|child| child.get_state(&state, &log_target) != SOPState::Terminated);

            if let Some(child) = active_child {
                // log::info!("next shild should be: {:?}", child);
                state = Box::pin(process_sop_node_tick(
                    sp_id, state, child, con, logging_tx, log_target,
                ))
                .await;
            }
        }

        SOP::Parallel(_sops) => todo!(),
        // {
        //     // Process ALL children that are not yet completed
        //     for child in sops {
        //         // The state is threaded through each call, so updates from one branch
        //         // are visible to the next within the same tick
        //         state = Box::pin(process_sop_node_tick(
        //             sp_id,
        //             state,
        //             child,
        //             con.clone(),
        //             logging_tx.clone(),
        //             log_target,
        //         ))
        //         .await;
        //     }
        // }
        SOP::Alternative(_sops) => todo!(),
        //     {
        //         // Check if a path is already active (i.e., not initial and not completed)
        //         let active_path = sops.iter().find(|child| {
        //             !is_sop_in_initial_state(sp_id, child, &state, log_target)
        //                 && !is_sop_completed(sp_id, child, &state, log_target)
        //         });

        //         if let Some(path) = active_path {
        //             // If a path is active, keep processing it
        //             state = Box::pin(process_sop_node_tick(
        //                 sp_id, state, path, con, logging_tx, log_target,
        //             ))
        //             .await;
        //         } else {
        //             // If no path is active, find the first one that can start
        //             if let Some(path_to_start) = sops
        //                 .iter()
        //                 .find(|child| can_sop_start(sp_id, child, &state, log_target))
        //             {
        //                 log::info!(target: log_target, "Found valid alternative path to start.");
        //                 state = Box::pin(process_sop_node_tick(
        //                     sp_id,
        //                     state,
        //                     path_to_start,
        //                     con,
        //                     logging_tx,
        //                     log_target,
        //                 ))
        //                 .await;
        //             }
        //         }
        //     }
    }

    state
}

// might not even need this for alternative because the processoperation hanfldless all the logic
// fn can_sop_start(sp_id: &str, sop: &SOP, state: &State, log_target: &str) -> bool {
//     match sop {
//         SOP::Operation(operation) => {
//             // We can reuse get_state here to check for Initial, but we MUST check eval manually
//             let current_state = sop.get_state(&state, &log_target);
//             current_state == SOPState::Initial && operation.eval(state, log_target)
//         }
//         SOP::Sequence(sops) => sops.first().map_or(false, |first| {
//             can_sop_start(sp_id, first, state, log_target)
//         }),
//         SOP::Parallel(sops) => sops
//             .iter()
//             .all(|child| can_sop_start(sp_id, child, state, log_target)),
//         SOP::Alternative(sops) => sops
//             .iter()
//             .any(|child| can_sop_start(sp_id, child, state, log_target)),
//     }
// }

pub fn uniquify_sop_operations(sop: SOP) -> SOP {
    match sop {
        SOP::Operation(op) => {
            let unique_id = nanoid::nanoid!(10); // 64^10 unique ids
            let new_name = format!("op_{}_{}", op.name, unique_id);
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
