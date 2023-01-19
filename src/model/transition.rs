use serde::{Deserialize, Serialize};

use crate::{
    get_predicate_vars_all, get_predicate_vars_planner, get_predicate_vars_runner, Action,
    Predicate, SPVariable, SPVariableType, State,
};
use std::fmt;

// Do I need transition types?
// Do I neew variable types like measured, controlled and effect?
// Do I want to implement a synthesis algorithm using some specifications, SCT?
// Do I want to plug back in Z# as the planner and specification handling tool?

#[derive(Debug, Clone, Eq, Hash, Serialize, Deserialize)]
pub struct Transition {
    pub name: String,
    pub guard: Predicate,
    pub runner_guard: Predicate,
    pub actions: Vec<Action>,
    pub runner_actions: Vec<Action>,
}

impl Transition {
    pub fn new(
        name: &str,
        guard: Predicate,
        runner_guard: Predicate,
        actions: Vec<Action>,
        runner_actions: Vec<Action>,
    ) -> Transition {
        Transition {
            name: name.to_string(),
            guard: {
                let variables = get_predicate_vars_runner(&guard);
                for var in variables {
                    panic!(
                        "Runner type variable '{}' can't be in the non-runner guard.",
                        var.name
                    )
                }
                guard
            },
            runner_guard: {
                let variables = get_predicate_vars_planner(&runner_guard);
                for var in variables {
                    panic!(
                        "Planner type variable '{}' can't be in the runner guard.",
                        var.name
                    )
                }
                runner_guard
            },
            actions: {
                for action in &actions {
                    match action.var.variable_type {
                        SPVariableType::Planner => (),
                        SPVariableType::Runner => panic!(
                            "Runner type variable '{}' can't be in the non-runner action.",
                            action.var.name
                        ),
                    }
                }
                actions
            },
            runner_actions: {
                for action in &runner_actions {
                    match action.var.variable_type {
                        SPVariableType::Planner => panic!(
                            "Planner type variable '{}' can't be in the runner action.",
                            action.var.name
                        ),
                        SPVariableType::Runner => (),
                    }
                }
                runner_actions
            },
        }
    }

    pub fn eval_planning(self, state: &State) -> bool {
        self.guard.eval(state)
    }

    pub fn eval_running(self, state: &State) -> bool {
        self.guard.eval(state) && self.runner_guard.eval(state)
    }

    pub fn take_planning(self, state: &State) -> State {
        let mut new_state = state.clone();
        match self.clone().eval_planning(state) {
            true => {
                for a in self.actions {
                    new_state = a.assign(&new_state)
                }
            }
            false => (),
        }
        new_state
    }

    pub fn take_running(self, state: &State) -> State {
        let mut new_state = state.clone();
        match self.clone().eval_planning(state) && self.clone().eval_running(state) {
            true => {
                for a in self.actions {
                    new_state = a.assign(&new_state)
                }
            }
            false => (),
        }
        new_state
    }
}

impl PartialEq for Transition {
    fn eq(&self, other: &Transition) -> bool {
        self.guard == other.guard
            && self.runner_guard == other.runner_guard
            && self.actions == other.actions
            && self.runner_actions == other.runner_actions
    }
}

// TODO: test
pub fn get_transition_vars_all(trans: &Transition) -> Vec<SPVariable> {
    let mut s = Vec::new();
    let guard_vars = get_predicate_vars_all(&trans.guard);
    let runner_guard_vars = get_predicate_vars_all(&trans.runner_guard);
    let action_vars: Vec<SPVariable> = trans.actions.iter().map(|x| x.var.to_owned()).collect();
    let runner_action_vars: Vec<SPVariable> = trans
        .runner_actions
        .iter()
        .map(|x| x.var.to_owned())
        .collect();
    s.extend(guard_vars);
    s.extend(runner_guard_vars);
    s.extend(action_vars);
    s.extend(runner_action_vars);
    s.sort();
    s.dedup();
    s
}

// TODO: test
pub fn get_transition_vars_planner(trans: &Transition) -> Vec<SPVariable> {
    let mut s = Vec::new();
    let guard_vars = get_predicate_vars_planner(&trans.guard);
    let runner_guard_vars = get_predicate_vars_planner(&trans.runner_guard);
    let action_vars: Vec<SPVariable> = trans.actions.iter().map(|x| x.var.to_owned()).collect();
    let runner_action_vars: Vec<SPVariable> = trans
        .runner_actions
        .iter()
        .map(|x| x.var.to_owned())
        .collect();
    s.extend(guard_vars);
    s.extend(runner_guard_vars);
    s.extend(action_vars);
    s.extend(runner_action_vars);
    s.sort();
    s.dedup();
    s
}

// TODO: test
pub fn get_transition_vars_runner(trans: &Transition) -> Vec<SPVariable> {
    let mut s = Vec::new();
    let guard_vars = get_predicate_vars_runner(&trans.guard);
    let runner_guard_vars = get_predicate_vars_runner(&trans.runner_guard);
    let action_vars: Vec<SPVariable> = trans.actions.iter().map(|x| x.var.to_owned()).collect();
    let runner_action_vars: Vec<SPVariable> = trans
        .runner_actions
        .iter()
        .map(|x| x.var.to_owned())
        .collect();
    s.extend(guard_vars);
    s.extend(runner_guard_vars);
    s.extend(action_vars);
    s.extend(runner_action_vars);
    s.sort();
    s.dedup();
    s
}

// TODO: test
pub fn get_transition_model_vars_all(model: &Vec<Transition>) -> Vec<SPVariable> {
    let mut s = Vec::new();
    model
        .iter()
        .for_each(|x| s.extend(get_transition_vars_all(x)));
    s.sort();
    s.dedup();
    s
}

// TODO: test
pub fn get_transition_model_vars_planner(model: &Vec<Transition>) -> Vec<SPVariable> {
    let mut s = Vec::new();
    model
        .iter()
        .for_each(|x| s.extend(get_transition_vars_planner(x)));
    s.sort();
    s.dedup();
    s
}

// TODO: test
pub fn get_transition_model_vars_runner(model: &Vec<Transition>) -> Vec<SPVariable> {
    let mut s = Vec::new();
    model
        .iter()
        .for_each(|x| s.extend(get_transition_vars_planner(x)));
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
