use crate::{Predicate, Action, State};

#[derive(Debug, Clone)]
pub struct Transition {
    pub name: String,
    pub guard: Predicate,
    pub actions: Vec<Action>
}

impl Transition {
    pub fn new(name: &str, guard: &Predicate, actions: &Vec<Action>) -> Transition {
        Transition { 
            name: name.to_string(), 
            guard: guard.to_owned(),
            actions: actions.to_owned()
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
            },
            false => panic!("Guard is false.")
        }
        new_state
    }

    // pub fn take(self, state: &State) -> Option<State> {
    //     match self.clone().eval(state) {
    //         true => {
    //             let mut failed_to_assign_all = false;
    //             let mut new_state = state.clone();
    //             for a in self.actions {
    //                 match a.assign(&new_state) {
    //                     Some(assigned) => {
    //                         new_state = assigned;
    //                     },
    //                     None => {
    //                         failed_to_assign_all = true;
    //                         break
    //                     }
    //                 }
    //             }
    //             match failed_to_assign_all {
    //                 true => None,
    //                 false => Some(new_state)
    //             }
    //         },
    //         false => None
    //     }
    // }
}