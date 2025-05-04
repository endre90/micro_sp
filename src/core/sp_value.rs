// use nalgebra::{Isometry3, Quaternion, UnitQuaternion, Vector3};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::{fmt, time::SystemTime};

/// Represents a variable value of a specific type.
#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum SPValue {
    Bool(BoolOrUnknown),
    Float64(FloatOrUnknown),
    Int64(IntOrUnknown),
    String(StringOrUnknown),
    Time(TimeOrUnknown),
    Array(ArrayOrUnknown),
    Map(MapOrUnknown), // The map is ordered
    Transform(TransformOrUnknown),
}

#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BoolOrUnknown {
    Bool(bool),
    UNKNOWN,
}

#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FloatOrUnknown {
    Float64(OrderedFloat<f64>),
    UNKNOWN,
}

#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum IntOrUnknown {
    Int64(i64),
    UNKNOWN,
}

#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum StringOrUnknown {
    String(String),
    UNKNOWN,
}

#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TimeOrUnknown {
    Time(SystemTime),
    UNKNOWN,
}

#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ArrayOrUnknown {
    Array(Vec<SPValue>),
    UNKNOWN,
}

#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MapOrUnknown {
    Map(Vec<(SPValue, SPValue)>),
    UNKNOWN,
}

#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TransformOrUnknown {
    Transform(SPTransformStamped),
    UNKNOWN,
}

#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SPTransform {
    pub translation: SPTranslation,
    pub rotation: SPRotation,
}

#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SPTranslation {
    pub x: OrderedFloat<f64>,
    pub y: OrderedFloat<f64>,
    pub z: OrderedFloat<f64>,
}

#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SPRotation {
    pub x: OrderedFloat<f64>,
    pub y: OrderedFloat<f64>,
    pub z: OrderedFloat<f64>,
    pub w: OrderedFloat<f64>,
}

impl Default for SPTransform {
    fn default() -> Self {
        SPTransform {
            translation: SPTranslation {
                x: ordered_float::OrderedFloat(0.0),
                y: ordered_float::OrderedFloat(0.0),
                z: ordered_float::OrderedFloat(0.0),
            },
            rotation: SPRotation {
                x: ordered_float::OrderedFloat(0.0),
                y: ordered_float::OrderedFloat(0.0),
                z: ordered_float::OrderedFloat(0.0),
                w: ordered_float::OrderedFloat(1.0),
            },
        }
    }
}

#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SPTransformStamped {
    pub active_transform: bool,
    pub enable_transform: bool,
    pub time_stamp: SystemTime,
    pub parent_frame_id: String,
    pub child_frame_id: String,
    pub transform: SPTransform,
    pub metadata: MapOrUnknown,
}

