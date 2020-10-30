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
/// Estimated (internal) kind.
#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub enum Kind {
    Measured,
    Command,
    Estimated,
}

#[derive(Derivative, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SPValue {
    Bool(bool),
    String(String),
}

// Used by Variables for defining type. Must be the same as SPValue
#[derive(Derivative, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SPValueType {
    Bool,
    String,
}

/// A trait for converting a value to SPValue
pub trait ToSPValue {
    fn to_spvalue(&self) -> SPValue;
}

impl ToSPValue for bool {
    fn to_spvalue(&self) -> SPValue {
        SPValue::Bool(*self)
    }
}

impl ToSPValue for String {
    fn to_spvalue(&self) -> SPValue {
        SPValue::String(self.clone())
    }
}

impl SPValue {
    pub fn is_type(&self, t: SPValueType) -> bool {
        match self {
            SPValue::Bool(_) => SPValueType::Bool == t,
            SPValue::String(_) => SPValueType::String == t,
        }
    }

    pub fn has_type(&self) -> SPValueType {
        match self {
            SPValue::Bool(_) => SPValueType::Bool,
            SPValue::String(_) => SPValueType::String,
        }
    }
}

#[derive(Derivative, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    pub value_type: SPValueType,
    pub domain: Vec<SPValue>,
    pub param: Parameter,
    pub r#type: String,
    pub kind: Kind,
}

impl Variable {
    pub fn new(
        name: &str,
        value_type: &SPValueType,
        domain: &Vec<SPValue>,
        param: Option<&Parameter>,
        r#type: Option<&String>,
        kind: Option<&Kind>,
    ) -> Variable {
        Variable {
            name: name.to_owned(),
            value_type: value_type.to_owned(),
            domain: domain.iter().map(|x| x.to_owned()).collect(),
            param: match param {
                Some(x) => x.to_owned(),
                None => Parameter::none(),
            },
            r#type: match r#type {
                Some(x) => x.to_owned(),
                None => String::from("NONE"),
            },
            kind: match kind {
                Some(x) => x.to_owned(),
                None => Kind::Estimated,
            },
        }
    }
    pub fn domain(&self) -> &[SPValue] {
        self.domain.as_slice()
    }
}

#[derive(Derivative, Debug, Clone, Eq, Serialize, Deserialize)]
#[derivative(PartialEq)]
pub struct Assignment {
    pub var: Variable,
    pub val: SPValue,
    #[derivative(PartialEq = "ignore")]
    pub lifetime: Duration,
}

impl Assignment {
    pub fn new(var: &Variable, val: &SPValue, lifetime: Option<&Duration>) -> Assignment {
        Assignment {
            var: var.to_owned(),
            val: match val.has_type() {
                SPValueType::Bool => match var.value_type {
                    SPValueType::Bool => val.to_owned(),
                    SPValueType::String => {
                        panic!("can't assign non-boolean value to boolean variable!")
                    }
                },
                SPValueType::String => match var.value_type {
                    SPValueType::Bool => {
                        panic!("can't assign boolean value to enum type variable!")
                    }
                    SPValueType::String => val.to_owned(),
                },
            },
            lifetime: match lifetime {
                Some(x) => x.to_owned(),
                None => Duration::new(6, 0),
            },
        }
    }
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

#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub struct BoolVariable {
    pub name: String,
    pub param: Parameter,
    pub kind: Kind,
}

