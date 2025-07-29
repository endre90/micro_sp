use crate::*;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::time::SystemTime;
use std::{collections::HashMap, fmt};

/// Represents the current state of the system.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct State {
    pub state: HashMap<String, SPAssignment>,
}

/// The Hash trait is implemented on State in order to enable the comparison
/// of different State instances using a hashing function.
impl Hash for State {
    fn hash<H: Hasher>(&self, s: &mut H) {
        self.state
            .keys()
            .into_iter()
            .map(|x| x.to_owned())
            .collect::<Vec<String>>()
            .hash(s);
        self.state
            .values()
            .into_iter()
            .map(|x| x.var.to_owned())
            .collect::<Vec<SPVariable>>()
            .hash(s);
        self.state
            .values()
            .into_iter()
            .map(|x| x.val.to_owned())
            .collect::<Vec<SPValue>>()
            .hash(s);
    }
}

impl State {
    pub fn new() -> State {
        let state = HashMap::new();
        State { state }
    }

    pub fn from_vec(vec: &Vec<(SPVariable, SPValue)>) -> State {
        let mut state = HashMap::new();
        vec.iter().for_each(|(var, val)| {
            state.insert(
                var.name.clone(),
                SPAssignment {
                    var: var.clone(),
                    val: val.clone(),
                },
            );
        });
        State { state }
    }

    /// Get the updated values between two states.
    /// pub fn get_changed_values(&self, other_state: &State) -> HashMap<SPVariable, (SPValue, SPValue)> {
    pub fn get_diff_values(&self, other_state: &State) -> HashMap<SPVariable, (SPValue, SPValue)> {
        let mut changed_values = HashMap::new();

        for (key, self_assignment) in &self.state {
            if let Some(other_assignment) = other_state.state.get(key) {
                if self_assignment.val != other_assignment.val {
                    changed_values.insert(
                        self_assignment.var.clone(),
                        (self_assignment.val.clone(), other_assignment.val.clone()),
                    );
                }
            }
        }

        changed_values
    }

    pub fn get_diff_variables(&self, other_state: &State) -> Vec<SPVariable> {
        let mut uncommon_vars = Vec::new();

        for (key, assignment) in &self.state {
            if !other_state.state.contains_key(key) {
                uncommon_vars.push(assignment.var.clone());
            }
        }

        for (key, assignment) in &other_state.state {
            if !self.state.contains_key(key) {
                uncommon_vars.push(assignment.var.clone());
            }
        }

        uncommon_vars
    }

    // Make a new partial state that only consists of updates.
    pub fn get_diff_partial_state(&self, new_state: &State) -> State {
        let mut updated_assignments = HashMap::new();
        for (key, new_assignment) in &new_state.state {
            if let Some(old_assignment) = self.state.get(key) {
                if old_assignment.val != new_assignment.val {
                    updated_assignments.insert(key.clone(), new_assignment.clone());
                }
            }
        }

        State {
            state: updated_assignments,
        }
    }

    pub fn add(&self, assignment: SPAssignment) -> State {
        match self.state.clone().get(&assignment.var.name) {
            Some(_) => {
                log::error!(target: &&format!("sp_state"), 
                    "Variable {} already in state! Skipped add.", assignment.var.name.to_string());
                self.clone()
            }
            None => {
                let mut state = self.state.clone();
                state.insert(assignment.var.name.to_string(), assignment.clone());
                State { state }
            }
        }
    }

    pub fn get_value(&self, name: &str) -> Option<SPValue> {
        match self.state.clone().get(name) {
            None => {
                log::error!(target: "sp_state::get_value", "Variable {} not in state!", name);
                None
            }
            Some(x) => Some(x.val.clone()),
        }
    }

