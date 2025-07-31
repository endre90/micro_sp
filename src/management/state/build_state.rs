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

#[cfg(test)]
mod tests_for_build_state {
    use super::{build_state, create_assignment};
    use crate::*;
    use std::collections::HashMap;

    #[test]
    fn test_build_state_full_success() {
        let key1 = "my_int".to_string();
        let val1 = SPValue::Int64(IntOrUnknown::Int64(123));
        let key2 = "my_str".to_string();
        let val2 = SPValue::String(StringOrUnknown::String("abc".to_string()));
        let key3 = "my_bool".to_string();
        let val3 = SPValue::Bool(BoolOrUnknown::Bool(true));

        let keys = vec![key1.clone(), key2.clone(), key3.clone()];
        let values = vec![
            Some(serde_json::to_string(&val1).unwrap()),
            Some(serde_json::to_string(&val2).unwrap()),
            Some(serde_json::to_string(&val3).unwrap()),
        ];

        let result_state = build_state(keys, values);

        let mut expected_map = HashMap::new();
        expected_map.insert(key1.clone(), create_assignment(&key1, val1));
        expected_map.insert(key2.clone(), create_assignment(&key2, val2));
        expected_map.insert(key3.clone(), create_assignment(&key3, val3));

        assert_eq!(result_state.state, expected_map);
    }

    #[test]
    fn test_build_state_with_missing_values() {
        let key1 = "my_int".to_string();
        let val1 = SPValue::Int64(IntOrUnknown::Int64(123));
        let key2 = "key_with_no_value".to_string();
        let key3 = "my_bool".to_string();
        let val3 = SPValue::Bool(BoolOrUnknown::Bool(true));

        let keys = vec![key1.clone(), key2.clone(), key3.clone()];
        let values = vec![
            Some(serde_json::to_string(&val1).unwrap()),
            None,
            Some(serde_json::to_string(&val3).unwrap()),
        ];

        let result_state = build_state(keys, values);

        let mut expected_map = HashMap::new();
        expected_map.insert(key1.clone(), create_assignment(&key1, val1));
        expected_map.insert(key3.clone(), create_assignment(&key3, val3));

        assert_eq!(result_state.state, expected_map);
    }

    #[test]
    fn test_build_state_with_deserialization_error() {
        let key1 = "good_val".to_string();
        let val1 = SPValue::Int64(IntOrUnknown::Int64(123));
        let key2 = "bad_val".to_string();

        let keys = vec![key1.clone(), key2.clone()];
        let values = vec![
            Some(serde_json::to_string(&val1).unwrap()),
            Some("{ not json }".to_string()),
        ];

        let result_state = build_state(keys, values);

        let mut expected_map = HashMap::new();
        expected_map.insert(key1.clone(), create_assignment(&key1, val1));

        assert_eq!(result_state.state, expected_map);
    }

    #[test]
    fn test_build_state_empty_input() {
        let keys = vec![];
        let values = vec![];

        let result_state = build_state(keys, values);

        assert!(result_state.state.is_empty());
    }

    #[test]
    fn test_build_state_all_values_none() {
        let keys = vec!["a".to_string(), "b".to_string()];
        let values = vec![None, None];

        let result_state = build_state(keys, values);

        assert!(result_state.state.is_empty());
    }
}
