use futures::stream::Stream;
use futures::StreamExt;
use r2r::ur_controller_msgs::action::URControl;
use r2r::ActionServerGoal;
use r2r::ParameterValue;
use std::sync::{Arc, Mutex};

use crate::State;
use crate::make_initial_state;

pub async fn ticker(
    ur_action_client: &r2r::ActionClient<URControl::Action>,
    shared_state: &Arc<Mutex<State>>,
    mut timer: r2r::Timer,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {

        timer.tick().await?;
    }
}