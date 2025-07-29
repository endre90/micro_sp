use std::{collections::HashMap, sync::Arc};

use crate::*;
use tokio::{
    time::{Duration, interval},
};

pub async fn tf_interface(
    sp_id: &str,
    connection_manager: &Arc<ConnectionManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = interval(Duration::from_millis(250));
    let log_target = format!("{}_tf_interface", sp_id);

    log::info!(target: &log_target, "Online.");

    let keys: Vec<String> = vec![
        format!("{}_tf_request_trigger", sp_id),
        format!("{}_tf_request_state", sp_id),
        format!("{}_tf_command", sp_id),
        format!("{}_tf_parent", sp_id),
        format!("{}_tf_child", sp_id),
        format!("{}_tf_lookup_result", sp_id),
        format!("{}_tf_insert_transforms", sp_id)
    ];

    let mut con = connection_manager.get_connection().await;
    loop {
        interval.tick().await;
        if let Err(_) = connection_manager.check_redis_health(&log_target).await {
            continue;
        }
        let state = match StateManager::get_state_for_keys(&mut con, &keys).await {
            Some(s) => s,
            None => continue,
        };

        let mut request_trigger = state.get_bool_or_default_to_false(
            &log_target,
            &format!("{}_tf_request_trigger", sp_id),
        );

        let mut request_state = state.get_string_or_default_to_unknown(
            &log_target,
            &format!("{}_tf_request_state", sp_id),
        );

        if request_trigger {
            request_trigger = false;
            if request_state == ServiceRequestState::Initial.to_string() {
                let command = state.get_string_or_default_to_unknown(
                    &log_target,
                    &format!("{}_tf_command", sp_id),
                );

                let parent = state.get_string_or_default_to_unknown(
                    &log_target,
                    &format!("{}_tf_parent", sp_id),
                );

                let child = state.get_string_or_default_to_unknown(
                    &log_target,
                    &format!("{}_tf_child", sp_id),
                );

                let mut tf_lookup_result = state.get_transform_or_default_to_default(
                    &log_target,
                    &format!("{}_tf_lookup_result", sp_id),
                );

                // let tf_insert_transform = state.get_transform_or_default_to_default(
                //     &log_target,
                //     &format!("{}_tf_insert_transform", sp_id),
                // );

                let tf_insert_transforms = state.get_array_or_default_to_empty(
                    &log_target,
                    &format!("{}_tf_insert_transforms", sp_id),
                );

                match command.as_str() {
                    "lookup" => {
                        match TransformsManager::lookup_transform(&mut con, &parent, &child).await {
                            Some(tf) => {
                                tf_lookup_result = tf;
                                request_state = ServiceRequestState::Succeeded.to_string();
                            }
                            None => {
                                log::error!(target: &log_target, 
                                    "Failed to lookup {} to {}.", parent, child);
                                request_state = ServiceRequestState::Failed.to_string();
                            }
                        }
                    }
                    "reparent" => {
                        match TransformsManager::reparent_transform(&mut con, &parent, &child).await {
                            true => {
                                request_state = ServiceRequestState::Succeeded.to_string();
                            }
                            false => {
                                log::error!(target: &log_target, 
                                    "Failed to reparent {} to {}.", child, parent);
                                request_state = ServiceRequestState::Failed.to_string();
                            }  
                        }
                    }

                    "insert" => {
                        let mut map = HashMap::new();
                        for transform in tf_insert_transforms {
                            match transform {
                                SPValue::Transform(tf_or_unknown) => match tf_or_unknown {
                                    TransformOrUnknown::Transform(t) => {
                                        map.insert(t.clone().child_frame_id, t);
                                    }
                                    TransformOrUnknown::UNKNOWN => (),
                                },
                                _ => ()
                            }
                        }
                        TransformsManager::insert_transforms(&mut con, map).await;
                        request_state = ServiceRequestState::Succeeded.to_string();
                        // match response_rx.await? {
                        //     // NICE WAY TO PROPAGATE SUCCESS/FAILURE
                        //     true => {
                        //         request_state = ServiceRequestState::Succeeded.to_string();
                        //     }
                        //     false => {
                        //         log::error!(target: &log_target,
                        //             "Failed to reparent {} to {}.", child, parent);
                        //         request_state = ServiceRequestState::Failed.to_string();
                        //     }
                        // }
                    }

                    // BETTER, DO LIKE THIS IN THE FUTURE
                    // "insert" => {
                    //     let (response_tx, response_rx) = oneshot::channel();
                    //     command_sender
                    //         .send(StateManagement::InsertTransform((tf_insert_transform, response_rx)))
                    //         .await?;
                    //     match response_rx.await? {
                    //         // NICE WAY TO PROPAGATE SUCCESS/FAILURE
                    //         true => {
                    //             request_state = ServiceRequestState::Succeeded.to_string();
                    //         }
                    //         false => {
                    //             log::error!(target: &log_target,
                    //                 "Failed to reparent {} to {}.", child, parent);
                    //             request_state = ServiceRequestState::Failed.to_string();
                    //         }
                    //     }
                    // }
                    _ => {
                        log::error!(target: &log_target, 
                            "TF interface command {} is invalid.", command);
                        request_state = ServiceRequestState::Failed.to_string()
                    }
                }

                let new_state = state
                    .update(
                        &format!("{}_tf_request_trigger", sp_id),
                        request_trigger.to_spvalue(),
                    )
                    .update(
                        &format!("{}_tf_request_state", sp_id),
                        request_state.to_spvalue(),
                    )
                    .update(
                        &format!("{}_tf_lookup_result", sp_id),
                        tf_lookup_result.to_spvalue(),
                    );

                let modified_state = state.get_diff_partial_state(&new_state);
                StateManager::set_state(&mut con, &modified_state).await;
            }
        }

        interval.tick().await;
    }
}
