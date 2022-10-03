use crate::{State, SPCommon};

#[derive(Debug, PartialEq, Clone)]
pub enum Predicate {
    TRUE,
    FALSE,
    NOT(Box<Predicate>),
    AND(Vec<Predicate>),
    OR(Vec<Predicate>),
    EQ(SPCommon, SPCommon),
}

impl Predicate {
    pub fn eval(self, state: &State) -> bool {
        match self {
            Predicate::TRUE => true,
            Predicate::FALSE => false,
            Predicate::NOT(p) => !p.eval(&state.clone()),
            Predicate::AND(p) => p.iter().all(|pp| pp.clone().eval(&state)),
            Predicate::OR(p) => p.iter().any(|pp| pp.clone().eval(&state)),
            Predicate::EQ(x, y) => match x {
                SPCommon::SPVariable(vx) => match y {
                    SPCommon::SPVariable(vy) => state.clone().get_val(&vx.name) == state.clone().get_val(&vy.name),
                    SPCommon::SPValue(vy) => state.clone().get_val(&vx.name) == vy,
                },
                SPCommon::SPValue(vx) => match y {
                    SPCommon::SPVariable(vy) => vx == state.clone().get_val(&vy.name),
                    SPCommon::SPValue(vy) => vx == vy,
                },
            },
        }
    }
}
