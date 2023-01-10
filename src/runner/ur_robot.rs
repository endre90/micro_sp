use futures::stream::Stream;
use futures::StreamExt;
use r2r::geometry_msgs::msg::TransformStamped;
use r2r::scene_manipulation_msgs::srv::LookupTransform;
use r2r::sensor_msgs::msg::JointState;
use r2r::simple_robot_simulator_msgs::action::SimpleRobotControl;
use r2r::ur_controller_msgs::action::URControl;
use r2r::ur_controller_msgs::msg::Payload;
use r2r::ur_script_msgs::action::ExecuteScript;
use r2r::ActionServerGoal;
use r2r::ParameterValue;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use crate::SPValue;
use crate::SPVariable;
use crate::State;
use crate::ur_action_trigger_key;

pub static NODE_ID: &'static str = "micro_sp";


pub async fn ur_action_client_callback(
    ur_action_client: &r2r::ActionClient<URControl::Action>,
    shared_state: &Arc<Mutex<State>>
) -> Result<(), Box<dyn std::error::Error>> {
    let shared_state_local = shared_state.lock().unwrap();
    let ur_action_trigger_key = ur_action_trigger_key().clone();
    match shared_state_local.state.get(&ur_action_trigger_key) {
        Some(trigger_value) => match trigger_value {
            SPValue::Bool(true) => {
                
            },
            _ => ()
        },
        None => ()
    }
    // let goal = urc_goal_to_srs_goal(g.goal.clone(), prefix);

    r2r::log_info!(NODE_ID, "Sending request to Simple Robot Simulator.");
    // let _ = g.publish_feedback(URControl::Feedback {
    //     current_state: "Sending request to Simple Robot Simulator.".into(),
    // });

    let (_goal, result, _feedback) = match ur_action_client.send_goal_request(goal) {
        Ok(x) => match x.await {
            Ok(y) => y,
            Err(e) => {
                r2r::log_info!(NODE_ID, "Could not send goal request.");
                return Err(Box::new(e));
            }
        },
        Err(e) => {
            r2r::log_info!(NODE_ID, "Did not get goal.");
            return Err(Box::new(e));
        }
    };

    match result.await {
        Ok((status, msg)) => match status {
            r2r::GoalStatus::Aborted => {
                r2r::log_info!(NODE_ID, "Goal succesfully aborted with: {:?}", msg);
                let _ = g.publish_feedback(URControl::Feedback {
                    current_state: "Goal succesfully aborted.".into(),
                });
                Ok(())
            }
            _ => {
                r2r::log_info!(
                    NODE_ID,
                    "Executing the Simple Robot Simulator action succeeded."
                );
                let _ = g.publish_feedback(URControl::Feedback {
                    current_state: "Executing the Simple Robot Simulator action succeeded.".into(),
                });
                Ok(())
            }
        },
        Err(e) => {
            r2r::log_error!(
                NODE_ID,
                "Simple Robot Simulator action failed with: {:?}",
                e,
            );
            let _ = g.publish_feedback(URControl::Feedback {
                current_state: "Simple Robot Simulator action failed. Aborting.".into(),
            });
            return Err(Box::new(e));
        }
    }
}