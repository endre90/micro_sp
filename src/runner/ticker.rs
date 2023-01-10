use futures::stream::Stream;
use futures::StreamExt;
use r2r::ur_controller_msgs::action::URControl;
use r2r::ActionServerGoal;
use r2r::ParameterValue;
use std::sync::{Arc, Mutex};

pub static NODE_ID: &'static str = "micro_sp";
// use crate::ur_action_trigger_key;
use crate::{ur_action_client_callback, State};

// pub static NODE_ID: &'static str = "micro_sp";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = r2r::Context::create()?;
    let mut node = r2r::Node::create(ctx, NODE_ID, "")?;

    let ur_action_client = node.create_action_client::<URControl::Action>("ur_control")?;
    let waiting_for_ur_action_client = node.is_available(&ur_action_client)?;

    let handle = std::thread::spawn(move || loop {
        node.spin_once(std::time::Duration::from_millis(100));
    });

    r2r::log_warn!(NODE_ID, "Waiting for the UR Action Service...");
    waiting_for_ur_action_client.await?;
    r2r::log_info!(NODE_ID, "UR Action available.");

    let shared_state = Arc::new(Mutex::new(State::new_empty()));

    let shared_state_clone = shared_state.clone();
    // let ur_action_trigger_key = ur_action_trigger_key().clone();
    tokio::task::spawn(async move {
        let result = ur_action_client_callback(
            &ur_action_client,
            &shared_state_clone,
        )
        .await;
        match result {
            Ok(()) => r2r::log_info!(NODE_ID, "Simple Controller Service call succeeded."),
            Err(e) => r2r::log_error!(
                NODE_ID,
                "Simple Controller Service call failed with: {}.",
                e
            ),
        };
    });

    handle.join().unwrap();

    r2r::log_warn!(NODE_ID, "Node started.");

    Ok(())
}