    pub fn get_bool_or_unknown(&self, name: &str) -> BoolOrUnknown {
        match self.get_value(name) {
            Some(value) => match value {
                SPValue::Bool(b) => b,
                _ => {
                    log::error!(target: "sp_state::get_bool_or_unknown", "Couldn't get boolean '{}' from the state, resulting to UNKNOWN.", name);
                    BoolOrUnknown::UNKNOWN
                }
            },
            None => BoolOrUnknown::UNKNOWN,
        }
    }

    pub fn get_bool_or_default_to_false(&self, name: &str) -> bool {
        match self.get_bool_or_unknown(name) {
            BoolOrUnknown::Bool(b) => b,
            _ => false,
        }
    }

    pub fn get_bool_or_value(&self, name: &str, value: bool) -> bool {
        match self.get_bool_or_unknown(name) {
            BoolOrUnknown::Bool(b) => b,
            _ => value,
        }
    }

    pub fn get_int_or_unknown(&self, name: &str) -> IntOrUnknown {
        match self.get_value(name) {
            Some(value) => match value {
                SPValue::Int64(i) => i,
                _ => {
                    log::error!(target: "sp_state::get_int_or_unknown", "Couldn't get int '{}' from the state, resulting to UNKNOWN.", name);
                    IntOrUnknown::UNKNOWN
                }
            },
            None => IntOrUnknown::UNKNOWN,
        }
    }

    pub fn get_int_or_default_to_zero(&self, name: &str) -> i64 {
        match self.get_int_or_unknown(name) {
            IntOrUnknown::Int64(i) => i,
            _ => 0,
        }
    }

    pub fn get_int_or_value(&self, name: &str, value: i64) -> i64 {
        match self.get_int_or_unknown(name) {
            IntOrUnknown::Int64(i) => i,
            _ => value,
        }
    }

    pub fn get_float_or_unknown(&self, name: &str) -> FloatOrUnknown {
        match self.get_value(name) {
            Some(value) => match value {
                SPValue::Float64(f) => f,
                _ => {
                    log::error!(target: "sp_state::get_float_or_unknown", "Couldn't get float '{}' from the state, resulting to UNKNOWN.", name);
                    FloatOrUnknown::UNKNOWN
                }
            },
            None => FloatOrUnknown::UNKNOWN,
        }
    }

    pub fn get_transform_or_unknown(&self, name: &str) -> TransformOrUnknown {
        match self.get_value(name) {
            Some(value) => match value {
                SPValue::Transform(f) => f,
                _ => {
                    log::error!(target: "sp_state::get_transform_or_unknown", "Couldn't get transform '{}' from the state, resulting to UNKNOWN.", name);
                    TransformOrUnknown::UNKNOWN
                }
            },
            None => TransformOrUnknown::UNKNOWN,
        }
    }

    pub fn get_transform_or_default_to_default(
        &self,
        name: &str,
    ) -> SPTransformStamped {
        match self.get_transform_or_unknown(name) {
            TransformOrUnknown::Transform(t) => t,
            _ => SPTransformStamped {
                active_transform: false,
                enable_transform: false,
                time_stamp: SystemTime::now(),
                parent_frame_id: "world".to_string(),
                child_frame_id: "failed_lookup".to_string(),
                transform: SPTransform::default(),
                metadata: MapOrUnknown::UNKNOWN,
            },
        }
    }

    pub fn get_float_or_default_to_zero(&self, name: &str) -> f64 {
        match self.get_float_or_unknown(name) {
            FloatOrUnknown::Float64(f) => f.into_inner(),
            _ => 0.0,
        }
    }

    pub fn get_float_or_value(&self, name: &str, value: f64) -> f64 {
        match self.get_float_or_unknown(name) {
            FloatOrUnknown::Float64(f) => f.into_inner(),
            _ => value,
        }
    }

