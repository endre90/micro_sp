use crate::*;
use serde::{Deserialize, Serialize};

/// Represents assigning a value to a variable.
#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SPAssignment {
    pub var: SPVariable,
    pub val: SPValue,
}

impl SPAssignment {
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

// #[cfg(test)]
// mod tests {

//     use crate::*;

//     #[test]
//     fn test_new_assignment() {
//         // Test creating an assignment with the correct value type
//         let bool_var = SPVariable::new_boolean("bool_var");
//         let bool_val = true.to_spvalue();
//         let bool_assignment = SPAssignment::new(bool_var.clone(), bool_val.clone());
//         assert_eq!(bool_assignment.var, bool_var);
//         assert_eq!(bool_assignment.val, bool_val);

//         let int_var = SPVariable::new_integer("int_var");
//         let int_val = 1.to_spvalue();
//         let int_assignment = SPAssignment::new(int_var.clone(), int_val.clone());
//         assert_eq!(int_assignment.var, int_var);
//         assert_eq!(int_assignment.val, int_val);

//         let float_var =
//             SPVariable::new_float("float_var");
//         let float_val = 1.0.to_spvalue();
//         let float_assignment = SPAssignment::new(float_var.clone(), float_val.clone());
//         assert_eq!(float_assignment.var, float_var);
//         assert_eq!(float_assignment.val, float_val);

//         let string_var =
//             SPVariable::new_string("string_var");
//         let string_val = "foo".to_spvalue();
//         let string_assignment = SPAssignment::new(string_var.clone(), string_val.clone());
//         assert_eq!(string_assignment.var, string_var);
//         assert_eq!(string_assignment.val, string_val);

//         let array_var = SPVariable::new_array(
//             "array_var",
//         );
//         let array_val = vec![1.to_spvalue()].to_spvalue();
//         let array_assignment = SPAssignment::new(array_var.clone(), array_val.clone());
//         assert_eq!(array_assignment.var, array_var);
//         assert_eq!(array_assignment.val, array_val);
//     }

//     #[test]
//     fn test_new_unknown_assignment() {
//         // Test creating an assignment with the correct value type
//         let bool_var = SPVariable::new_boolean("bool_var");
//         let bool_val = SPValue::Bool(BoolOrUnknown::UNKNOWN);
//         let bool_assignment = SPAssignment::new(bool_var.clone(), bool_val.clone());
//         assert_eq!(bool_assignment.var, bool_var);
//         assert_eq!(bool_assignment.val, bool_val);

//         let int_var = SPVariable::new_integer("int_var");
//         let int_val = SPValue::Int64(IntOrUnknown::UNKNOWN);
//         let int_assignment = SPAssignment::new(int_var.clone(), int_val.clone());
//         assert_eq!(int_assignment.var, int_var);
//         assert_eq!(int_assignment.val, int_val);

//         let float_var =
//             SPVariable::new_float("float_var");
//         let float_val = SPValue::Float64(FloatOrUnknown::UNKNOWN);
//         let float_assignment = SPAssignment::new(float_var.clone(), float_val.clone());
//         assert_eq!(float_assignment.var, float_var);
//         assert_eq!(float_assignment.val, float_val);

//         let string_var =
//             SPVariable::new_string("string_var");
//         let string_val = SPValue::String(StringOrUnknown::UNKNOWN);
//         let string_assignment = SPAssignment::new(string_var.clone(), string_val.clone());
//         assert_eq!(string_assignment.var, string_var);
//         assert_eq!(string_assignment.val, string_val);

//         let array_var = SPVariable::new_array(
//             "array_var",
//         );
//         let array_val = SPValue::Array(ArrayOrUnknown::UNKNOWN);
//         let array_assignment = SPAssignment::new(array_var.clone(), array_val.clone());
//         assert_eq!(array_assignment.var, array_var);
//         assert_eq!(array_assignment.val, array_val);
//     }

//     #[test]
//     #[should_panic]
//     fn test_new_assignment_should_panic() {
//         let var = SPVariable::new_boolean("test_var");
//         let compatible_val = SPValue::Bool(BoolOrUnknown::Bool(true));
//         let incompatible_val = SPValue::Int64(IntOrUnknown::Int64(42));

//         // Test creating a compatible assignment
//         let assignment = SPAssignment::new(var.clone(), compatible_val.clone());
//         assert_eq!(assignment.var, var.clone());
//         assert_eq!(assignment.val, compatible_val);

//         // Test creating an incompatible assignment, which should panic
//         SPAssignment::new(var.clone(), incompatible_val);
//     }
// }
