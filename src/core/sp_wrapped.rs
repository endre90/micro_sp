//! Operands that are either a literal or a variable reference.
//!
//! Predicates and actions are written over [`SPWrapped`]: each side of a
//! comparison, and each right-hand side of an assignment, is either a literal
//! [`SPValue`] or a reference to an [`SPVariable`] that is looked up in the
//! [`State`] at evaluation time. The `Array` and `Map` variants build composite
//! values out of other operands.

use crate::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::fmt;

/// An operand in a predicate or action: a literal, a variable, or a composite
/// of those.
///
/// ```
/// use micro_sp::*;
///
/// let state = State::from_vec(&vec![
///     (SPVariable::new("count", SPValueType::Int64), 7.to_spvalue()),
/// ]);
///
/// // A literal evaluates to itself; a variable is read from the state.
/// assert_eq!(1.to_spvalue().wrap().evaluate(&state, "docs"), 1.to_spvalue());
/// let count = SPVariable::new("count", SPValueType::Int64);
/// assert_eq!(count.wrap().evaluate(&state, "docs"), 7.to_spvalue());
/// assert_eq!(count.wrap().get_variables(), vec![count]);
/// ```
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Clone, Hash, Serialize, Deserialize)]
pub enum SPWrapped {
    /// A reference to a variable, resolved against the state when evaluated.
    SPVariable(SPVariable),
    /// A literal value.
    SPValue(SPValue),
    /// An array built from other operands, evaluated element by element.
    Array(Vec<SPWrapped>),
    /// A map built from other operands; both keys and values are evaluated.
    Map(Vec<(SPWrapped, SPWrapped)>),
}

impl SPWrapped {
    /// Resolves the operand against `state`, producing a concrete [`SPValue`].
    ///
    /// Composites recurse. `log_target` is the logging target used for the
    /// underlying state reads.
    ///
    /// # Panics
    ///
    /// Panics if a referenced variable is not in the state, like every other
    /// state read in the crate.
    pub fn evaluate(&self, state: &State, log_target: &str) -> SPValue {
        match self {
            SPWrapped::SPVariable(var) => state
                .get_value(&var.name, log_target)
                .unwrap_or_else(|| panic!("Variable '{}' not in state.", var.name)),

            SPWrapped::SPValue(val) => val.clone(),

            SPWrapped::Array(arr) => {
                let evaluated_items: Vec<SPValue> = arr
                    .iter()
                    .map(|item| item.evaluate(state, log_target))
                    .collect();
                SPValue::Array(ArrayOrUnknown::Array(evaluated_items))
            }
            
            SPWrapped::Map(map) => {
                let evaluated_pairs: Vec<(SPValue, SPValue)> = map
                    .iter()
                    .map(|(k, v)| (k.evaluate(state, log_target), v.evaluate(state, log_target)))
                    .collect();
                SPValue::Map(MapOrUnknown::Map(evaluated_pairs))
            }
        }
    }

    /// Collects every variable this operand refers to, in order.
    ///
    /// Recurses through composites, including map *keys*. The runners build
    /// their key sets from this, so a variable missed here is a key that never
    /// gets read.
    pub fn get_variables(&self) -> Vec<SPVariable> {
        match self {
            SPWrapped::SPVariable(v) => vec![v.clone()],
            SPWrapped::SPValue(_) => vec![],
            SPWrapped::Array(arr) => arr.iter().flat_map(|x| x.get_variables()).collect(),
            SPWrapped::Map(map) => map
                .iter()
                .flat_map(|(k, v)| {
                    let mut vars = k.get_variables();
                    vars.extend(v.get_variables());
                    vars
                })
                .collect(),
        }
    }
}

/// Wraps a literal value as an [`SPWrapped`] operand.
///
/// Implemented for [`SPValue`] and for the same primitives as [`ToSPValue`].
/// The variable counterpart is [`ToSPWrappedVar`].
pub trait ToSPWrapped {
    /// Wraps `self` as a literal [`SPWrapped::SPValue`].
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
    /// Wrap this variable as an [`SPWrapped::SPVariable`] operand.
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
                    FloatOrUnknown::Float64(f_val) => write!(fmtr, "{}", f_val.into_inner() as f64),
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
            _ => write!(fmtr, "TODO!"),
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

/// The composite `SPWrapped` variants.
///
/// `SPWrapped::Array` and `SPWrapped::Map` are the experimental half of this
/// enum: they let a predicate or an action build a value out of *other*
/// variables rather than out of a literal. They are the only variants whose
/// `evaluate` recurses, and the only ones `get_variables` has to walk - which
/// matters beyond evaluation, because `get_variables` is what feeds the
/// runners' key sets, and a variable missed there is a key the runner never
/// reads.
#[cfg(test)]
mod composite_tests {
    use crate::*;