impl BoolVariable {
    /// Make a new boolean variable.
    pub fn new(name: &str, param: Option<&Parameter>, kind: &Kind) -> BoolVariable {
        BoolVariable {
            name: name.to_owned(),
            param: match param {
                Some(x) => x.to_owned(),
                None => Parameter::none(),
            },
            kind: kind.to_owned(),
        }
    }
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
/// lifetime since it has last updated.
#[derive(Derivative, Debug, Clone, PartialOrd, Eq, Ord, Serialize, Deserialize)]
#[derivative(PartialEq)]
pub struct EnumValue {
    pub var: EnumVariable,
    pub val: String,
    #[derivative(PartialEq = "ignore")]
    pub lifetime: Duration,
}

/// A value assigned to the boolean variable from its domain with a tracked
/// lifetime since it has last updated.
#[derive(Derivative, Debug, Clone, PartialOrd, Eq, Ord, Serialize, Deserialize)]
#[derivative(PartialEq)]
pub struct BoolValue {
    pub var: BoolVariable,
    pub val: bool,
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

impl BoolValue {
    /// Assign a value to a boolean varible.
    pub fn new(var: &BoolVariable, val: &bool, lifetime: Option<&Duration>) -> BoolValue {
        BoolValue {
            var: var.to_owned(),
            val: *val,
            lifetime: match lifetime {
                Some(x) => x.to_owned(),
                None => Duration::new(6, 0),
            },
        }
    }
}

// #[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord, Serialize, Deserialize)]
// pub enum Variable {
//     BoolVariable(BoolVariable),
//     EnumVariable(EnumVariable)
// }

// #[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord, Serialize, Deserialize)]
// pub enum Value {
//     BoolValue(BoolValue),
//     EnumValue(EnumValue)
// }

/// A collection of variables of the same control kind.
#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub struct State {
    pub vec: Vec<EnumValue>,
    pub kind: Kind,
}

/// A collection of variables of the same control kind.
#[derive(Debug, PartialEq, Clone, Eq, Serialize, Deserialize)]
pub struct NewState {
    pub vec: Vec<Assignment>,
    pub kind: Kind,
}

impl NewState {
    pub fn new(vec: &Vec<Assignment>, kind: &Kind) -> NewState {
        match vec.len() > 0 {
            false => NewState {
                vec: vec![],
                kind: kind.to_owned(),
            },
            true => match vec.iter().all(|x| x.var.kind == *kind) {
                false => panic!("can't make a state of other than variable kind"),
                true => NewState {
                    vec: vec.to_owned(),
                    kind: kind.to_owned(),
                },
            },
        }
    }
}

// impl State {
//     pub fn new(vec: &Vec<Value>, kind: &Kind) -> State {
//         match vec.len() > 0 {
//             false => State {
//                 vec: vec![],
//                 kind: kind.to_owned(),
//             },
//             true => match vec.iter().all(|x| match x {
//                 Value::EnumValue(y) => y.var.kind == *kind,
//                 Value::BoolValue(y) => y.var.kind == *kind,
//             }) {
//                 false => panic!("can't make a state of other than variable kind"),
//                 true => State {
//                     vec: vec.to_owned(),
//                     kind: kind.to_owned(),
//                 },
//             },
//         }
//     }
// }

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

/// A struct containing the measured, command and estimated state. (Temporary?)
#[derive(Debug, PartialEq, Clone, Eq, Serialize, Deserialize)]
pub struct NewCompleteState {
    pub measured: NewState,
    pub command: NewState,
    pub estimated: NewState,
}

impl NewCompleteState {
    /// Create a new empty complete state. (Temporary?)
    pub fn empty() -> NewCompleteState {
        NewCompleteState {
            measured: NewState::new(&vec![], &Kind::Measured),
            command: NewState::new(&vec![], &Kind::Command),
            estimated: NewState::new(&vec![], &Kind::Estimated),
        }
    }
    /// Collect a complete state from measured, handshake,
    /// command and estimated state. States can also be empty.
    pub fn from_states(
        measured: &NewState,
        command: &NewState,
        estimated: &NewState,
    ) -> NewCompleteState {
        NewCompleteState {
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
    pub fn from_vec(vec: &Vec<Assignment>) -> NewCompleteState {
        NewCompleteState {
            measured: NewState::new(
                &vec.iter()
                    .filter(|x| x.var.kind == Kind::Measured)
                    .map(|x| x.to_owned())
                    .collect(),
                &Kind::Measured,
            ),
            command: NewState::new(
                &vec.iter()
                    .filter(|x| x.var.kind == Kind::Command)
                    .map(|x| x.to_owned())
                    .collect(),
                &Kind::Command,
            ),
            estimated: NewState::new(
                &vec.iter()
                    .filter(|x| x.var.kind == Kind::Estimated)
                    .map(|x| x.to_owned())
                    .collect(),
                &Kind::Estimated,
            ),
        }
    }
}
