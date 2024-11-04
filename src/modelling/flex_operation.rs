use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Debug, PartialEq, Clone, Eq, Hash, Serialize, Deserialize)]
pub struct FlexOperation {
    pub name: String,
    pub state: OperationState,
    pub timeout: Option<OrderedFloat<f64>>,
    pub timeout_transitions: Vec<Transition>,
    pub retries: i64,
    pub preconditions: Vec<Transition>,
    pub postconditions: Vec<Transition>,
    pub fail_transitions: Vec<Transition>,
    pub reset_transitions: Vec<Transition>,
}

impl FlexOperation {
    pub fn new(
        name: &str,
        timeout: Option<f64>,
        timeout_transitions: Vec<Transition>,
        retries: Option<i64>,
        preconditions: Vec<Transition>,
        postconditions: Vec<Transition>,
        fail_transitions: Vec<Transition>,
        reset_transitions: Vec<Transition>,
    ) -> FlexOperation {
        FlexOperation {
            name: name.to_string(),
            state: OperationState::UNKNOWN,
            timeout: match timeout {
                None => Some(OrderedFloat::from(MAX_ALLOWED_OPERATION_DURATION)),
                Some(x) => Some(OrderedFloat::from(x)),
            },
            timeout_transitions,
            retries: match retries {
                Some(x) => x,
                None => 0,
            },
            preconditions,
            postconditions,
            fail_transitions,
            reset_transitions,
        }
    }

    /// Check the guard of the planning precondidion transition.
    pub fn eval_planning(&self, state: &State) -> bool {
        // (bool, Option<String>) {
        if state.get_value(&self.name) == OperationState::Initial.to_spvalue() {
            for precondition in &self.preconditions {
                if precondition.clone().eval_planning(state) {
                    return true;
                    //  return (true, Some(precondition.name))
                }
            }
        }
        false
    }

    /// Check the guard of the running precondidion transition.
    pub fn eval_running(&self, state: &State) -> bool {
        if state.get_value(&self.name) == OperationState::Initial.to_spvalue() {
            for precondition in &self.preconditions {
                if precondition.clone().eval_running(state) {
                    return true;
                }
            }
        }
        false
    }

    /// Check the running postondition guard.
    pub fn can_be_completed(&self, state: &State) -> bool {
        if state.get_value(&self.name) == OperationState::Executing.to_spvalue() {
            for postcondition in &self.postconditions {
                if postcondition.clone().eval_running(&state) {
                    return true;
                }
            }
        }
        false
    }

    /// Check the running fail_transition guard.
    pub fn can_be_failed(&self, state: &State) -> bool {
        if state.get_value(&self.name) == OperationState::Executing.to_spvalue() {
            for fail_transition in &self.fail_transitions {
                if fail_transition.clone().eval_running(&state) {
                    return true;
                }
            }
        }
        false
    }

    /// Check the running reset_transition guard.
    pub fn can_be_reset(&self, state: &State) -> bool {
        if state.get_value(&self.name) == OperationState::Completed.to_spvalue() {
            for reset_transition in &self.reset_transitions {
                if reset_transition.clone().eval_running(&state) {
                    return true;
                }
            }
        }
        false
    }

    /// Check the running timeout_transition guard.
    pub fn can_be_timeouted(&self, state: &State) -> bool {
        if state.get_value(&self.name) == OperationState::Completed.to_spvalue() {
            for timeout_transition in &self.timeout_transitions {
                if timeout_transition.clone().eval_running(&state) {
                    return true;
                }
            }
        }
        false
    }

    /// Start executing the operation. Check for eval_running() first.
    pub fn start_running(&self, state: &State) -> State {
        let assignment = state.get_assignment(&self.name);
        if assignment.val == OperationState::Initial.to_spvalue() {
            for precondition in &self.preconditions {
                if precondition.clone().eval_running(state) {
                    let action = Action::new(
                        assignment.var,
                        OperationState::Executing.to_spvalue().wrap(),
                    );
                    return action.assign(&precondition.clone().take_running(state));
                }
            }
        }
        state.clone()
    }

