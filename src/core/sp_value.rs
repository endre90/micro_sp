use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::SystemTime;

/// Represents a variable value of a specific type.
#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SPValue {
    Bool(bool),
    Float64(OrderedFloat<f64>),
    Int64(i64),
    String(String),
    Time(SystemTime),
    Array(SPValueType, Vec<SPValue>),
    UNKNOWN,
}

impl Default for SPValue {
    fn default() -> Self {
        SPValue::UNKNOWN
    }
}

/// Displaying the value of an SPValue instance in a user-friendly way.
impl fmt::Display for SPValue {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SPValue::Bool(b) if *b => write!(fmtr, "true"),
            SPValue::Bool(_) => write!(fmtr, "false"),
            SPValue::Float64(f) => write!(fmtr, "{}", f.0),
            SPValue::Int64(i) => write!(fmtr, "{}", i),
            SPValue::String(s) => write!(fmtr, "{}", s),
            SPValue::Time(t) => write!(fmtr, "{:?}", t.elapsed().unwrap_or_default()),
            SPValue::Array(_, a) => write!(fmtr, "{:?}", a),
            SPValue::UNKNOWN => write!(fmtr, "UNKNOWN"),
        }
    }
}

/// Used by SPVariables for defining their type. Must be the same as SPValue.
#[derive(Debug, PartialEq, Copy, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SPValueType {
    Bool,
    Float64,
    Int64,
    String,
    Time,
    Array,
    UNKNOWN,
}

impl Default for SPValueType {
    fn default() -> Self {
        SPValueType::UNKNOWN
    }
}

impl SPValueType {
    pub fn from_str(x: &str) -> SPValueType {
        match x {
            "bool" => SPValueType::Bool,
            "f64" => SPValueType::Float64,
            "i64" => SPValueType::Int64,
            "string" => SPValueType::String,
            "time" => SPValueType::Time,
            "array" => SPValueType::Array,
            _ => SPValueType::UNKNOWN,
        }
    }
}

impl fmt::Display for SPValueType {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SPValueType::Bool => write!(fmtr, "bool"),
            SPValueType::Float64 => write!(fmtr, "f64"),
            SPValueType::Int64 => write!(fmtr, "i64"),
            SPValueType::String => write!(fmtr, "string"),
            SPValueType::Time => write!(fmtr, "time"),
            SPValueType::Array => write!(fmtr, "array"),
            SPValueType::UNKNOWN => write!(fmtr, "UNKNOWN"),
        }
    }
}

impl SPValue {
    /// Checks whether the value is of the specified type.
    pub fn is_type(&self, t: SPValueType) -> bool {
        match self {
            SPValue::Bool(_) => SPValueType::Bool == t,
            SPValue::Float64(_) => SPValueType::Float64 == t,
            SPValue::Int64(_) => SPValueType::Int64 == t,
            SPValue::String(_) => SPValueType::String == t,
            SPValue::Time(_) => SPValueType::Time == t,
            SPValue::Array(_, _) => SPValueType::Array == t,
            SPValue::UNKNOWN => SPValueType::UNKNOWN == t,
        }
    }

    /// Returns the type of the `SPValue`.
    pub fn has_type(&self) -> SPValueType {
        match self {
            SPValue::Bool(_) => SPValueType::Bool,
            SPValue::Float64(_) => SPValueType::Float64,
            SPValue::Int64(_) => SPValueType::Int64,
            SPValue::String(_) => SPValueType::String,
            SPValue::Time(_) => SPValueType::Time,
            SPValue::Array(_, _) => SPValueType::Array,
            SPValue::UNKNOWN => SPValueType::UNKNOWN,
        }
    }

    /// Checks whether the value is of the array type.
    pub fn is_array(&self) -> bool {
        match self {
            SPValue::Array(_, _) => true,
            _ => false,
        }
    }

