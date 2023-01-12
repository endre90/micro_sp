use crate::{SPValue, SPVariable};
use ordered_float::OrderedFloat;
use std::fmt;

/// SPWrapped can either be a SPVariable or a SPValue.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum SPWrapped {
    SPVariable(SPVariable),
    SPValue(SPValue),
}

pub trait ToSPWrapped {
    fn to_spwrapped(&self) -> SPWrapped;
}

impl ToSPWrapped for SPValue {
    fn to_spwrapped(&self) -> SPWrapped {
        SPWrapped::SPValue(self.clone())
    }
}

impl ToSPWrapped for bool {
    fn to_spwrapped(&self) -> SPWrapped {
        SPWrapped::SPValue(SPValue::Bool(*self))
    }
}

impl ToSPWrapped for i32 {
    fn to_spwrapped(&self) -> SPWrapped {
        SPWrapped::SPValue(SPValue::Int32(*self))
    }
}

impl ToSPWrapped for f64 {
    fn to_spwrapped(&self) -> SPWrapped {
        SPWrapped::SPValue(SPValue::Float64(OrderedFloat(*self)))
    }
}

impl ToSPWrapped for String {
    fn to_spwrapped(&self) -> SPWrapped {
        SPWrapped::SPValue(SPValue::String(self.clone()))
    }
}

impl ToSPWrapped for &str {
    fn to_spwrapped(&self) -> SPWrapped {
        SPWrapped::SPValue(SPValue::String((*self).to_string()))
    }
}

// impl<T> ToSPWrapped for Vec<T>
// where
//     T: ToSPWrapped,
// {
//     fn to_spwrapped(&self) -> SPWrapped {
//         let res = self
//             .iter()
//             .map(|x| x.to_spwrapped())
//             .collect::<Vec<SPWrapped>>();
//         res.to_spwrapped()
//     }
// }

// impl ToSPWrapped for Vec<SPWrapped> {
//     fn to_spwrapped(&self) -> SPWrapped {
//         if self.is_empty() {
//             SPWrapped::SPValue(SPValue::Array(SPValue::Unknown, self.clone().))
//         } else {
//             let spvaltype = self[0].has_type();
//             assert!(self.iter().all(|e| e.has_type() == spvaltype));
//             SPValue::Array(spvaltype, self.clone())
//         }
//     }
// }

pub trait ToSPWrappedVar {
    fn to_spwrapped(&self) -> SPWrapped;
}

impl ToSPWrappedVar for SPVariable {
    fn to_spwrapped(&self) -> SPWrapped {
        SPWrapped::SPVariable(self.clone())
    }
}

impl fmt::Display for SPWrapped {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SPWrapped::SPValue(val) => match val {
                SPValue::Bool(b) if *b => write!(fmtr, "true"),
                SPValue::Bool(_) => write!(fmtr, "false"),
                SPValue::Float64(f) => write!(fmtr, "{}", f),
                SPValue::Int32(i) => write!(fmtr, "{}", i),
                SPValue::String(s) => write!(fmtr, "{}", s),
                SPValue::Time(t) => write!(fmtr, "{:?} s", t.elapsed().unwrap_or_default()),
                SPValue::Array(_, a) => write!(fmtr, "{:?}", a),
                SPValue::Unknown => write!(fmtr, "[unknown]"),
            },
            SPWrapped::SPVariable(var) => write!(fmtr, "{}", var.name.to_owned()),
        }
    }
}