/// Displaying the value of an SPValue instance in a user-friendly way.
impl fmt::Display for SPValue {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SPValue::Bool(b) => match b {
                BoolOrUnknown::Bool(b_val) => match b_val {
                    true => write!(fmtr, "true"),
                    false => write!(fmtr, "false"),
                },
                BoolOrUnknown::UNKNOWN => write!(fmtr, "UNKNOWN"),
            },
            SPValue::Float64(f) => match f {
                FloatOrUnknown::Float64(f_val) => write!(fmtr, "{}", f_val.0),
                FloatOrUnknown::UNKNOWN => write!(fmtr, "UNKNOWN"),
            },
            SPValue::Int64(i) => match i {
                IntOrUnknown::Int64(i_val) => write!(fmtr, "{}", i_val),
                IntOrUnknown::UNKNOWN => write!(fmtr, "UNKNOWN"),
            },
            SPValue::String(s) => match s {
                StringOrUnknown::String(s_val) => write!(fmtr, "{}", s_val),
                StringOrUnknown::UNKNOWN => write!(fmtr, "UNKNOWN"),
            },
            SPValue::Time(t) => match t {
                TimeOrUnknown::Time(t_val) => {
                    write!(fmtr, "{:?}", t_val.elapsed().unwrap_or_default())
                }
                TimeOrUnknown::UNKNOWN => write!(fmtr, "UNKNOWN"),
            },
            SPValue::Array(a) => match a {
                ArrayOrUnknown::Array(a_val) => {
                    let items_str = a_val
                        .iter()
                        .map(|item| item.to_string())
                        .collect::<Vec<_>>()
                        .join(", ");
                    write!(fmtr, "[{}]", items_str)
                }
                ArrayOrUnknown::UNKNOWN => write!(fmtr, "UNKNOWN"),
            },
            SPValue::Map(m) => match m {
                MapOrUnknown::Map(m_val) => {
                    let items_str = m_val
                        .iter()
                        .map(|(k, v)| format!("({}, {})", k.is_string(), v.is_string()))
                        .collect::<Vec<_>>()
                        .join(", ");
                    write!(fmtr, "[{}]", items_str)
                }
                MapOrUnknown::UNKNOWN => write!(fmtr, "UNKNOWN"),
            },
            SPValue::Transform(t) => match t {
                TransformOrUnknown::Transform(ts_val) => {
                    let trans = &ts_val.transform.translation;
                    let trans_str =
                        format!("({:.3}, {:.3}, {:.3})", trans.x.0, trans.y.0, trans.z.0);

                    let rot = &ts_val.transform.rotation;
                    let rot_str = format!(
                        "({:.3}, {:.3}, {:.3}, {:.3})",
                        rot.x.0, rot.y.0, rot.z.0, rot.w.0
                    );

                    let time_str = format!("{:?}", ts_val.time_stamp.elapsed().unwrap_or_default());

                    let meta_str = match &ts_val.metadata {
                        MapOrUnknown::Map(map_val) => {
                            let items = map_val
                                .iter()
                                .map(|(k, v)| format!("{}: {}", k, v))
                                .collect::<Vec<_>>()
                                .join(", ");
                            format!("{{{}}}", items)
                        }
                        MapOrUnknown::UNKNOWN => "UNKNOWN".to_string(),
                    };

                    write!(
                        fmtr,
                        "TF(active={}, time={}, parent={}, child={}, translation:{}, rotation:{}, meta={})",
                        ts_val.active_transform,
                        time_str,
                        ts_val.parent_frame_id,
                        ts_val.child_frame_id,
                        trans_str,
                        rot_str,
                        meta_str
                    )
                }
                TransformOrUnknown::UNKNOWN => write!(fmtr, "UNKNOWN"),
            },
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
    Map,
    Transform,
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
            "map" => SPValueType::Map,
            "transform" => SPValueType::Transform,
            _ => panic!("Unsupported SPValueType!"),
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
            SPValueType::Map => write!(fmtr, "map"),
            SPValueType::Transform => write!(fmtr, "transform"),
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
            SPValue::Array(_) => SPValueType::Array == t,
            SPValue::Map(_) => SPValueType::Map == t,
            SPValue::Transform(_) => SPValueType::Transform == t,
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
            SPValue::Array(_) => SPValueType::Array,
            SPValue::Map(_) => SPValueType::Map,
            SPValue::Transform(_) => SPValueType::Transform,
        }
    }

    /// Checks whether the value is of the array type.
    pub fn is_array(&self) -> bool {
        match self {
            SPValue::Array(_) => true,
            _ => false,
        }
    }

    /// Checks whether the value is of the string type.
    pub fn is_string(&self) -> bool {
        match self {
            SPValue::String(_) => true,
            _ => false,
        }
    }

    /// Checks whether the value is of the transform type.
    pub fn is_transform(&self) -> bool {
        match self {
            SPValue::Transform(_) => true,
            _ => false,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            SPValue::Bool(b) => match b {
                BoolOrUnknown::Bool(b_val) => format!("{}", b_val),
                BoolOrUnknown::UNKNOWN => "UNKNOWN".to_string(),
            },
            SPValue::Int64(i) => match i {
                IntOrUnknown::Int64(i_val) => format!("{}", i_val),
                IntOrUnknown::UNKNOWN => "UNKNOWN".to_string(),
            },
            SPValue::Float64(f) => match f {
                FloatOrUnknown::Float64(f_val) => format!("{}", f_val.into_inner()),
                FloatOrUnknown::UNKNOWN => "UNKNOWN".to_string(),
            },
            SPValue::String(s) => match s {
                StringOrUnknown::String(s_val) => format!("{}", s_val),
                StringOrUnknown::UNKNOWN => "UNKNOWN".to_string(),
            },
            SPValue::Time(t) => match t {
                TimeOrUnknown::Time(t_val) => format!("{:?}", t_val.elapsed().unwrap_or_default()),
                TimeOrUnknown::UNKNOWN => "UNKNOWN".to_string(),
            },
            SPValue::Array(a) => match a {
                ArrayOrUnknown::Array(a_val) => {
                    let items_str = a_val
                        .iter()
                        .map(|item| item.to_string())
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("[{}]", items_str)
                }
                ArrayOrUnknown::UNKNOWN => "UNKNOWN".to_string(),
            },
            SPValue::Map(m) => match m {
                MapOrUnknown::Map(m_val) => {
                    let items_str = m_val
                        .iter()
                        .map(|(k, v)| format!("({}, {})", k.is_string(), v.is_string()))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("[{}]", items_str)
                }
                MapOrUnknown::UNKNOWN => "UNKNOWN".to_string(),
            },
            SPValue::Transform(t) => match t {
                TransformOrUnknown::Transform(ts_val) => {
                    let trans = &ts_val.transform.translation;
                    let trans_str =
                        format!("({:.3}, {:.3}, {:.3})", trans.x.0, trans.y.0, trans.z.0);

                    let rot = &ts_val.transform.rotation;
                    let rot_str = format!(
                        "({:.3}, {:.3}, {:.3}, {:.3})",
                        rot.x.0, rot.y.0, rot.z.0, rot.w.0
                    );

                    let time_str = format!("{:?}", ts_val.time_stamp.elapsed().unwrap_or_default());

                    let meta_str = match &ts_val.metadata {
                        MapOrUnknown::Map(map_val) => {
                            let items = map_val
                                .iter()
                                .map(|(k, v)| format!("{}: {}", k, v))
                                .collect::<Vec<_>>()
                                .join(", ");
                            format!("{{{}}}", items)
                        }
                        MapOrUnknown::UNKNOWN => "UNKNOWN".to_string(),
                    };

                    format!(
                        "TF(active={}, time={}, parent={}, child={}, translation:{}, rotation:{}, meta={})",
                        ts_val.active_transform,
                        time_str,
                        ts_val.parent_frame_id,
                        ts_val.child_frame_id,
                        trans_str,
                        rot_str,
                        meta_str
                    )
                }
                TransformOrUnknown::UNKNOWN => "UNKNOWN".to_string(),
            },
        }
    }
}