    /// Returns a `String` representation of the `SPValue`.
    pub fn to_string(&self) -> String {
        match self {
            SPValue::Bool(x) => x.to_string(),
            SPValue::Float64(x) => x.to_string(),
            SPValue::Int64(x) => x.to_string(),
            SPValue::String(x) => x.to_string(),
            SPValue::Time(x) => format!("{:?}", x.elapsed().unwrap_or_default()),
            SPValue::Array(_, arr) => format!(
                "[{}]",
                arr.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            SPValue::UNKNOWN => "UNKNOWN".to_string(),
        }
    }
}

/// This trait defines a set of conversions from some Rust primitive types and containers to `SPValue`.
pub trait ToSPValue {
    fn to_spvalue(&self) -> SPValue;
}

impl ToSPValue for bool {
    fn to_spvalue(&self) -> SPValue {
        SPValue::Bool(*self)
    }
}

impl ToSPValue for i64 {
    fn to_spvalue(&self) -> SPValue {
        SPValue::Int64(*self)
    }
}

impl ToSPValue for f64 {
    fn to_spvalue(&self) -> SPValue {
        SPValue::Float64(OrderedFloat(*self))
    }
}

impl ToSPValue for String {
    fn to_spvalue(&self) -> SPValue {
        SPValue::String(self.clone())
    }
}

impl ToSPValue for &str {
    fn to_spvalue(&self) -> SPValue {
        SPValue::String((*self).to_string())
    }
}

impl ToSPValue for std::time::SystemTime {
    fn to_spvalue(&self) -> SPValue {
        SPValue::Time(*self)
    }
}

impl<T> ToSPValue for Vec<T>
where
    T: ToSPValue,
{
    fn to_spvalue(&self) -> SPValue {
        let res = self
            .iter()
            .map(|x| x.to_spvalue())
            .collect::<Vec<SPValue>>();
        res.to_spvalue()
    }
}

impl ToSPValue for Vec<SPValue> {
    fn to_spvalue(&self) -> SPValue {
        if self.is_empty() {
            SPValue::Array(SPValueType::UNKNOWN, self.clone())
        } else {
            let spvaltype = self[0].has_type();
            assert!(self.iter().all(|e| e.has_type() == spvaltype));
            SPValue::Array(spvaltype, self.clone())
        }
    }
}


#[cfg(test)]
mod tests {

    use ordered_float::OrderedFloat;

    use crate::{SPValue, SPValueType, ToSPValue};
    use std::time::SystemTime;
    
    #[test]
    fn test_is_type_bool() {
        let val = SPValue::Bool(true);
        assert!(val.is_type(SPValueType::Bool));
        assert!(!val.is_type(SPValueType::Float64));
    }
    
    #[test]
    fn test_is_type_float64() {
        let val = SPValue::Float64(OrderedFloat(3.14));
        assert!(val.is_type(SPValueType::Float64));
        assert!(!val.is_type(SPValueType::Int64));
    }
    
    #[test]
    fn test_is_type_int64() {
        let val = SPValue::Int64(42);
        assert!(val.is_type(SPValueType::Int64));
        assert!(!val.is_type(SPValueType::String));
    }
    
    #[test]
    fn test_is_type_string() {
        let val = SPValue::String("Hello, world!".to_string());
        assert!(val.is_type(SPValueType::String));
        assert!(!val.is_type(SPValueType::Bool));
    }
    
    #[test]
    fn test_is_type_time() {
        let val = SPValue::Time(SystemTime::now());
        assert!(val.is_type(SPValueType::Time));
        assert!(!val.is_type(SPValueType::Array));
    }
    
    #[test]
    fn test_is_type_array() {
        let val = SPValue::Array(
            SPValueType::Int64,
            vec![SPValue::Int64(1), SPValue::Int64(2)],
        );
        assert!(val.is_type(SPValueType::Array));
        assert!(!val.is_type(SPValueType::Time));
    }
    
    #[test]
    fn test_is_type_UNKNOWN() {
        let val = SPValue::UNKNOWN;
        assert!(val.is_type(SPValueType::UNKNOWN));
        assert!(!val.is_type(SPValueType::Int64));
    }
    
    #[test]
    fn test_has_type_bool() {
        let value = SPValue::Bool(true);
        assert_eq!(value.has_type(), SPValueType::Bool);
    }
    
