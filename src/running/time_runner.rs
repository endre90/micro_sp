use std::sync::Arc;

use crate::*;
use tokio::time::{Duration, interval};

static TICK_INTERVAL_MS: u64 = 100;

pub async fn time_interface_runner(
    sp_id: &str,
    connection_manager: &Arc<ConnectionManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = interval(Duration::from_millis(TICK_INTERVAL_MS));
    let log_target = format!("{}_time_interface", sp_id);

    log::info!(target: &log_target,  "Online.");

    // TODO: add more sleepers and other timer related commands, don't have jut one sleeper
    let keys: Vec<String> = vec![
        format!("{}_time_request_trigger", sp_id),
        format!("{}_time_request_state", sp_id),
        format!("{}_time_command", sp_id),
        format!("{}_time_duration_ms", sp_id),
        format!("{}_time_elapsed_ms", sp_id),
    ];

    loop {
        interval.tick().await;
        if let Err(_) = connection_manager.check_redis_health(&log_target).await {
            continue;
        }
        let mut con = connection_manager.get_connection().await;
        let state = match StateManager::get_state_for_keys(&mut con, &keys, &log_target).await {
            Some(s) => s,
            None => continue,
        };

        let mut request_trigger = state
            .get_bool_or_default_to_false(&format!("{}_time_request_trigger", sp_id), &log_target);

        let mut request_state = state.get_string_or_default_to_unknown(
            &format!("{}_time_request_state", sp_id),
            &log_target,
        );

        let command =
            state.get_string_or_default_to_unknown(&format!("{}_time_command", sp_id), &log_target);

        let duration_ms =
            state.get_int_or_default_to_zero(&format!("{}_time_duration_ms", sp_id), &log_target);

        let mut elapsed_ms =
            state.get_int_or_default_to_zero(&format!("{}_time_elapsed_ms", sp_id), &log_target);

        if request_trigger {
            request_trigger = false;
            if request_state == ActionRequestState::Initial.to_string() {
                match command.as_str() {
                    "sleep" => {
                        if duration_ms > 0 {
                            log::info!(target: &log_target, "Starting sleep timer for {} ms.", duration_ms);
                            request_state = ActionRequestState::Executing.to_string();
                            elapsed_ms = 0;
                        } else {
                            log::error!(target: &log_target, "Invalid sleep duration: {}. Must be > 0.", duration_ms);
                            request_state = ActionRequestState::Failed.to_string();
                        }
                    }
                    _ => {
                        log::error!(target: &log_target, "Time interface command '{}' is invalid.", command);
                        request_state = ActionRequestState::Failed.to_string();
                    }
                }
            }
        }

        if request_state == ActionRequestState::Executing.to_string() {
            elapsed_ms += TICK_INTERVAL_MS as i64;

            if elapsed_ms >= duration_ms {
                elapsed_ms = duration_ms;
                request_state = ActionRequestState::Succeeded.to_string();
                log::info!(target: &log_target, "Sleep timer finished.");
            }
        }

        let new_state = state
            .update(
                &format!("{}_time_request_trigger", sp_id),
                request_trigger.to_spvalue(),
            )
            .update(
                &format!("{}_time_request_state", sp_id),
                request_state.to_spvalue(),
            )
            .update(
                &format!("{}_time_elapsed_ms", sp_id),
                elapsed_ms.to_spvalue(),
            );

        let modified_state = state.get_diff_partial_state(&new_state);
        StateManager::set_state(&mut con, &modified_state).await;
    }
}