/// This trait defines a set of conversions from some Rust primitive types and containers to `SPValue`.
pub trait ToSPValue {
    fn to_spvalue(&self) -> SPValue;
}

impl ToSPValue for bool {
    fn to_spvalue(&self) -> SPValue {
        SPValue::Bool(BoolOrUnknown::Bool(*self))
    }
}

impl ToSPValue for Option<bool> {
    fn to_spvalue(&self) -> SPValue {
        match self {
            Some(value) => SPValue::Bool(BoolOrUnknown::Bool(*value)),
            None => SPValue::Bool(BoolOrUnknown::UNKNOWN),
        }
    }
}

impl ToSPValue for i64 {
    fn to_spvalue(&self) -> SPValue {
        SPValue::Int64(IntOrUnknown::Int64(*self))
    }
}

impl ToSPValue for f64 {
    fn to_spvalue(&self) -> SPValue {
        SPValue::Float64(FloatOrUnknown::Float64(OrderedFloat(*self)))
    }
}

impl ToSPValue for String {
    fn to_spvalue(&self) -> SPValue {
        if self == "Unknown" || self == "unknown" || self == "UNKNOWN" {
            SPValue::String(StringOrUnknown::UNKNOWN)
        } else {
            SPValue::String(StringOrUnknown::String(self.clone()))
        }
    }
}

