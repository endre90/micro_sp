use crate::{SPAssignment, SPValueType, SPVariable, State, ToSPValue};

pub fn make_initial_state() -> State {
    let state = State::new_empty();
    let state = state.add(&SPAssignment {
        var: SPVariable::new(
            "ur_action_trigger_key",
            &SPValueType::Bool,
            &vec![true.to_spval(), false.to_spval()],
        ),
        val: false.to_spval(),
    });
    let state = state.add(&SPAssignment {
        var: SPVariable::new(
            "ur_command",
            &SPValueType::String,
            &vec!["movej".to_spval(), "movel".to_spval()],
        ),
        val: "movej".to_spval(),
    });
    let state = state.add(&SPAssignment {
        var: SPVariable::new(
            "ur_velocity",
            &SPValueType::Float64,
            &vec![0.1.to_spval(), 0.2.to_spval(), 0.3.to_spval()],
        ),
        val: 0.1.to_spval(),
    });
    let state = state.add(&SPAssignment {
        var: SPVariable::new(
            "ur_acceleration",
            &SPValueType::Float64,
            &vec![0.2.to_spval(), 0.4.to_spval(), 0.6.to_spval()],
        ),
        val: 0.2.to_spval(),
    });
    let state = state.add(&SPAssignment {
        var: SPVariable::new(
            "ur_goal_feature_id",
            &SPValueType::String,
            &vec!["a".to_spval(), "b".to_spval(), "c".to_spval()],
        ),
        val: "a".to_spval(),
    });
    let state = state.add(&SPAssignment {
        var: SPVariable::new(
            "ur_tcp_id",
            &SPValueType::String,
            &vec!["svt_tcp".to_spval()],
        ),
        val: "svt_tcp".to_spval(),
    });
    state
}