    pub fn get_string_or_unknown(&self, name: &str) -> StringOrUnknown {
        match self.get_value(name) {
            Some(value) => match value {
                SPValue::String(s) => s,
                _ => {
                    log::error!(target: "sp_state::get_string_or_unknown", "Couldn't get string '{}' from the state, resulting to UNKNOWN.", name);
                    StringOrUnknown::UNKNOWN
                }
            },
            None => StringOrUnknown::UNKNOWN,
        }
    }

    pub fn get_string_or_default_to_unknown(&self, name: &str) -> String {
        match self.get_string_or_unknown(name) {
            StringOrUnknown::String(s) => s,
            _ => SPValue::String(StringOrUnknown::UNKNOWN).to_string(),
        }
    }

    pub fn get_string_or_value(&self, name: &str, value: String) -> String {
        match self.get_string_or_unknown(name) {
            StringOrUnknown::String(s) => s,
            _ => value,
        }
    }

    pub fn get_array_or_unknown(&self, name: &str) -> ArrayOrUnknown {
        match self.get_value(name) {
            Some(value) => match value {
                SPValue::Array(a) => a,
                _ => {
                    log::error!(target: "sp_state::get_array_or_unknown", "Couldn't get array '{}' from the state, resulting to UNKNOWN.", name);
                    ArrayOrUnknown::UNKNOWN
                }
            },
            None => ArrayOrUnknown::UNKNOWN,
        }
    }

    pub fn get_array_or_default_to_empty(&self, name: &str) -> Vec<SPValue> {
        match self.get_array_or_unknown(name) {
            ArrayOrUnknown::Array(a) => a,
            _ => {
                vec![]
            }
        }
    }

    pub fn get_array_or_value(
        &self,
        name: &str,
        value: Vec<SPValue>,
    ) -> Vec<SPValue> {
        match self.get_array_or_unknown(name) {
            ArrayOrUnknown::Array(a) => a,
            _ => value,
        }
    }

    pub fn get_map_or_unknown(&self, name: &str) -> MapOrUnknown {
        match self.get_value(name) {
            Some(value) => match value {
                SPValue::Map(m) => m,
                _ => {
                    log::error!(target: "sp_state::get_map_or_unknown", "Couldn't get map '{}' from the state, resulting to UNKNOWN.", name);
                    MapOrUnknown::UNKNOWN
                }
            },
            None => MapOrUnknown::UNKNOWN,
        }
    }

    pub fn get_map_or_default_to_empty(&self, name: &str) -> Vec<(SPValue, SPValue)> {
        match self.get_map_or_unknown(name) {
            MapOrUnknown::Map(m) => m,
            _ => {
                vec![]
            }
        }
    }

    pub fn get_map_or_value(
        &self,
        name: &str,
        value: Vec<(SPValue, SPValue)>,
    ) -> Vec<(SPValue, SPValue)> {
        match self.get_map_or_unknown(name) {
            MapOrUnknown::Map(m) => m,
            _ => value,
        }
    }

    pub fn get_time_or_unknown(&self, name: &str) -> TimeOrUnknown {
        match self.get_value(name) {
            Some(value) => match value {
                SPValue::Time(t) => t,
                _ => {
                    log::error!(target: "sp_state::get_time_or_unknown", "Couldn't get time '{}' from the state, resulting to UNKNOWN.", name);
                    TimeOrUnknown::UNKNOWN
                }
            },
            None => TimeOrUnknown::UNKNOWN,
        }
    }

    pub fn get_assignment(&self, name: &str) -> SPAssignment {
        match self.state.clone().get(name) {
            None => panic!("Variable {} not in state!", name),
            Some(x) => x.clone(),
        }
    }

    pub fn get_all_vars(&self) -> Vec<SPVariable> {
        self.state
            .iter()
            .map(|(_, assignment)| assignment.var.clone())
            .collect()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.state.clone().contains_key(name)
    }

