use serde::{Deserialize, Serialize};

use crate::{Action, State, ToSPValue, ToSPWrapped, Transition};

#[derive(Debug, PartialEq, Clone, Eq, Hash, Serialize, Deserialize)]
pub struct Operation {
    pub name: String,
    pub precondition: Transition,
    pub postcondition: Transition,
}

impl Operation {
    pub fn new(name: &str, precondition: Transition, postcondition: Transition) -> Operation {
        Operation {
            name: name.to_string(),
            precondition,
            postcondition,
        }
    }

    pub fn eval_planning(self, state: &State) -> bool {
        if state.get_value(&self.name) == "initial".to_spvalue() {
            self.precondition.eval_planning(state)
        } else {
            false
        }
    }

    pub fn eval_running(self, state: &State) -> bool {
        if state.get_value(&self.name) == "initial".to_spvalue() {
            self.precondition.eval_running(state)
        } else {
            false
        }
    }

    pub fn take_planning(self, state: &State) -> State {
        self.postcondition
            .take_planning(&self.precondition.take_planning(state))
    }

    pub fn start_running(self, state: &State) -> State {
        let assignment = state.get_all(&self.name);
        if assignment.val == "initial".to_spvalue() {
            let action = Action::new(assignment.var, "executing".wrap());
            action.assign(&self.precondition.take_running(state))
        } else {
            state.clone()
        }
    }

    pub fn complete_running(self, state: &State) -> State {
        let assignment = state.get_all(&self.name);
        if assignment.val == "executing".to_spvalue() {
            let action = Action::new(assignment.var, "initial".wrap());
            self.postcondition.take_running(&action.assign(&state))
        } else {
            state.clone()
        }
    }

    pub fn can_be_completed(self, state: &State) -> bool {
        state.get_value(&self.name) == "executing".to_spvalue()
            && self.postcondition.eval_running(&state)
    }

    // TODO: test...
    pub fn relax(self, vars: &Vec<String>) -> Operation {
        let r_precondition = self.precondition.relax(vars);
        let r_postcondition = self.postcondition.relax(vars);
        Operation {
            name: self.name,
            precondition: r_precondition,
            postcondition: r_postcondition,
        }
    }

