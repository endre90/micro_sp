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
    /// Create and returns a new State instance.
    pub fn new() -> State {
        let mut state = HashMap::new();
        state.insert("empty".to_string(), SPAssignment::new(
            SPVariable {
                name: "empty".to_string(),
                value_type: SPValueType::Bool,
            },
            false.to_spvalue(),
        ));
        State {
            state
        }
    }

    /// Create a new State from a vector of (SPVariable, SPValue) tuples.
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
    pub fn get_diff(&self, new_state: &State) -> HashMap<SPVariable, (SPValue, SPValue)> {
        let mut modified = HashMap::new();
        for (key, new_assignment) in &new_state.state {
            let old_value = self.get_value(&key);
            if old_value != new_assignment.val {
                modified.insert(
                    new_assignment.var.clone(),
                    (old_value.clone(), new_assignment.val.clone()),
                );
            }
        }
        modified
    }

    /// Make a new partial state that only consists of updates.
    pub fn get_diff_partial_state(&self, new_state: &State) -> State {
        let mut modified = HashMap::new();
        for (key, new_assignment) in &new_state.state {
            let old_value = self.get_value(&key);
            if old_value != new_assignment.val {
                modified.insert(new_assignment.var.name.clone(), new_assignment.clone());
            }
        }
        State { state: modified }
    }

    /// Add an SPAssignment to the State, returning a new State.
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

    /// Returns the SPValue of the variable with the given name from the state.
    /// Panics if the variable is not in the state.
    pub fn get_value(&self, name: &str) -> SPValue {
        match self.state.clone().get(name) {
            None => panic!("Variable {} not in state!", name),
            Some(x) => x.val.clone(),
        }
    }

    pub fn get_bool_or_unknown(&self, target: &str, name: &str) -> BoolOrUnknown {
        match self.get_value(name) {
            SPValue::Bool(b) => b,
            _ => {
                log::error!(target: target, "Couldn't get boolean '{}' from the state, resulting to UNKNOWN.", name);
                BoolOrUnknown::UNKNOWN
            }
        }
    }

    pub fn get_bool_or_default_to_false(&self, target: &str, name: &str) -> bool {
        match self.get_bool_or_unknown(target, name) {
            BoolOrUnknown::Bool(b) => b,
            _ => false,
        }
    }

    pub fn get_bool_or_value(&self, target: &str, name: &str, value: bool) -> bool {
        match self.get_bool_or_unknown(target, name) {
            BoolOrUnknown::Bool(b) => b,
            _ => value,
        }
    }

    pub fn get_int_or_unknown(&self, target: &str, name: &str) -> IntOrUnknown {
        match self.get_value(name) {
            SPValue::Int64(i) => i,
            _ => {
                log::error!(target: target, "Couldn't get int '{}' from the state, resulting to UNKNOWN.", name);
                IntOrUnknown::UNKNOWN
            }
        }
    }

    pub fn get_int_or_default_to_zero(&self, target: &str, name: &str) -> i64 {
        match self.get_int_or_unknown(target, name) {
            IntOrUnknown::Int64(i) => i,
            _ => 0,
        }
    }

    pub fn get_int_or_value(&self, target: &str, name: &str, value: i64) -> i64 {
        match self.get_int_or_unknown(target, name) {
            IntOrUnknown::Int64(i) => i,
            _ => value,
        }
    }

    pub fn get_float_or_unknown(&self, target: &str, name: &str) -> FloatOrUnknown {
        match self.get_value(name) {
            SPValue::Float64(f) => f,
            _ => {
                log::error!(target: target, "Couldn't get float '{}' from the state, resulting to UNKNOWN.", name);
                FloatOrUnknown::UNKNOWN
            }
        }
    }

    pub fn get_transform_or_unknown(&self, target: &str, name: &str) -> TransformOrUnknown {
        match self.get_value(name) {
            SPValue::Transform(f) => f,
            _ => {
                log::error!(target: target, "Couldn't get transform '{}' from the state, resulting to UNKNOWN.", name);
                TransformOrUnknown::UNKNOWN
            }
        }
    }

    pub fn get_transform_or_default_to_default(&self, target: &str, name: &str) -> SPTransformStamped {
        match self.get_transform_or_unknown(target, name) {
            TransformOrUnknown::Transform(t) => t,
            _ => SPTransformStamped { 
                active_transform: false, 
                enable_transform: false, 
                time_stamp: SystemTime::now(), 
                parent_frame_id: "world".to_string(), 
                child_frame_id: "failed_lookup".to_string(), 
                transform: SPTransform::default(), 
                metadata: MapOrUnknown::UNKNOWN 
            },
        }
    }

    pub fn get_float_or_default_to_zero(&self, target: &str, name: &str) -> f64 {
        match self.get_float_or_unknown(target, name) {
            FloatOrUnknown::Float64(f) => f.into_inner(),
            _ => 0.0,
        }
    }

    pub fn get_float_or_value(&self, target: &str, name: &str, value: f64) -> f64 {
        match self.get_float_or_unknown(target, name) {
            FloatOrUnknown::Float64(f) => f.into_inner(),
            _ => value,
        }
    }

    pub fn get_string_or_unknown(&self, target: &str, name: &str) -> StringOrUnknown {
        match self.get_value(name) {
            SPValue::String(s) => s,
            _ => {
                log::error!(target: target, "Couldn't get string '{}' from the state, resulting to UNKNOWN.", name);
                StringOrUnknown::UNKNOWN
            }
        }
    }

    pub fn get_string_or_default_to_unknown(&self, target: &str, name: &str) -> String {
        match self.get_string_or_unknown(target, name) {
            StringOrUnknown::String(s) => s,
            _ => SPValue::String(StringOrUnknown::UNKNOWN).to_string(),
        }
    }

    pub fn get_string_or_value(&self, target: &str, name: &str, value: String) -> String {
        match self.get_string_or_unknown(target, name) {
            StringOrUnknown::String(s) => s,
            _ => value,
        }
    }

    pub fn get_array_or_unknown(&self, target: &str, name: &str) -> ArrayOrUnknown {
        match self.get_value(name) {
            SPValue::Array(a) => a,
            _ => {
                log::error!(target: target, "Couldn't get array '{}' from the state, resulting to UNKNOWN.", name);
                ArrayOrUnknown::UNKNOWN
            }
        }
    }

    pub fn get_array_or_default_to_empty(&self, target: &str, name: &str) -> Vec<SPValue> {
        match self.get_array_or_unknown(target, name) {
            ArrayOrUnknown::Array(a) => a,
            _ => {
                vec![]
            }
        }
    }

    pub fn get_array_or_value(
        &self,
        target: &str,
        name: &str,
        value: Vec<SPValue>,
    ) -> Vec<SPValue> {
        match self.get_array_or_unknown(target, name) {
            ArrayOrUnknown::Array(a) => a,
            _ => value,
        }
    }

    pub fn get_map_or_unknown(&self, target: &str, name: &str) -> MapOrUnknown {
        match self.get_value(name) {
            SPValue::Map(m) => m,
            _ => {
                log::error!(target: target, "Couldn't get map '{}' from the state, resulting to UNKNOWN.", name);
                MapOrUnknown::UNKNOWN
            }
        }
    }

    pub fn get_map_or_default_to_empty(&self, target: &str, name: &str) -> Vec<(SPValue, SPValue)> {
        match self.get_map_or_unknown(target, name) {
            MapOrUnknown::Map(m) => m,
            _ => {
                vec![]
            }
        }
    }

    pub fn get_map_or_value(
        &self,
        target: &str,
        name: &str,
        value: Vec<(SPValue, SPValue)>,
    ) -> Vec<(SPValue, SPValue)> {
        match self.get_map_or_unknown(target, name) {
            MapOrUnknown::Map(m) => m,
            _ => value,
        }
    }

    pub fn get_time_or_unknown(&self, target: &str, name: &str) -> TimeOrUnknown {
        match self.get_value(name) {
            SPValue::Time(t) => t,
            _ => {
                log::error!(target: target, "Couldn't get time '{}' from the state, resulting to UNKNOWN.", name);
                TimeOrUnknown::UNKNOWN
            }
        }
    }

    /// Get the assignment of a variable in the state,
    /// or panic if the variable is not found.
    pub fn get_assignment(&self, name: &str) -> SPAssignment {
        match self.state.clone().get(name) {
            None => panic!("Variable {} not in state!", name),
            Some(x) => x.clone(),
        }
    }

    /// Get all variables from the state.
    pub fn get_all_vars(&self) -> Vec<SPVariable> {
        self.state
            .iter()
            .map(|(_, assignment)| assignment.var.clone())
            .collect()
    }

    /// Check whether a variable with the given name is contained in the state.
    pub fn contains(&self, name: &str) -> bool {
        self.state.clone().contains_key(name)
    }

    /// Update the value of a variable and return a new State.
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

    /// Extend the state. If variables already exist, either keep
    /// the existing values or overwrite them.
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

    /// Extract the goal predicate from the String value.
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