    pub fn update(&self, name: &str, val: SPValue) -> State {
        match self.state.get(name) {
            Some(assignment) => {
                let mut state = self.state.clone();
                state.insert(
                    name.to_string(),
                    SPAssignment {
                        var: assignment.var.clone(),
                        val: val.clone(),
                    },
                );
                State { state }
            }
            None => panic!("Variable {} not in state.", name),
        }
    }

    pub fn extend(&self, other: State, overwrite_existing: bool) -> State {
        let existing = self.state.clone();
        let extension = other.state;
        let mut state = HashMap::<String, SPAssignment>::new();
        if overwrite_existing {
            existing.iter().for_each(|(k, v)| {
                state.insert(k.clone(), v.clone());
            });
            extension.iter().for_each(|(k, v)| {
                state.insert(k.clone(), v.clone());
            });
            State { state }
        } else {
            extension.iter().for_each(|(k, v)| {
                state.insert(k.clone(), v.clone());
            });
            existing.iter().for_each(|(k, v)| {
                state.insert(k.clone(), v.clone());
            });
            State { state }
        }
    }

    pub fn extract_goal(&self, name: &str) -> Predicate {
        match self.state.get(&format!("{}_current_goal_predicate", name)) {
            Some(g_spvalue) => match &g_spvalue.val {
                SPValue::String(StringOrUnknown::String(g_value)) => {
                    match pred_parser::pred(&g_value, &self) {
                        Ok(goal_predicate) => goal_predicate,
                        Err(_) => Predicate::TRUE,
                    }
                }
                _ => Predicate::TRUE,
            },
            None => Predicate::TRUE,
        }
    }
}

impl fmt::Display for State {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: String = {
            let mut children: Vec<_> = self
                .state
                .iter()
                .map(|(k, v)| match &v.val {
                    SPValue::Array(arr) => match arr {
                        ArrayOrUnknown::UNKNOWN => format!("    {}: {}", k, v.val),
                        ArrayOrUnknown::Array(some_array) => {
                            let mut sub_children: Vec<String> = vec![format!("    {}:", k)];
                            sub_children.extend(
                                some_array
                                    .iter()
                                    .map(|value| format!("        {}", value))
                                    .collect::<Vec<String>>(),
                            );
                            format!("{}", sub_children.join("\n"))
                        }
                    },
                    _ => format!("    {}: {}", k, v.val),
                })
                .collect();
            children.sort();
            format!("{}", children.join("\n"))
        };

        write!(fmtr, "State: {{\n{}\n}}\n", &s)
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use std::time::SystemTime;

    fn create_dummy_transform() -> SPTransformStamped {
        SPTransformStamped {
            active_transform: true,
            enable_transform: true,
            time_stamp: SystemTime::now(),
            parent_frame_id: "world".to_string(),
            child_frame_id: "robot".to_string(),
            transform: SPTransform::default(),
            metadata: MapOrUnknown::Map(vec![("quality".to_spvalue(), "good".to_spvalue())]),
        }
    }

    fn get_initial_state() -> State {
        let name = SPVariable::new("name", SPValueType::String);
        let height = SPVariable::new("height", SPValueType::Int64);
        let smart = SPVariable::new("smart", SPValueType::Bool);
        let weight = SPVariable::new("weight", SPValueType::Float64);
        let items = SPVariable::new("items", SPValueType::Array);
        let data = SPVariable::new("data", SPValueType::Map);
        let pose = SPVariable::new("pose", SPValueType::Transform);
        let time = SPVariable::new("time", SPValueType::Time);

        State::from_vec(&vec![
            (name, "John".to_spvalue()),
            (height, 185.to_spvalue()),
            (smart, true.to_spvalue()),
            (weight, 80.0.to_spvalue()),
            (items, vec![1.to_spvalue()].to_spvalue()),
            (
                data,
                vec![("a".to_spvalue(), "b".to_spvalue())].to_spvalue(),
            ),
            (pose, create_dummy_transform().to_spvalue()),
            (time, SystemTime::now().to_spvalue()),
        ])
    }

