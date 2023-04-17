use serde::{Deserialize, Serialize};

use crate::{
    get_predicate_vars_all, get_predicate_vars_planner, get_predicate_vars_runner, Action,
    Predicate, SPVariable, SPVariableType, State,
};
use std::fmt;

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
            guard,
            runner_guard,
            actions,
            runner_actions,
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
        for a in self.actions {
            new_state = a.assign(&new_state)
        }
        new_state
    }

    pub fn take_running(self, state: &State) -> State {
        let mut new_state = state.clone();
        for a in self.actions {
            match a.var.variable_type {
                SPVariableType::Measured => (),
                _ => new_state = a.assign(&new_state),
            }
        }
        for a in self.runner_actions {
            match a.var.variable_type {
                SPVariableType::Measured => (),
                _ => new_state = a.assign(&new_state),
            }
        }
        new_state
    }

    // TODO: test...
    pub fn relax(self, vars: &Vec<String>) -> Transition {
        let r_guard = self.guard.remove(vars);
        let r_runner_guard = self.runner_guard.remove(vars);
        let mut r_actions = vec![];
        let mut r_runner_actions = vec![];
        self.actions
            .iter()
            .for_each(|x| match vars.contains(&x.var.name) {
                false => r_actions.push(x.clone()),
                true => (),
            });
        self.runner_actions
            .iter()
            .for_each(|x| match vars.contains(&x.var.name) {
                false => r_runner_actions.push(x.clone()),
                true => (),
            });
        Transition {
            name: self.name,
            guard: match r_guard {
                Some(x) => x,
                None => Predicate::TRUE,
            },
            runner_guard: match r_runner_guard {
                Some(x) => x,
                None => Predicate::TRUE,
            },
            actions: r_actions,
            runner_actions: r_runner_actions,
        }
    }
    
// TODO: test...
    pub fn contains_planning(self, var: &String) -> bool {
        let guard_vars: Vec<String> = get_predicate_vars_all(&self.guard)
            .iter()
            .map(|p| p.name.to_owned())
            .collect();
        let actions_vars: Vec<String> =
            self.actions.iter().map(|a| a.var.name.to_owned()).collect();
        guard_vars.contains(var) || actions_vars.contains(var)
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
