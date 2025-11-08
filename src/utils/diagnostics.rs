use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::{ConnectionManager, OperationState, SPValue, StateManager, StringOrUnknown, ToSPValue};

#[derive(Debug, Serialize, Deserialize)]
pub struct OperationMsg {
    pub operation_name: String,
    pub timestamp: DateTime<Utc>,
    pub severity: log::Level,
    pub state: OperationState,
    pub log: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OperationLog {
    pub operation_name: String,
    pub log: Vec<OperationMsg>,
}

pub async fn operation_diagnostics_receiver_task(
    mut rx: mpsc::Receiver<OperationMsg>,
    connection_manager: &Arc<ConnectionManager>,
    sp_id: &str,
) {
    let log_target = format!("{}_diagnostics_operations_receiver", sp_id);
    log::error!(target: &log_target, "1");
    while let Some(msg) = rx.recv().await {
        log::error!(target: &log_target, "2");
        if let Err(_) = connection_manager.check_redis_health(&log_target).await {
            log::error!(target: &log_target, "2x");
            continue;
        }
        log::error!(target: &log_target, "3");
        let mut con = connection_manager.get_connection().await;
        if let Some(log_spvalue) =
            StateManager::get_sp_value(&mut con, &format!("{}_diagnostics_operations", sp_id)).await
        {
            log::error!(target: &log_target, "4");
            if let SPValue::String(StringOrUnknown::String(string_log)) = log_spvalue {
                log::error!(target: &log_target, "5");
                if let Ok(mut log) = serde_json::from_str::<Vec<Vec<OperationLog>>>(&string_log) {
                    log::error!(target: &log_target, "6");
                    if let Some(last_vector) = log.last_mut() {
                        log::error!(target: &log_target, "7");
                        if last_vector.is_empty() {
                            log::error!(target: &log_target, "8");
                            last_vector.push(OperationLog {
                                operation_name: msg.operation_name.clone(),
                                log: vec![msg],
                            });
                        } else {
                            log::error!(target: &log_target, "9");
                            match last_vector
                                .iter_mut()
                                .find(|log| log.operation_name == msg.operation_name)
                            {
                                Some(exists) => {
                                    log::error!(target: &log_target, "10");
                                    exists.log.push(msg);
                                }
                                None => {
                                    log::error!(target: &log_target, "11");
                                    last_vector.push(OperationLog {
                                        operation_name: msg.operation_name.clone(),
                                        log: vec![msg],
                                    });
                                }
                            }
                        }
                        match serde_json::to_string(&log) {
                            Ok(serialized) => {
                                StateManager::set_sp_value(
                                    &mut con,
                                    &format!("{}_diagnostics_operations", sp_id),
                                    &serialized.to_spvalue(),
                                )
                                .await
                            }
                            Err(e) => {
                                log::error!(target: &log_target, "Serialization failed with {e}.")
                            }
                        }
                    }
                }
            };
        }
    }
}
