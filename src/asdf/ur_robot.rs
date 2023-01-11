use crate::SPValue;
use crate::SPVariable;
use crate::State;
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
// use crate::ur_action_trigger_key;

pub static NODE_ID: &'static str = "micro_sp";

pub async fn ur_action_client_callback(
    ur_action_client: &r2r::ActionClient<URControl::Action>,
    shared_state: &Arc<Mutex<State>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let shared_state_local = shared_state.lock().unwrap();
    match shared_state_local.state.get("ur_action_trigger_key") {
        Some(assignment) => match assignment.val {
            SPValue::Bool(true) => {
                let ur_command = shared_state_local.state.get("ur_command").unwrap();
                let ur_velocity = shared_state_local.state.get("ur_velocity").unwrap();
                let ur_acceleration = shared_state_local.state.get("ur_acceleration").unwrap();
                let ur_goal_feature_id =
                    shared_state_local.state.get("ur_goal_feature_id").unwrap();
                let ur_tcp_id = shared_state_local.state.get("ur_tcp_id").unwrap();

                let goal = URControl::Goal {
                    command: match ur_command.val.clone() {
                        SPValue::String(cmd) => cmd.clone(),
                        _ => "none".to_string(), // TODO: panic or handle this better, don't send none
                    },
                    goal_feature_id: match ur_goal_feature_id.val.clone() {
                        SPValue::String(gfid) => gfid.clone(),
                        _ => "none".to_string(),
                    },
                    tcp_id: match ur_tcp_id.val.clone() {
                        SPValue::String(tcpid) => tcpid.clone(),
                        _ => "none".to_string(),
                    },
                    velocity: match ur_velocity.val {
                        SPValue::Float64(vel) => *vel,
                        _ => 0.0,
                    },
                    acceleration: match ur_acceleration.val {
                        SPValue::Float64(acc) => *acc,
                        _ => 0.0,
                    },
                    acceleration_scaling: 1.0,
                    velocity_scaling: 1.0,
                    ..Default::default()
                };

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
                            // let _ = g.publish_feedback(URControl::Feedback {
                            //     current_state: "Goal succesfully aborted.".into(),
                            // });
                            Ok(())
                        }
                        _ => {
                            r2r::log_info!(
                                NODE_ID,
                                "Executing the Simple Robot Simulator action succeeded."
                            );
                            // let _ = g.publish_feedback(URControl::Feedback {
                            //     current_state:
                            //         "Executing the Simple Robot Simulator action succeeded.".into(),
                            // });
                            Ok(())
                        }
                    },
                    Err(e) => {
                        r2r::log_error!(
                            NODE_ID,
                            "Simple Robot Simulator action failed with: {:?}",
                            e,
                        );
                        // let _ = g.publish_feedback(URControl::Feedback {
                        //     current_state: "Simple Robot Simulator action failed. Aborting.".into(),
                        // });
                        return Err(Box::new(e));
                    }
                }
            }
            _ => Ok(()),
        },
        None => Ok(()),
    }
}
