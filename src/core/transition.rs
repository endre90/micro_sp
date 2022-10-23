use crate::{get_predicate_vars, Action, Predicate, SPVariable, State};
use std::fmt;

#[derive(Debug, Clone, Eq)]
pub struct Transition {
    pub name: String,
    pub guard: Predicate,
    pub actions: Vec<Action>,
}

impl Transition {
    pub fn new(name: &str, guard: Predicate, actions: Vec<Action>) -> Transition {
        Transition {
            name: name.to_string(),
            guard: guard.to_owned(),
            actions: actions.to_owned(),
        }
    }

    pub fn eval(self, state: &State) -> bool {
        self.guard.eval(state)
    }

    pub fn take(self, state: &State) -> State {
        let mut new_state = state.clone();
        match self.clone().eval(state) {
            true => {
                for a in self.actions {
                    new_state = a.assign(&new_state)
                }
            }
            false => panic!("Guard is false."),
        }
        new_state
    }
}

impl PartialEq for Transition {
    fn eq(&self, other: &Transition) -> bool {
        self.guard == other.guard && self.actions == other.actions
    }
}

pub fn get_transition_vars(trans: &Transition) -> Vec<SPVariable> {
    let mut s = Vec::new();
    let guard_vars = get_predicate_vars(&trans.guard);
    let action_vars: Vec<SPVariable> = trans.actions.iter().map(|x| x.var.to_owned()).collect();
    s.extend(guard_vars);
    s.extend(action_vars);
    s.sort();
    s.dedup();
    s
}

pub fn get_model_vars(model: &Vec<Transition>) -> Vec<SPVariable> {
    let mut s = Vec::new();
    model.iter().for_each(|x| s.extend(get_transition_vars(x)));
    s.sort();
    s.dedup();
    s
}

impl fmt::Display for Transition {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut action_string = "".to_string();
        let mut actions = self.actions.clone();
        match actions.pop() {
            Some(last_action) => {
                action_string = actions
                    .iter()
                    .map(|x| format!("{}, ", x.to_string()))
                    .collect::<String>();
                let last_action_string = &format!("{}", last_action.to_string());
                action_string.extend(last_action_string.chars());
            }
            None => (),
        }
        write!(fmtr, "{}: {} / [{}]", self.name, self.guard, action_string)
    }
}