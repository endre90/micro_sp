use chrono::{DateTime, Utc};
use colored::Colorize;
use console::measure_text_width;
use serde::{Deserialize, Serialize};
use std::{fmt::Write, sync::Arc};
use tokio::sync::mpsc;

use crate::{
    ConnectionManager, OperationState, SPValue, StateManager, StringOrUnknown, ToSPValue,
    running::process_operation::OperationProcessingType,
};

#[derive(Debug, Serialize, Deserialize)]
pub enum LogMsg {
    OperationMsg(OperationMsg),
    TransitionMsg(TransitionMsg),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OperationMsg {
    pub operation_name: String,
    pub operation_processing_type: OperationProcessingType,
    pub state: OperationState,
    pub timestamp: DateTime<Utc>,
    pub severity: log::Level,
    pub log: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransitionMsg {
    pub transition_name: String,
    pub timestamp: DateTime<Utc>,
    pub severity: log::Level,
    pub log: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OperationLog {
    pub operation_name: String,
    pub log: Vec<OperationMsg>,
}

// with agg for testing purposes, comment out the agg for running
pub async fn operation_log_receiver_task(
    mut rx: mpsc::Receiver<LogMsg>,
    connection_manager: &Arc<ConnectionManager>,
    sp_id: &str,
) {
    let log_target = format!("{}_logger_receiver", sp_id);
    while let Some(log_msg) = rx.recv().await {
        match log_msg {
            LogMsg::OperationMsg(msg) => {
                if let Err(_) = connection_manager.check_redis_health(&log_target).await {
                    continue;
                }
                let mut con = connection_manager.get_connection().await;

                let which_op_type_logger = match msg.operation_processing_type {
                    OperationProcessingType::Planned => {
                        format!("{}_logger_planned_operations", sp_id)
                    }
                    OperationProcessingType::Automatic => {
                        format!("{}_logger_automatic_operations", sp_id)
                    }
                    OperationProcessingType::SOP => format!("{}_logger_sop_operations", sp_id),
                };

                let agg_key = format!("{}_agg", which_op_type_logger);

                if let Some(log_spvalue) =
                    StateManager::get_sp_value(&mut con, &which_op_type_logger).await
                {
                    if let SPValue::String(StringOrUnknown::String(string_log)) = log_spvalue {
                        if let Ok(mut log) =
                            serde_json::from_str::<Vec<Vec<OperationLog>>>(&string_log)
                        {
                            let is_last_empty = log.last().map_or(true, |v| v.is_empty());

                            if is_last_empty {
                                if log.is_empty() {
                                    log.push(Vec::new());
                                }
                                log.last_mut().unwrap().push(OperationLog {
                                    operation_name: msg.operation_name.clone(),
                                    log: vec![msg],
                                });
                            } else {
                                let needs_partition = log.last().unwrap().iter().any(|op| {
                                    matches!(
                                        op.log.last().map(|s| &s.state),
                                        Some(
                                            OperationState::Completed
                                                | OperationState::Bypassed
                                                | OperationState::Fatal
                                                | OperationState::Cancelled
                                        )
                                    )
                                });

                                if needs_partition {
                                    log.push(Vec::new());
                                    let log_len = log.len();
                                    let (prefix, suffix) = log.split_at_mut(log_len - 1);
                                    let old_last_vec = &mut prefix[prefix.len() - 1];
                                    let new_last_vec = &mut suffix[0];
                                    let ops_to_partition = std::mem::take(old_last_vec);

                                    for op in ops_to_partition {
                                        match op.log.last().map(|s| &s.state) {
                                            Some(
                                                OperationState::Completed
                                                | OperationState::Bypassed
                                                | OperationState::Fatal
                                                | OperationState::Cancelled,
                                            ) => {
                                                old_last_vec.push(op);
                                            }
                                            _ => {
                                                new_last_vec.push(op);
                                            }
                                        }
                                    }
                                }

                                let last_vector = log.last_mut().unwrap();

                                match last_vector
                                    .iter_mut()
                                    .find(|log| log.operation_name == msg.operation_name)
                                {
                                    Some(exists) => {
                                        exists.log.push(msg);
                                    }
                                    None => {
                                        last_vector.push(OperationLog {
                                            operation_name: msg.operation_name.clone(),
                                            log: vec![msg],
                                        });
                                    }
                                }
                            }

                            if let Ok(serialized) = serde_json::to_string(&log) {
                                StateManager::set_sp_value(
                                    &mut con,
                                    &which_op_type_logger,
                                    &serialized.to_spvalue(),
                                )
                                .await;
                            }
                            // Aggregate log => has purpose for testing, but it might aggregate a lot and slow down redis?
                            let mut agg_log: Vec<Vec<Vec<OperationLog>>> =
                                if let Some(log_spvalue) =
                                    StateManager::get_sp_value(&mut con, &agg_key).await
                                {
                                    if let SPValue::String(StringOrUnknown::String(string_log)) =
                                        log_spvalue
                                    {
                                        serde_json::from_str(&string_log).unwrap_or_default()
                                    } else {
                                        Vec::new()
                                    }
                                } else {
                                    Vec::new()
                                };

                            agg_log.push(log);

                            if let Ok(serialized) = serde_json::to_string(&agg_log) {
                                StateManager::set_sp_value(
                                    &mut con,
                                    &agg_key,
                                    &serialized.to_spvalue(),
                                )
                                .await;
                            }
                        }
                    };
                }
            }
            LogMsg::TransitionMsg(msg) => {
                if let Err(_) = connection_manager.check_redis_health(&log_target).await {
                    continue;
                }
                let mut con = connection_manager.get_connection().await;
                let redis_key = format!("{}_logger_automatic_transitions", sp_id);
                let mut log: Vec<TransitionMsg> = if let Some(log_spvalue) =
                    StateManager::get_sp_value(&mut con, &redis_key).await
                {
                    if let SPValue::String(StringOrUnknown::String(string_log)) = log_spvalue {
                        serde_json::from_str(&string_log).unwrap_or_default()
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                };

                log.push(msg);

                if let Ok(serialized) = serde_json::to_string(&log) {
                    StateManager::set_sp_value(&mut con, &redis_key, &serialized.to_spvalue())
                        .await;
                }
            }
        }
    }
}

// pub async fn operation_log_receiver_task(
//     mut rx: mpsc::Receiver<LogMsg>,
//     // op_proc_type: OperationProcessingType,
//     connection_manager: &Arc<ConnectionManager>,
//     sp_id: &str,
// ) {
//     let log_target = format!("{}_logger_receiver", sp_id);
//     while let Some(log_msg) = rx.recv().await {
//         match log_msg {
//             LogMsg::OperationMsg(msg) => {
//                 if let Err(_) = connection_manager.check_redis_health(&log_target).await {
//                     continue;
//                 }
//                 let mut con = connection_manager.get_connection().await;

//                 let which_op_type_logger = match msg.operation_processing_type {
//                     OperationProcessingType::Planned => {
//                         &format!("{}_logger_planned_operations", sp_id)
//                     }
//                     OperationProcessingType::Automatic => {
//                         &format!("{}_logger_automatic_operations", sp_id)
//                     }
//                     OperationProcessingType::SOP => &format!("{}_logger_sop_operations", sp_id),
//                 };

//                 if let Some(log_spvalue) =
//                     StateManager::get_sp_value(&mut con, &which_op_type_logger).await
//                 {
//                     if let SPValue::String(StringOrUnknown::String(string_log)) = log_spvalue {
//                         if let Ok(mut log) =
//                             serde_json::from_str::<Vec<Vec<OperationLog>>>(&string_log)
//                         {
//                             let is_last_empty = log.last().map_or(true, |v| v.is_empty());

//                             if is_last_empty {
//                                 if log.is_empty() {
//                                     log.push(Vec::new());
//                                 }
//                                 log.last_mut().unwrap().push(OperationLog {
//                                     operation_name: msg.operation_name.clone(),
//                                     log: vec![msg],
//                                 });
//                             } else {
//                                 let needs_partition = log.last().unwrap().iter().any(|op| {
//                                     matches!(
//                                         op.log.last().map(|s| &s.state),
//                                         Some(
//                                             OperationState::Completed
//                                                 | OperationState::Bypassed
//                                                 | OperationState::Fatal
//                                                 | OperationState::Cancelled
//                                         )
//                                     )
//                                 });

//                                 if needs_partition {
//                                     log.push(Vec::new());
//                                     let log_len = log.len();
//                                     let (prefix, suffix) = log.split_at_mut(log_len - 1);

//                                     let old_last_vec = &mut prefix[prefix.len() - 1];
//                                     let new_last_vec = &mut suffix[0];

//                                     let ops_to_partition = std::mem::take(old_last_vec);

//                                     for op in ops_to_partition {
//                                         match op.log.last().map(|s| &s.state) {
//                                             Some(
//                                                 OperationState::Completed
//                                                 | OperationState::Bypassed
//                                                 | OperationState::Fatal | OperationState::Cancelled,
//                                             ) => {
//                                                 old_last_vec.push(op);
//                                             }
//                                             _ => {
//                                                 new_last_vec.push(op);
//                                             }
//                                         }
//                                     }
//                                 }

//                                 let last_vector = log.last_mut().unwrap();

//                                 match last_vector
//                                     .iter_mut()
//                                     .find(|log| log.operation_name == msg.operation_name)
//                                 {
//                                     Some(exists) => {
//                                         exists.log.push(msg);
//                                     }
//                                     None => {
//                                         last_vector.push(OperationLog {
//                                             operation_name: msg.operation_name.clone(),
//                                             log: vec![msg],
//                                         });
//                                     }
//                                 }
//                             }

//                             match serde_json::to_string(&log) {
//                                 Ok(serialized) => {
//                                     StateManager::set_sp_value(
//                                         &mut con,
//                                         &which_op_type_logger,
//                                         &serialized.to_spvalue(),
//                                     )
//                                     .await
//                                 }
//                                 Err(e) => {
//                                     log::error!(target: &log_target, "Serialization failed with {e}.")
//                                 }
//                             }
//                         }
//                     };
//                 }
//             }
//             LogMsg::TransitionMsg(msg) => {
//                 if let Err(_) = connection_manager.check_redis_health(&log_target).await {
//                     continue;
//                 }
//                 let mut con = connection_manager.get_connection().await;
//                 let redis_key = format!("{}_logger_automatic_transitions", sp_id);
//                 let mut log: Vec<TransitionMsg> = if let Some(log_spvalue) =
//                     StateManager::get_sp_value(&mut con, &redis_key).await
//                 {
//                     if let SPValue::String(StringOrUnknown::String(string_log)) = log_spvalue {
//                         serde_json::from_str(&string_log).unwrap_or_default()
//                     } else {
//                         Vec::new()
//                     }
//                 } else {
//                     Vec::new()
//                 };

//                 log.push(msg);

//                 match serde_json::to_string(&log) {
//                     Ok(serialized) => {
//                         StateManager::set_sp_value(&mut con, &redis_key, &serialized.to_spvalue())
//                             .await
//                     }
//                     Err(e) => {
//                         log::error!(target: &log_target, "Serialization failed for transition with {e}.")
//                     }
//                 }
//             }
//         }
//     }
// }

fn format_timestamp(utc_ts: &DateTime<Utc>) -> String {
    let cet = chrono::FixedOffset::east_opt(1 * 3600).unwrap();
    let cet_ts = utc_ts.with_timezone(&cet);
    cet_ts.format("%H:%M:%S%.3f").to_string()
}

pub fn format_log_rows(log_rows: &Vec<Vec<OperationLog>>) -> String {
    let mut output = String::new();
    let column_separator = " ".to_string();
    let total_rows = log_rows.len();

    for (i, row) in log_rows.iter().enumerate() {
        let header = if i == total_rows - 1 {
            "Running".to_string()
        } else {
            let relative_index = i as isize - (total_rows as isize - 1);
            format!("Done {}", relative_index)
        };

        if row.is_empty() {
            writeln!(&mut output, "(No logs in this row)").unwrap();
            continue;
        }

        let mut max_log_height = 0;
        let mut rendered_logs: Vec<Vec<String>> = Vec::new();
        let mut max_widths: Vec<usize> = Vec::new();
                    let max_line_width = 42;

        for op_log in row {
            let mut log_lines: Vec<String> = Vec::new();

            let title = format!("{}: {}", header, truncate_center(&op_log.operation_name, 30))
                .bold()
                .blue();
            let underline = format!("{:-<width$}", "", width = max_line_width);

            // let title_width = measure_text_width(&title.to_string());
            // let underline_width = measure_text_width(&underline);

            // let mut max_line_width = std::cmp::max(title_width, underline_width);

            log_lines.push(title.to_string());
            log_lines.push(underline.to_string());

            for msg in &op_log.log {
                let ts = format_timestamp(&msg.timestamp);

                let state_raw = format!("{:?}", msg.state);
                let state_colored = match msg.state {
                    OperationState::Initial => state_raw.green(),
                    OperationState::Executing => state_raw.green(),
                    OperationState::Completed => state_raw.green(),
                    OperationState::Timedout => state_raw.yellow(),
                    OperationState::Fatal => state_raw.red().bold(),
                    OperationState::Disabled => state_raw.yellow(),
                    OperationState::Bypassed => state_raw.yellow(),
                    OperationState::Failed => state_raw.yellow(),
                    OperationState::Cancelled => state_raw.yellow(),
                    OperationState::Terminated(_) => state_raw.green(),
                    // OperationState::Void => state_raw.green(),
                    OperationState::UNKNOWN => state_raw.red(),
                };

                let log_colored = match msg.severity {
                    log::Level::Error => msg.log.red().bold(),
                    log::Level::Warn => msg.log.yellow(),
                    _ => msg.log.normal(),
                };
                let colored_line = format!(
                    "[{ts} | {state:<10}] {log:<13.13}",
                    ts = ts.dimmed(),
                    state = state_colored,
                    log = log_colored
                );

                // let line_width = measure_text_width(&colored_line);
                // max_line_width = std::cmp::max(max_line_width, line_width);
                // max_line_width
                log_lines.push(colored_line);
            }

            max_log_height = std::cmp::max(max_log_height, log_lines.len());
            max_widths.push(max_line_width);
            rendered_logs.push(log_lines);
        }

        let mut padded_logs: Vec<Vec<String>> = Vec::new();
        for (mut log_lines, &width) in rendered_logs.into_iter().zip(&max_widths) {
            let mut padded_box_lines = Vec::new();

            for line in log_lines.iter_mut() {
                let current_width = measure_text_width(line);
                let padding = " ".repeat(width.saturating_sub(current_width));
                line.push_str(&padding);
            }

            while log_lines.len() < max_log_height {
                log_lines.push(" ".repeat(width));
            }

            let border_top = format!("+{:-<width$}+", "", width = width + 2);
            padded_box_lines.push(border_top.clone());
            for line in log_lines {
                padded_box_lines.push(format!("| {} |", line));
            }
            padded_box_lines.push(border_top);

            padded_logs.push(padded_box_lines);
        }

        for i in 0..(max_log_height + 2) {
            let mut line_to_print = String::new();
            for (col_idx, padded_log) in padded_logs.iter().enumerate() {
                line_to_print.push_str(&padded_log[i]);
                if col_idx < padded_logs.len() - 1 {
                    line_to_print.push_str(&column_separator);
                }
            }
            writeln!(&mut output, "{}", line_to_print).unwrap();
        }
    }

    output
}

pub fn format_transition_log(log: &Vec<TransitionMsg>) -> String {
    let mut output = String::new();

    if log.is_empty() {
        writeln!(&mut output, "(No transitions logged)").unwrap();
        return output;
    }

    let mut max_line_width;
    let mut rendered_lines: Vec<String> = Vec::new();

    let title = "Transitions".bold().blue();
    let underline = format!("{:-<width$}", "", width = "Transitions".len());

    max_line_width = std::cmp::max(
        measure_text_width(&title.to_string()),
        measure_text_width(&underline),
    );

    rendered_lines.push(title.to_string());
    rendered_lines.push(underline.to_string());

    for msg in log {
        let ts = format_timestamp(&msg.timestamp);

        let log_colored = match msg.severity {
            log::Level::Error => msg.log.red().bold(),
            log::Level::Warn => msg.log.yellow(),
            _ => msg.log.normal(),
        };

        let colored_line = format!(
            "[{ts}] {name}: {log}",
            ts = ts.dimmed(),
            name = msg.transition_name.blue(),
            log = log_colored
        );

        let line_width = measure_text_width(&colored_line);
        max_line_width = std::cmp::max(max_line_width, line_width);
        rendered_lines.push(colored_line);
    }

    let mut padded_box_lines = Vec::new();
    let border_top = format!("+{:-<width$}+", "", width = max_line_width + 2);

    padded_box_lines.push(border_top.clone());

    for mut line in rendered_lines {
        let current_width = measure_text_width(&line);
        let padding = " ".repeat(max_line_width.saturating_sub(current_width));
        line.push_str(&padding);
        padded_box_lines.push(format!("| {} |", line));
    }

    padded_box_lines.push(border_top);

    for line in padded_box_lines {
        writeln!(&mut output, "{}", line).unwrap();
    }

    output
}

fn truncate_center(input: &str, max_len: usize) -> String {
    if input.len() <= max_len {
        return input.to_string();
    }

    let separator = "...";
    let sep_len = separator.len();

    if max_len < sep_len {
        return input.chars().take(max_len).collect();
    }

    let available_chars = max_len - sep_len;
    let keep_left = available_chars / 2;
    let keep_right = available_chars - keep_left;

    format!(
        "{}{}{}",
        &input[..keep_left],
        separator,
        &input[input.len() - keep_right..]
    )
}

#[test]
fn test_log_formatter_with_colors() {
    let base_time = chrono::TimeZone::with_ymd_and_hms(&Utc, 2025, 11, 8, 15, 36, 0).unwrap();
    let ts = |sec, nano| {
        chrono::Timelike::with_nanosecond(
            &chrono::Timelike::with_second(&base_time, sec).unwrap(),
            nano,
        )
        .unwrap()
    };

    let op_name_1 = "op_emulate_timeout".to_string();
    let op_log_1 = OperationLog {
        operation_name: op_name_1.clone(),
        log: vec![
            OperationMsg {
                operation_name: op_name_1.clone(),
                operation_processing_type: OperationProcessingType::Automatic,
                state: OperationState::Initial,
                timestamp: ts(36, 769_744_256),
                severity: log::Level::Info,
                log: "Starting operation (i).".to_string(),
            },
            OperationMsg {
                operation_name: op_name_1.clone(),
                operation_processing_type: OperationProcessingType::Automatic,
                state: OperationState::Executing,
                timestamp: ts(36, 969_458_646),
                severity: log::Level::Info,
                log: "Waiting to be completed.".to_string(),
            },
            OperationMsg {
                operation_name: op_name_1.clone(),
                operation_processing_type: OperationProcessingType::Automatic,
                state: OperationState::Executing,
                timestamp: ts(37, 569_480_353),
                severity: log::Level::Warn,
                log: "Timeout for operation (e).".to_string(),
            },
            OperationMsg {
                operation_name: op_name_1.clone(),
                operation_processing_type: OperationProcessingType::Automatic,
                state: OperationState::Timedout,
                timestamp: ts(37, 769_515_608),
                severity: log::Level::Warn,
                log: "Operation timedout.".to_string(),
            },
            OperationMsg {
                operation_name: op_name_1.clone(),
                operation_processing_type: OperationProcessingType::Automatic,
                state: OperationState::Fatal,
                timestamp: ts(37, 969_277_368),
                severity: log::Level::Error,
                log: "Operation unrecoverable.".to_string(),
            },
        ],
    };

    let op_name_2 = "op_upload".to_string();
    let op_log_2 = OperationLog {
        operation_name: op_name_2.clone(),
        log: vec![
            OperationMsg {
                operation_name: op_name_2.clone(),
                operation_processing_type: OperationProcessingType::Automatic,
                state: OperationState::Initial,
                timestamp: ts(38, 100_000_000),
                severity: log::Level::Info,
                log: "Starting operation (i).".to_string(),
            },
            OperationMsg {
                operation_name: op_name_2.clone(),
                operation_processing_type: OperationProcessingType::Automatic,
                state: OperationState::Executing,
                timestamp: ts(38, 200_000_000),
                severity: log::Level::Info,
                log: "Waiting to be completed.".to_string(),
            },
            OperationMsg {
                operation_name: op_name_2.clone(),
                operation_processing_type: OperationProcessingType::Automatic,
                state: OperationState::Executing,
                timestamp: ts(39, 300_000_000),
                severity: log::Level::Info,
                log: "Uploading... 50%".to_string(),
            },
            OperationMsg {
                operation_name: op_name_2.clone(),
                operation_processing_type: OperationProcessingType::Automatic,
                state: OperationState::Completed,
                timestamp: ts(40, 400_000_000),
                severity: log::Level::Info,
                log: "Upload complete.".to_string(),
            },
        ],
    };

    let op_name_3 = "op_cleanup".to_string();
    let op_log_3 = OperationLog {
        operation_name: op_name_3.clone(),
        log: vec![
            OperationMsg {
                operation_name: op_name_3.clone(),
                operation_processing_type: OperationProcessingType::Automatic,
                state: OperationState::Initial,
                timestamp: ts(41, 0),
                severity: log::Level::Info,
                log: "Starting operation.".to_string(),
            },
            OperationMsg {
                operation_name: op_name_3.clone(),
                operation_processing_type: OperationProcessingType::Automatic,
                state: OperationState::Executing,
                timestamp: ts(41, 100_000_000),
                severity: log::Level::Info,
                log: "Waiting to be completed.".to_string(),
            },
            OperationMsg {
                operation_name: op_name_3.clone(),
                operation_processing_type: OperationProcessingType::Automatic,
                state: OperationState::Completed,
                timestamp: ts(41, 200_000_000),
                severity: log::Level::Info,
                log: "Cleanup complete.".to_string(),
            },
        ],
    };

    let all_log_rows: Vec<Vec<OperationLog>> = vec![vec![op_log_1, op_log_2], vec![op_log_3]];

    let formatted_string = format_log_rows(&all_log_rows);

    println!("{}", formatted_string);
}