    /// Complete executing the operation. Check for can_be_completed() first.
    pub fn complete_running(&self, state: &State) -> State {
        let assignment = state.get_assignment(&self.name);
        if assignment.val == OperationState::Executing.to_spvalue() {
            for postcondition in &self.postconditions {
                if postcondition.clone().eval_running(&state) {
                    let action = Action::new(
                        assignment.var,
                        OperationState::Completed.to_spvalue().wrap(),
                    );
                    return postcondition.clone().take_running(&action.assign(&state));
                }
            }
        }
        state.clone()
    }

    /// Fail the executing operation. Check for can_be_failed() first.
    pub fn fail_running(&self, state: &State) -> State {
        let assignment = state.get_assignment(&self.name);
        if assignment.val == OperationState::Executing.to_spvalue() {
            for fail_transition in &self.fail_transitions {
                if fail_transition.clone().eval_running(&state) {
                    let action =
                        Action::new(assignment.var, OperationState::Failed.to_spvalue().wrap());
                    return fail_transition.clone().take_running(&action.assign(&state));
                }
            }
        }
        state.clone()
    }

    /// Retry the execution of the operation, allows for retries without immediate replanning.
    pub fn retry_running(&self, state: &State) -> State {
        let assignment = state.get_assignment(&self.name);
        if assignment.val == OperationState::Failed.to_spvalue() {
            let action = Action::new(assignment.var, OperationState::Initial.to_spvalue().wrap());
            action.assign(&state)
        } else {
            state.clone()
        }
    }

    /// Reset the completed operation. Check for can_be_reset() first.
    pub fn reset_running(&self, state: &State) -> State {
        let assignment = state.get_assignment(&self.name);
        if assignment.val == OperationState::Completed.to_spvalue() {
            for reset_transition in &self.reset_transitions {
                if reset_transition.clone().eval_running(&state) {
                    let action =
                        Action::new(assignment.var, OperationState::Initial.to_spvalue().wrap());
                    return reset_transition.clone().take_running(&action.assign(&state));
                }
            }
        }
        state.clone()
    }
}

#[cfg(test)]
mod tests {

    use crate::*;

    pub fn make_initial_state() -> State {
        let state = State::new();
        let state = state.add(SPAssignment::new(
            v!("runner_goal"),
            "var:ur_current_pose == c".to_spvalue(),
        ));
        let state = state.add(SPAssignment::new(
            av!("runner_plan"),
            Vec::<String>::new().to_spvalue(),
        ));
        let state = state.add(SPAssignment::new(bv!("runner_replan"), true.to_spvalue()));
        let state = state.add(SPAssignment::new(
            bv!("runner_replanned"),
            false.to_spvalue(),
        ));
        let state = state.add(SPAssignment::new(
            bv!("ur_action_trigger"),
            false.to_spvalue(),
        ));
        let state = state.add(SPAssignment::new(
            v!("ur_action_state"),
            "initial".to_spvalue(),
        ));
        let state = state.add(SPAssignment::new(v!("ur_current_pose"), "a".to_spvalue()));
        let state = state.add(SPAssignment::new(v!("ur_command"), "movej".to_spvalue()));
        let state = state.add(SPAssignment::new(fv!("ur_velocity"), 0.2.to_spvalue()));
        let state = state.add(SPAssignment::new(fv!("ur_acceleration"), 0.4.to_spvalue()));
        let state = state.add(SPAssignment::new(
            v!("ur_goal_feature_id"),
            "a".to_spvalue(),
        ));
        let state = state.add(SPAssignment::new(v!("ur_tcp_id"), "svt_tcp".to_spvalue()));
        state
    }

