use crate::{get_predicate_vars, Action, Predicate, SPVariable, State};

#[derive(Debug, Clone, PartialEq)]
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
