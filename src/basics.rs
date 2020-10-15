use super::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Variables with the same parameter belong to the same group during compositional planning.
/// As such, they will be included in the model together after the next refinement.
#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub value: bool,
}

impl Parameter {
    /// Make a new paremeter that will enable or disable variables during compositional planning.
    pub fn new(name: &str, value: &bool) -> Parameter {
        Parameter {
            name: name.to_owned(),
            value: *value,
        }
    }
    /// Make a dummy parameter that will include variables in every step during compositional
    /// planning, or for incremental planning where no parameter is needed.
    pub fn none() -> Parameter {
        Parameter {
            name: "NONE".to_owned(),
            value: true,
        }
    }
}

/// Control and modelling paradigm. More about this later, might end up with only one.
#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub enum Paradigm {
    Raar,
    Invar
}

/// Variables, transitions and states can be of Measured (input), Command (output) and
/// Estimated (internal) kind.
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
            domain: 
            // {
            //     let mut domain2 = domain.clone();
            //     domain2.push("dummy");
            //     let domain3 = domain2.iter().map(|x| { println!("{}", x); x.to_string()}).collect();
            //     domain3
            // },
            domain
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
            command: State::new(&vec![], &Kind::Command),
            estimated: State::new(&vec![], &Kind::Estimated),
        }
    }
    /// Collect a complete state from measured, command and estimated state. States can also be empty.
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

/// A transition that updates the state according to the guard and update predicates.
/// The transition has a kind since it is assumed that transitions are changing one
/// variable at a time.
#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord)]
pub struct Transition {
    pub name: String,
    pub guard: Predicate,
    pub update: Predicate,
    pub kind: Kind,
}

impl Transition {
    /// Make a new named transition from guard and update predicates.
    pub fn new(name: &str, guard: &Predicate, update: &Predicate) -> Transition {
        let updates = get_predicate_vars(&update);
        Transition {
            name: name.to_string(),
            guard: guard.to_owned(),
            update: update.to_owned(),
            kind: match updates.len() > 0 {
                true => updates[0].kind.to_owned(),
                false => panic!("no update?"),
            },
        }
    }
}
