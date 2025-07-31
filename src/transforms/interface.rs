use std::sync::Arc;

use crate::*;
use tokio::time::{Duration, interval};

pub async fn tf_interface(
    sp_id: &str,
    connection_manager: &Arc<ConnectionManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = interval(Duration::from_millis(250));
    let log_target = format!("{}_tf_interface", sp_id);

    log::info!(target: &log_target,  "Online.");

    let keys: Vec<String> = vec![
        format!("{}_tf_request_trigger", sp_id),
        format!("{}_tf_request_state", sp_id),
        format!("{}_tf_command", sp_id),
        format!("{}_tf_parent", sp_id),
        format!("{}_tf_child", sp_id),
        format!("{}_tf_lookup_result", sp_id),
        format!("{}_tf_insert_transforms", sp_id),
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

        let mut request_trigger = state
            .get_bool_or_default_to_false(&format!("{}_tf_request_trigger", sp_id), &log_target);

        let mut request_state = state
            .get_string_or_default_to_unknown(&format!("{}_tf_request_state", sp_id), &log_target);

        if request_trigger {
            request_trigger = false;
            if request_state == ServiceRequestState::Initial.to_string() {
                let command = state.get_string_or_default_to_unknown(
                    &format!("{}_tf_command", sp_id),
                    &log_target,
                );

                let parent = state
                    .get_string_or_default_to_unknown(&format!("{}_tf_parent", sp_id), &log_target);

                let child = state
                    .get_string_or_default_to_unknown(&format!("{}_tf_child", sp_id), &log_target);

                let mut tf_lookup_result = state.get_transform_or_default_to_default(
                    &format!("{}_tf_lookup_result", sp_id),
                    &log_target,
                );

                let tf_insert_transforms_sp_values = state.get_array_or_default_to_empty(
                    &format!("{}_tf_insert_transforms", sp_id),
                    &log_target,
                );

                let mut tf_insert_transforms = vec![];
                tf_insert_transforms_sp_values.iter().for_each(|x| match x {
                    SPValue::Transform(TransformOrUnknown::Transform(transform)) => {
                        tf_insert_transforms.push(transform.to_owned())
                    }
                    _ => (),
                });

                match command.as_str() {
                    "lookup" => {
                        match TransformsManager::lookup_transform(&mut con, &parent, &child).await {
                            Ok(tf) => {
                                tf_lookup_result = tf;
                                request_state = ServiceRequestState::Succeeded.to_string();
                            }
                            Err(e) => {
                                log::error!(target: &log_target,
                                    "Failed to lookup {} to {}.", parent, child);
                                log::error!(target: &log_target, "{e}");
                                request_state = ServiceRequestState::Failed.to_string();
                            }
                        }
                    }
                    "reparent" => {
                        match TransformsManager::reparent_transform(&mut con, &parent, &child).await
                        {
                            Ok(()) => request_state = ServiceRequestState::Succeeded.to_string(),
                            Err(e) => {
                                log::error!(target:  &log_target,
                                    "Failed to reparent {} to {}.", child, parent);
                                log::error!(target:  &log_target, "{e}");
                                request_state = ServiceRequestState::Failed.to_string();
                            }
                        }
                    }
                    "insert" => {
                        match TransformsManager::insert_transforms(&mut con, &tf_insert_transforms)
                            .await
                        {
                            Ok(()) => request_state = ServiceRequestState::Succeeded.to_string(),
                            Err(e) => {
                                log::error!(target:  &log_target,
                                    "Failed to insert transforms {:?}.", tf_insert_transforms);
                                log::error!(target:  &log_target, "{e}");
                                request_state = ServiceRequestState::Failed.to_string();
                            }
                        }
                    }                 
                    _ => {
                        log::error!(target:  &log_target,
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
