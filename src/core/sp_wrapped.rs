use crate::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
// SPWrapped can either be a SPVariable or a SPValue.
pub enum SPWrapped {
    SPVariable(SPVariable),
    SPValue(SPValue),
}

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
        SPWrapped::SPValue(SPValue::Bool(BoolOrUnknown::Bool(*self)))
    }
}

impl ToSPWrapped for i64 {
    fn wrap(&self) -> SPWrapped {
        SPWrapped::SPValue(SPValue::Int64(IntOrUnknown::Int64(*self)))
    }
}

impl ToSPWrapped for f64 {
    fn wrap(&self) -> SPWrapped {
        SPWrapped::SPValue(SPValue::Float64(FloatOrUnknown::Float64(OrderedFloat(
            *self,
        ))))
    }
}

impl ToSPWrapped for String {
    fn wrap(&self) -> SPWrapped {
        SPWrapped::SPValue(SPValue::String(StringOrUnknown::String(self.clone())))
    }
}

impl ToSPWrapped for &str {
    fn wrap(&self) -> SPWrapped {
        SPWrapped::SPValue(SPValue::String(StringOrUnknown::String(
            (*self).to_string(),
        )))
    }
}

impl ToSPWrapped for std::time::SystemTime {
    fn wrap(&self) -> SPWrapped {
        SPWrapped::SPValue(SPValue::Time(TimeOrUnknown::Time(*self)))
    }
}

impl ToSPWrapped for SPTransformStamped {
    fn wrap(&self) -> SPWrapped {
        SPWrapped::SPValue(SPValue::Transform(TransformOrUnknown::Transform(
            self.clone(),
        )))
    }
}

impl ToSPWrapped for Vec<SPValue> {
    fn wrap(&self) -> SPWrapped {
        if self.is_empty() {
            SPWrapped::SPValue(SPValue::Array(ArrayOrUnknown::Array(vec![])))
        } else {
            SPWrapped::SPValue(SPValue::Array(ArrayOrUnknown::Array(self.clone())))
        }
    }
}

impl ToSPWrapped for Vec<(SPValue, SPValue)> {
    fn wrap(&self) -> SPWrapped {
        SPWrapped::SPValue(SPValue::Map(MapOrUnknown::Map(self.clone())))
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

impl fmt::Display for SPWrapped {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SPWrapped::SPValue(val) => match val {
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
                        write!(fmtr, "{}", items_str)
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

                        let time_str =
                            format!("{:?}", ts_val.time_stamp.elapsed().unwrap_or_default());

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
            },
            SPWrapped::SPVariable(var) => write!(fmtr, "{}", var.name.to_owned()),
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
            metadata: MapOrUnknown::Map(vec![("quality".to_spvalue(), "good".to_spvalue())]),
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
        assert_eq!(
            empty_vec_sp.wrap(),
            SPWrapped::SPValue(empty_vec_sp.to_spvalue())
        );

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
    fn test_display_for_spwrapped() {
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
}
