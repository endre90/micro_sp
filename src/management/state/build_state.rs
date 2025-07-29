use std::collections::HashMap;

use crate::{
    SPAssignment, SPValue, SPValueType, SPVariable, State, av, bv, fv, iv, mv, tfv, tv, v,
};

fn create_assignment(key: &str, value: SPValue) -> SPAssignment {
    let variable = match &value {
        SPValue::Bool(_) => bv!(key),
        SPValue::Float64(_) => fv!(key),
        SPValue::Int64(_) => iv!(key),
        SPValue::String(_) => v!(key),
        SPValue::Time(_) => tv!(key),
        SPValue::Array(_) => av!(key),
        SPValue::Map(_) => mv!(key),
        SPValue::Transform(_) => tfv!(key),
    };
    SPAssignment::new(variable, value)
}

pub(super) fn build_state(keys: Vec<String>, values: Vec<Option<String>>) -> State {
    let mut state_map = HashMap::new();

    for (key, maybe_value) in keys.into_iter().zip(values.into_iter()) {
        let Some(value_str) = maybe_value else {
            continue;
        };

        if let Ok(sp_value) = serde_json::from_str::<SPValue>(&value_str) {
            let assignment = create_assignment(&key, sp_value);
            state_map.insert(key, assignment);
        } else {
            log::warn!("Failed to deserialize value for key '{}'.", key);
        }
    }

    State { state: state_map }
}
