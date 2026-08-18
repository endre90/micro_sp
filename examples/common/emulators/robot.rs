use micro_sp::{ConnectionManager, ServiceRequestState, StateManager, ToSPValue};
use rand::Rng;
use rand::prelude::SliceRandom;
use std::{sync::Arc, time::Duration};
use tokio::time::interval;

use super::super::EMULATOR_TICK_INTERVAL;

#[derive(Debug, Clone)]
pub struct RobotRequest {
    pub command: String,
    pub speed: f64,
    pub position: String,
    pub emulate_execution_time: i64,
    pub emulated_execution_time: i64,
    pub emulate_failure_rate: i64,
    pub emulated_failure_rate: i64,
    pub emulate_failure_cause: i64,
    pub emulated_failure_cause: Vec<String>,
    pub emulate_mounted_tool: bool,
    pub emulated_mounted_tool: String,
}

#[derive(Debug, Clone)]
pub struct RobotResponse {
    pub success: bool,
    pub failure_cause: String,
    pub info: String,
    pub checked_mounted_tool: String,
}

pub async fn robot_emulator(
    connection_manager: &Arc<ConnectionManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = interval(Duration::from_millis(EMULATOR_TICK_INTERVAL));
    let log_target = "robot_emulator";
    log::info!(target: &log_target, "Online.");

    let keys: Vec<String> = vec![
        "robot_request_trigger",
        "robot_request_state",
        "robot_total_fail_counter",
        "robot_subsequent_fail_counter",
        "robot_command_command",
        "robot_speed_command",
        "robot_position_command",
        "robot_position_estimated",
        "robot_mounted_one_time_measured",
        "robot_emulate_execution_time",
        "robot_emulated_execution_time",
        "robot_emulate_failure_rate",
        "robot_emulated_failure_rate",
        "robot_emulate_failure_cause",
        "robot_emulated_failure_cause",
        "robot_emulate_mounted_tool",
        "robot_emulated_mounted_tool",
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
            state.get_bool_or_default_to_false("robot_request_trigger", log_target);
        let mut request_state =
            state.get_string_or_default_to_unknown("robot_request_state", log_target);

        let mut total_fail_counter =
            state.get_int_or_default_to_zero("robot_total_fail_counter", log_target);
        let mut subsequent_fail_counter =
            state.get_int_or_default_to_zero("robot_subsequent_fail_counter", log_target);
        let mut robot_position_estimated =
            state.get_string_or_default_to_unknown("robot_position_estimated", log_target);
        let mut robot_mounted_one_time_measured =
            state.get_string_or_default_to_unknown("robot_mounted_one_time_measured", log_target);

        if request_trigger {
            request_trigger = false;
            if request_state == ServiceRequestState::Initial.to_string() {
                let emulated_failure_cause_sp_value = state
                    .get_array_or_default_to_empty("robot_emulated_failure_cause", log_target);

                let emulated_failure_cause: Vec<String> = emulated_failure_cause_sp_value
                    .iter()
                    .filter(|val| val.is_string())
                    .map(|y| y.to_string())
                    .collect();

                let request = RobotRequest {
                    command: state
                        .get_string_or_default_to_unknown("robot_command_command", log_target),
                    speed: state.get_float_or_default_to_zero("robot_speed_command", log_target),
                    position: state
                        .get_string_or_default_to_unknown("robot_position_command", log_target),
                    emulate_execution_time: state
                        .get_int_or_default_to_zero("robot_emulate_execution_time", log_target),
                    emulated_execution_time: state
                        .get_int_or_default_to_zero("robot_emulated_execution_time", log_target),
                    emulate_failure_rate: state
                        .get_int_or_default_to_zero("robot_emulate_failure_rate", log_target),
                    emulated_failure_rate: state
                        .get_int_or_default_to_zero("robot_emulated_failure_rate", log_target),
                    emulate_mounted_tool: state
                        .get_bool_or_default_to_false("robot_emulate_mounted_tool", log_target),
                    emulated_mounted_tool: state.get_string_or_default_to_unknown(
                        "robot_emulated_mounted_tool",
                        &log_target,
                    ),
                    emulate_failure_cause: state
                        .get_int_or_default_to_zero("robot_emulate_failure_cause", log_target),
                    emulated_failure_cause,
                };

                let response = emulate_robot_operation(&request).await;

                request_state = if response.success {
                    subsequent_fail_counter = 0;
                    match request.command.as_str() {
                        "move" => robot_position_estimated = request.position,
                        "check_mounted_tool" => {
                            robot_mounted_one_time_measured = response.checked_mounted_tool
                        }
                        "pick" | "place" | "mount" | "unmount" => (),
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
            .update("robot_request_trigger", request_trigger.to_spvalue())
            .update("robot_request_state", request_state.to_spvalue())
            .update("robot_total_fail_counter", total_fail_counter.to_spvalue())
            .update(
                "robot_subsequent_fail_counter",
                subsequent_fail_counter.to_spvalue(),
            )
            .update(
                "robot_position_estimated",
                robot_position_estimated.to_spvalue(),
            )
            .update(
                "robot_mounted_one_time_measured",
                robot_mounted_one_time_measured.to_spvalue(),
            );

        let modified_state = state.get_diff_partial_state(&new_state);
        StateManager::set_state(&mut con, &modified_state).await;
    }
}

pub async fn emulate_robot_operation(request: &RobotRequest) -> RobotResponse {
    let mut fail = match request.emulate_failure_rate {
        0 => false,
        1 => true,
        2 => rand::thread_rng().gen_range(0..=100) <= request.emulated_failure_rate as u64,
        _ => false,
    };

    let mut checked_mounted_tool = "UNKNOWN".to_string();
    match request.command.as_str() {
        "move" => {
            log::info!(target: "robot_emulator", "Got request to move to {}.", request.position)
        }
        "pick" => log::info!(target: "robot_emulator", "Got request to pick."),
        "place" => log::info!(target: "robot_emulator", "Got request to place."),
        "mount" => log::info!(target: "robot_emulator", "Got request to mount."),
        "unmount" => log::info!(target: "robot_emulator", "Got request to unmount."),
        "check_mounted_tool" => {
            log::info!(target: "robot_emulator", "Got request to check mounted tool.");
            if !fail {
                if request.emulate_mounted_tool {
                    checked_mounted_tool = request.emulated_mounted_tool.clone()
                } else {
                    checked_mounted_tool = vec!["gripper_tool", "suction_tool", "none"]
                        .choose(&mut rand::thread_rng())
                        .unwrap()
                        .to_string();
                }
            }
        }
        _ => {
            log::warn!(target: "robot_emulator", "Unknown command: '{}'", request.command);
            fail = true;
        }
    };

    // log::warn!(target: "robot_emulator", "DELAY: '{}'", request.emulate_execution_time);

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
        "pick" => (
            "Succeeded to pick.".to_string(),
            format!("Failed to pick due to {}.", cause),
        ),
        "place" => (
            "Succeeded to place.".to_string(),
            format!("Failed to place due to {}.", cause),
        ),
        "mount" => (
            "Succeeded to mount tool.".to_string(),
            format!("Failed to mount tool due to {}.", cause),
        ),
        "unmount" => (
            "Succeeded to unmount tool.".to_string(),
            format!("Failed to unmount tool due to {}.", cause),
        ),
        "check_mounted_tool" => (
            "Succeeded to check mounted tool.".to_string(),
            format!("Failed to check mounted tool due to {}.", cause),
        ),
        _ => (
            "Failed, unknown command".to_string(),
            "Failed, unknown command".to_string(),
        ),
    };

    if !fail {
        log::info!(target: "robot_emulator", "{}", success_info);
        RobotResponse {
            success: true,
            failure_cause: "".to_string(),
            info: success_info,
            checked_mounted_tool,
        }
    } else {
        log::error!(target: "robot_emulator", "{}", failure_info);
        RobotResponse {
            success: false,
            failure_cause: cause,
            info: failure_info,
            checked_mounted_tool,
        }
    }
}