    #[test]
    fn test_state_new_and_from_vec() {
        let new_state = State::new();
        assert!(new_state.state.is_empty());

        let initial_state = get_initial_state();
        assert_eq!(initial_state.state.len(), 8);
    }

    #[test]
    fn test_get_value_and_assignment() {
        let state = get_initial_state();
        assert_eq!(state.get_value("height"), Some(185.to_spvalue()));
        let assignment = state.get_assignment("height");
        assert_eq!(assignment.var.name, "height");
        assert_eq!(assignment.val, 185.to_spvalue());
    }

    #[test]
    #[should_panic]
    fn test_get_value_panic() {
        let state = State::new();
        state.get_value( "nonexistent").unwrap();
    }

    #[test]
    #[should_panic]
    fn test_get_assignment_panic() {
        let state = State::new();
        state.get_assignment("nonexistent");
    }

    #[test]
    fn test_add() {
        let state = State::new();
        let var = SPVariable::new("v", SPValueType::Bool);
        let assignment = SPAssignment::new(var, true.to_spvalue());
        let new_state = state.add(assignment.clone());
        assert_eq!(new_state.state.len(), 1);
        let same_state = new_state.add(assignment);
        assert_eq!(same_state.state.len(), 1);
    }

    #[test]
    fn test_update() {
        let state = get_initial_state();
        let updated_state = state.update("height", 190.to_spvalue());
        assert_eq!(updated_state.get_value("height"), Some(190.to_spvalue()));
    }

    #[test]
    #[should_panic]
    fn test_update_panic() {
        let state = State::new();
        state.update("nonexistent", 1.to_spvalue());
    }

    #[test]
    fn test_contains_and_get_all_vars() {
        let state = get_initial_state();
        assert!(state.contains("name"));
        assert!(!state.contains("age"));
        let vars = state.get_all_vars();
        assert_eq!(vars.len(), 8);
        assert!(vars.contains(&SPVariable::new("name", SPValueType::String)));
    }

    #[test]
    fn test_get_diff_values() {
        let var_a = SPVariable::new("a", SPValueType::Int64);
        let var_b = SPVariable::new("b", SPValueType::Bool);
        let var_c = SPVariable::new("c", SPValueType::String);

        let state1 = State::from_vec(&vec![
            (var_a.clone(), 1.to_spvalue()),
            (var_b.clone(), true.to_spvalue()),
            (var_c.clone(), "hello".to_spvalue()),
        ]);

        let var_d = SPVariable::new("d", SPValueType::Float64);
        let state2 = State::from_vec(&vec![
            (var_a.clone(), 2.to_spvalue()),    // Changed value
            (var_b.clone(), true.to_spvalue()), // Same value
            (var_d.clone(), 3.14.to_spvalue()), // Not in state1
        ]);

        let changed = state1.get_diff_values(&state2);

        assert_eq!(changed.len(), 1);
        assert!(changed.contains_key(&var_a));

        let (old_val, new_val) = changed.get(&var_a).unwrap();
        assert_eq!(*old_val, 1.to_spvalue());
        assert_eq!(*new_val, 2.to_spvalue());

        let no_changes = state1.get_diff_values(&state1);
        assert!(no_changes.is_empty());
    }

    #[test]
    fn test_get_diff_variables() {
        let var_a = SPVariable::new("a", SPValueType::Int64);
        let var_b = SPVariable::new("b", SPValueType::Bool);
        let var_c = SPVariable::new("c", SPValueType::String);

        let state1 = State::from_vec(&vec![
            (var_a.clone(), 1.to_spvalue()),
            (var_b.clone(), true.to_spvalue()),
        ]);

        let state2 = State::from_vec(&vec![
            (var_b.clone(), false.to_spvalue()),
            (var_c.clone(), "hello".to_spvalue()),
        ]);

        let mut uncommon = state1.get_diff_variables(&state2);
        uncommon.sort(); // Sort for consistent test results

        let mut expected = vec![var_a.clone(), var_c];
        expected.sort();

        assert_eq!(uncommon, expected);

        // Test with no uncommon variables
        let no_uncommon = state1.get_diff_variables(&state1);
        assert!(no_uncommon.is_empty());

        // Test with an empty state
        let empty_state = State::new();
        let mut uncommon_with_empty = state1.get_diff_variables(&empty_state);
        uncommon_with_empty.sort();

        let mut expected_with_empty = vec![var_a.clone(), var_b.clone()];
        expected_with_empty.sort();

        assert_eq!(uncommon_with_empty, expected_with_empty);
    }

