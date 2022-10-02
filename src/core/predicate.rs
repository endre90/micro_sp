use crate::{State, VarOrVal};

#[derive(Debug, PartialEq, Clone)]
pub enum Predicate {
    NOT(Box<Predicate>),
    AND(Vec<Predicate>),
    OR(Vec<Predicate>),
    EQ(VarOrVal, VarOrVal),
}

impl Predicate {
    pub fn eval(self, state: &State) -> bool {
        match self {
            Predicate::NOT(p) => !p.eval(&state),
            Predicate::AND(p) => p.iter().all(|pp| pp.clone().eval(&state)),
            Predicate::OR(p) => p.iter().any(|pp| pp.clone().eval(&state)),
            Predicate::EQ(x, y) => match x {
                VarOrVal::String(vx) => match y {
                    VarOrVal::String(vy) => state.clone().get(&vx) == state.clone().get(&vy),
                    VarOrVal::SPValue(vy) => state.clone().get(&vx) == vy,
                },
                VarOrVal::SPValue(vx) => match y {
                    VarOrVal::String(vy) => vx == state.clone().get(&vy),
                    VarOrVal::SPValue(vy) => vx == vy,
                },
            },
        }
    }
}