    // TODO: test...
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
            v_runner!("runner_goal"),
            "var:ur_current_pose == c".to_spvalue(),
        ));
        let state = state.add(SPAssignment::new(
            av_runner!("runner_plan"),
            Vec::<String>::new().to_spvalue(),
        ));
        let state = state.add(SPAssignment::new(
            bv_runner!("runner_replan"),
            true.to_spvalue(),
        ));
        let state = state.add(SPAssignment::new(
            bv_runner!("runner_replanned"),
            false.to_spvalue(),
        ));
        let state = state.add(SPAssignment::new(
            bv_estimated!("ur_action_trigger"),
            false.to_spvalue(),
        ));
        let state = state.add(SPAssignment::new(
            v_estimated!("ur_action_state", vec!("initial", "executing", "done")),
            "initial".to_spvalue(),
        ));
        let state = state.add(SPAssignment::new(
            v_estimated!("ur_current_pose", vec!("a", "b", "c")),
            "a".to_spvalue(),
        ));
        let state = state.add(SPAssignment::new(
            v_estimated!("ur_command", vec!("movej", "movel")),
            "movej".to_spvalue(),
        ));
        let state = state.add(SPAssignment::new(
            fv_estimated!("ur_velocity", vec!(0.1, 0.2, 0.3)),
            0.2.to_spvalue(),
        ));
        let state = state.add(SPAssignment::new(
            fv_estimated!("ur_acceleration", vec!(0.2, 0.4, 0.6)),
            0.4.to_spvalue(),
        ));
        let state = state.add(SPAssignment::new(
            v_estimated!("ur_goal_feature_id", vec!("a", "b", "c")),
            "a".to_spvalue(),
        ));
        let state = state.add(SPAssignment::new(
            v_estimated!("ur_tcp_id", vec!("svt_tcp")),
            "svt_tcp".to_spvalue(),
        ));
        state
    }

    #[test]
    fn test_operation_new() {
        let state = make_initial_state();
        Operation::new(
        "op_move_to_b",
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
        )
    );
    }

    #[test]
    fn test_operation_eval_planning() {
        let state = make_initial_state();
        let op_move_to_b = v_runner!("op_move_to_b");
        let state = state.add(assign!(op_move_to_b, "initial".to_spvalue()));
        let op = Operation::new(
        "op_move_to_b",
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
        )
    );

        // Adding the opeation states in the model
        let m = Model::new("asdf", state.clone(), vec![], vec![op.clone()], vec!());
        assert_eq!(op.eval_planning(&m.state), true)
    }

    #[should_panic]
    #[test]
    fn test_operation_eval_planning_panic() {
        let state = make_initial_state();
        let op_move_to_b = v_runner!("op_move_to_b");
        let state = state.add(assign!(op_move_to_b, "initial".to_spvalue()));
        let op = Operation::new(
        "op_move_to_b",
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
        )
    );

        // Adding the opeation states in the model
        let m = Model::new("asdf", state.clone(), vec![], vec![op.clone()], vec!());
        assert_eq!(op.eval_planning(&m.state), true)
    }

    #[test]
    fn test_operation_eval_running() {
        let state = make_initial_state();
        let op_move_to_b = v_runner!("op_move_to_b");
        let state = state.add(assign!(op_move_to_b, "initial".to_spvalue()));
        let op = Operation::new(
        "op_move_to_b",
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
        )
    );

        // Adding the opeation states in the model
        let m = Model::new("asdf", state.clone(), vec![], vec![op.clone()], vec!());
        assert_eq!(op.eval_running(&m.state), true)
    }

    #[should_panic]
    #[test]
    fn test_operation_eval_running_panic() {
        let state = make_initial_state();
        let op_move_to_b = v_runner!("op_move_to_b");
        let state = state.add(assign!(op_move_to_b, "initial".to_spvalue()));
        let op = Operation::new(
        "op_move_to_b",
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
        )
    );

        // Adding the opeation states in the model
        let m = Model::new("asdf", state.clone(), vec![], vec![op.clone()], vec!());
        assert_eq!(op.eval_running(&m.state), true)
    }

    #[test]
    fn test_operation_take_planning() {
        let state = make_initial_state();
        let op_move_to_b = v_runner!("op_move_to_b");
        let state = state.add(assign!(op_move_to_b, "initial".to_spvalue()));
        let op = Operation::new(
        "op_move_to_b",
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
        )
    );

        // Adding the opeation states in the model
        let m = Model::new("asdf", state.clone(), vec![], vec![op.clone()], vec!());
        let new_state = match op.clone().eval_planning(&m.state) {
            true => op.take_planning(&m.state),
            false => m.state,
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
        let op_move_to_b = v_runner!("op_move_to_b");
        let state = state.add(assign!(op_move_to_b, "initial".to_spvalue()));
        let op = Operation::new(
        "op_move_to_b",
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
        )
    );

        // Adding the opeation states in the model
        let m = Model::new("asdf", state.clone(), vec![], vec![op.clone()], vec!());
        let new_state = match op.clone().eval_running(&m.state) {
            true => op.start_running(&m.state),
            false => m.state,
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
        let op_move_to_b = v_runner!("op_move_to_b");
        let state = state.add(assign!(op_move_to_b, "initial".to_spvalue()));
        let op = Operation::new(
        "op_move_to_b",
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
        )
    );

        // Adding the opeation states in the model
        let m = Model::new("asdf", state.clone(), vec![], vec![op.clone()], vec!());
        let new_state = match op.clone().eval_running(&m.state) {
            true => op.clone().start_running(&m.state),
            false => m.state,
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
            "initial".to_spvalue()
        );
    }
}
