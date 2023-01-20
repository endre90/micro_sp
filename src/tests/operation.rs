#![allow(unused_imports)]
#![allow(dead_code)]
use crate::{
    av_run, bv, bv_run, fv, pred_parser, t, v, v_run, Action, Operation, SPAssignment, SPValueType,
    SPVariable, SPVariableType, State, ToSPValue, Transition, OperationModel,
};
use std::collections::{HashMap, HashSet};

pub fn make_initial_state() -> State {
    let state = State::new();
    let state = state.add(SPAssignment::new(
        v_run!("runner_goal"),
        "var:ur_current_pose == c".to_spvalue(),
    ));
    let state = state.add(SPAssignment::new(
        av_run!("runner_plan"),
        Vec::<String>::new().to_spvalue(),
    ));
    let state = state.add(SPAssignment::new(
        bv_run!("runner_replan"),
        true.to_spvalue(),
    ));
    let state = state.add(SPAssignment::new(
        bv_run!("runner_replanned"),
        false.to_spvalue(),
    ));
    let state = state.add(SPAssignment::new(
        bv!("ur_action_trigger"),
        false.to_spvalue(),
    ));
    let state = state.add(SPAssignment::new(
        v!("ur_action_state", vec!("initial", "executing", "done")),
        "initial".to_spvalue(),
    ));
    let state = state.add(SPAssignment::new(
        v!("ur_current_pose", vec!("a", "b", "c")),
        "a".to_spvalue(),
    ));
    let state = state.add(SPAssignment::new(
        v!("ur_command", vec!("movej", "movel")),
        "movej".to_spvalue(),
    ));
    let state = state.add(SPAssignment::new(
        fv!("ur_velocity", vec!(0.1, 0.2, 0.3)),
        0.2.to_spvalue(),
    ));
    let state = state.add(SPAssignment::new(
        fv!("ur_acceleration", vec!(0.2, 0.4, 0.6)),
        0.4.to_spvalue(),
    ));
    let state = state.add(SPAssignment::new(
        v!("ur_goal_feature_id", vec!("a", "b", "c")),
        "a".to_spvalue(),
    ));
    let state = state.add(SPAssignment::new(
        v!("ur_tcp_id", vec!("svt_tcp")),
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
    let m = OperationModel::new("asdf", state.clone(), vec!(), vec!(op.clone()));
    assert_eq!(op.eval_planning(&m.initial_state), true)
}

#[should_panic]
#[test]
fn test_operation_eval_planning_panic() {
    let state = make_initial_state();
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
    let m = OperationModel::new("asdf", state.clone(), vec!(), vec!(op.clone()));
    assert_eq!(op.eval_planning(&m.initial_state), true)
}

#[test]
fn test_operation_eval_running() {
    let state = make_initial_state();
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
    let m = OperationModel::new("asdf", state.clone(), vec!(), vec!(op.clone()));
    assert_eq!(op.eval_running(&m.initial_state), true)
}

#[should_panic]
#[test]
fn test_operation_eval_running_panic() {
    let state = make_initial_state();
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
    let m = OperationModel::new("asdf", state.clone(), vec!(), vec!(op.clone()));
    assert_eq!(op.eval_running(&m.initial_state), true)
}

#[test]
fn test_operation_take_planning() {
    let state = make_initial_state();
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
    let m = OperationModel::new("asdf", state.clone(), vec!(), vec!(op.clone()));
    let new_state = match op.clone().eval_planning(&m.initial_state) {
        true => op.take_planning(&m.initial_state),
        false => {
            m.initial_state
        }
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
    let m = OperationModel::new("asdf", state.clone(), vec!(), vec!(op.clone()));
    let new_state = match op.clone().eval_running(&m.initial_state) {
        true => op.start_running(&m.initial_state),
        false => {
            m.initial_state
        }
    };
    assert_eq!(new_state.get_value("ur_current_pose"), "a".to_spvalue());
    assert_eq!(new_state.get_value("ur_action_trigger"), true.to_spvalue());
    assert_eq!(new_state.get_value("ur_command"), "movej".to_spvalue());
    assert_eq!(new_state.get_value("ur_goal_feature_id"), "b".to_spvalue());
    assert_eq!(new_state.get_value("ur_tcp_id"), "svt_tcp".to_spvalue());
    assert_eq!(new_state.get_value("op_move_to_b"), "executing".to_spvalue());
}

#[test]
fn test_operation_complete() {
    let state = make_initial_state();
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
    let m = OperationModel::new("asdf", state.clone(), vec!(), vec!(op.clone()));
    let new_state = match op.clone().eval_running(&m.initial_state) {
        true => op.clone().start_running(&m.initial_state),
        false => {
            m.initial_state
        }
    };
    assert_eq!(new_state.get_value("ur_current_pose"), "a".to_spvalue());
    assert_eq!(new_state.get_value("ur_action_trigger"), true.to_spvalue());
    assert_eq!(new_state.get_value("ur_command"), "movej".to_spvalue());
    assert_eq!(new_state.get_value("ur_goal_feature_id"), "b".to_spvalue());
    assert_eq!(new_state.get_value("ur_tcp_id"), "svt_tcp".to_spvalue());
    assert_eq!(new_state.get_value("op_move_to_b"), "executing".to_spvalue());

    let new_state_2 = op.complete_running(&new_state);
    assert_eq!(new_state_2.get_value("ur_current_pose"), "b".to_spvalue());
    assert_eq!(new_state_2.get_value("ur_action_trigger"), false.to_spvalue());
    assert_eq!(new_state_2.get_value("ur_command"), "movej".to_spvalue());
    assert_eq!(new_state_2.get_value("ur_goal_feature_id"), "b".to_spvalue());
    assert_eq!(new_state_2.get_value("ur_tcp_id"), "svt_tcp".to_spvalue());
    assert_eq!(new_state_2.get_value("op_move_to_b"), "initial".to_spvalue());
}