    #[test]
    fn test_has_type_float64() {
        let value = SPValue::Float64(OrderedFloat(3.14));
        assert_eq!(value.has_type(), SPValueType::Float64);
    }
    
    #[test]
    fn test_has_type_int64() {
        let value = SPValue::Int64(42);
        assert_eq!(value.has_type(), SPValueType::Int64);
    }
    
    #[test]
    fn test_has_type_string() {
        let value = SPValue::String("Hello, world!".to_string());
        assert_eq!(value.has_type(), SPValueType::String);
    }
    
    #[test]
    fn test_has_type_time() {
        let value = SPValue::Time(SystemTime::UNIX_EPOCH);
        assert_eq!(value.has_type(), SPValueType::Time);
    }
    
    #[test]
    fn test_has_type_array() {
        let value = SPValue::Array(
            SPValueType::Int64,
            vec![SPValue::Int64(1), SPValue::Int64(2)],
        );
        assert_eq!(value.has_type(), SPValueType::Array);
    }
    
    #[test]
    fn test_has_type_UNKNOWN() {
        let value = SPValue::UNKNOWN;
        assert_eq!(value.has_type(), SPValueType::UNKNOWN);
    }
    
    #[test]
    fn test_is_array_returns_true_for_array_value() {
        let array_value = SPValue::Array(
            SPValueType::Int64,
            vec![SPValue::Int64(1), SPValue::Int64(2)],
        );
        assert_eq!(array_value.is_array(), true);
    }
    
    #[test]
    fn test_is_array_returns_false_for_non_array_values() {
        let bool_value = SPValue::Bool(true);
        assert_eq!(bool_value.is_array(), false);
    
        let float_value = SPValue::Float64(OrderedFloat(3.14));
        assert_eq!(float_value.is_array(), false);
    
        let int_value = SPValue::Int64(42);
        assert_eq!(int_value.is_array(), false);
    
        let string_value = SPValue::String("Hello, world!".to_string());
        assert_eq!(string_value.is_array(), false);
    
        let time_value = SPValue::Time(SystemTime::UNIX_EPOCH);
        assert_eq!(time_value.is_array(), false);
    
        let unknown_value = SPValue::UNKNOWN;
        assert_eq!(unknown_value.is_array(), false);
    }
    
    #[test]
    fn test_to_string_returns_correct_string_for_bool() {
        let bool_value = SPValue::Bool(true);
        assert_eq!(bool_value.to_string(), "true".to_string());
    
        let bool_value = SPValue::Bool(false);
        assert_eq!(bool_value.to_string(), "false".to_string());
    }
    
    #[test]
    fn test_to_string_returns_correct_string_for_float() {
        let float_value = SPValue::Float64(OrderedFloat(3.14));
        assert_eq!(float_value.to_string(), "3.14".to_string());
    }
    
    #[test]
    fn test_to_string_returns_correct_string_for_int() {
        let int_value = SPValue::Int64(42);
        assert_eq!(int_value.to_string(), "42".to_string());
    }
    
    #[test]
    fn test_to_string_returns_correct_string_for_string() {
        let string_value = SPValue::String("Hello, world!".to_string());
        assert_eq!(string_value.to_string(), "Hello, world!".to_string());
    }
    
    #[should_panic]
    #[test]
    fn test_to_string_returns_correct_string_for_time() {
        todo!()
    }
    
    #[test]
    fn test_to_string_returns_correct_string_for_array() {
        let array_value = SPValue::Array(
            SPValueType::Int64,
            vec![SPValue::Int64(1), SPValue::Int64(2), SPValue::Int64(3)],
        );
        assert_eq!(array_value.to_string(), "[1, 2, 3]".to_string());
    }
    
    #[test]
    fn test_to_string_returns_correct_string_for_UNKNOWN() {
        let UNKNOWN_value = SPValue::UNKNOWN;
        assert_eq!(UNKNOWN_value.to_string(), "[UNKNOWN]".to_string());
    }
    
