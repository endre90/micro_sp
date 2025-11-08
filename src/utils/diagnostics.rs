use std::{fmt::Write, sync::Arc};
use colored::Colorize;
use console::measure_text_width;
use chrono::{DateTime, TimeZone, Timelike, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::{ConnectionManager, OperationState, SPValue, StateManager, StringOrUnknown, ToSPValue};

#[derive(Debug, Serialize, Deserialize)]
pub struct OperationMsg {
    pub operation_name: String,
    pub state: OperationState,
    pub timestamp: DateTime<Utc>,
    pub severity: log::Level,
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
    while let Some(msg) = rx.recv().await {
        if let Err(_) = connection_manager.check_redis_health(&log_target).await {
            continue;
        }
        let mut con = connection_manager.get_connection().await;
        if let Some(log_spvalue) =
            StateManager::get_sp_value(&mut con, &format!("{}_diagnostics_operations", sp_id)).await
        {
            if let SPValue::String(StringOrUnknown::String(string_log)) = log_spvalue {
                if let Ok(mut log) = serde_json::from_str::<Vec<Vec<OperationLog>>>(&string_log) {
                    if let Some(last_vector) = log.last_mut() {
                        if last_vector.is_empty() {
                            last_vector.push(OperationLog {
                                operation_name: msg.operation_name.clone(),
                                log: vec![msg],
                            });
                        } else {
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

// fn format_timestamp(utc_ts: &DateTime<Utc>) -> String {
//     let cet = chrono::FixedOffset::east_opt(1 * 3600).unwrap();

//     // Convert the UTC DateTime into a DateTime with the new timezone
//     let cet_ts = utc_ts.with_timezone(&cet);

//     // Format the new timezone-aware DateTime
//     cet_ts.format("%H:%M:%S%.3f").to_string()
// }

// /// Main function to format the entire Vec<Vec<OperationLog>>
// fn format_log_rows(log_rows: &Vec<Vec<OperationLog>>) -> String {
//     let mut output = String::new();
//     let column_separator = " ".to_string();

//     for (i, row) in log_rows.iter().enumerate() {
//         writeln!(&mut output, "--- Log Row {} ---\n", i + 1).unwrap();

//         if row.is_empty() {
//             writeln!(&mut output, "(No logs in this row)").unwrap();
//             continue;
//         }

//         let mut max_log_height = 0;
//         let mut rendered_logs: Vec<Vec<String>> = Vec::new();
//         let mut max_widths: Vec<usize> = Vec::new();

//         // 1. First pass: Render
//         for op_log in row {
//             let mut log_lines: Vec<String> = Vec::new();
//             let title = format!("Operation: {}", op_log.operation_name);
//             let underline = format!("{:-<width$}", "", width = title.len().saturating_sub(10));

//             log_lines.push(title);
//             log_lines.push(underline);

//             let mut max_line_width = log_lines[0].len();

//             for msg in &op_log.log {
//                 let ts = format_timestamp(&msg.timestamp);

//                 // Format the message line (using log::log::Level)
//                 let line = format!(
//                     "[{ts} | {state:<9} | {sev:<5}] {log}",
//                     ts = ts,
//                     state = format!("{:?}", msg.state),
//                     sev = format!("{:?}", msg.severity), // This now formats the log::Level
//                     log = msg.log
//                 );

//                 max_line_width = std::cmp::max(max_line_width, line.len());
//                 log_lines.push(line);
//             }

//             max_log_height = std::cmp::max(max_log_height, log_lines.len());
//             max_widths.push(max_line_width);
//             rendered_logs.push(log_lines);
//         }

//         // 2. Second pass: Pad
//         let mut padded_logs: Vec<Vec<String>> = Vec::new();
//         for (mut log_lines, &width) in rendered_logs.into_iter().zip(&max_widths) {
//             let mut padded_box_lines = Vec::new();

//             for line in log_lines.iter_mut() {
//                 write!(
//                     line,
//                     "{:width$}",
//                     "",
//                     width = width.saturating_sub(line.len())
//                 )
//                 .unwrap();
//             }

//             while log_lines.len() < max_log_height {
//                 log_lines.push(" ".repeat(width));
//             }

//             let border_top = format!("+{:-<width$}+", "", width = width + 2);
//             padded_box_lines.push(border_top.clone());
//             for line in log_lines {
//                 padded_box_lines.push(format!("| {} |", line));
//             }
//             padded_box_lines.push(border_top);

//             padded_logs.push(padded_box_lines);
//         }

//         // 3. Final pass: Stitch
//         for i in 0..(max_log_height + 2) {
//             let mut line_to_print = String::new();
//             for (col_idx, padded_log) in padded_logs.iter().enumerate() {
//                 line_to_print.push_str(&padded_log[i]);
//                 if col_idx < padded_logs.len() - 1 {
//                     line_to_print.push_str(&column_separator);
//                 }
//             }
//             writeln!(&mut output, "{}", line_to_print).unwrap();
//         }
//         writeln!(&mut output, "\n").unwrap();
//     }

//     output
// }

// // --- 4. The Test Function (Updated) ---
// #[test]
// fn test_log_formatter() {
//     // Helper to create our DateTime<Utc+1> objects
//     let cet = chrono::FixedOffset::east_opt(1 * 3600).unwrap(); // CET is UTC+1

//     // Helper function to create timestamps from nanoseconds
//     let base_time = chrono::TimeZone::with_ymd_and_hms(&cet, 2025, 11, 8, 16, 36, 0).unwrap();
//     let ts = |sec, nano| {
//         chrono::Timelike::with_nanosecond(
//             &chrono::Timelike::with_second(&base_time, sec).unwrap(),
//             nano,
//         )
//         .unwrap()
//     };

//     // --- Build the sample data with new types ---

//     let op_name_1 = "op_emulate_timeout".to_string();
//     let op_log_1 = OperationLog {
//         operation_name: op_name_1.clone(),
//         log: vec![
//             OperationMsg {
//                 operation_name: op_name_1.clone(),
//                 state: OperationState::Initial,
//                 timestamp: ts(36, 769_744_256).into(),
//                 severity: log::log::Level::Info,
//                 log: "Starting initialized operation 'op_emulate_timeout'.".to_string(),
//             },
//             OperationMsg {
//                 operation_name: op_name_1.clone(),
//                 state: OperationState::Executing,
//                 timestamp: ts(36, 969_458_646).into(),
//                 severity: log::log::Level::Info,
//                 log: "Waiting for operation 'op_emulate_timeout' to be...".to_string(),
//             },
//             OperationMsg {
//                 operation_name: op_name_1.clone(),
//                 state: OperationState::Executing,
//                 timestamp: ts(37, 569_480_353).into(),
//                 severity: log::log::Level::Warn,
//                 log: "Timeout for executing operation 'op_emulate_timeout'.".to_string(),
//             },
//             OperationMsg {
//                 operation_name: op_name_1.clone(),
//                 state: OperationState::Timedout,
//                 timestamp: Utc::now(),
//                 severity: log::log::Level::Warn,
//                 log: "Operation 'op_emulate_timeout' timedout.".to_string(),
//             },
//             OperationMsg {
//                 operation_name: op_name_1.clone(),
//                 state: OperationState::Unrecoverable,
//                 timestamp: ts(37, 969_277_368).into(),
//                 severity: log::log::Level::Error,
//                 log: "Operation 'op_emulate_timeout' is unrecoverable...".to_string(),
//             },
//         ],
//     };

//     let op_name_2 = "op_upload".to_string();
//     let op_log_2 = OperationLog {
//         operation_name: op_name_2.clone(),
//         log: vec![
//             OperationMsg {
//                 operation_name: op_name_2.clone(),
//                 state: OperationState::Initial,
//                 timestamp: ts(38, 100_000_000).into(),
//                 severity: log::log::Level::Info,
//                 log: "Starting initialized operation 'op_upload'.".to_string(),
//             },
//             OperationMsg {
//                 operation_name: op_name_2.clone(),
//                 state: OperationState::Executing,
//                 timestamp: ts(38, 200_000_000).into(),
//                 severity: log::log::Level::Info,
//                 log: "Waiting for operation 'op_upload' to be...".to_string(),
//             },
//             OperationMsg {
//                 operation_name: op_name_2.clone(),
//                 state: OperationState::Executing,
//                 timestamp: ts(39, 300_000_000).into(),
//                 severity: log::log::Level::Info,
//                 log: "Uploading... 50%".to_string(),
//             },
//             OperationMsg {
//                 operation_name: op_name_2.clone(),
//                 state: OperationState::Completed,
//                 timestamp: ts(40, 400_000_000).into(),
//                 severity: log::log::Level::Info,
//                 log: "Upload complete.".to_string(),
//             },
//         ],
//     };

//     let op_name_3 = "op_cleanup".to_string();
//     let op_log_3 = OperationLog {
//         operation_name: op_name_3.clone(),
//         log: vec![
//             OperationMsg {
//                 operation_name: op_name_3.clone(),
//                 state: OperationState::Initial,
//                 timestamp: ts(41, 0).into(),
//                 severity: log::log::Level::Info,
//                 log: "Starting initialized operation 'op_cleanup'.".to_string(),
//             },
//             OperationMsg {
//                 operation_name: op_name_3.clone(),
//                 state: OperationState::Executing,
//                 timestamp: ts(41, 100_000_000).into(),
//                 severity: log::log::Level::Info,
//                 log: "Waiting for operation 'op_cleanup' to be completed.".to_string(),
//             },
//             OperationMsg {
//                 operation_name: op_name_3.clone(),
//                 state: OperationState::Completed,
//                 timestamp: ts(41, 200_000_000).into(),
//                 severity: log::log::Level::Info,
//                 log: "Cleanup complete.".to_string(),
//             },
//         ],
//     };

//     // Create the final nested Vec structure
//     let all_log_rows: Vec<Vec<OperationLog>> = vec![
//         vec![op_log_1, op_log_2], // Row 1: Two logs side-by-side
//         vec![op_log_3],           // Row 2: One log
//     ];

//     // Format and print!
//     let formatted_string = format_log_rows(&all_log_rows);

//     // This will print the output when you run `cargo test -- --nocapture`
//     println!("{}", formatted_string);

//     // Assertions
//     assert!(formatted_string.contains("Log Row 1"));
//     assert!(formatted_string.contains("op_emulate_timeout"));
//     assert!(formatted_string.contains("16:36:37.969")); // Check timestamp format
//     assert!(formatted_string.contains("| Error]")); // Check log::log::Level format
//     assert!(formatted_string.contains("| Warn ]"));
//     // assert!(formatted_ExampleString.contains("| Info ]"));
//     assert!(formatted_string.contains("op_upload"));
//     assert!(formatted_string.contains("Log Row 2"));
//     assert!(formatted_string.contains("op_cleanup"));
// }


fn format_timestamp(utc_ts: &DateTime<Utc>) -> String {
    let cet = chrono::FixedOffset::east_opt(1 * 3600).unwrap();
    let cet_ts = utc_ts.with_timezone(&cet);
    cet_ts.format("%H:%M:%S%.3f").to_string()
}

fn format_log_rows(log_rows: &Vec<Vec<OperationLog>>) -> String {
    let mut output = String::new();
    let column_separator = " ".to_string(); 

    for (i, row) in log_rows.iter().enumerate() {
        writeln!(&mut output, "--- Log Row {} ---\n", i + 1).unwrap();

        if row.is_empty() {
            writeln!(&mut output, "(No logs in this row)").unwrap();
            continue;
        }

        let mut max_log_height = 0;
        let mut rendered_logs: Vec<Vec<String>> = Vec::new();
        let mut max_widths: Vec<usize> = Vec::new();

        for op_log in row {
            let mut log_lines: Vec<String> = Vec::new();
            
            let title = format!("Operation: {}", op_log.operation_name).bold().cyan();
            let underline = format!("{:-<width$}", "", width = op_log.operation_name.len() + 1);

            let title_width = measure_text_width(&title.to_string());
            let underline_width = measure_text_width(&underline);
            let mut max_line_width = std::cmp::max(title_width, underline_width);

            log_lines.push(title.to_string());
            log_lines.push(underline.to_string());


            for msg in &op_log.log {
                let ts = format_timestamp(&msg.timestamp);
                
                let state_raw = format!("{:?}", msg.state);
                let sev_raw = format!("{:?}", msg.severity);

                let state_colored = match msg.state {
                    OperationState::Timedout => state_raw.yellow(),
                    OperationState::Unrecoverable => state_raw.red().bold(),
                    OperationState::Completed => state_raw.green(),
                    _ => state_raw.normal(),
                };

                let sev_colored = match msg.severity {
                    log::Level::Error => sev_raw.red().bold(),
                    log::Level::Warn => sev_raw.yellow().bold(),
                    log::Level::Info => sev_raw.cyan(),
                    log::Level::Debug => sev_raw.blue(),
                    log::Level::Trace => sev_raw.magenta(),
                };

                let log_colored = match msg.severity {
                    log::Level::Error => msg.log.red().bold(),
                    log::Level::Warn => msg.log.yellow(),
                    _ => msg.log.normal(),
                };

                let colored_line = format!(
                    "[{ts} | {state:<9.9} | {sev:<5}] {log}",
                    ts = ts.dimmed(),
                    state = state_colored,
                    sev = sev_colored,
                    log = log_colored
                );
                
                let line_width = measure_text_width(&colored_line);
                max_line_width = std::cmp::max(max_line_width, line_width);
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
        writeln!(&mut output, "\n").unwrap();
    }

    output
}


#[test]
fn test_log_formatter_with_colors() {
    let base_time = Utc.with_ymd_and_hms(2025, 11, 8, 15, 36, 0).unwrap();
    let ts = |sec, nano| base_time.with_second(sec).unwrap().with_nanosecond(nano).unwrap();

    let op_name_1 = "op_emulate_timeout".to_string();
    let op_log_1 = OperationLog {
        operation_name: op_name_1.clone(),
        log: vec![
            OperationMsg { operation_name: op_name_1.clone(), state: OperationState::Initial, timestamp: ts(36, 769_744_256), severity: log::Level::Info, log: "Starting initialized operation 'op_emulate_timeout'.".to_string() },
            OperationMsg { operation_name: op_name_1.clone(), state: OperationState::Executing, timestamp: ts(36, 969_458_646), severity: log::Level::Info, log: "Waiting for operation 'op_emulate_timeout' to be...".to_string() },
            OperationMsg { operation_name: op_name_1.clone(), state: OperationState::Executing, timestamp: ts(37, 569_480_353), severity: log::Level::Warn, log: "Timeout for executing operation 'op_emulate_timeout'.".to_string() },
            OperationMsg { operation_name: op_name_1.clone(), state: OperationState::Timedout, timestamp: ts(37, 769_515_608), severity: log::Level::Warn, log: "Operation 'op_emulate_timeout' timedout.".to_string() },
            OperationMsg { operation_name: op_name_1.clone(), state: OperationState::Unrecoverable, timestamp: ts(37, 969_277_368), severity: log::Level::Error, log: "Operation 'op_emulate_timeout' is unrecoverable...".to_string() },
        ],
    };

    let op_name_2 = "op_upload".to_string();
    let op_log_2 = OperationLog {
        operation_name: op_name_2.clone(),
        log: vec![
            OperationMsg { operation_name: op_name_2.clone(), state: OperationState::Initial, timestamp: ts(38, 100_000_000), severity: log::Level::Info, log: "Starting initialized operation 'op_upload'.".to_string() },
            OperationMsg { operation_name: op_name_2.clone(), state: OperationState::Executing, timestamp: ts(38, 200_000_000), severity: log::Level::Info, log: "Waiting for operation 'op_upload' to be...".to_string() },
            OperationMsg { operation_name: op_name_2.clone(), state: OperationState::Executing, timestamp: ts(39, 300_000_000), severity: log::Level::Info, log: "Uploading... 50%".to_string() },
            OperationMsg { operation_name: op_name_2.clone(), state: OperationState::Completed, timestamp: ts(40, 400_000_000), severity: log::Level::Info, log: "Upload complete.".to_string() },
        ],
    };
    
    let op_name_3 = "op_cleanup".to_string();
    let op_log_3 = OperationLog {
        operation_name: op_name_3.clone(),
        log: vec![
            OperationMsg { operation_name: op_name_3.clone(), state: OperationState::Initial, timestamp: ts(41, 0), severity: log::Level::Info, log: "Starting initialized operation 'op_cleanup'.".to_string() },
            OperationMsg { operation_name: op_name_3.clone(), state: OperationState::Executing, timestamp: ts(41, 100_000_000), severity: log::Level::Info, log: "Waiting for operation 'op_cleanup' to be completed.".to_string() },
            OperationMsg { operation_name: op_name_3.clone(), state: OperationState::Completed, timestamp: ts(41, 200_000_000), severity: log::Level::Info, log: "Cleanup complete.".to_string() },
        ],
    };

    let all_log_rows: Vec<Vec<OperationLog>> = vec![
        vec![op_log_1, op_log_2],
        vec![op_log_3],
    ];

    let formatted_string = format_log_rows(&all_log_rows);
    
    println!("{}", formatted_string);

    assert!(formatted_string.contains("Log Row 1"));
    assert!(formatted_string.contains("op_emulate_timeout"));
    assert!(formatted_string.contains("16:36:37.969")); 
    assert!(formatted_string.contains("| Error]")); 
    assert!(formatted_string.contains("| Warn ]"));
    assert!(formatted_string.contains("| Info ]"));
    assert!(formatted_string.contains("op_upload"));
    assert!(formatted_string.contains("Log Row 2"));
    assert!(formatted_string.contains("op_cleanup"));
}