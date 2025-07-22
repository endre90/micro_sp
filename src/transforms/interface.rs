use crate::*;
use tokio::{
    sync::{mpsc, oneshot},
    time::{Duration, interval},
};

pub async fn tf_interface(
    sp_id: &str,
    command_sender: mpsc::Sender<StateManagement>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = interval(Duration::from_millis(100));

    log::info!(target: &&format!("tf_interface"), "Online.");

    loop {
        let (response_tx, response_rx) = oneshot::channel();
        command_sender
            .send(StateManagement::GetState(response_tx))
            .await?;
        let state = response_rx.await?;

        let mut request_trigger = state.get_bool_or_default_to_false(
            &format!("{}_tf_interface", sp_id),
            &format!("{}_tf_request_trigger", sp_id),
        );

        let mut request_state = state.get_string_or_default_to_unknown(
            &format!("{}_tf_interface", sp_id),
            &format!("{}_tf_request_state", sp_id),
        );

        if request_trigger {
            request_trigger = false;
            if request_state == ServiceRequestState::Initial.to_string() {
                let command = state.get_string_or_default_to_unknown(
                    &format!("{}_tf_interface", sp_id),
                    &format!("{}_tf_command", sp_id),
                );

                let parent = state.get_string_or_default_to_unknown(
                    &format!("{}_tf_interface", sp_id),
                    &format!("{}_tf_parent", sp_id),
                );

                let child = state.get_string_or_default_to_unknown(
                    &format!("{}_tf_interface", sp_id),
                    &format!("{}_tf_child", sp_id),
                );

                let mut tf_lookup_result = state.get_transform_or_default_to_default(
                    &format!("{}_tf_interface", sp_id),
                    &format!("{}_tf_lookup_result", sp_id),
                );

                let tf_insert_transform = state.get_transform_or_default_to_default(
                    &format!("{}_tf_interface", sp_id),
                    &format!("{}_tf_insert_transform", sp_id),
                );

                match command.as_str() {
                    "lookup" => {
                        let (response_tx, response_rx) = oneshot::channel();
                        command_sender
                            .send(StateManagement::LookupTransform((
                                parent.clone(),
                                child.clone(),
                                response_tx,
                            )))
                            .await?;
                        match response_rx.await? {
                            Some(tf) => {
                                tf_lookup_result = tf;
                                request_state = ServiceRequestState::Succeeded.to_string();
                            }
                            None => {
                                log::error!(target: &format!("{}_tf_interface", sp_id), 
                                    "Failed to lookup {} to {}.", parent, child);
                                request_state = ServiceRequestState::Failed.to_string();
                            }
                        }
                    }
                    "reparent" => {
                        let (response_tx, response_rx) = oneshot::channel();
                        command_sender
                            .send(StateManagement::ReparentTransform((
                                parent.clone(),
                                child.clone(),
                                response_tx,
                            )))
                            .await?;
                        match response_rx.await? {
                            // NICE WAY TO PROPAGATE SUCCESS/FAILURE
                            true => {
                                request_state = ServiceRequestState::Succeeded.to_string();
                            }
                            false => {
                                log::error!(target: &format!("{}_tf_interface", sp_id), 
                                    "Failed to reparent {} to {}.", child, parent);
                                request_state = ServiceRequestState::Failed.to_string();
                            }
                        }
                    }

                    "insert" => {
                        // let (response_tx, response_rx) = oneshot::channel();
                        command_sender
                            .send(StateManagement::InsertTransform(
                                tf_insert_transform))
                            .await?;
                        request_state = ServiceRequestState::Succeeded.to_string();
                        // match response_rx.await? {
                        //     // NICE WAY TO PROPAGATE SUCCESS/FAILURE
                        //     true => {
                        //         request_state = ServiceRequestState::Succeeded.to_string();
                        //     }
                        //     false => {
                        //         log::error!(target: &format!("{}_tf_interface", sp_id), 
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
                    //             log::error!(target: &format!("{}_tf_interface", sp_id),
                    //                 "Failed to reparent {} to {}.", child, parent);
                    //             request_state = ServiceRequestState::Failed.to_string();
                    //         }
                    //     }
                    // }
                    _ => {
                        log::error!(target: &format!("{}_tf_interface", sp_id), 
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
                command_sender
                    .send(StateManagement::SetPartialState(modified_state))
                    .await?;
            }
        }

        interval.tick().await;
    }
}