    #[test]
    fn test_operation_new() {
        let state = make_initial_state();
        Operation::new(
        "op_move_to_b",
        None,
        None,
        t!(
            "start_moving_to_b",
            "var:ur_action_trigger == false && var:ur_action_state == initial && var:ur_current_pose != b",
            "true",
            vec!(
                "var:ur_command <- movej",
                "var:ur_action_trigger <- true",
                "var:ur_goal_feature_id <- b",
                "var:ur_tcp_id <- svt_tcp"
            ),
            Vec::<&str>::new(),
            &state
        ),
        t!(
            "complete_moving_to_b",
            "var:ur_action_state == done",
            "true",
            vec!(
                "var:ur_action_trigger <- false",
                "var:ur_current_pose <- b"
            ),
            Vec::<&str>::new(),
            &state
        ),
        Transition::empty(),Transition::empty()
    );
    }

    #[test]
    fn test_operation_eval_planning() {
        let state = make_initial_state();
        let op_move_to_b = v!("op_move_to_b");
        let state = state.add(assign!(op_move_to_b, "initial".to_spvalue()));
        let op = Operation::new(
        "op_move_to_b",
        None,
        None,
        t!(
            "start_moving_to_b",
            "var:ur_action_trigger == false && var:ur_action_state == initial && var:ur_current_pose != b",
            "true",
            vec!(
                "var:ur_command <- movej",
                "var:ur_action_trigger <- true",
                "var:ur_goal_feature_id <- b",
                "var:ur_tcp_id <- svt_tcp"
            ),
            Vec::<&str>::new(),
            &state
        ),
        t!(
            "complete_moving_to_b",
            "var:ur_action_state == done",
            "true",
            vec!(
                "var:ur_action_trigger <- false",
                "var:ur_current_pose <- b"
            ),
            Vec::<&str>::new(),
            &state
        ),
        Transition::empty(), Transition::empty()
    );

        // Adding the opeation states in the model
        // let m = Model::new("asdf", vec![], vec![op.clone()]);
        assert_eq!(op.eval_planning(&state), true)
    }

    #[should_panic]
    #[test]
    fn test_operation_eval_planning_panic() {
        let state = make_initial_state();
        let op_move_to_b = v!("op_move_to_b");
        let state = state.add(assign!(op_move_to_b, "initial".to_spvalue()));
        let op = Operation::new(
        "op_move_to_b",
        None,
        None,
        t!(
            "start_moving_to_b",
            "var:ur_action_trigger == false && var:ur_action_state == initial && var:ur_current_pose == b",
            "true",
            vec!(
                "var:ur_command <- movej",
                "var:ur_action_trigger <- true",
                "var:ur_goal_feature_id <- b",
                "var:ur_tcp_id <- svt_tcp"
            ),
            Vec::<&str>::new(),
            &state
        ),
        t!(
            "complete_moving_to_b",
            "var:ur_action_state == done",
            "true",
            vec!(
                "var:ur_action_trigger <- false",
                "var:ur_current_pose <- b"
            ),
            Vec::<&str>::new(),
            &state
        ),
        Transition::empty(), Transition::empty()
    );

        // Adding the opeation states in the model
        // let m = Model::new("asdf", vec![], vec![op.clone()]);
        assert_eq!(op.eval_planning(&state), true)
    }

    #[test]
    fn test_operation_eval_running() {
        let state = make_initial_state();
        let op_move_to_b = v!("op_move_to_b");
        let state = state.add(assign!(op_move_to_b, "initial".to_spvalue()));
        let op = Operation::new(
        "op_move_to_b",
        None,
        None,
        t!(
            "start_moving_to_b",
            "var:ur_action_trigger == false && var:ur_action_state == initial && var:ur_current_pose != b",
            "var:runner_replan == true",
            vec!(
                "var:ur_command <- movej",
                "var:ur_action_trigger <- true",
                "var:ur_goal_feature_id <- b",
                "var:ur_tcp_id <- svt_tcp"
            ),
            Vec::<&str>::new(),
            &state
        ),
        t!(
            "complete_moving_to_b",
            "var:ur_action_state == done",
            "true",
            vec!(
                "var:ur_action_trigger <- false",
                "var:ur_current_pose <- b"
            ),
            Vec::<&str>::new(),
            &state
        ),
        Transition::empty(), Transition::empty()
    );

        // Adding the opeation states in the model
        // let m = Model::new("asdf", vec![], vec![op.clone()]);
        assert_eq!(op.eval_running(&state), true)
    }

