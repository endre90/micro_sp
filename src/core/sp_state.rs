use crate::{SPAssignment, SPValue, SPVariable, SPVariableType};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::{collections::HashMap, fmt};

/// Represents the current state of the system.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct State {
    pub state: HashMap<String, SPAssignment>,
}

/// The Hash trait is implemented on State in order to enable the comparison of different State instances using a hashing function.
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
    /// Creates and returns a new State instance.
    pub fn new() -> State {
        State {
            state: HashMap::new(),
        }
    }

    /// The from_vec function creates a new State object from a vector of (SPVariable, SPValue) tuples.
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

    /// Adds an SPAssignment to the State, returning a new State instance.
    pub fn add(&self, assignment: SPAssignment) -> State {
        match self.state.clone().get(&assignment.var.name) {
            Some(_) => panic!(
                "Variable {} already in state!",
                assignment.var.name.to_string()
            ),
            None => {
                let mut state = self.state.clone();
                state.insert(assignment.var.name.to_string(), assignment.clone());
                State { state }
            }
        }
    }

    /// Returns the value of the variable with the given name from the state, and panics if the variable is not in the state.
    pub fn get_value(&self, name: &str) -> SPValue {
        match self.state.clone().get(name) {
            None => panic!("Variable {} not in state!", name),
            Some(x) => x.val.clone(),
        }
    }

    /// Returns the assignment of a variable in the state or panics if the variable is not found.
    pub fn get_all(&self, name: &str) -> SPAssignment {
        match self.state.clone().get(name) {
            None => panic!("Variable {} not in state!", name),
            Some(x) => x.clone(),
        }
    }

    /// Checks whether a variable with the given name is contained in the state.
    pub fn contains(&self, name: &str) -> bool {
        self.state.clone().contains_key(name)
    }

    /// Updates the value of a variable in the state. If the variable is of type Runner, the value can be any.
    /// If the value is not in the variable's domain, the update is ignored, unless the value is "unknown".
    /// If the variable is not in the state, a panic occurs. Returns a new state with the updated value.
    ///
    /// Maybe this is not how we should do it...
    //     pub fn update(&self, name: &str, val: SPValue) -> State {
    //         match self.state.clone().get(name) {
    //             Some(assignment) => match assignment.var.variable_type {
    //                 SPVariableType::Runner => {
    //                     let mut state = self.state.clone();
    //                     state.insert(
    //                         name.to_string(),
    //                         SPAssignment {
    //                             var: assignment.var.clone(),
    //                             val: val.clone(),
    //                         },
    //                     );
    //                     State { state }
    //                 }
    //                 _ => match assignment.var.domain.contains(&val) {
    //                     true => {
    //                         let mut state = self.state.clone();
    //                         state.insert(
    //                             name.to_string(),
    //                             SPAssignment {
    //                                 var: assignment.var.clone(),
    //                                 val: val.clone(),
    //                             },
    //                         );
    //                         State { state }
    //                     }
    //                     false => match val {
    //                         SPValue::UNDEFINED => {
    //                             let mut state = self.state.clone();
    //                             state.insert(
    //                                 name.to_string(),
    //                                 SPAssignment {
    //                                     var: assignment.var.clone(),
    //                                     val: val.clone(),
    //                                 },
    //                             );
    //                             State { state }
    //                         }
    //                         SPValue::String(x) => match x.as_str() {
    //                             "unknown" => self.clone(),
    //                             _ => {
    //                                 println!("Value {} to update the variable {} is not in its domain. State not updated!", x, assignment.var.name);
    //                                 self.clone()
    //                             }
    //                         },
    //                         _ => {
    //                             println!("Value {} to update the variable {} is not in its domain. State not updated!", val, assignment.var.name);
    //                             self.clone()
    //                         }
    //                     },
    //                 },
    //             },
    //             None => panic!("Variable {} not in state.", name),
    //         }
    //     }
    // }

    pub fn update(&self, name: &str, val: SPValue) -> State {
        match self.state.clone().get(name) {
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
                    SPValue::Array(_, some_array) => {
                        let mut sub_children: Vec<String> = vec![format!("    {}:", k)];
                        sub_children.extend(
                            some_array
                                .iter()
                                .map(|value| format!("        {}", value))
                                .collect::<Vec<String>>(),
                        );
                        format!("{}", sub_children.join("\n"))
                    }
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

    use crate::{assign, bv_estimated, fv_estimated, iv_estimated, v_estimated};
    use crate::{SPAssignment, SPValue, SPValueType, SPVariable, SPVariableType, State, ToSPValue};

    fn john_doe() -> Vec<(SPVariable, SPValue)> {
        let name = v_estimated!("name", vec!("John", "Jack"));
        let surname = v_estimated!("surname", vec!("Doe", "Crawford"));
        let height = iv_estimated!("height", vec!(180, 185, 190));
        let weight = fv_estimated!("weight", vec!(80.0, 82.5, 85.0));
        let smart = bv_estimated!("smart");

        vec![
            (name, "John".to_spvalue()),
            (surname, "Doe".to_spvalue()),
            (height, 185.to_spvalue()),
            (weight, 80.0.to_spvalue()),
            (smart, true.to_spvalue()),
        ]
    }

    fn _john_doe_faulty() -> Vec<(SPVariable, SPValue)> {
        let name = v_estimated!("name", vec!("John", "Jack"));
        let surname = v_estimated!("surname", vec!("Doe", "Crawford"));
        let height = iv_estimated!("height", vec!(180, 185, 190));
        let weight = fv_estimated!("weight", vec!(80.0, 82.5, 85.0));
        let smart = bv_estimated!("smart");

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
        assert_eq!(new_state.state.len(), 0)
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
                var: iv_estimated!("height", vec!(180, 185, 190)),
                val: 185.to_spvalue()
            },
            state.get_all("height")
        );
        assert_ne!(
            SPAssignment {
                var: iv_estimated!("height", vec!(180, 185, 190)),
                val: 186.to_spvalue()
            },
            state.get_all("height")
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
        let wealth = iv_estimated!("wealth", vec!(1000, 2000));
        state.add(assign!(wealth, 2000.to_spvalue()));
        assert_ne!(state.state.len(), 6)
    }

    #[test]
    fn test_state_add() {
        let john_doe = john_doe();
        let state = State::from_vec(&john_doe);
        let wealth = iv_estimated!("wealth", vec!(1000, 2000));
        let state = state.add(assign!(wealth, 2000.to_spvalue()));
        assert_eq!(state.state.len(), 6)
    }

    #[test]
    #[should_panic]
    fn test_state_add_already_exists() {
        let john_doe = john_doe();
        let state = State::from_vec(&john_doe);
        let wealth = iv_estimated!("height", vec!(1000, 2000));
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
