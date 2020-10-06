use super::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub value: bool,
}

#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub enum ControlKind {
    Measured,  // input
    Command,   // output
    Estimated, // internal
    None,
}


#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub struct EnumVariable {
    pub name: String,
    pub r#type: String,
    pub domain: Vec<String>,
    pub param: Parameter,
    pub kind: ControlKind,
}

#[derive(Derivative, Debug, Clone, PartialOrd, Eq, Ord, Serialize, Deserialize)]
#[derivative(PartialEq)]
pub struct EnumVariableValue {
    pub var: EnumVariable,
    pub val: String,
    #[derivative(PartialEq="ignore")]
    pub lifetime: Duration,
}

#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub struct State {
    pub vec: Vec<EnumVariableValue>,
    pub kind: ControlKind,
}

#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub struct CompleteState {
    pub measured: State,
    pub command: State,
    pub estimated: State,
}

#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord)]
pub struct Transition {
    pub name: String,
    pub guard: Predicate,
    pub update: Predicate,
    pub kind: ControlKind,
}

impl State {
    pub fn new(kind: &ControlKind) -> State {
        State {
            vec: vec![],
            kind: kind.to_owned(),
        }
    }
    pub fn from(vec: &Vec<EnumVariableValue>, kind: &ControlKind) -> State {
        State {
            vec: vec.to_owned(),
            kind: kind.to_owned(),
        }
    }
}

impl CompleteState {
    pub fn new() -> CompleteState {
        CompleteState {
            measured: State::new(&ControlKind::Measured),
            command: State::new(&ControlKind::Command),
            estimated: State::new(&ControlKind::Estimated),
        }
    }
    pub fn from(measured: &State, command: &State, estimated: &State) -> CompleteState {
        CompleteState {
            measured: match measured.kind == ControlKind::Measured {
                true => measured.to_owned(),
                false => panic!("kind must match when constructing state"),
            },
            command: match measured.kind == ControlKind::Command {
                true => command.to_owned(),
                false => panic!("kind must match when constructing state"),
            },
            estimated: match measured.kind == ControlKind::Estimated {
                true => estimated.to_owned(),
                false => panic!("kind must match when constructing state"),
            },
        }
    }
}

// revise transition (not sure about the panics and stuff)
impl Transition {
    pub fn new(name: &str, guard: &Predicate, update: &Predicate) -> Transition {
        Transition {
            name: name.to_string(),
            guard: guard.to_owned(),
            update: update.to_owned(),
            kind: {
                // get kind from the kind of the updated variable
                let diff = get_predicate_vars(&guard).intersect(get_predicate_vars(&update));
                match diff.len() {
                    0 => panic!("no update"),
                    1 => diff[0].kind.to_owned(),
                    _ => panic!("multiple actions in one step not implemented"),
                }
            },
        }
    }
}

impl Parameter {
    pub fn new(name: &str, value: &bool) -> Parameter {
        Parameter {
            name: name.to_owned(),
            value: *value,
        }
    }
}

// revise enum variable (don't like the EMPTY and TRUE stuff)
impl EnumVariable {
    pub fn new(
        name: &str,
        domain: &Vec<&str>,
        param: Option<&Parameter>,
        kind: Option<&ControlKind>,
    ) -> EnumVariable {
        EnumVariable {
            name: match name == "EMPTY" {
                true => panic!(
                    "Error 69e2abf9-498b-4d5c-88c7-30ea70ed27fb: 
                EnumVariable name 'EMPTY' is reserved."
                ), // why?
                false => name.to_owned(),
            },
            r#type: name.to_owned(),
            domain: domain
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>(),
            param: match param {
                Some(x) => x.to_owned(),
                None => Parameter::new("TRUE", &true),
            },
            kind: match kind {
                Some(x) => x.to_owned(),
                None => ControlKind::None,
            },
        }
    }
}

impl EnumVariableValue {
    // for output from planner
    pub fn new(var: &EnumVariable, val: &str) -> EnumVariableValue {
        EnumVariableValue {
            var: var.to_owned(),
            val: val.to_owned(),
            lifetime: Duration::new(6, 0),
        }
    }
    // for input
    pub fn timed(var: &EnumVariable, val: &str, lifetime: Duration) -> EnumVariableValue {
        EnumVariableValue {
            var: var.to_owned(),
            val: val.to_owned(),
            lifetime: lifetime.to_owned(),
        }
    }
}

#[test]
fn test_new_enum_variable() {
    let var1 = EnumVariable::new("var1", &vec!["a", "b", "c"], None, None);
    println!("{:?}", var1);
}
