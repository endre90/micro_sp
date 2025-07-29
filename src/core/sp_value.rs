use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::{fmt, time::SystemTime};

// Represents a value of a specific type.
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
pub enum FloatOrUnknown {
    Float64(OrderedFloat<f64>),
    UNKNOWN,
}

#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BoolOrUnknown {
    Bool(bool),
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

    pub fn is_array(&self) -> bool {
        match self {
            SPValue::Array(_) => true,
            _ => false,
        }
    }

    pub fn is_string(&self) -> bool {
        match self {
            SPValue::String(_) => true,
            _ => false,
        }
    }

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

impl ToSPValue for Option<i64> {
    fn to_spvalue(&self) -> SPValue {
        match self {
            Some(value) => SPValue::Int64(IntOrUnknown::Int64(*value)),
            None => SPValue::Int64(IntOrUnknown::UNKNOWN),
        }
    }
}

impl ToSPValue for f64 {
    fn to_spvalue(&self) -> SPValue {
        SPValue::Float64(FloatOrUnknown::Float64(OrderedFloat(*self)))
    }
}

impl ToSPValue for Option<f64> {
    fn to_spvalue(&self) -> SPValue {
        match self {
            Some(value) => SPValue::Float64(FloatOrUnknown::Float64(OrderedFloat(*value))),
            None => SPValue::Float64(FloatOrUnknown::UNKNOWN),
        }
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

impl ToSPValue for Option<String> {
    fn to_spvalue(&self) -> SPValue {
        match self {
            Some(value) => {
                if value == "Unknown" || value == "unknown" || value == "UNKNOWN" {
                    SPValue::String(StringOrUnknown::UNKNOWN)
                } else {
                    SPValue::String(StringOrUnknown::String(value.clone()))
                }
            }
            None => SPValue::String(StringOrUnknown::UNKNOWN),
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

impl ToSPValue for Option<std::time::SystemTime> {
    fn to_spvalue(&self) -> SPValue {
        match self {
            Some(value) => SPValue::Time(TimeOrUnknown::Time(*value)),
            None => SPValue::Time(TimeOrUnknown::UNKNOWN),
        }
    }
}

impl ToSPValue for SPTransformStamped {
    fn to_spvalue(&self) -> SPValue {
        SPValue::Transform(TransformOrUnknown::Transform(self.clone()))
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

impl<K, V> ToSPValue for Vec<(K, V)>
where
    K: ToSPValue,
    V: ToSPValue,
{
    fn to_spvalue(&self) -> SPValue {
        let res = self
            .iter()
            .map(|(k, v)| (k.to_spvalue(), v.to_spvalue()))
            .collect::<Vec<(SPValue, SPValue)>>();
        SPValue::Map(MapOrUnknown::Map(res))
    }
}

impl ToSPValue for Vec<(SPValue, SPValue)> {
    fn to_spvalue(&self) -> SPValue {
        SPValue::Map(MapOrUnknown::Map(self.clone()))
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use ordered_float::OrderedFloat;
    use std::time::{Duration, SystemTime};

    fn create_dummy_transform() -> SPTransformStamped {
        SPTransformStamped {
            active_transform: true,
            enable_transform: true,
            time_stamp: SystemTime::now(),
            parent_frame_id: "world".to_string(),
            child_frame_id: "robot".to_string(),
            transform: SPTransform::default(),
            metadata: MapOrUnknown::Map(vec![("quality".to_spvalue(), "good".to_spvalue())]),
        }
    }

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
    fn test_is_type_map() {
        let val = SPValue::Map(MapOrUnknown::Map(vec![]));
        let val2 = SPValue::Map(MapOrUnknown::UNKNOWN);
        assert!(val.is_type(SPValueType::Map));
        assert!(val2.is_type(SPValueType::Map));
        assert!(!val.is_type(SPValueType::Transform));
        assert!(!val2.is_type(SPValueType::Transform));
    }

    #[test]
    fn test_is_type_transform() {
        let val = SPValue::Transform(TransformOrUnknown::Transform(create_dummy_transform()));
        let val2 = SPValue::Transform(TransformOrUnknown::UNKNOWN);
        assert!(val.is_type(SPValueType::Transform));
        assert!(val2.is_type(SPValueType::Transform));
        assert!(!val.is_type(SPValueType::Map));
        assert!(!val2.is_type(SPValueType::Map));
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
    fn test_has_type_map() {
        let val = SPValue::Map(MapOrUnknown::Map(vec![]));
        let val2 = SPValue::Map(MapOrUnknown::UNKNOWN);
        assert_eq!(val.has_type(), SPValueType::Map);
        assert_eq!(val2.has_type(), SPValueType::Map);
    }

    #[test]
    fn test_has_type_transform() {
        let val = SPValue::Transform(TransformOrUnknown::Transform(create_dummy_transform()));
        let val2 = SPValue::Transform(TransformOrUnknown::UNKNOWN);
        assert_eq!(val.has_type(), SPValueType::Transform);
        assert_eq!(val2.has_type(), SPValueType::Transform);
    }

    #[test]
    fn test_is_array() {
        let array_val = SPValue::Array(ArrayOrUnknown::Array(vec![]));
        let unknown_array_val = SPValue::Array(ArrayOrUnknown::UNKNOWN);
        let non_array_val = SPValue::Int64(IntOrUnknown::Int64(1));

        assert!(array_val.is_array());
        assert!(unknown_array_val.is_array());
        assert!(!non_array_val.is_array());
    }

    #[test]
    fn test_is_string() {
        let string_val = SPValue::String(StringOrUnknown::String("hello".to_string()));
        let unknown_string_val = SPValue::String(StringOrUnknown::UNKNOWN);
        let non_string_val = SPValue::Int64(IntOrUnknown::Int64(1));

        assert!(string_val.is_string());
        assert!(unknown_string_val.is_string());
        assert!(!non_string_val.is_string());
    }

    #[test]
    fn test_is_transform() {
        let transform_val =
            SPValue::Transform(TransformOrUnknown::Transform(create_dummy_transform()));
        let unknown_transform_val = SPValue::Transform(TransformOrUnknown::UNKNOWN);
        let non_transform_val = SPValue::Int64(IntOrUnknown::Int64(1));

        assert!(transform_val.is_transform());
        assert!(unknown_transform_val.is_transform());
        assert!(!non_transform_val.is_transform());
    }

    #[test]
    fn test_spvalue_to_string_methods() {
        assert_eq!(true.to_spvalue().to_string(), "true");
        assert_eq!(SPValue::Bool(BoolOrUnknown::UNKNOWN).to_string(), "UNKNOWN");

        assert_eq!(42.to_spvalue().to_string(), "42");
        assert_eq!(SPValue::Int64(IntOrUnknown::UNKNOWN).to_string(), "UNKNOWN");

        assert_eq!(3.14.to_spvalue().to_string(), "3.14");
        assert_eq!(
            SPValue::Float64(FloatOrUnknown::UNKNOWN).to_string(),
            "UNKNOWN"
        );

        assert_eq!("hello".to_spvalue().to_string(), "hello");
        assert_eq!(
            SPValue::String(StringOrUnknown::UNKNOWN).to_string(),
            "UNKNOWN"
        );

        let time_val = SystemTime::UNIX_EPOCH + Duration::from_secs(1);
        assert!(!time_val.to_spvalue().to_string().is_empty());
        assert_eq!(SPValue::Time(TimeOrUnknown::UNKNOWN).to_string(), "UNKNOWN");

        let array_val = vec![1.to_spvalue(), "a".to_spvalue()].to_spvalue();
        assert_eq!(array_val.to_string(), "[1, a]");
        assert_eq!(
            SPValue::Array(ArrayOrUnknown::UNKNOWN).to_string(),
            "UNKNOWN"
        );

        let map_val = vec![
            ("key1".to_spvalue(), 1.to_spvalue()),
            ("key2".to_spvalue(), true.to_spvalue()),
        ]
        .to_spvalue();
        assert_eq!(map_val.to_string(), "[(true, false), (true, false)]");
        assert_eq!(SPValue::Map(MapOrUnknown::UNKNOWN).to_string(), "UNKNOWN");

        let transform_val = create_dummy_transform().to_spvalue();
        assert!(transform_val.to_string().starts_with("TF(active=true"));
        assert_eq!(
            SPValue::Transform(TransformOrUnknown::UNKNOWN).to_string(),
            "UNKNOWN"
        );

        let mut transform_val = create_dummy_transform();
        transform_val.metadata = MapOrUnknown::UNKNOWN;

        assert!(
            transform_val
                .to_spvalue()
                .to_string()
                .starts_with("TF(active=true")
        );
        assert_eq!(
            SPValue::Transform(TransformOrUnknown::UNKNOWN).to_string(),
            "UNKNOWN"
        );
    }

    #[test]
    fn test_spvalue_display_trait() {
        assert_eq!(
            format!("{}", SPValue::Bool(BoolOrUnknown::Bool(true))),
            "true"
        );
        assert_eq!(
            format!("{}", SPValue::Bool(BoolOrUnknown::Bool(false))),
            "false"
        );
        assert_eq!(
            format!("{}", SPValue::Bool(BoolOrUnknown::UNKNOWN)),
            "UNKNOWN"
        );

        let float_val = SPValue::Float64(FloatOrUnknown::Float64(OrderedFloat(3.14)));
        assert_eq!(format!("{}", float_val), "3.14");
        assert_eq!(
            format!("{}", SPValue::Float64(FloatOrUnknown::UNKNOWN)),
            "UNKNOWN"
        );

        assert_eq!(format!("{}", SPValue::Int64(IntOrUnknown::Int64(42))), "42");
        assert_eq!(
            format!("{}", SPValue::Int64(IntOrUnknown::UNKNOWN)),
            "UNKNOWN"
        );

        let str_val = SPValue::String(StringOrUnknown::String("hello".to_string()));
        assert_eq!(format!("{}", str_val), "hello");
        assert_eq!(
            format!("{}", SPValue::String(StringOrUnknown::UNKNOWN)),
            "UNKNOWN"
        );

        let time_val = SPValue::Time(TimeOrUnknown::Time(SystemTime::now()));
        assert!(!format!("{}", time_val).is_empty()); // Can't assert exact time
        assert_eq!(
            format!("{}", SPValue::Time(TimeOrUnknown::UNKNOWN)),
            "UNKNOWN"
        );

        let array_val = SPValue::Array(ArrayOrUnknown::Array(vec![1.to_spvalue()]));
        assert_eq!(format!("{}", array_val), "[1]");
        assert_eq!(
            format!("{}", SPValue::Array(ArrayOrUnknown::UNKNOWN)),
            "UNKNOWN"
        );

        let map_val = SPValue::Map(MapOrUnknown::Map(vec![(
            "key".to_spvalue(),
            "value".to_spvalue(),
        )]));
        assert_eq!(format!("{}", map_val), "[(true, true)]");
        assert_eq!(
            format!("{}", SPValue::Map(MapOrUnknown::UNKNOWN)),
            "UNKNOWN"
        );

        let transform_val =
            SPValue::Transform(TransformOrUnknown::Transform(create_dummy_transform()));
        assert!(format!("{}", transform_val).starts_with("TF(active=true"));
        assert!(format!("{}", transform_val).contains("meta={quality: good}"));

        let mut tf_with_unknown_meta = create_dummy_transform();
        tf_with_unknown_meta.metadata = MapOrUnknown::UNKNOWN;
        let transform_val_unknown_meta =
            SPValue::Transform(TransformOrUnknown::Transform(tf_with_unknown_meta));
        assert!(format!("{}", transform_val_unknown_meta).contains("meta=UNKNOWN"));

        let unknown_transform = SPValue::Transform(TransformOrUnknown::UNKNOWN);
        assert_eq!(format!("{}", unknown_transform), "UNKNOWN");
    }

    #[test]
    fn test_spvaluetype_from_str() {
        assert_eq!(SPValueType::from_str("bool"), SPValueType::Bool);
        assert_eq!(SPValueType::from_str("f64"), SPValueType::Float64);
        assert_eq!(SPValueType::from_str("i64"), SPValueType::Int64);
        assert_eq!(SPValueType::from_str("string"), SPValueType::String);
        assert_eq!(SPValueType::from_str("time"), SPValueType::Time);
        assert_eq!(SPValueType::from_str("array"), SPValueType::Array);
        assert_eq!(SPValueType::from_str("map"), SPValueType::Map);
        assert_eq!(SPValueType::from_str("transform"), SPValueType::Transform);
    }

    #[test]
    #[should_panic(expected = "Unsupported SPValueType!")]
    fn test_spvaluetype_from_str_panic() {
        SPValueType::from_str("unsupported");
    }

    #[test]
    fn test_spvaluetype_display_trait() {
        assert_eq!(format!("{}", SPValueType::Bool), "bool");
        assert_eq!(format!("{}", SPValueType::Float64), "f64");
        assert_eq!(format!("{}", SPValueType::Int64), "i64");
        assert_eq!(format!("{}", SPValueType::String), "string");
        assert_eq!(format!("{}", SPValueType::Time), "time");
        assert_eq!(format!("{}", SPValueType::Array), "array");
        assert_eq!(format!("{}", SPValueType::Map), "map");
        assert_eq!(format!("{}", SPValueType::Transform), "transform");
    }

    #[test]
    fn test_to_spvalue_for_options() {
        assert_eq!(
            Some(true).to_spvalue(),
            SPValue::Bool(BoolOrUnknown::Bool(true))
        );
        assert_eq!(
            None::<bool>.to_spvalue(),
            SPValue::Bool(BoolOrUnknown::UNKNOWN)
        );

        assert_eq!(
            Some(10).to_spvalue(),
            SPValue::Int64(IntOrUnknown::Int64(10))
        );
        assert_eq!(
            None::<i64>.to_spvalue(),
            SPValue::Int64(IntOrUnknown::UNKNOWN)
        );

        assert_eq!(
            Some(1.23).to_spvalue(),
            SPValue::Float64(FloatOrUnknown::Float64(OrderedFloat(1.23)))
        );
        assert_eq!(
            None::<f64>.to_spvalue(),
            SPValue::Float64(FloatOrUnknown::UNKNOWN)
        );

        assert_eq!(
            Some("text".to_string()).to_spvalue(),
            SPValue::String(StringOrUnknown::String("text".to_string()))
        );
        assert_eq!(
            Some("UNKNOWN".to_string()).to_spvalue(),
            SPValue::String(StringOrUnknown::UNKNOWN)
        );
        assert_eq!(
            None::<String>.to_spvalue(),
            SPValue::String(StringOrUnknown::UNKNOWN)
        );

        let now = SystemTime::now();
        assert_eq!(
            Some(now).to_spvalue(),
            SPValue::Time(TimeOrUnknown::Time(now))
        );
        assert_eq!(
            None::<SystemTime>.to_spvalue(),
            SPValue::Time(TimeOrUnknown::UNKNOWN)
        );
    }

    #[test]
    fn test_to_spvalue_for_string() {
        assert_eq!(
            "hello".to_string().to_spvalue(),
            SPValue::String(StringOrUnknown::String("hello".to_string()))
        );
        assert_eq!(
            "unknown".to_string().to_spvalue(),
            SPValue::String(StringOrUnknown::UNKNOWN)
        );
        assert_eq!(
            "Unknown".to_string().to_spvalue(),
            SPValue::String(StringOrUnknown::UNKNOWN)
        );
        assert_eq!(
            "UNKNOWN".to_string().to_spvalue(),
            SPValue::String(StringOrUnknown::UNKNOWN)
        );
    }

    #[test]
    fn test_to_spvalue_for_str_slice() {
        assert_eq!(
            "hello".to_spvalue(),
            SPValue::String(StringOrUnknown::String("hello".to_string()))
        );
        assert_eq!(
            "unknown".to_spvalue(),
            SPValue::String(StringOrUnknown::UNKNOWN)
        );
        assert_eq!(
            "Unknown".to_spvalue(),
            SPValue::String(StringOrUnknown::UNKNOWN)
        );
    }

    #[test]
    fn test_to_spvalue_for_vecs() {
        let v_i64 = vec![1, 2, 3];
        let sp_v = SPValue::Array(ArrayOrUnknown::Array(vec![
            1.to_spvalue(),
            2.to_spvalue(),
            3.to_spvalue(),
        ]));
        assert_eq!(v_i64.to_spvalue(), sp_v);

        let sp_v_clone = sp_v.clone();
        assert_eq!(
            vec![1.to_spvalue(), 2.to_spvalue(), 3.to_spvalue()].to_spvalue(),
            sp_v_clone
        );

        assert_eq!(
            Vec::<SPValue>::new().to_spvalue(),
            SPValue::Array(ArrayOrUnknown::Array(vec![]))
        );
    }

    #[test]
    fn test_to_spvalue_for_vec_of_tuples() {
        let v_tuples: Vec<(i64, &str)> = vec![(1, "a"), (2, "b")];
        let sp_map = SPValue::Map(MapOrUnknown::Map(vec![
            (1.to_spvalue(), "a".to_spvalue()),
            (2.to_spvalue(), "b".to_spvalue()),
        ]));
        assert_eq!(v_tuples.to_spvalue(), sp_map);
    }

    #[test]
    fn test_to_spvalue_for_vec_of_spvalue_tuples() {
        let tuples: Vec<(SPValue, SPValue)> = vec![
            ("key1".to_spvalue(), 123.to_spvalue()),
            ("key2".to_spvalue(), true.to_spvalue()),
        ];
        let expected_map = SPValue::Map(MapOrUnknown::Map(tuples.clone()));
        assert_eq!(tuples.to_spvalue(), expected_map);

        let empty_tuples: Vec<(SPValue, SPValue)> = vec![];
        let expected_empty_map = SPValue::Map(MapOrUnknown::Map(vec![]));
        assert_eq!(empty_tuples.to_spvalue(), expected_empty_map);
    }

    #[test]
    fn test_sptransform_default() {
        let default_tf = SPTransform::default();
        assert_eq!(default_tf.translation.x, OrderedFloat(0.0));
        assert_eq!(default_tf.translation.y, OrderedFloat(0.0));
        assert_eq!(default_tf.translation.z, OrderedFloat(0.0));
        assert_eq!(default_tf.rotation.x, OrderedFloat(0.0));
        assert_eq!(default_tf.rotation.y, OrderedFloat(0.0));
        assert_eq!(default_tf.rotation.z, OrderedFloat(0.0));
        assert_eq!(default_tf.rotation.w, OrderedFloat(1.0));
    }
}
