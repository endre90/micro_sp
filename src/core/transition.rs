use crate::{Action, Predicate, State};

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
