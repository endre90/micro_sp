use crate::{SPValue, SPVariable, State};

#[derive(Debug, PartialEq, Clone, Hash)]
pub enum SPCommon {
    SPVariable(SPVariable),
    SPValue(SPValue),
}

pub trait ToSPCommon {
    fn to_comval(&self) -> SPCommon;
}

impl ToSPCommon for bool {
    fn to_comval(&self) -> SPCommon {
        SPCommon::SPValue(SPValue::Bool(*self))
    }
}

impl ToSPCommon for i32 {
    fn to_comval(&self) -> SPCommon {
        SPCommon::SPValue(SPValue::Int32(*self))
    }
}

impl ToSPCommon for String {
    fn to_comval(&self) -> SPCommon {
        SPCommon::SPValue(SPValue::String(self.clone()))
    }
}

impl ToSPCommon for &str {
    fn to_comval(&self) -> SPCommon {
        SPCommon::SPValue(SPValue::String((*self).to_string()))
    }
}

pub trait ToSPCommonVar {
    fn to_comvar(&self, state: &State) -> SPCommon;
}

impl ToSPCommonVar for String {
    fn to_comvar(&self, state: &State) -> SPCommon {
        SPVariable::to_common_from_name(self, state)
    }
}

impl ToSPCommonVar for &str {
    fn to_comvar(&self, state: &State) -> SPCommon {
        SPVariable::to_common_from_name(self, state)
    }
}

// pub trait ToSPCommonVar {
//     fn to_comvar(&self) -> SPCommon;
// }

// impl ToSPCommonVar for SPVariable {
//     fn to_comvar(&self) -> SPCommon {
//         SPCommon::SPVariable(self.clone())
//     }
// }