    #[test]
    fn test_to_spvalue_bool() {
        assert_eq!(true.to_spvalue(), SPValue::Bool(true));
        assert_eq!(false.to_spvalue(), SPValue::Bool(false));
    }
    
    #[test]
    fn test_to_spvalue_i64() {
        assert_eq!((-1).to_spvalue(), SPValue::Int64(-1));
        assert_eq!(0.to_spvalue(), SPValue::Int64(0));
        assert_eq!(42.to_spvalue(), SPValue::Int64(42));
    }
    
    #[test]
    fn test_to_spvalue_f64() {
        assert_eq!(0.0.to_spvalue(), SPValue::Float64(OrderedFloat(0.0)));
        assert_eq!((-1.5).to_spvalue(), SPValue::Float64(OrderedFloat(-1.5)));
        assert_eq!(3.14.to_spvalue(), SPValue::Float64(OrderedFloat(3.14)));
    }
    
    #[test]
    fn test_to_spvalue_string() {
        assert_eq!("".to_spvalue(), SPValue::String("".to_string()));
        assert_eq!("hello".to_spvalue(), SPValue::String("hello".to_string()));
    }
    
    #[test]
    fn test_to_spvalue_str() {
        assert_eq!("".to_spvalue(), SPValue::String("".to_string()));
        assert_eq!("hello".to_spvalue(), SPValue::String("hello".to_string()));
    }
    
    #[test]
    fn test_to_spvalue_system_time() {
        let epoch = std::time::UNIX_EPOCH;
        assert_eq!(epoch.to_spvalue(), SPValue::Time(epoch));
    }
    
    #[test]
    fn test_display_bool_true() {
        let value = SPValue::Bool(true);
        assert_eq!(format!("{}", value), "true");
    }
    
    #[test]
    fn test_display_bool_false() {
        let value = SPValue::Bool(false);
        assert_eq!(format!("{}", value), "false");
    }
    
    #[test]
    fn test_display_float() {
        let value = SPValue::Float64(OrderedFloat(3.14));
        assert_eq!(format!("{}", value), "3.14");
    }
    
    #[test]
    fn test_display_int() {
        let value = SPValue::Int64(42);
        assert_eq!(format!("{}", value), "42");
    }
    
    #[test]
    fn test_display_string() {
        let value = SPValue::String(String::from("hello"));
        assert_eq!(format!("{}", value), "hello");
    }
    
    #[test]
    fn test_display_array() {
        let value = SPValue::Array(
            SPValueType::Int64,
            vec![SPValue::Int64(1), SPValue::Int64(2), SPValue::Int64(3)],
        );
        assert_eq!(format!("{}", value), "[Int64(1), Int64(2), Int64(3)]");
    }
    
    #[test]
    fn test_display_UNKNOWN() {
        let value = SPValue::UNKNOWN;
        assert_eq!(format!("{}", value), "[UNKNOWN]");
    }
    
    #[test]
    fn test_display_type_bool() {
        let value_type = SPValueType::Bool;
        assert_eq!(format!("{}", value_type), "bool");
    }
    
    #[test]
    fn test_display_type_float64() {
        let value_type = SPValueType::Float64;
        assert_eq!(format!("{}", value_type), "f64");
    }
    
    #[test]
    fn test_display_type_int64() {
        let value_type = SPValueType::Int64;
        assert_eq!(format!("{}", value_type), "i64");
    }
    
    #[test]
    fn test_display_type_string() {
        let value_type = SPValueType::String;
        assert_eq!(format!("{}", value_type), "string");
    }
    
    #[test]
    fn test_display_type_time() {
        let value_type = SPValueType::Time;
        assert_eq!(format!("{}", value_type), "time");
    }
    
    #[test]
    fn test_display_type_array() {
        let value_type = SPValueType::Array;
        assert_eq!(format!("{}", value_type), "array");
    }
    
    #[test]
    fn test_display_type_UNKNOWN() {
        let value_type = SPValueType::UNKNOWN;
        assert_eq!(format!("{}", value_type), "[UNKNOWN]");
    }
}