    #[test]
    fn test_get_diff_partial_state() {
        let var_a = SPVariable::new("a", SPValueType::Int64);
        let var_b = SPVariable::new("b", SPValueType::Bool);

        let state1 = State::from_vec(&vec![
            (var_a.clone(), 1.to_spvalue()),
            (var_b.clone(), true.to_spvalue()),
        ]);

        let var_c = SPVariable::new("c", SPValueType::String);
        let state2 = State::from_vec(&vec![
            (var_a.clone(), 2.to_spvalue()),     // Updated
            (var_b.clone(), true.to_spvalue()),  // Unchanged
            (var_c.clone(), "new".to_spvalue()), // New, should be ignored
        ]);

        let updated_state = state1.get_diff_partial_state(&state2);

        assert_eq!(
            updated_state.state.len(),
            1,
            "Only the updated variable should be in the new state"
        );
        assert!(
            updated_state.contains("a"),
            "The updated variable 'a' should be present"
        );
        assert_eq!(
            updated_state.get_value("a"),
            Some(2.to_spvalue()),
            "The value of 'a' should be the new value"
        );
    }

    #[test]
    fn test_extend() {
        let state1 = State::from_vec(&vec![(
            SPVariable::new("a", SPValueType::Int64),
            1.to_spvalue(),
        )]);
        let state2 = State::from_vec(&vec![
            (SPVariable::new("a", SPValueType::Int64), 2.to_spvalue()),
            (SPVariable::new("b", SPValueType::Int64), 3.to_spvalue()),
        ]);

        let extended_overwrite = state1.extend(state2.clone(), true);
        assert_eq!(extended_overwrite.state.len(), 2);
        assert_eq!(extended_overwrite.get_value("a"), Some(2.to_spvalue()));

        let extended_no_overwrite = state1.extend(state2.clone(), false);
        assert_eq!(extended_no_overwrite.state.len(), 2);
        assert_eq!(extended_no_overwrite.get_value("a"), Some(1.to_spvalue()));
    }