    const TARGET: &str = "test";

    fn state() -> State {
        State::from_vec(&vec![
            (SPVariable::new("a", SPValueType::Int64), 1.to_spvalue()),
            (SPVariable::new("b", SPValueType::Int64), 2.to_spvalue()),
            (SPVariable::new("k", SPValueType::String), "key".to_spvalue()),
        ])
    }

    fn var(name: &str, state: &State) -> SPWrapped {
        state.get_assignment(name, TARGET).var.wrap()
    }

    #[test]
    fn an_array_evaluates_each_of_its_elements() {
        let state = state();
        let array = SPWrapped::Array(vec![
            var("a", &state),
            var("b", &state),
            42.to_spvalue().wrap(),
        ]);

        assert_eq!(
            array.evaluate(&state, TARGET),
            vec![1.to_spvalue(), 2.to_spvalue(), 42.to_spvalue()].to_spvalue()
        );
    }

    #[test]
    fn a_map_evaluates_both_its_keys_and_its_values() {
        let state = state();
        let map = SPWrapped::Map(vec![
            (var("k", &state), var("a", &state)),
            ("literal".to_spvalue().wrap(), var("b", &state)),
        ]);

        assert_eq!(
            map.evaluate(&state, TARGET),
            SPValue::Map(MapOrUnknown::Map(vec![
                ("key".to_spvalue(), 1.to_spvalue()),
                ("literal".to_spvalue(), 2.to_spvalue()),
            ]))
        );
    }

    #[test]
    fn composites_nest() {
        let state = state();
        let nested = SPWrapped::Array(vec![SPWrapped::Map(vec![(
            var("k", &state),
            SPWrapped::Array(vec![var("a", &state)]),
        )])]);

        assert_eq!(
            nested.evaluate(&state, TARGET),
            vec![SPValue::Map(MapOrUnknown::Map(vec![(
                "key".to_spvalue(),
                vec![1.to_spvalue()].to_spvalue()
            )]))]
            .to_spvalue()
        );
    }

    #[test]
    fn an_empty_composite_evaluates_to_an_empty_value() {
        let state = state();
        assert_eq!(
            SPWrapped::Array(vec![]).evaluate(&state, TARGET),
            Vec::<SPValue>::new().to_spvalue()
        );
        assert_eq!(
            SPWrapped::Map(vec![]).evaluate(&state, TARGET),
            SPValue::Map(MapOrUnknown::Map(vec![]))
        );
    }

    /// `get_variables` has to reach through both composites, in both halves of
    /// a map entry - this is what decides whether a runner reads the key.
    #[test]
    fn get_variables_reaches_through_both_composites() {
        let state = state();

        assert_eq!(var("a", &state).get_variables().len(), 1);
        assert!(1.to_spvalue().wrap().get_variables().is_empty());

        let array = SPWrapped::Array(vec![var("a", &state), 9.to_spvalue().wrap(), var("b", &state)]);
        let names: Vec<String> = array.get_variables().iter().map(|v| v.name.clone()).collect();
        assert_eq!(names, vec!["a".to_string(), "b".to_string()]);

        let map = SPWrapped::Map(vec![(var("k", &state), var("a", &state))]);
        let names: Vec<String> = map.get_variables().iter().map(|v| v.name.clone()).collect();
        assert_eq!(
            names,
            vec!["k".to_string(), "a".to_string()],
            "a variable used as a map *key* counts too"
        );
    }

    /// A composite has no `Display` form of its own - it renders as the
    /// placeholder. Worth pinning because these strings end up in the "please
    /// satisfy the runner guard" message a user reads.
    #[test]
    fn a_composite_renders_as_a_placeholder() {
        let state = state();
        assert_eq!(SPWrapped::Array(vec![var("a", &state)]).to_string(), "TODO!");
        assert_eq!(
            SPWrapped::Map(vec![(var("k", &state), var("a", &state))]).to_string(),
            "TODO!"
        );
    }

    /// Evaluating a variable that is not in the state panics, like every other
    /// read path in the crate.
    #[test]
    #[should_panic(expected = "not in state")]
    fn evaluating_a_missing_variable_panics() {
        let state = state();
        let missing = SPWrapped::SPVariable(SPVariable::new("nope", SPValueType::Int64));
        let _ = missing.evaluate(&state, TARGET);
    }
}
