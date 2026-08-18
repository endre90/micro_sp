use micro_sp::{ConnectionManager, ServiceRequestState, StateManager, ToSPValue};
use rand::Rng;
use rand::prelude::SliceRandom;
use std::{sync::Arc, time::Duration};
use tokio::time::interval;

use super::super::EMULATOR_TICK_INTERVAL;

#[derive(Debug, Clone)]
pub struct GantryRequest {
    pub command: String,
    pub speed: f64,
    pub position: String,
    pub emulate_execution_time: i64,
    pub emulated_execution_time: i64,
    pub emulate_failure_rate: i64,
    pub emulated_failure_rate: i64,
    pub emulate_failure_cause: i64,
    pub emulated_failure_cause: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct GantryResponse {
    pub success: bool,
    pub failure_cause: String,
    pub info: String,
}

pub async fn gantry_emulator(
    connection_manager: &Arc<ConnectionManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = interval(Duration::from_millis(EMULATOR_TICK_INTERVAL));
    let log_target = "gantry_emulator";
    log::info!(target: &log_target, "Online.");

    let keys: Vec<String> = vec![
        "gantry_request_trigger",
        "gantry_request_state",
        "gantry_total_fail_counter",
        "gantry_subsequent_fail_counter",
        "gantry_command_command",
        "gantry_speed_command",
        "gantry_position_command",
        "gantry_position_estimated",
        "gantry_calibrated_estimated",
        "gantry_locked_estimated",
        "gantry_emulate_execution_time",
        "gantry_emulated_execution_time",
        "gantry_emulate_failure_rate",
        "gantry_emulated_failure_rate",
        "gantry_emulate_failure_cause",
        "gantry_emulated_failure_cause",
    ]
    .iter()
    .map(|k| k.to_string())
    .collect();

    loop {
        interval.tick().await;
        if connection_manager.check_redis_health(log_target).await.is_err() {
            continue;
        }
        let mut con = connection_manager.get_connection().await;
        let state = match StateManager::get_state_for_keys(&mut con, &keys, log_target).await {
            Some(s) => s,
            None => continue,
        };

        let mut request_trigger =
            state.get_bool_or_default_to_false("gantry_request_trigger", log_target);
        let mut request_state =
            state.get_string_or_default_to_unknown("gantry_request_state", log_target);

        let mut total_fail_counter =
            state.get_int_or_default_to_zero("gantry_total_fail_counter", log_target);
        let mut subsequent_fail_counter =
            state.get_int_or_default_to_zero("gantry_subsequent_fail_counter", log_target);
        let mut gantry_position_estimated =
            state.get_string_or_default_to_unknown("gantry_position_estimated", log_target);
        let mut gantry_calibrated_estimated =
            state.get_bool_or_default_to_false("gantry_calibrated_estimated", log_target);
        let mut gantry_locked_estimated =
            state.get_bool_or_default_to_false("gantry_locked_estimated", log_target);

        if request_trigger {
            request_trigger = false;
            if request_state == ServiceRequestState::Initial.to_string() {
                let emulated_failure_cause_sp_value = state
                    .get_array_or_default_to_empty("gantry_emulated_failure_cause", log_target);

                let emulated_failure_cause: Vec<String> = emulated_failure_cause_sp_value
                    .iter()
                    .filter(|val| val.is_string())
                    .map(|y| y.to_string())
                    .collect();

                let request = GantryRequest {
                    command: state
                        .get_string_or_default_to_unknown("gantry_command_command", log_target),
                    speed: state.get_float_or_default_to_zero("gantry_speed_command", log_target),
                    position: state
                        .get_string_or_default_to_unknown("gantry_position_command", log_target),
                    emulate_execution_time: state
                        .get_int_or_default_to_zero("gantry_emulate_execution_time", log_target),
                    emulated_execution_time: state
                        .get_int_or_default_to_zero("gantry_emulated_execution_time", log_target),
                    emulate_failure_rate: state
                        .get_int_or_default_to_zero("gantry_emulate_failure_rate", log_target),
                    emulated_failure_rate: state
                        .get_int_or_default_to_zero("gantry_emulated_failure_rate", log_target),
                    emulate_failure_cause: state
                        .get_int_or_default_to_zero("gantry_emulate_failure_cause", log_target),
                    emulated_failure_cause,
                };

                let response = emulate_gantry_operation(&request).await;

                request_state = if response.success {
                    subsequent_fail_counter = 0;
                    match request.command.as_str() {
                        "move" => gantry_position_estimated = request.position,
                        "calibrate" => gantry_calibrated_estimated = true,
                        "lock" => gantry_locked_estimated = true,
                        "unlock" => gantry_locked_estimated = false,
                        _ => (),
                    }
                    ServiceRequestState::Succeeded.to_string()
                } else {
                    subsequent_fail_counter += 1;
                    total_fail_counter += 1;
                    ServiceRequestState::Failed.to_string()
                };
            }
        }
        let new_state = state
            .update("gantry_request_trigger", request_trigger.to_spvalue())
            .update("gantry_request_state", request_state.to_spvalue())
            .update("gantry_total_fail_counter", total_fail_counter.to_spvalue())
            .update(
                "gantry_subsequent_fail_counter",
                subsequent_fail_counter.to_spvalue(),
            )
            .update(
                "gantry_position_estimated",
                gantry_position_estimated.to_spvalue(),
            )
            .update(
                "gantry_calibrated_estimated",
                gantry_calibrated_estimated.to_spvalue(),
            )
            .update(
                "gantry_locked_estimated",
                gantry_locked_estimated.to_spvalue(),
            );

        let modified_state = state.get_diff_partial_state(&new_state);
        StateManager::set_state(&mut con, &modified_state).await;
    }
}

pub async fn emulate_gantry_operation(request: &GantryRequest) -> GantryResponse {

    let mut fail = match request.emulate_failure_rate {
        0 => false, // Never fail
        1 => true,  // Always fail
        2 => rand::thread_rng().gen_range(0..=100) <= request.emulated_failure_rate as u64,
        _ => false,
    };

    match request.command.as_str() {
        "move" => {
            log::info!(target: "gantry_emulator", "Got request to move to {}.", request.position)
        }
        "calibrate" => log::info!(target: "gantry_emulator", "Got request to calibrate."),
        "lock" => log::info!(target: "gantry_emulator", "Got request to lock."),
        "unlock" => log::info!(target: "gantry_emulator", "Got request to unlock."),
        _ => {
            log::warn!(target: "gantry_emulator", "Unknown command: '{}'", request.command);
            fail = true;
        }
    };

    let delay_ms: u64 = match request.emulate_execution_time {
        0 => 0,
        1 => request.emulated_execution_time as u64,
        2 => rand::thread_rng().gen_range(0..=request.emulated_execution_time) as u64,
        _ => 0,
    };
    if delay_ms > 0 {
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    }

    let cause = if fail {
        match request.emulate_failure_cause {
            0 => "generic_failure".to_string(),
            1 => request
                .emulated_failure_cause
                .get(0)
                .cloned()
                .unwrap_or_else(|| "config_error".to_string()),
            2 => request
                .emulated_failure_cause
                .choose(&mut rand::thread_rng())
                .cloned()
                .unwrap_or_else(|| "random_error".to_string()),
            _ => "generic_failure".to_string(),
        }
    } else {
        "".to_string()
    };

    let (success_info, failure_info) = match request.command.as_str() {
        "move" => (
            format!("Succeeded to move to {}.", request.position),
            format!("Failed to move to {} due to {}.", request.position, cause),
        ),
        "calibrate" => (
            "Succeeded to calibrate.".to_string(),
            format!("Failed to calibrate due to {}.", cause),
        ),
        "lock" => (
            "Succeeded to lock.".to_string(),
            format!("Failed to lock due to {}.", cause),
        ),
        "unlock" => (
            "Succeeded to unlock.".to_string(),
            format!("Failed to unlock due to {}.", cause),
        ),
        _ => (
            "Failed, unknown command".to_string(),
            "Failed, unknown command".to_string(),
        ),
    };

    if !fail {
        log::info!(target: "gantry_emulator", "{}", success_info);
        GantryResponse {
            success: true,
            failure_cause: "".to_string(),
            info: success_info,
        }
    } else {
        log::error!(target: "gantry_emulator", "{}", failure_info);
        GantryResponse {
            success: false,
            failure_cause: cause,
            info: failure_info,
        }
    }
}