    #[test]
    fn test_getters() {
        let state = get_initial_state();
        let wrong_type_state = State::from_vec(&vec![(
            SPVariable::new("smart", SPValueType::Int64),
            0.to_spvalue(),
        )]);

        assert_eq!(
            state.get_bool_or_unknown("smart"),
            BoolOrUnknown::Bool(true)
        );
        assert_eq!(
            wrong_type_state.get_bool_or_unknown("smart"),
            BoolOrUnknown::UNKNOWN
        );
        assert!(state.get_bool_or_default_to_false("smart"));
        assert!(!wrong_type_state.get_bool_or_default_to_false("smart"));
        assert!(state.get_bool_or_value("smart", false));
        assert!(!wrong_type_state.get_bool_or_value("smart", false));

        assert_eq!(
            state.get_int_or_unknown("height"),
            IntOrUnknown::Int64(185)
        );
        assert_eq!(state.get_int_or_default_to_zero("height"), 185);
        assert_eq!(state.get_int_or_value("height", 0), 185);

        assert_eq!(
            state.get_float_or_unknown("weight"),
            FloatOrUnknown::Float64(80.0.into())
        );
        assert_eq!(state.get_float_or_default_to_zero("weight"), 80.0);
        assert_eq!(state.get_float_or_value("weight", 0.0), 80.0);

        assert_eq!(
            state.get_string_or_unknown("name"),
            StringOrUnknown::String("John".to_string())
        );
        assert_eq!(
            state.get_string_or_default_to_unknown("name"),
            "John".to_string()
        );
        assert_eq!(
            state.get_string_or_value("name", "".to_string()),
            "John".to_string()
        );

        assert_eq!(
            state.get_array_or_unknown("items"),
            ArrayOrUnknown::Array(vec![1.to_spvalue()])
        );
        assert_eq!(
            state.get_array_or_default_to_empty("items"),
            vec![1.to_spvalue()]
        );
        assert_eq!(
            state.get_array_or_value("items", vec![]),
            vec![1.to_spvalue()]
        );

        assert_eq!(
            state.get_map_or_unknown("data"),
            MapOrUnknown::Map(vec![("a".to_spvalue(), "b".to_spvalue())])
        );
        assert_eq!(
            state.get_map_or_default_to_empty("data"),
            vec![("a".to_spvalue(), "b".to_spvalue())]
        );
        assert_eq!(
            state.get_map_or_value("data", vec![]),
            vec![("a".to_spvalue(), "b".to_spvalue())]
        );

        assert!(matches!(
            state.get_time_or_unknown("time"),
            TimeOrUnknown::Time(_)
        ));

        assert!(matches!(
            state.get_transform_or_unknown("pose"),
            TransformOrUnknown::Transform(_)
        ));
        let default_tf = state.get_transform_or_default_to_default("pose");
        assert_eq!(default_tf.parent_frame_id, "world");
    }

    #[test]
    fn test_getters_defaults() {
        let state = State::new();
        assert_eq!(state.get_string_or_default_to_unknown("x"), "UNKNOWN");
        assert_eq!(state.get_array_or_default_to_empty("x"), vec![]);
        assert_eq!(state.get_map_or_default_to_empty("x"), vec![]);
        let default_tf = state.get_transform_or_default_to_default("x");
        assert_eq!(default_tf.child_frame_id, "failed_lookup");
    }

    #[test]
    fn test_display() {
        let state = get_initial_state();
        let display_str = format!("{}", state);
        assert!(display_str.starts_with("State: {\n"));
        assert!(display_str.contains("    name: John\n"));
        assert!(display_str.contains("    height: 185\n"));
        assert!(display_str.ends_with("}\n"));

        let arr_state = State::from_vec(&vec![(
            SPVariable::new("arr", SPValueType::Array),
            vec![1.to_spvalue(), 2.to_spvalue()].to_spvalue(),
        )]);
        let arr_display = format!("{}", arr_state);
        assert!(arr_display.contains("    arr:\n        1\n        2"));

        let unk_arr_state = State::from_vec(&vec![(
            SPVariable::new("unk", SPValueType::Array),
            SPValue::Array(ArrayOrUnknown::UNKNOWN),
        )]);
        assert!(format!("{}", unk_arr_state).contains("    unk: UNKNOWN"));
    }

    #[test]
    fn test_extract_goal() {
        // Should be tested using pred_parser
        let state_no_goal = get_initial_state();
        assert_eq!(state_no_goal.extract_goal("g"), Predicate::TRUE);

        let state_with_bad_goal = state_no_goal.add(SPAssignment::new(
            SPVariable::new("g_current_goal_predicate", SPValueType::Int64),
            1.to_spvalue(),
        ));
        assert_eq!(state_with_bad_goal.extract_goal("g"), Predicate::TRUE);
    }
}

// #[cfg(test)]
// mod tests {

//     use crate::*;

//     fn john_doe() -> Vec<(SPVariable, SPValue)> {
//         let name = v!("name");
//         let surname = v!("surname");
//         let height = iv!("height");
//         let weight = fv!("weight");
//         let smart = bv!("smart");