    #[should_panic]
    #[test]
    fn test_operation_eval_running_panic() {
        let state = make_initial_state();
        let op_move_to_b = v!("op_move_to_b");
        let state = state.add(assign!(op_move_to_b, "initial".to_spvalue()));
        let op = Operation::new(
        "op_move_to_b",
        None,
        None,
        t!(
            "start_moving_to_b",
            "var:ur_action_trigger == false && var:ur_action_state == initial && var:ur_current_pose != b",
            "var:runner_replan == false",
            vec!(
                "var:ur_command <- movej",
                "var:ur_action_trigger <- true",
                "var:ur_goal_feature_id <- b",
                "var:ur_tcp_id <- svt_tcp"
            ),
            Vec::<&str>::new(),
            &state
        ),
        t!(
            "complete_moving_to_b",
            "var:ur_action_state == done",
            "true",
            vec!(
                "var:ur_action_trigger <- false",
                "var:ur_current_pose <- b"
            ),
            Vec::<&str>::new(),
            &state
        ),
        Transition::empty(), Transition::empty()
    );

        // Adding the opeation states in the model
        // let m = Model::new("asdf", vec![], vec![op.clone()]);
        assert_eq!(op.eval_running(&state), true)
    }

    #[test]
    fn test_operation_take_planning() {
        let state = make_initial_state();
        let op_move_to_b = v!("op_move_to_b");
        let state = state.add(assign!(op_move_to_b, "initial".to_spvalue()));
        let op = Operation::new(
        "op_move_to_b",
        None,
        None,
        t!(
            "start_moving_to_b",
            "var:ur_action_trigger == false && var:ur_action_state == initial && var:ur_current_pose != b",
            "true",
            vec!(
                "var:ur_command <- movej",
                "var:ur_action_trigger <- true",
                "var:ur_goal_feature_id <- b",
                "var:ur_tcp_id <- svt_tcp"
            ),
            Vec::<&str>::new(),
            &state
        ),
        t!(
            "complete_moving_to_b",
            "var:ur_action_state == done",
            "true",
            vec!(
                "var:ur_action_trigger <- false",
                "var:ur_current_pose <- b"
            ),
            Vec::<&str>::new(),
            &state
        ),
        Transition::empty(), Transition::empty()
    );

        // Adding the opeation states in the model
        // let m = Model::new("asdf", vec![], vec![op.clone()]);
        let new_state = match op.clone().eval_planning(&state) {
            true => op.take_planning(&state),
            false => state,
        };
        assert_eq!(new_state.get_value("ur_current_pose"), "b".to_spvalue());
        assert_eq!(new_state.get_value("ur_action_trigger"), false.to_spvalue());
        assert_eq!(new_state.get_value("ur_command"), "movej".to_spvalue());
        assert_eq!(new_state.get_value("ur_goal_feature_id"), "b".to_spvalue());
        assert_eq!(new_state.get_value("ur_tcp_id"), "svt_tcp".to_spvalue());
    }

