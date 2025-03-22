// use crate::{SPValue, SPVariable};
use crate::*;
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
                SPValue::Unknown(_) => write!(fmtr, "UNKNOWN"),
            },
            SPWrapped::SPVariable(var) => write!(fmtr, "{}", var.name.to_owned()),
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::*;

    #[test]
    fn test_wrap_values() {
        let int = 123;
        let float = 0.123;
        let bool = false;
        let string = "asdf";
        assert_eq!(SPWrapped::SPValue(SPValue::Int64(123)), int.wrap());
        assert_eq!(
            SPWrapped::SPValue(SPValue::Float64(ordered_float::OrderedFloat(0.123))),
            float.wrap()
        );
        assert_eq!(SPWrapped::SPValue(SPValue::Bool(false)), bool.wrap());
        assert_eq!(
            SPWrapped::SPValue(SPValue::String("asdf".to_string())),
            string.wrap()
        );
    }

    #[test]
    fn test_wrap_spvalues() {
        let int_val = 123.to_spvalue();
        let float_val = 0.123.to_spvalue();
        let bool_val = false.to_spvalue();
        let string_val = "asdf".to_spvalue();
        assert_eq!(SPWrapped::SPValue(SPValue::Int64(123)), int_val.wrap());
        assert_eq!(
            SPWrapped::SPValue(SPValue::Float64(ordered_float::OrderedFloat(0.123))),
            float_val.wrap()
        );
        assert_eq!(SPWrapped::SPValue(SPValue::Bool(false)), bool_val.wrap());
        assert_eq!(
            SPWrapped::SPValue(SPValue::String("asdf".to_string())),
            string_val.wrap()
        );
    }

    #[test]
    fn test_wrap_variables() {
        let string_var = v!("position");
        let int_var = iv!("counter");
        let bool_var = bv!("toggle");
        let float_var = fv!("speed");
        assert_eq!(SPWrapped::SPVariable(string_var.clone()), string_var.wrap());
        assert_eq!(SPWrapped::SPVariable(string_var.clone()), string_var.wrap());
        assert_eq!(SPWrapped::SPVariable(int_var.clone()), int_var.wrap());
        assert_eq!(SPWrapped::SPVariable(bool_var.clone()), bool_var.wrap());
        assert_eq!(SPWrapped::SPVariable(float_var.clone()), float_var.wrap());
    }
}