/// Displaying the State in a user-friendly way.
impl fmt::Display for State {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: String = {
            // let sorted = self.state.sort();
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

    fn john_doe() -> Vec<(SPVariable, SPValue)> {
        let name = v!("name");
        let surname = v!("surname");
        let height = iv!("height");
        let weight = fv!("weight");
        let smart = bv!("smart");

        vec![
            (name, "John".to_spvalue()),
            (surname, "Doe".to_spvalue()),
            (height, 185.to_spvalue()),
            (weight, 80.0.to_spvalue()),
            (smart, true.to_spvalue()),
        ]
    }

    fn _john_doe_faulty() -> Vec<(SPVariable, SPValue)> {
        let name = v!("name");
        let surname = v!("surname");
        let height = iv!("height");
        let weight = fv!("weight");
        let smart = bv!("smart");

        vec![
            (name, "John".to_spvalue()),
            (surname, "Doe".to_spvalue()),
            (height, 185.to_spvalue()),
            (weight, 81.0.to_spvalue()),
            (smart, true.to_spvalue()),
        ]
    }

    #[test]
    fn test_state_new() {
        let new_state = State::new();
        assert_eq!(new_state.state.len(), 1) // Has the empty var
    }

    #[test]
    fn test_state_from_vec() {
        let john_doe = john_doe();
        let new_state = State::from_vec(&john_doe);
        assert_eq!(new_state.state.len(), 5)
    }

