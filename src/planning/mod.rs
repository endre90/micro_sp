pub mod operation;
pub mod transition;

#[cfg(test)]
mod tests {

    use crate::*;

    #[test]
fn test_planning_simple() {
    let pos = v_estimated!("pos", vec!("a", "b", "c", "d", "e", "f"));
    let s = State::from_vec(&vec![(pos.clone(), "a".to_spvalue())]);

    let t1 = t_plan!(
        "a_to_b",
        eq!(pos.wrap(), "a".wrap()),
        vec!(a!(pos.clone(), "b".wrap()))
    );
    let t2 = t_plan!(
        "b_to_c",
        eq!(pos.wrap(), "b".wrap()),
        vec!(a!(pos.clone(), "c".wrap()))
    );
    let t3 = t_plan!(
        "c_to_d",
        eq!(pos.wrap(), "c".wrap()),
        vec!(a!(pos.clone(), "d".wrap()))
    );
    let t4 = t_plan!(
        "d_to_e",
        eq!(pos.wrap(), "d".wrap()),
        vec!(a!(pos.clone(), "e".wrap()))
    );
    let t5 = t_plan!(
        "e_to_f",
        eq!(pos.wrap(), "e".wrap()),
        vec!(a!(pos.clone(), "f".wrap()))
    );
    let t6 = t_plan!(
        "a_to_c",
        eq!(pos.wrap(), "a".wrap()),
        vec!(a!(pos.clone(), "c".wrap()))
    );
    let t7 = t_plan!(
        "d_to_f",
        eq!(pos.wrap(), "d".wrap()),
        vec!(a!(pos.clone(), "f".wrap()))
    );

    let result = bfs_transition_planner(
        s.clone(),
        eq!(pos.wrap(), "f".wrap()),
        vec![
            t1.clone(),
            t2.clone(),
            t3.clone(),
            t4.clone(),
            t5.clone(),
            t6.clone(),
            t7.clone(),
        ],
        10,
    );
    assert_eq!(result.found, true);
    assert_eq!(result.length, 3);
    assert_eq!(result.plan, vec!("a_to_c", "c_to_d", "d_to_f"));

    let result = bfs_transition_planner(
        s.clone(),
        eq!(&pos.wrap(), "a".wrap()),
        vec![
            t1.clone(),
            t2.clone(),
            t3.clone(),
            t4.clone(),
            t5.clone(),
            t6.clone(),
            t7.clone(),
        ],
        10,
    );
    assert_eq!(result.found, true);
    assert_eq!(result.length, 0);
    assert_eq!(result.plan, Vec::<&str>::new());

    let result = bfs_transition_planner(
        s.clone(),
        eq!(&pos.wrap(), "f".wrap()),
        vec![t1.clone(), t2.clone()],
        10,
    );
    assert_eq!(result.found, false);
    assert_eq!(result.length, 0);
    assert_eq!(result.plan, Vec::<&str>::new());
}

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
        v_estimated!("ur_current_pose", vec!("a", "b", "c", "d")),
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
        v_estimated!("ur_goal_feature_id", vec!("a", "b", "c", "d")),
        "a".to_spvalue(),
    ));
    let state = state.add(SPAssignment::new(
        v_estimated!("ur_tcp_id", vec!("svt_tcp")),
        "svt_tcp".to_spvalue(),
    ));
    state
}

#[test]
fn test_operation_planner() {
    let state = make_initial_state();
    let op_move_to_b = v_runner!("op_move_to_b");
    let op_move_to_c = v_runner!("op_move_to_c");
    let op_move_to_d = v_runner!("op_move_to_d");
    let state = state.add(assign!(op_move_to_b, "initial".to_spvalue()));
    let state = state.add(assign!(op_move_to_c, "initial".to_spvalue()));
    let state = state.add(assign!(op_move_to_d, "initial".to_spvalue()));
    let op_move_to_b = Operation::new(
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

    let op_move_to_c = Operation::new(
        "op_move_to_c",
        t!(
            "start_moving_to_c",
            "var:ur_action_trigger == false && var:ur_action_state == initial && var:ur_current_pose == b",
            "true",
            vec!(
                "var:ur_command <- movej", 
                "var:ur_action_trigger <- true", 
                "var:ur_goal_feature_id <- c", 
                "var:ur_tcp_id <- svt_tcp"
            ),
            Vec::<&str>::new(),
            &state
        ),
        t!(
            "complete_moving_to_c",
            "var:ur_action_state == done",
            "true",
            vec!(
                "var:ur_action_trigger <- false", 
                "var:ur_current_pose <- c"
            ),
            Vec::<&str>::new(),
            &state
        )
    );

    let op_move_to_d = Operation::new(
        "op_move_to_d",
        t!(
            "start_moving_to_d",
            "var:ur_action_trigger == false && var:ur_action_state == initial && var:ur_current_pose == c",
            "true",
            vec!(
                "var:ur_command <- movej", 
                "var:ur_action_trigger <- true", 
                "var:ur_goal_feature_id <- d", 
                "var:ur_tcp_id <- svt_tcp"
            ),
            Vec::<&str>::new(),
            &state
        ),
        t!(
            "complete_moving_to_d",
            "var:ur_action_state == done",
            "true",
            vec!(
                "var:ur_action_trigger <- false", 
                "var:ur_current_pose <- d"
            ),
            Vec::<&str>::new(),
            &state
        )
    );

    // Adding the opeation states in the model
    let m = Model::new(
        "asdf",
        state.clone(),
        vec![],
        vec![
            op_move_to_b.clone(),
            op_move_to_c.clone(),
            op_move_to_d.clone(),
        ],
        vec!()
    );

    let goal = pred_parser::pred("var:ur_current_pose == d", &m.state).unwrap();
    let result = bfs_operation_planner(m.state, goal, m.operations, 30);
    assert_eq!(
        vec!("op_move_to_b", "op_move_to_c", "op_move_to_d"),
        result.plan
    );
}


}