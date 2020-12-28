use super::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Variables, transitions and states can be of Measured (input), Command (output) and
/// Estimated (internal) kind.
#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub enum Kind {
    Measured,
    Command,
    Estimated,
}

#[derive(Derivative, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SPValue {
    Bool(bool),
    String(String),
}

// Used by Variables for defining type. Must be the same as SPValue
#[derive(Derivative, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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

#[derive(Derivative, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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

#[derive(Derivative, Debug, Clone, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, Clone, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct State {
    pub measured: Vec<Assignment>,
    pub command: Vec<Assignment>,
    pub estimated: Vec<Assignment>,
}

impl State {
    pub fn empty() -> State {
        State {
            measured: vec![],
            command: vec![],
            estimated: vec![],
        }
    }
    pub fn from_vec(vec: &Vec<Assignment>) -> State {
        State {
            measured: vec
                .iter()
                .filter(|x| x.var.kind == Kind::Measured)
                .map(|x| x.to_owned())
                .collect(),
            command: vec
                .iter()
                .filter(|x| x.var.kind == Kind::Command)
                .map(|x| x.to_owned())
                .collect(),
            estimated: vec
                .iter()
                .filter(|x| x.var.kind == Kind::Estimated)
                .map(|x| x.to_owned())
                .collect(),
        }
    }
}

/// A transition that updates the state according to the guard and update predicates.
/// When incremental planning, the guard and update predicates
/// are concjunctions of predicated from the guard and update vector. During
/// compositional planning, the guard and update predicates are a conjunction
/// of activated predicates from the vectors.
#[derive(Debug, PartialEq, Ord, PartialOrd, Clone, Eq)]
pub struct Transition {
    pub name: String,
    pub guard: Predicate,
    pub update: Predicate,
}

impl Transition {
    /// Make a new named transition from guard and update predicates.
    pub fn new(name: &str, guard: &Predicate, update: &Predicate) -> Transition {
        Transition {
            name: name.to_string(),
            guard: guard.to_owned(),
            update: update.to_owned(),
        }
    }
}

/// A frame holds states about what happens in a step.
#[derive(PartialEq, Eq, Clone, Debug, PartialOrd, Ord)]
pub struct PlanningFrame {
    pub source: State,
    pub sink: State,
    pub trans: Vec<String>,
}

/// Define strategy to use when planning.
#[derive(PartialEq, Eq, Clone, Debug, PartialOrd, Ord)]
pub struct PlanningStrategy {
    pub model: String, // part of Model?
    pub logic: String, // part of Model?
    pub base: String,
    pub encoding: String,
    pub rate: String,
    pub subgoaling: String,
    pub decompose: String,
    pub parallelism: String,
}
/// A planning problem that is given to the incremental solver.
#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord)]
pub struct PlanningProblem {
    pub name: String,
    pub init: Predicate,
    pub goal: Predicate,
    pub trans: Vec<Transition>,
    pub invars: Predicate
}

/// A result is generated when the planner finds a satisfiable model.
#[derive(PartialEq, Eq, Clone, Debug, PartialOrd, Ord)]
pub struct PlanningResult {
    pub name: String,
    pub alg: String,
    pub plan_found: bool,
    pub plan_length: u64,
    pub trace: Vec<PlanningFrame>,
    pub time_to_solve: std::time::Duration,
    pub model_size: u64
}

impl PlanningProblem {
    /// Make a new planning problem from defined componenets.
    pub fn new(
        name: &str,
        init: &Predicate,
        goal: &Predicate,
        trans: &Vec<Transition>,
        invars: &Predicate
    ) -> PlanningProblem {
        PlanningProblem {
            name: name.to_string(),
            init: init.to_owned(),
            goal: goal.to_owned(),
            trans: trans.to_owned(),
            invars: invars.to_owned()
        }
    }
}