use crate::*;
use tokio::{
    sync::{mpsc, oneshot},
    time::{interval, Duration},
};

pub async fn sop_runner(
    sp_id: &str,
    model: &Model,
    command_sender: mpsc::Sender<StateManagement>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = interval(Duration::from_millis(100));

    log::info!(target: &&format!("{}_sop_runner", sp_id), "Online.");

    loop {
        let (response_tx, response_rx) = oneshot::channel();
        command_sender
            .send(StateManagement::GetState(response_tx))
            .await?;
        let state = response_rx.await?;

        let request_trigger = state.get_bool_or_default_to_false(
            &format!("{}_sop_runner", sp_id),
            &format!("{}_sop_request_trigger", sp_id),
        );

        let request_state = state.get_string_or_default_to_unknown(
            &format!("{}_sop_runner", sp_id),
            &format!("{}_sop_request_state", sp_id),
        );

        if request_trigger {
            if request_state == ActionRequestState::Initial.to_string() {
                let sop_id = state.get_string_or_default_to_unknown(
                    &format!("{}_sop_runner", sp_id),
                    &format!("{}_sop_id", sp_id),
                );

                let sop = model
                    .sops
                    .iter()
                    .find(|sop| sop.id == sop_id.to_string())
                    .unwrap()
                    .to_owned();

                command_sender
                    .send(StateManagement::Set((
                        format!("{sp_id}_sop_request_state"),
                        ActionRequestState::Executing.to_string().to_spvalue(),
                    )))
                    .await?;
                command_sender
                    .send(StateManagement::Set((
                        format!("{sp_id}_sop_request_trigger"),
                        false.to_spvalue(),
                    )))
                    .await?;

                match sop.sop.execute_sop(sp_id, command_sender.clone()).await {
                    Ok(sop_state) => match sop_state {
                        OperationState::Completed => {
                            command_sender
                                .send(StateManagement::Set((
                                    format!("{sp_id}_sop_request_state"),
                                    ActionRequestState::Succeeded.to_string().to_spvalue(),
                                )))
                                .await?;
                        }
                        _ => {
                            command_sender
                                .send(StateManagement::Set((
                                    format!("{sp_id}_sop_request_state"),
                                    ActionRequestState::Failed.to_string().to_spvalue(),
                                )))
                                .await?;
                        }
                    },
                    Err(e) => {
                        log::error!(target: &&format!("{}_sop_runner", sop_id), "SOP {sop_id} has failed with: {e}.");
                        command_sender
                            .send(StateManagement::Set((
                                format!("{sp_id}_sop_request_state"),
                                ActionRequestState::Failed.to_string().to_spvalue(),
                            )))
                            .await?;
                    }
                }
            }
        }

        interval.tick().await;
    }
}
