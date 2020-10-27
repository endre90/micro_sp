use super::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Control and modelling paradigm. More about this later, might end up with only one.
#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub enum Paradigm {
    Raar,
    Invar,
}

/// Variables, transitions and states can be of Measured (input), Command (output) and
/// Estimated (internal) kind. Handshake is the "MeasuredCommand" kind.
#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub enum Kind {
    Measured,
    Command,
    Estimated,
}

/// An enumeration kind variable with a name (ex. banana), type (ex. fruit),
/// domain (ex. [green, ripe, spoiled]), activation parameter for refinement during
/// compositional planning (ex. food == false) and control kind (ex. estimated).
#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub struct EnumVariable {
    pub name: String,
    pub r#type: String,
    pub domain: Vec<String>,
    pub param: Parameter,
    pub kind: Kind,
}

impl EnumVariable {
    /// Make a new enumeration kind variable.
    pub fn new(
        name: &str,
        domain: &Vec<&str>,
        r#type: &str,
        param: Option<&Parameter>,
        kind: &Kind,
    ) -> EnumVariable {
        EnumVariable {
            name: name.to_owned(),
            r#type: r#type.to_owned(),
            domain: domain
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>(),
            param: match param {
                Some(x) => x.to_owned(),
                None => Parameter::none(),
            },
            kind: kind.to_owned(),
        }
    }
}

/// A value assigned to the enumeration kind variable from its domain with a tracked
/// lifetime since its last updated.
#[derive(Derivative, Debug, Clone, PartialOrd, Eq, Ord, Serialize, Deserialize)]
#[derivative(PartialEq)]
pub struct EnumValue {
    pub var: EnumVariable,
    pub val: String,
    #[derivative(PartialEq = "ignore")]
    pub lifetime: Duration,
}

impl EnumValue {
    /// Assign a value to a varible from its domain.
    pub fn new(var: &EnumVariable, val: &str, lifetime: Option<&Duration>) -> EnumValue {
        EnumValue {
            var: var.to_owned(),
            val: match var.domain.contains(&val.to_owned()) {
                true => val.to_owned(),
                false => panic!("value {:?} not in the domain of the variable", val),
            },
            lifetime: match lifetime {
                Some(x) => x.to_owned(),
                None => Duration::new(6, 0),
            },
        }
    }
}

/// A collection of variables of the same control kind.
#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub struct State {
    pub vec: Vec<EnumValue>,
    pub kind: Kind,
}

impl State {
    pub fn new(vec: &Vec<EnumValue>, kind: &Kind) -> State {
        match vec.len() > 0 {
            false => State {
                vec: vec![],
                kind: kind.to_owned(),
            },
            true => match vec.iter().all(|x| x.var.kind == *kind) {
                false => panic!("can't make a state of other than variable kind"),
                true => State {
                    vec: vec.to_owned(),
                    kind: kind.to_owned(),
                },
            },
        }
    }
}

/// A struct containing the measured, command and estimated state. (Temporary?)
#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub struct CompleteState {
    pub measured: State,
    pub command: State,
    pub estimated: State,
}

impl CompleteState {
    /// Create a new empty complete state. (Temporary?)
    pub fn empty() -> CompleteState {
        CompleteState {
            measured: State::new(&vec![], &Kind::Measured),
            // handshake: State::new(&vec![], &Kind::Handshake),
            command: State::new(&vec![], &Kind::Command),
            estimated: State::new(&vec![], &Kind::Estimated),
        }
    }
    /// Collect a complete state from measured, handshake,
    /// command and estimated state. States can also be empty.
    pub fn from_states(measured: &State, command: &State, estimated: &State) -> CompleteState {
        CompleteState {
            measured: match measured.kind == Kind::Measured {
                true => measured.to_owned(),
                false => panic!("kind must match when constructing state"),
            },
            command: match command.kind == Kind::Command {
                true => command.to_owned(),
                false => panic!("kind must match when constructing state"),
            },
            estimated: match estimated.kind == Kind::Estimated {
                true => estimated.to_owned(),
                false => panic!("kind must match when constructing state"),
            },
        }
    }
    /// Collect a complete state from a vector of values.
    pub fn from_vec(vec: &Vec<EnumValue>) -> CompleteState {
        CompleteState {
            measured: State::new(
                &vec.iter()
                    .filter(|x| x.var.kind == Kind::Measured)
                    .map(|x| x.to_owned())
                    .collect(),
                &Kind::Measured,
            ),
            command: State::new(
                &vec.iter()
                    .filter(|x| x.var.kind == Kind::Command)
                    .map(|x| x.to_owned())
                    .collect(),
                &Kind::Command,
            ),
            estimated: State::new(
                &vec.iter()
                    .filter(|x| x.var.kind == Kind::Estimated)
                    .map(|x| x.to_owned())
                    .collect(),
                &Kind::Estimated,
            ),
        }
    }
}