    #[test]
    fn test_state_display() {
        let john_doe = john_doe();
        let new_state = State::from_vec(&john_doe);
        print!("{}", new_state)
    }

    #[test]
    #[should_panic]
    fn test_state_from_vec_panic() {
        let john_doe = john_doe();
        let new_state = State::from_vec(&john_doe);
        assert_eq!(new_state.state.len(), 6)
    }

    #[test]
    fn test_state_get_value() {
        let john_doe = john_doe();
        let state = State::from_vec(&john_doe);
        assert_eq!(185.to_spvalue(), state.get_value("height"));
        assert_ne!(186.to_spvalue(), state.get_value("height"));
    }

    #[test]
    fn test_state_get_all() {
        let john_doe = john_doe();
        let state = State::from_vec(&john_doe);
        assert_eq!(
            SPAssignment {
                var: iv!("height"),
                val: 185.to_spvalue()
            },
            state.get_assignment("height")
        );
        assert_ne!(
            SPAssignment {
                var: iv!("height"),
                val: 186.to_spvalue()
            },
            state.get_assignment("height")
        );
    }

    #[test]
    fn test_state_contains() {
        let john_doe = john_doe();
        let state = State::from_vec(&john_doe);
        assert_eq!(true, state.contains("height"));
        assert_ne!(true, state.contains("wealth"));
    }

    #[test]
    fn test_state_add_not_mutable() {
        let john_doe = john_doe();
        let state = State::from_vec(&john_doe);
        let wealth = iv!("wealth");
        state.add(assign!(wealth, 2000.to_spvalue()));
        assert_ne!(state.state.len(), 6)
    }

    #[test]
    fn test_state_add() {
        let john_doe = john_doe();
        let state = State::from_vec(&john_doe);
        let wealth = iv!("wealth");
        let state = state.add(assign!(wealth, 2000.to_spvalue()));
        assert_eq!(state.state.len(), 6)
    }

    #[test]
    #[should_panic]
    fn test_state_add_already_exists() {
        let john_doe = john_doe();
        let state = State::from_vec(&john_doe);
        let wealth = iv!("height");
        let state = state.add(assign!(wealth, 2000.to_spvalue()));
        assert_eq!(state.state.len(), 6)
    }

    #[test]
    fn test_state_update() {
        let john_doe = john_doe();
        let state = State::from_vec(&john_doe);
        let state = state.update("height", 190.to_spvalue());
        assert_eq!(state.get_value("height"), 190.to_spvalue())
    }
}
