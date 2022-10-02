use crate::SPValue;

#[derive(Debug, PartialEq, Clone)]
pub enum VarOrVal {
    String(String),
    SPValue(SPValue),
}

pub trait ToVar {
    fn to_var(&self) -> VarOrVal;
}

impl ToVar for String {
    fn to_var(&self) -> VarOrVal {
        VarOrVal::String(self.clone())
    }
}

impl ToVar for &str {
    fn to_var(&self) -> VarOrVal {
        VarOrVal::String((*self).to_string())
    }
}

pub trait ToVal {
    fn to_val(&self) -> VarOrVal;
}

impl ToVal for bool {
    fn to_val(&self) -> VarOrVal {
        VarOrVal::SPValue(SPValue::Bool(*self))
    }
}

// impl ToVal for f32 {
//     fn to_val(&self) -> VarOrVal {
//         VarOrVal::SPValue(SPValue::Float32(*self))
//     }
// }

impl ToVal for i32 {
    fn to_val(&self) -> VarOrVal {
        VarOrVal::SPValue(SPValue::Int32(*self))
    }
}

impl ToVal for String {
    fn to_val(&self) -> VarOrVal {
        VarOrVal::SPValue(SPValue::String(self.clone()))
    }
}

impl ToVal for &str {
    fn to_val(&self) -> VarOrVal {
        VarOrVal::SPValue(SPValue::String((*self).to_string()))
    }
}