impl ToSPValue for &str {
    fn to_spvalue(&self) -> SPValue {
        let s = self.to_string();
        if s == "Unknown" || s == "unknown" || s == "UNKNOWN" {
            SPValue::String(StringOrUnknown::UNKNOWN)
        } else {
            SPValue::String(StringOrUnknown::String(s.clone()))
        }
    }
}

impl ToSPValue for std::time::SystemTime {
    fn to_spvalue(&self) -> SPValue {
        SPValue::Time(TimeOrUnknown::Time(*self))
    }
}

impl ToSPValue for TimeOrUnknown {
    fn to_spvalue(&self) -> SPValue {
        SPValue::Time(self.clone())
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
            SPValue::Array(ArrayOrUnknown::Array(vec![]))
        } else {
            SPValue::Array(ArrayOrUnknown::Array(self.clone()))
        }
    }
}

impl ToSPValue for SPTransformStamped {
    fn to_spvalue(&self) -> SPValue {
        SPValue::Transform(TransformOrUnknown::Transform(self.clone()))
    }
}

impl ToSPValue for TransformOrUnknown {
    fn to_spvalue(&self) -> SPValue {
        SPValue::Transform(self.clone())
    }
}

pub trait ToIntOrUnknown {
    fn to_int_or_unknown(self) -> IntOrUnknown;
}

pub trait ToInt64 {
    fn to_int_or_zero(self) -> i64;
}

impl ToIntOrUnknown for SPValue {
    fn to_int_or_unknown(self) -> IntOrUnknown {
        match self {
            SPValue::Int64(int_or_unknown) => int_or_unknown,
            _ => {
                log::error!(target: &&format!("sp_values"), "SPValue is not of type Int64. Taken UNKNOWN.");
                IntOrUnknown::UNKNOWN
            }
        }
    }
}

