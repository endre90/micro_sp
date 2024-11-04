use std::fmt;

use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::*;

/// Initial:   The operation planned and ready to be executed.
/// Disabled:  The operation is ready for execution, but the precondition guard is not yet enabled.
/// Executing: The precondition guard is enabled and the actions of the precondition are taken.
/// Completed: The postcondition guard is enabled and the actions of the postcondition are taken.
///            The operation is successfully completed.
/// Timedout:  The operation was in the executing state for more time than its deadline allows.
/// Failed:    The operations has failed due to an error.
#[derive(Debug, PartialEq, Clone, Eq, Hash, Serialize, Deserialize)]
pub enum OperationState {
    Initial,
    Disabled,
    Executing,
    Completed,
    Timedout,
    Failed,
    UNKNOWN,
}

impl Default for OperationState {
    fn default() -> Self {
        OperationState::UNKNOWN
    }
}

impl OperationState {
    pub fn from_str(x: &str) -> OperationState {
        match x {
            "initial" => OperationState::Initial,
            "disabled" => OperationState::Disabled,
            "executing" => OperationState::Executing,
            "timedout" => OperationState::Timedout,
            "failed" => OperationState::Failed,
            "completed" => OperationState::Completed,
            _ => OperationState::UNKNOWN,
        }
    }
    pub fn to_spvalue(self) -> SPValue {
        self.to_string().to_spvalue()
    }
}

impl fmt::Display for OperationState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OperationState::Initial => write!(f, "initial"),
            OperationState::Disabled => write!(f, "disabled"),
            OperationState::Executing => write!(f, "executing"),
            OperationState::Timedout => write!(f, "timedout"),
            OperationState::Failed => write!(f, "failed"),
            OperationState::Completed => write!(f, "completed"),
            OperationState::UNKNOWN => write!(f, "UNKNOWN"),
        }
    }
}

/// An operation O captures the behavior of tasks that can take some time
/// to complete, and it is a convenient modeling abstraction for both planning
/// and execution. A model of an operation can be in its initial (init)
/// or executing (exec) state, nominally (look at OperationState for info
/// about all the states and operacion can be in). The precondition is a running
/// transition associated with the start of the operation, switching it to the
/// executing state. The operation will be in its executing state until the
/// guard of the postcondition running transition is satisfied. The satisfac-
/// tion of the postcondition implies that the operation is completed and
/// can return to the initial state.
#[derive(Debug, PartialEq, Clone, Eq, Hash, Serialize, Deserialize)]
pub struct Operation {
    pub name: String,
    pub state: OperationState,
    pub deadline: Option<OrderedFloat<f64>>,
    pub retries: i64,
    pub precondition: Transition,
    pub postcondition: Transition,
    pub fail_transition: Transition, // pub fail_transitions: Vec<Transition>, one option to solve this
    // pub deadline_transition: Transition,
    pub reset_transition: Transition,
}

impl Operation {
    pub fn new(
        name: &str,
        deadline: Option<f64>,
        retries: Option<i64>,
        precondition: Transition,
        postcondition: Transition,
        fail_transition: Transition,
        reset_transition: Transition,
    ) -> Operation {
        Operation {
            name: name.to_string(),
            state: OperationState::UNKNOWN,
            deadline: match deadline {
                None => Some(OrderedFloat::from(MAX_ALLOWED_OPERATION_DURATION)),
                Some(x) => Some(OrderedFloat::from(x)),
            },
            retries: match retries {
                Some(x) => x,
                None => 0,
            },
            precondition,
            postcondition,
            fail_transition,
            reset_transition,
        }
    }

    /// Check the guard of the planning precondidion transition.
    pub fn eval_planning(&self, state: &State) -> bool {
        if state.get_value(&self.name) == OperationState::Initial.to_spvalue() {
            self.clone().precondition.eval_planning(state)
        } else {
            false
        }
    }

    /// Check the guard of the running precondidion transition.
    pub fn eval_running(&self, state: &State) -> bool {
        if state.get_value(&self.name) == OperationState::Initial.to_spvalue() {
            self.clone().precondition.eval_running(state)
        } else {
            false
        }
    }

    /// Execute the planing actions of both the pre and post conditions.
    pub fn take_planning(&self, state: &State) -> State {
        self.clone()
            .postcondition
            .take_planning(&self.clone().precondition.take_planning(state))
    }

    /// Start executing the operation. Check for eval_running() first.
    pub fn start_running(&self, state: &State) -> State {
        let assignment = state.get_assignment(&self.name);
        if assignment.val == "initial".to_spvalue() {
            let action = Action::new(assignment.var, "executing".wrap());
            action.assign(&self.clone().precondition.take_running(state))
        } else {
            state.clone()
        }
    }

    /// Complete executing the operation. Check for can_be_completed() first.
    pub fn complete_running(&self, state: &State) -> State {
        let assignment = state.get_assignment(&self.name);
        if assignment.val == "executing".to_spvalue() {
            let action = Action::new(assignment.var, "completed".wrap());
            self.clone()
                .postcondition
                .take_running(&action.assign(&state))
        } else {
            state.clone()
        }
    }

    /// Fail the executing operation. Check for can_be_failed() first.
    pub fn fail_running(&self, state: &State) -> State {
        let assignment = state.get_assignment(&self.name);
        if assignment.val == "executing".to_spvalue() {
            let action = Action::new(assignment.var, "failed".wrap());
            self.clone()
                .fail_transition
                .take_running(&action.assign(&state))
        } else {
            state.clone()
        }
    }

    /// Retry the execution of the operation, allows for retries without immediate replanning.
    pub fn retry_running(&self, state: &State) -> State {
        let assignment = state.get_assignment(&self.name);
        if assignment.val == "failed".to_spvalue() {
            let action = Action::new(assignment.var, "initial".wrap());
            action.assign(&state)
        } else {
            state.clone()
        }
    }

    /// Reset the completed operation. Check for can_be_reset() first.
    pub fn reset_running(&self, state: &State) -> State {
        let assignment = state.get_assignment(&self.name);
        if assignment.val == "completed".to_spvalue() {
            let action = Action::new(assignment.var, "initial".wrap());
            self.clone()
                .reset_transition
                .take_running(&action.assign(&state))
        } else {
            state.clone()
        }
    }

    /// Check the running postondition guard.
    pub fn can_be_completed(&self, state: &State) -> bool {
        state.get_value(&self.name) == "executing".to_spvalue()
            && self.clone().postcondition.eval_running(&state)
    }

    /// Check the running fail_transition guard.
    pub fn can_be_failed(&self, state: &State) -> bool {
        state.get_value(&self.name) == "executing".to_spvalue()
            && self.clone().fail_transition.eval_running(&state)
    }

    /// Check the running reset_transition guard.
    pub fn can_be_reset(&self, state: &State) -> bool {
        state.get_value(&self.name) == "completed".to_spvalue()
            && self.clone().reset_transition.eval_running(&state)
    }

    // // TODO: test relax function
    // pub fn relax(self, vars: &Vec<String>) -> Operation {
    //     let r_precondition = self.precondition.relax(vars);
    //     let r_postcondition = self.postcondition.relax(vars);
    //     let r_reset_transition = self.reset_transition.relax(vars);
    //     Operation {
    //         name: self.name,
    //         state: self.state,
    //         deadline: self.deadline,
    //         precondition: r_precondition,
    //         postcondition: r_postcondition,
    //         reset_transition: r_reset_transition,
    //     }
    // }

    // TODO: contains planning function
    pub fn contains_planning(self, var: &String) -> bool {
        self.precondition.contains_planning(var) || self.postcondition.contains_planning(var)
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