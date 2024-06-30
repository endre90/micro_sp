use crate::{SPValue, SPVariable};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::fmt;

/// SPWrapped can either be a SPVariable or a SPValue.
#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub enum SPWrapped {
    SPVariable(SPVariable),
    SPValue(SPValue),
}

/// This trait defines a set of conversions from some Rust primitive types and containers to `SPWrapped`.
pub trait ToSPWrapped {
    fn wrap(&self) -> SPWrapped;
}

impl ToSPWrapped for SPValue {
    fn wrap(&self) -> SPWrapped {
        SPWrapped::SPValue(self.clone())
    }
}

impl ToSPWrapped for bool {
    fn wrap(&self) -> SPWrapped {
        SPWrapped::SPValue(SPValue::Bool(*self))
    }
}

impl ToSPWrapped for i64 {
    fn wrap(&self) -> SPWrapped {
        SPWrapped::SPValue(SPValue::Int64(*self))
    }
}

impl ToSPWrapped for f64 {
    fn wrap(&self) -> SPWrapped {
        SPWrapped::SPValue(SPValue::Float64(OrderedFloat(*self)))
    }
}

impl ToSPWrapped for String {
    fn wrap(&self) -> SPWrapped {
        SPWrapped::SPValue(SPValue::String(self.clone()))
    }
}

impl ToSPWrapped for &str {
    fn wrap(&self) -> SPWrapped {
        SPWrapped::SPValue(SPValue::String((*self).to_string()))
    }
}

/// This trait defines a set of conversions from `SPVariable` to `SPWrapped`.
pub trait ToSPWrappedVar {
    fn wrap(&self) -> SPWrapped;
}

impl ToSPWrappedVar for SPVariable {
    fn wrap(&self) -> SPWrapped {
        SPWrapped::SPVariable(self.clone())
    }
}

/// Displaying SPWrapped in a user-friendly way.
impl fmt::Display for SPWrapped {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SPWrapped::SPValue(val) => match val {
                SPValue::Bool(b) if *b => write!(fmtr, "true"),
                SPValue::Bool(_) => write!(fmtr, "false"),
                SPValue::Float64(f) => write!(fmtr, "{}", f),
                SPValue::Int64(i) => write!(fmtr, "{}", i),
                SPValue::String(s) => write!(fmtr, "{}", s),
                SPValue::Time(t) => write!(fmtr, "{:?} s", t.elapsed().unwrap_or_default()),
                SPValue::Array(_, a) => write!(fmtr, "{:?}", a),
                SPValue::UNKNOWN => write!(fmtr, "UNKNOWN"),
            },
            SPWrapped::SPVariable(var) => write!(fmtr, "{}", var.name.to_owned()),
        }
    }
}

// #[cfg(test)]
// mod tests {

//     use crate::{
//         bv_estimated, bv_runner, fv_estimated, fv_runner, iv_estimated, iv_runner, v_estimated,
//         v_runner,
//     };
//     use crate::{
//         SPValue, SPValueType, SPVariable, SPVariableType, SPWrapped, ToSPValue, ToSPWrapped,
//         ToSPWrappedVar,
//     };

//     #[test]
//     fn test_wrap_values() {
//         let int = 123;
//         let float = 0.123;
//         let bool = false;
//         let string = "asdf";
//         assert_eq!(SPWrapped::SPValue(SPValue::Int64(123)), int.wrap());
//         assert_eq!(
//             SPWrapped::SPValue(SPValue::Float64(ordered_float::OrderedFloat(0.123))),
//             float.wrap()
//         );
//         assert_eq!(SPWrapped::SPValue(SPValue::Bool(false)), bool.wrap());
//         assert_eq!(
//             SPWrapped::SPValue(SPValue::String("asdf".to_string())),
//             string.wrap()
//         );
//     }

//     #[test]
//     fn test_wrap_spvalues() {
//         let int_val = 123.to_spvalue();
//         let float_val = 0.123.to_spvalue();
//         let bool_val = false.to_spvalue();
//         let string_val = "asdf".to_spvalue();
//         assert_eq!(SPWrapped::SPValue(SPValue::Int64(123)), int_val.wrap());
//         assert_eq!(
//             SPWrapped::SPValue(SPValue::Float64(ordered_float::OrderedFloat(0.123))),
//             float_val.wrap()
//         );
//         assert_eq!(SPWrapped::SPValue(SPValue::Bool(false)), bool_val.wrap());
//         assert_eq!(
//             SPWrapped::SPValue(SPValue::String("asdf".to_string())),
//             string_val.wrap()
//         );
//     }

//     #[test]
//     fn test_wrap_variables() {
//         let string_var = v_estimated!("position", vec!("a", "b", "c"));
//         let string_var_run = v_runner!("position");
//         let int_var = iv_estimated!("counter", vec!(1, 2, 3));
//         let int_var_run = iv_runner!("counter");
//         let bool_var = bv_estimated!("toggle");
//         let bool_var_run = bv_runner!("toggle");
//         let float_var = fv_estimated!("speed", vec!(0.1, 0.3));
//         let float_var_run = fv_runner!("speed");
//         assert_eq!(SPWrapped::SPVariable(string_var.clone()), string_var.wrap());
//         assert_eq!(
//             SPWrapped::SPVariable(string_var_run.clone()),
//             string_var_run.wrap()
//         );
//         assert_eq!(SPWrapped::SPVariable(int_var.clone()), int_var.wrap());
//         assert_eq!(
//             SPWrapped::SPVariable(int_var_run.clone()),
//             int_var_run.wrap()
//         );
//         assert_eq!(SPWrapped::SPVariable(bool_var.clone()), bool_var.wrap());
//         assert_eq!(
//             SPWrapped::SPVariable(bool_var_run.clone()),
//             bool_var_run.wrap()
//         );
//         assert_eq!(SPWrapped::SPVariable(float_var.clone()), float_var.wrap());
//         assert_eq!(
//             SPWrapped::SPVariable(float_var_run.clone()),
//             float_var_run.wrap()
//         );
//     }
// }