impl ToInt64 for SPValue {
    fn to_int_or_zero(self) -> i64 {
        match self {
            SPValue::Int64(int_or_unknown) => match int_or_unknown {
                IntOrUnknown::Int64(val) => val,
                IntOrUnknown::UNKNOWN => 0,
            },
            _ => {
                log::error!(target: &&format!("sp_values"), "SPValue is not of type Int64. Taken 0.");
                0
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use ordered_float::OrderedFloat;

    use crate::*;
    use std::time::SystemTime;

    #[test]
    fn test_is_type_bool() {
        let val = SPValue::Bool(BoolOrUnknown::Bool(true));
        let val2 = SPValue::Bool(BoolOrUnknown::UNKNOWN);
        assert!(val.is_type(SPValueType::Bool));
        assert!(val2.is_type(SPValueType::Bool));
        assert!(!val.is_type(SPValueType::Float64));
        assert!(!val2.is_type(SPValueType::Float64));
    }

    #[test]
    fn test_is_type_float64() {
        let val = SPValue::Float64(FloatOrUnknown::Float64(OrderedFloat(3.14)));
        let val2 = SPValue::Float64(FloatOrUnknown::UNKNOWN);
        assert!(val.is_type(SPValueType::Float64));
        assert!(val2.is_type(SPValueType::Float64));
        assert!(!val.is_type(SPValueType::Int64));
        assert!(!val2.is_type(SPValueType::Int64));
    }

    #[test]
    fn test_is_type_int64() {
        let val = SPValue::Int64(IntOrUnknown::Int64(42));
        let val2 = SPValue::Int64(IntOrUnknown::UNKNOWN);
        assert!(val.is_type(SPValueType::Int64));
        assert!(val2.is_type(SPValueType::Int64));
        assert!(!val.is_type(SPValueType::String));
        assert!(!val2.is_type(SPValueType::String));
    }

    #[test]
    fn test_is_type_string() {
        let val = SPValue::String(StringOrUnknown::String("Hello, world!".to_string()));
        let val2 = SPValue::String(StringOrUnknown::UNKNOWN);
        assert!(val.is_type(SPValueType::String));
        assert!(val2.is_type(SPValueType::String));
        assert!(!val.is_type(SPValueType::Bool));
        assert!(!val2.is_type(SPValueType::Bool));
    }

    #[test]
    fn test_is_type_time() {
        let val = SPValue::Time(TimeOrUnknown::Time(SystemTime::now()));
        let val2 = SPValue::Time(TimeOrUnknown::UNKNOWN);
        assert!(val.is_type(SPValueType::Time));
        assert!(val2.is_type(SPValueType::Time));
        assert!(!val.is_type(SPValueType::Array));
        assert!(!val2.is_type(SPValueType::Array));
    }

    #[test]
    fn test_is_type_array() {
        let val = SPValue::Array(ArrayOrUnknown::Array(vec![
            SPValue::Int64(IntOrUnknown::Int64(1)),
            SPValue::Int64(IntOrUnknown::Int64(3)),
            SPValue::Int64(IntOrUnknown::UNKNOWN),
        ]));
        let val2 = SPValue::Array(ArrayOrUnknown::UNKNOWN);
        assert!(val.is_type(SPValueType::Array));
        assert!(val2.is_type(SPValueType::Array));
        assert!(!val.is_type(SPValueType::Time));
        assert!(!val2.is_type(SPValueType::Time));
    }

    #[test]
    fn test_has_type_bool() {
        let val = SPValue::Bool(BoolOrUnknown::Bool(true));
        let val2 = SPValue::Bool(BoolOrUnknown::UNKNOWN);
        assert_eq!(val.has_type(), SPValueType::Bool);
        assert_eq!(val2.has_type(), SPValueType::Bool);
    }

    #[test]
    fn test_has_type_float64() {
        let val = SPValue::Float64(FloatOrUnknown::Float64(OrderedFloat(3.14)));
        let val2 = SPValue::Float64(FloatOrUnknown::UNKNOWN);
        assert_eq!(val.has_type(), SPValueType::Float64);
        assert_eq!(val2.has_type(), SPValueType::Float64);
    }

    #[test]
    fn test_has_type_int64() {
        let val = SPValue::Int64(IntOrUnknown::Int64(42));
        let val2 = SPValue::Int64(IntOrUnknown::UNKNOWN);
        assert_eq!(val.has_type(), SPValueType::Int64);
        assert_eq!(val2.has_type(), SPValueType::Int64);
    }

    #[test]
    fn test_has_type_string() {
        let val = SPValue::String(StringOrUnknown::String("Hello, world!".to_string()));
        let val2 = SPValue::String(StringOrUnknown::UNKNOWN);
        assert_eq!(val.has_type(), SPValueType::String);
        assert_eq!(val2.has_type(), SPValueType::String);
    }

    #[test]
    fn test_has_type_time() {
        let val = SPValue::Time(TimeOrUnknown::Time(SystemTime::UNIX_EPOCH));
        let val2 = SPValue::Time(TimeOrUnknown::UNKNOWN);
        assert_eq!(val.has_type(), SPValueType::Time);
        assert_eq!(val2.has_type(), SPValueType::Time);
    }

    #[test]
    fn test_has_type_array() {
        let val = SPValue::Array(ArrayOrUnknown::Array(vec![
            SPValue::Int64(IntOrUnknown::Int64(1)),
            SPValue::Int64(IntOrUnknown::UNKNOWN),
        ]));
        let val2 = SPValue::Array(ArrayOrUnknown::UNKNOWN);
        assert_eq!(val.has_type(), SPValueType::Array);
        assert_eq!(val2.has_type(), SPValueType::Array);
    }

    #[test]
    fn test_is_array_returns_true_for_array_value() {
        let val = SPValue::Array(ArrayOrUnknown::Array(vec![
            SPValue::Int64(IntOrUnknown::Int64(1)),
            SPValue::Int64(IntOrUnknown::UNKNOWN),
        ]));
        let val2 = SPValue::Array(ArrayOrUnknown::UNKNOWN);
        assert_eq!(val.is_array(), true);
        assert_eq!(val2.is_array(), true);
    }

    #[test]
    fn test_is_array_returns_false_for_non_array_values() {
        let bool_value = SPValue::Bool(BoolOrUnknown::Bool(true));
        let bool_value2 = SPValue::Bool(BoolOrUnknown::UNKNOWN);
        assert_eq!(bool_value.is_array(), false);
        assert_eq!(bool_value2.is_array(), false);

        let float_value = SPValue::Float64(FloatOrUnknown::Float64(OrderedFloat(3.14)));
        let float_value2 = SPValue::Float64(FloatOrUnknown::UNKNOWN);
        assert_eq!(float_value.is_array(), false);
        assert_eq!(float_value2.is_array(), false);

        let int_value = SPValue::Int64(IntOrUnknown::Int64(42));
        let int_value2 = SPValue::Int64(IntOrUnknown::UNKNOWN);
        assert_eq!(int_value.is_array(), false);
        assert_eq!(int_value2.is_array(), false);

        let string_value = SPValue::String(StringOrUnknown::String("Hello, world!".to_string()));
        let string_value2 = SPValue::String(StringOrUnknown::UNKNOWN);
        assert_eq!(string_value.is_array(), false);
        assert_eq!(string_value2.is_array(), false);

        let time_value = SPValue::Time(TimeOrUnknown::Time(SystemTime::UNIX_EPOCH));
        let time_value2 = SPValue::Time(TimeOrUnknown::UNKNOWN);
        assert_eq!(time_value.is_array(), false);
        assert_eq!(time_value2.is_array(), false);
    }

    #[test]
    fn test_to_string_returns_correct_string_for_bool() {
        let bool_value = SPValue::Bool(BoolOrUnknown::Bool(true));
        assert_eq!(bool_value.to_string(), "true".to_string());

        let bool_value = SPValue::Bool(BoolOrUnknown::Bool(false));
        assert_eq!(bool_value.to_string(), "false".to_string());

        let bool_value = SPValue::Bool(BoolOrUnknown::UNKNOWN);
        assert_eq!(bool_value.to_string(), "UNKNOWN".to_string());
    }

    #[test]
    fn test_to_string_returns_correct_string_for_float() {
        let float_value = SPValue::Float64(FloatOrUnknown::Float64(OrderedFloat(3.14)));
        assert_eq!(float_value.to_string(), "3.14".to_string());

        let float_value = SPValue::Float64(FloatOrUnknown::UNKNOWN);
        assert_eq!(float_value.to_string(), "UNKNOWN".to_string());
    }

    #[test]
    fn test_to_string_returns_correct_string_for_int() {
        let int_value = SPValue::Int64(IntOrUnknown::Int64(42));
        assert_eq!(int_value.to_string(), "42".to_string());

        let int_value = SPValue::Int64(IntOrUnknown::UNKNOWN);
        assert_eq!(int_value.to_string(), "UNKNOWN".to_string());
    }

    #[test]
    fn test_to_string_returns_correct_string_for_string() {
        let string_value = SPValue::String(StringOrUnknown::String("Hello, world!".to_string()));
        assert_eq!(string_value.to_string(), "Hello, world!".to_string());

        let string_value = SPValue::String(StringOrUnknown::UNKNOWN);
        assert_eq!(string_value.to_string(), "UNKNOWN".to_string());
    }

    #[should_panic]
    #[test]
    fn test_to_string_returns_correct_string_for_time() {
        todo!()
    }

    #[test]
    fn test_to_string_returns_correct_string_for_array() {
        let array_value = SPValue::Array(ArrayOrUnknown::Array(vec![
            SPValue::Int64(IntOrUnknown::Int64(1)),
            SPValue::Int64(IntOrUnknown::Int64(2)),
            SPValue::Int64(IntOrUnknown::UNKNOWN),
        ]));
        assert_eq!(array_value.to_string(), "[1, 2, UNKNOWN]".to_string());
    }

    #[test]
    fn test_to_string_returns_correct_string_for_unknown() {
        let unknown_value = SPValue::String(StringOrUnknown::UNKNOWN);
        assert_eq!(unknown_value.to_string(), "UNKNOWN".to_string());
    }

    #[test]
    fn test_to_spvalue_bool() {
        assert_eq!(true.to_spvalue(), SPValue::Bool(BoolOrUnknown::Bool(true)));
        assert_eq!(
            false.to_spvalue(),
            SPValue::Bool(BoolOrUnknown::Bool(false))
        );
    }

    #[test]
    fn test_to_spvalue_i64() {
        assert_eq!((-1).to_spvalue(), SPValue::Int64(IntOrUnknown::Int64(-1)));
        assert_eq!(0.to_spvalue(), SPValue::Int64(IntOrUnknown::Int64(0)));
        assert_eq!(42.to_spvalue(), SPValue::Int64(IntOrUnknown::Int64(42)));
    }

    #[test]
    fn test_to_spvalue_f64() {
        assert_eq!(
            0.0.to_spvalue(),
            SPValue::Float64(FloatOrUnknown::Float64(OrderedFloat(0.0)))
        );
        assert_eq!(
            (-1.5).to_spvalue(),
            SPValue::Float64(FloatOrUnknown::Float64(OrderedFloat(-1.5)))
        );
        assert_eq!(
            3.14.to_spvalue(),
            SPValue::Float64(FloatOrUnknown::Float64(OrderedFloat(3.14)))
        );
    }

    #[test]
    fn test_to_spvalue_string() {
        assert_eq!(
            "".to_spvalue(),
            SPValue::String(StringOrUnknown::String("".to_string()))
        );
        assert_eq!(
            "hello".to_spvalue(),
            SPValue::String(StringOrUnknown::String("hello".to_string()))
        );
        assert_eq!(
            "UNKNOWN".to_spvalue(),
            SPValue::String(StringOrUnknown::UNKNOWN)
        );
    }

    #[test]
    fn test_to_spvalue_system_time() {
        let epoch = std::time::UNIX_EPOCH;
        assert_eq!(
            epoch.to_spvalue(),
            SPValue::Time(TimeOrUnknown::Time(epoch))
        );
    }

    #[test]
    fn test_display_bool_true() {
        let value = SPValue::Bool(BoolOrUnknown::Bool(true));
        assert_eq!(format!("{}", value), "true");
    }

    #[test]
    fn test_display_bool_false() {
        let value = SPValue::Bool(BoolOrUnknown::Bool(false));
        assert_eq!(format!("{}", value), "false");
    }

    #[test]
    fn test_display_bool_unknown() {
        let value = SPValue::Bool(BoolOrUnknown::UNKNOWN);
        assert_eq!(format!("{}", value), "UNKNOWN");
    }

    #[test]
    fn test_display_float() {
        let value = SPValue::Float64(FloatOrUnknown::Float64(OrderedFloat(3.14)));
        assert_eq!(format!("{}", value), "3.14");
    }

    #[test]
    fn test_display_float_unknown() {
        let value = SPValue::Float64(FloatOrUnknown::UNKNOWN);
        assert_eq!(format!("{}", value), "UNKNOWN");
    }

    #[test]
    fn test_display_int() {
        let value = SPValue::Int64(IntOrUnknown::Int64(42));
        assert_eq!(format!("{}", value), "42");
    }

    #[test]
    fn test_display_int_unknown() {
        let value = SPValue::Int64(IntOrUnknown::UNKNOWN);
        assert_eq!(format!("{}", value), "UNKNOWN");
    }

    #[test]
    fn test_display_string() {
        let value = SPValue::String(StringOrUnknown::String(String::from("hello")));
        assert_eq!(format!("{}", value), "hello");
    }

    #[test]
    fn test_display_string_unknown() {
        let value = SPValue::String(StringOrUnknown::UNKNOWN);
        assert_eq!(format!("{}", value), "UNKNOWN");
    }

    #[test]
    fn test_display_array() {
        let value = SPValue::Array(ArrayOrUnknown::Array(vec![
            SPValue::Int64(IntOrUnknown::Int64(1)),
            SPValue::Int64(IntOrUnknown::Int64(2)),
            SPValue::Int64(IntOrUnknown::Int64(3)),
        ]));
        assert_eq!(format!("{}", value), "[1, 2, 3]");
    }

    #[test]
    fn test_display_array_unknown() {
        let value = SPValue::Array(ArrayOrUnknown::UNKNOWN);
        assert_eq!(format!("{}", value), "UNKNOWN");
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
}