    #[test]
    fn test_operation_start() {
        let state = make_initial_state();
        let op_move_to_b = v!("op_move_to_b");
        let state = state.add(assign!(op_move_to_b, "initial".to_spvalue()));
        let op = Operation::new(
        "op_move_to_b",
        None,
        None,
        t!(
            "start_moving_to_b",
            "var:ur_action_trigger == false && var:ur_action_state == initial && var:ur_current_pose != b",
            "true",
            vec!(
                "var:ur_command <- movej",
                "var:ur_action_trigger <- true",
                "var:ur_goal_feature_id <- b",
                "var:ur_tcp_id <- svt_tcp"
            ),
            Vec::<&str>::new(),
            &state
        ),
        t!(
            "complete_moving_to_b",
            "var:ur_action_state == done",
            "true",
            vec!(
                "var:ur_action_trigger <- false",
                "var:ur_current_pose <- b"
            ),
            Vec::<&str>::new(),
            &state
        ),
        Transition::empty(), Transition::empty()
    );

        // Adding the opeation states in the model
        // let m = Model::new("asdf", vec![], vec![op.clone()]);
        let new_state = match op.clone().eval_running(&state) {
            true => op.start_running(&state),
            false => state,
        };
        assert_eq!(new_state.get_value("ur_current_pose"), "a".to_spvalue());
        assert_eq!(new_state.get_value("ur_action_trigger"), true.to_spvalue());
        assert_eq!(new_state.get_value("ur_command"), "movej".to_spvalue());
        assert_eq!(new_state.get_value("ur_goal_feature_id"), "b".to_spvalue());
        assert_eq!(new_state.get_value("ur_tcp_id"), "svt_tcp".to_spvalue());
        assert_eq!(
            new_state.get_value("op_move_to_b"),
            "executing".to_spvalue()
        );
    }

    #[test]
    fn test_operation_complete() {
        let state = make_initial_state();
        let op_move_to_b = v!("op_move_to_b");
        let state = state.add(assign!(op_move_to_b, "initial".to_spvalue()));
        let op = Operation::new(
        "op_move_to_b",
        None,
        None,
        t!(
            "start_moving_to_b",
            "var:ur_action_trigger == false && var:ur_action_state == initial && var:ur_current_pose != b",
            "true",
            vec!(
                "var:ur_command <- movej",
                "var:ur_action_trigger <- true",
                "var:ur_goal_feature_id <- b",
                "var:ur_tcp_id <- svt_tcp"
            ),
            Vec::<&str>::new(),
            &state
        ),
        t!(
            "complete_moving_to_b",
            "var:ur_action_state == done",
            "true",
            vec!(
                "var:ur_action_trigger <- false",
                "var:ur_current_pose <- b"
            ),
            Vec::<&str>::new(),
            &state
        ),
        Transition::empty(), Transition::empty()
    );

        // Adding the opeation states in the model
        // let m = Model::new("asdf", vec![], vec![op.clone()]);
        let new_state = match op.clone().eval_running(&state) {
            true => op.clone().start_running(&state),
            false => state,
        };
        assert_eq!(new_state.get_value("ur_current_pose"), "a".to_spvalue());
        assert_eq!(new_state.get_value("ur_action_trigger"), true.to_spvalue());
        assert_eq!(new_state.get_value("ur_command"), "movej".to_spvalue());
        assert_eq!(new_state.get_value("ur_goal_feature_id"), "b".to_spvalue());
        assert_eq!(new_state.get_value("ur_tcp_id"), "svt_tcp".to_spvalue());
        assert_eq!(
            new_state.get_value("op_move_to_b"),
            "executing".to_spvalue()
        );

        let new_state_2 = op.complete_running(&new_state);
        assert_eq!(new_state_2.get_value("ur_current_pose"), "b".to_spvalue());
        assert_eq!(
            new_state_2.get_value("ur_action_trigger"),
            false.to_spvalue()
        );
        assert_eq!(new_state_2.get_value("ur_command"), "movej".to_spvalue());
        assert_eq!(
            new_state_2.get_value("ur_goal_feature_id"),
            "b".to_spvalue()
        );
        assert_eq!(new_state_2.get_value("ur_tcp_id"), "svt_tcp".to_spvalue());
        assert_eq!(
            new_state_2.get_value("op_move_to_b"),
            "completed".to_spvalue()
        );
    }
}
