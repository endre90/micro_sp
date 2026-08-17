//! Pairing a variable with a value.
//!
//! An [`SPAssignment`] binds an [`SPVariable`] to an [`SPValue`] of the
//! variable's declared type. A [`State`] is a map of these, keyed by variable
//! name.

use crate::*;
use serde::{Deserialize, Serialize};

/// A variable bound to a value of its declared type.
///
/// ```
/// use micro_sp::*;
///
/// let mut state = State::new();
/// state.add_mut(
///     SPAssignment::new(SPVariable::new("pos", SPValueType::String), "a".to_spvalue()),
///     "docs",
/// );
/// assert_eq!(state.get_value("pos", "docs"), Some("a".to_spvalue()));
/// ```
#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SPAssignment {
    /// The variable being assigned to.
    pub var: SPVariable,
    /// The value assigned, of `var`'s type.
    pub val: SPValue,
}

impl SPAssignment {
    /// Creates an assignment, checking that the value matches the variable's
    /// type.
    ///
    /// # Panics
    ///
    /// Panics if `val`'s [`SPValueType`] differs from `var`'s. An `UNKNOWN`
    /// value of the right type is accepted.
    pub fn new(var: SPVariable, val: SPValue) -> SPAssignment {
        match var.has_type() == val.has_type() {
            true => SPAssignment { var, val },
            false => panic!(
                "Wrong value type '{}' can't be assigned to a variable with type '{}'.",
                var.has_type(),
                val.has_type()
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use std::time::SystemTime;

    fn create_dummy_transform() -> SPTransformStamped {
        SPTransformStamped {
            active_transform: true,
            enable_transform: true,
            time_stamp: SystemTime::now(),
            parent_frame_id: "world".to_string(),
            child_frame_id: "robot".to_string(),
            transform: SPTransform::default(),
            metadata: MapOrUnknown::Map(vec![(
                "quality".to_spvalue(),
                "good".to_spvalue(),
            )]),
        }
    }

    #[test]
    fn test_tospwrapped_implementations() {
        let sp_value = 123.to_spvalue();
        assert_eq!(sp_value.wrap(), SPWrapped::SPValue(sp_value.clone()));

        assert_eq!(true.wrap(), SPWrapped::SPValue(true.to_spvalue()));

        assert_eq!(42.wrap(), SPWrapped::SPValue(42.to_spvalue()));

        assert_eq!(3.14.wrap(), SPWrapped::SPValue(3.14.to_spvalue()));

        let s = "hello".to_string();
        assert_eq!(s.wrap(), SPWrapped::SPValue(s.to_spvalue()));

        assert_eq!("world".wrap(), SPWrapped::SPValue("world".to_spvalue()));

        let now = SystemTime::now();
        assert_eq!(now.wrap(), SPWrapped::SPValue(now.to_spvalue()));

        let transform = create_dummy_transform();
        assert_eq!(transform.wrap(), SPWrapped::SPValue(transform.to_spvalue()));

        let vec_sp = vec![1.to_spvalue(), true.to_spvalue()];
        assert_eq!(vec_sp.wrap(), SPWrapped::SPValue(vec_sp.to_spvalue()));
        let empty_vec_sp: Vec<SPValue> = vec![];
        assert_eq!(empty_vec_sp.wrap(), SPWrapped::SPValue(empty_vec_sp.to_spvalue()));

        let vec_tuples = vec![("k".to_spvalue(), "v".to_spvalue())];
        assert_eq!(
            vec_tuples.wrap(),
            SPWrapped::SPValue(SPValue::Map(MapOrUnknown::Map(vec_tuples)))
        );
    }

    #[test]
    fn test_tospwrappedvar_implementation() {
        let var = SPVariable::new("my_var", SPValueType::Bool);
        assert_eq!(var.wrap(), SPWrapped::SPVariable(var.clone()));
    }

    #[test]
    fn test_display_for_spwrapped_full_coverage() {
        let var = SPVariable::new("var_name", SPValueType::String);
        assert_eq!(format!("{}", var.wrap()), "var_name");

        assert_eq!(format!("{}", true.wrap()), "true");
        assert_eq!(format!("{}", false.wrap()), "false");
        let unknown_bool = SPWrapped::SPValue(SPValue::Bool(BoolOrUnknown::UNKNOWN));
        assert_eq!(format!("{}", unknown_bool), "UNKNOWN");

        assert_eq!(format!("{}", 3.14.wrap()), "3.14");
        let unknown_float = SPWrapped::SPValue(SPValue::Float64(FloatOrUnknown::UNKNOWN));
        assert_eq!(format!("{}", unknown_float), "UNKNOWN");

        assert_eq!(format!("{}", 42.wrap()), "42");
        let unknown_int = SPWrapped::SPValue(SPValue::Int64(IntOrUnknown::UNKNOWN));
        assert_eq!(format!("{}", unknown_int), "UNKNOWN");

        assert_eq!(format!("{}", "hello".wrap()), "hello");
        let unknown_string = SPWrapped::SPValue(SPValue::String(StringOrUnknown::UNKNOWN));
        assert_eq!(format!("{}", unknown_string), "UNKNOWN");

        let time_val = SystemTime::now();
        assert!(!format!("{}", time_val.wrap()).is_empty());
        let unknown_time = SPWrapped::SPValue(SPValue::Time(TimeOrUnknown::UNKNOWN));
        assert_eq!(format!("{}", unknown_time), "UNKNOWN");

        let array_val = vec![1.to_spvalue(), "a".to_spvalue()];
        assert_eq!(format!("{}", array_val.wrap()), "1, a");
        let unknown_array = SPWrapped::SPValue(SPValue::Array(ArrayOrUnknown::UNKNOWN));
        assert_eq!(format!("{}", unknown_array), "UNKNOWN");

        let map_val = vec![("k".to_spvalue(), 1.to_spvalue())];
        assert_eq!(format!("{}", map_val.wrap()), "[(true, false)]");
        let unknown_map = SPWrapped::SPValue(SPValue::Map(MapOrUnknown::UNKNOWN));
        assert_eq!(format!("{}", unknown_map), "UNKNOWN");

        let transform = create_dummy_transform();
        assert!(format!("{}", transform.wrap()).starts_with("TF(active=true"));
        assert!(format!("{}", transform.wrap()).contains("meta={quality: good}"));
        
        let mut tf_unknown_meta = create_dummy_transform();
        tf_unknown_meta.metadata = MapOrUnknown::UNKNOWN;
        assert!(format!("{}", tf_unknown_meta.wrap()).contains("meta=UNKNOWN"));

        let unknown_transform = SPWrapped::SPValue(SPValue::Transform(TransformOrUnknown::UNKNOWN));
        assert_eq!(format!("{}", unknown_transform), "UNKNOWN");
    }

    #[test]
    fn test_new_assignment_success() {
        let bool_var = SPVariable::new("v", SPValueType::Bool);
        SPAssignment::new(bool_var.clone(), true.to_spvalue());
        SPAssignment::new(bool_var.clone(), SPValue::Bool(BoolOrUnknown::UNKNOWN));

        let int_var = SPVariable::new("v", SPValueType::Int64);
        SPAssignment::new(int_var.clone(), 42.to_spvalue());
        SPAssignment::new(int_var.clone(), SPValue::Int64(IntOrUnknown::UNKNOWN));

        let float_var = SPVariable::new("v", SPValueType::Float64);
        SPAssignment::new(float_var.clone(), 3.14.to_spvalue());
        SPAssignment::new(float_var.clone(), SPValue::Float64(FloatOrUnknown::UNKNOWN));

        let string_var = SPVariable::new("v", SPValueType::String);
        SPAssignment::new(string_var.clone(), "hello".to_spvalue());
        SPAssignment::new(string_var.clone(), SPValue::String(StringOrUnknown::UNKNOWN));

        let array_var = SPVariable::new("v", SPValueType::Array);
        SPAssignment::new(array_var.clone(), vec![1.to_spvalue()].to_spvalue());
        SPAssignment::new(array_var.clone(), SPValue::Array(ArrayOrUnknown::UNKNOWN));

        let map_var = SPVariable::new("v", SPValueType::Map);
        SPAssignment::new(map_var.clone(), vec![("k".to_spvalue(), "v".to_spvalue())].to_spvalue());
        SPAssignment::new(map_var.clone(), SPValue::Map(MapOrUnknown::UNKNOWN));
        
        let time_var = SPVariable::new("v", SPValueType::Time);
        SPAssignment::new(time_var.clone(), SystemTime::now().to_spvalue());
        SPAssignment::new(time_var.clone(), SPValue::Time(TimeOrUnknown::UNKNOWN));

        let transform_var = SPVariable::new("v", SPValueType::Transform);
        SPAssignment::new(transform_var.clone(), create_dummy_transform().to_spvalue());
        SPAssignment::new(transform_var.clone(), SPValue::Transform(TransformOrUnknown::UNKNOWN));
    }

    #[test]
    #[should_panic]
    fn test_new_assignment_panic_on_mismatch() {
        let var = SPVariable::new("test_var", SPValueType::Bool);
        let incompatible_val = SPValue::Int64(IntOrUnknown::Int64(42));
        SPAssignment::new(var, incompatible_val);
    }
}