//         vec![
//             (name, "John".to_spvalue()),
//             (surname, "Doe".to_spvalue()),
//             (height, 185.to_spvalue()),
//             (weight, 80.0.to_spvalue()),
//             (smart, true.to_spvalue()),
//         ]
//     }

//     fn _john_doe_faulty() -> Vec<(SPVariable, SPValue)> {
//         let name = v!("name");
//         let surname = v!("surname");
//         let height = iv!("height");
//         let weight = fv!("weight");
//         let smart = bv!("smart");

//         vec![
//             (name, "John".to_spvalue()),
//             (surname, "Doe".to_spvalue()),
//             (height, 185.to_spvalue()),
//             (weight, 81.0.to_spvalue()),
//             (smart, true.to_spvalue()),
//         ]
//     }

//     #[test]
//     fn test_state_new() {
//         let new_state = State::new();
//         assert_eq!(new_state.state.len(), 0)

//     }

//     #[test]
//     fn test_state_from_vec() {
//         let john_doe = john_doe();
//         let new_state = State::from_vec(&john_doe);
//         assert_eq!(new_state.state.len(), 5)
//     }

//     #[test]
//     fn test_state_display() {
//         let john_doe = john_doe();
//         let new_state = State::from_vec(&john_doe);
//         print!("{}", new_state)
//     }

//     #[test]
//     #[should_panic]
//     fn test_state_from_vec_panic() {
//         let john_doe = john_doe();
//         let new_state = State::from_vec(&john_doe);
//         assert_eq!(new_state.state.len(), 6)
//     }

//     #[test]
//     fn test_state_get_value() {
//         let john_doe = john_doe();
//         let state = State::from_vec(&john_doe);
//         assert_eq!(185.to_spvalue(), state.get_value("height"));
//         assert_ne!(186.to_spvalue(), state.get_value("height"));
//     }

//     #[test]
//     fn test_state_get_all() {
//         let john_doe = john_doe();
//         let state = State::from_vec(&john_doe);
//         assert_eq!(
//             SPAssignment {
//                 var: iv!("height"),
//                 val: 185.to_spvalue()
//             },
//             state.get_assignment("height")
//         );
//         assert_ne!(
//             SPAssignment {
//                 var: iv!("height"),
//                 val: 186.to_spvalue()
//             },
//             state.get_assignment("height")
//         );
//     }

//     #[test]
//     fn test_state_contains() {
//         let john_doe = john_doe();
//         let state = State::from_vec(&john_doe);
//         assert_eq!(true, state.contains("height"));
//         assert_ne!(true, state.contains("wealth"));
//     }

//     #[test]
//     fn test_state_add_not_mutable() {
//         let john_doe = john_doe();
//         let state = State::from_vec(&john_doe);
//         let wealth = iv!("wealth");
//         state.add(assign!(wealth, 2000.to_spvalue()));
//         assert_ne!(state.state.len(), 6)
//     }

//     #[test]
//     fn test_state_add() {
//         let john_doe = john_doe();
//         let state = State::from_vec(&john_doe);
//         let wealth = iv!("wealth");
//         let state = state.add(assign!(wealth, 2000.to_spvalue()));
//         assert_eq!(state.state.len(), 6)
//     }

//     #[test]
//     #[should_panic]
//     fn test_state_add_already_exists() {
//         let john_doe = john_doe();
//         let state = State::from_vec(&john_doe);
//         let wealth = iv!("height");
//         let state = state.add(assign!(wealth, 2000.to_spvalue()));
//         assert_eq!(state.state.len(), 6)
//     }

//     #[test]
//     fn test_state_update() {
//         let john_doe = john_doe();
//         let state = State::from_vec(&john_doe);
//         let state = state.update("height", 190.to_spvalue());
//         assert_eq!(state.get_value("height"), 190.to_spvalue())
//     }
// }
