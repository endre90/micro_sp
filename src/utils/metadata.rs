use crate::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use std::collections::HashSet;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct PotentialTransformMetadata {
    pub next_frame: Option<HashSet<String>>, // next frame, good for visualizing path plans
    pub frame_type: Option<String>,          // can be used separate waypoint, tag, human, etc.
    pub visualize_mesh: bool,
    pub visualize_zone: bool,
    pub zone: f64,      // when are you "at" the frame, threshold, in meters
    pub mesh_type: i32, // 1 - cube, 2 - sphere, 3 - cylinder or 10 - mesh (provide path)
    pub override_meshes_dir: Option<String>, // To privide custom meshes dir for more publishers
    pub mesh_file: Option<String>,
    pub mesh_scale: f32,
    pub mesh_r: f32,
    pub mesh_g: f32,
    pub mesh_b: f32,
    pub mesh_a: f32,
    pub secondary_transforms: Vec<SPTransform>,
    pub mesh_use_embedded_materials: bool
}

impl Default for PotentialTransformMetadata {
    fn default() -> Self {
        PotentialTransformMetadata {
            next_frame: None,
            frame_type: None,
            visualize_mesh: false,
            visualize_zone: false,
            zone: 0.0,
            mesh_type: 10,
            override_meshes_dir: None,
            mesh_file: None,
            mesh_scale: 0.001,
            mesh_r: 1.0,
            mesh_g: 1.0,
            mesh_b: 1.0,
            mesh_a: 1.0,
            secondary_transforms: vec![],
            mesh_use_embedded_materials: false
        }
    }
}

fn parse_point(value: &SPValue) -> Option<SPTranslation> {
    let SPValue::Map(MapOrUnknown::Map(map)) = value else {
        return None;
    };

    let mut point = SPTranslation {
        x: OrderedFloat(0.0),
        y: OrderedFloat(0.0),
        z: OrderedFloat(0.0),
    };
    for (key, val) in map {
        let SPValue::String(StringOrUnknown::String(k)) = key else {
            continue;
        };
        let SPValue::Float64(FloatOrUnknown::Float64(v)) = val else {
            continue;
        };
        match k.as_str() {
            "x" => point.x = *v,
            "y" => point.y = *v,
            "z" => point.z = *v,
            _ => {}
        }
    }
    Some(point)
}

fn parse_quaternion(value: &SPValue) -> Option<SPRotation> {
    let SPValue::Map(MapOrUnknown::Map(map)) = value else {
        return None;
    };

    let mut quat = SPRotation {
        x: OrderedFloat(0.0),
        y: OrderedFloat(0.0),
        z: OrderedFloat(0.0),
        w: OrderedFloat(1.0),
    };
    for (key, val) in map {
        let SPValue::String(StringOrUnknown::String(k)) = key else {
            continue;
        };
        let SPValue::Float64(FloatOrUnknown::Float64(v)) = val else {
            continue;
        };
        match k.as_str() {
            "x" => quat.x = *v,
            "y" => quat.y = *v,
            "z" => quat.z = *v,
            "w" => quat.w = *v,
            _ => {}
        }
    }
    Some(quat)
}

fn parse_transform(value: &SPValue) -> Option<SPTransform> {
    let SPValue::Map(MapOrUnknown::Map(tf_map)) = value else {
        return None;
    };

    let mut translation = None;
    let mut rotation = None;

    for (key, val) in tf_map {
        let SPValue::String(StringOrUnknown::String(k)) = key else {
            continue;
        };
        match k.as_str() {
            "translation" => translation = parse_point(val),
            "rotation" => rotation = parse_quaternion(val),
            _ => {}
        }
    }

    Some(SPTransform {
        translation: translation?,
        rotation: rotation?,
    })
}

fn parse_secondary_transforms(value: &SPValue) -> Vec<SPTransform> {
    let SPValue::Array(ArrayOrUnknown::Array(frames)) = value else {
        return vec![];
    };

    frames
        .iter()
        .filter_map(|frame_item| {
            let SPValue::Map(MapOrUnknown::Map(map)) = frame_item else {
                return None;
            };
            for (key, val) in map {
                let SPValue::String(StringOrUnknown::String(k)) = key else {
                    continue;
                };
                if k == "transform" {
                    return parse_transform(val);
                }
            }
            None
        })
        .collect()
}

pub fn decode_metadata(map_value: &MapOrUnknown) -> PotentialTransformMetadata {
    let mut metadata = PotentialTransformMetadata::default();

    let map = match map_value {
        MapOrUnknown::Map(map) => map,
        MapOrUnknown::UNKNOWN => return metadata,
    };

    for (key_sp, sp_value) in map {
        let key_str = match key_sp {
            SPValue::String(StringOrUnknown::String(s)) => s.as_str(),
            _ => continue,
        };

        match key_str {
            "next_frame" => {
                if let SPValue::Array(ArrayOrUnknown::Array(arr)) = sp_value {
                    let mut string_set = HashSet::new();
                    for item_sp in arr {
                        if let SPValue::String(StringOrUnknown::String(s)) = item_sp {
                            string_set.insert(s.clone());
                        }
                    }
                    if !string_set.is_empty() {
                        metadata.next_frame = Some(string_set);
                    }
                }
            }
            "frame_type" => {
                if let SPValue::String(StringOrUnknown::String(s)) = sp_value {
                    metadata.frame_type = Some(s.clone());
                }
            }
            "visualize_mesh" => {
                if let SPValue::Bool(BoolOrUnknown::Bool(b)) = sp_value {
                    metadata.visualize_mesh = *b;
                }
            }
            "mesh_use_embedded_materials" => {
                if let SPValue::Bool(BoolOrUnknown::Bool(b)) = sp_value {
                    metadata.mesh_use_embedded_materials = *b;
                }
            }
            "visualize_zone" => {
                if let SPValue::Bool(BoolOrUnknown::Bool(b)) = sp_value {
                    metadata.visualize_zone = *b;
                }
            }
            "zone" => {
                if let SPValue::Float64(FloatOrUnknown::Float64(of)) = sp_value {
                    metadata.zone = of.into_inner();
                }
            }
            "mesh_type" => {
                if let SPValue::Int64(IntOrUnknown::Int64(i)) = sp_value {
                    if let Ok(i32_val) = (*i).try_into() {
                        metadata.mesh_type = i32_val;
                    }
                }
            }
            "mesh_file" => {
                if let SPValue::String(StringOrUnknown::String(s)) = sp_value {
                    metadata.mesh_file = Some(s.clone());
                }
            }
            "override_meshes_dir" => {
                if let SPValue::String(string_or_unknown) = sp_value {
                    match string_or_unknown {
                        StringOrUnknown::UNKNOWN => metadata.override_meshes_dir = None,
                        StringOrUnknown::String(s) => {
                            metadata.override_meshes_dir = Some(s.clone())
                        }
                    }
                }
            }
            "mesh_scale" => {
                if let SPValue::Float64(FloatOrUnknown::Float64(of)) = sp_value {
                    metadata.mesh_scale = of.into_inner() as f32;
                }
            }
            "mesh_r" => {
                if let SPValue::Float64(FloatOrUnknown::Float64(of)) = sp_value {
                    metadata.mesh_r = of.into_inner() as f32;
                }
            }
            "mesh_g" => {
                if let SPValue::Float64(FloatOrUnknown::Float64(of)) = sp_value {
                    metadata.mesh_g = of.into_inner() as f32;
                }
            }
            "mesh_b" => {
                if let SPValue::Float64(FloatOrUnknown::Float64(of)) = sp_value {
                    metadata.mesh_b = of.into_inner() as f32;
                }
            }
            "mesh_a" => {
                if let SPValue::Float64(FloatOrUnknown::Float64(of)) = sp_value {
                    metadata.mesh_a = of.into_inner() as f32;
                }
            }

            "secondary_transforms" => {
                metadata.secondary_transforms = parse_secondary_transforms(sp_value);
            }
            _ => {}
        }
    }

    metadata
}

/// `decode_metadata` is the boundary between a transform's untyped `metadata`
/// map - which arrives from a JSON file on disk or from another process writing
/// Redis directly - and the typed struct the visualisation code uses. It is
/// written entirely as "if the value happens to have the type I expect, take
/// it; otherwise silently keep the default", so the interesting cases are not
/// the happy path but the malformed ones: a field of the wrong type, a field
/// missing, a nested map missing half its keys.
///
/// These tests pin the whole field table (a field that stops being decoded is a
/// visualisation that quietly reverts to a default) and each of those tolerance
/// rules.
#[cfg(test)]
mod tests {
    use super::*;

    fn key(name: &str) -> SPValue {
        SPValue::String(StringOrUnknown::String(name.to_string()))
    }

    fn float(value: f64) -> SPValue {
        SPValue::Float64(FloatOrUnknown::Float64(OrderedFloat(value)))
    }

    fn int(value: i64) -> SPValue {
        SPValue::Int64(IntOrUnknown::Int64(value))
    }

    fn boolean(value: bool) -> SPValue {
        SPValue::Bool(BoolOrUnknown::Bool(value))
    }

    fn text(value: &str) -> SPValue {
        SPValue::String(StringOrUnknown::String(value.to_string()))
    }

    fn map(entries: Vec<(&str, SPValue)>) -> MapOrUnknown {
        MapOrUnknown::Map(entries.into_iter().map(|(k, v)| (key(k), v)).collect())
    }

    fn map_value(entries: Vec<(&str, SPValue)>) -> SPValue {
        SPValue::Map(map(entries))
    }

    fn array(items: Vec<SPValue>) -> SPValue {
        SPValue::Array(ArrayOrUnknown::Array(items))
    }

    fn transform_value(
        translation: Vec<(&str, SPValue)>,
        rotation: Vec<(&str, SPValue)>,
    ) -> SPValue {
        map_value(vec![
            ("translation", map_value(translation)),
            ("rotation", map_value(rotation)),
        ])
    }

    #[test]
    fn an_unknown_map_decodes_to_the_defaults() {
        assert_eq!(
            decode_metadata(&MapOrUnknown::UNKNOWN),
            PotentialTransformMetadata::default()
        );
    }

    #[test]
    fn an_empty_map_decodes_to_the_defaults() {
        assert_eq!(
            decode_metadata(&map(vec![])),
            PotentialTransformMetadata::default()
        );
    }

    /// The defaults are what every un-set field falls back to, so they are part
    /// of the contract rather than an implementation detail: `mesh_type` 10
    /// means "mesh file", and a scale of 0.001 is millimetres-to-metres.
    #[test]
    fn the_documented_defaults_hold() {
        let metadata = PotentialTransformMetadata::default();
        assert_eq!(metadata.next_frame, None);
        assert_eq!(metadata.frame_type, None);
        assert!(!metadata.visualize_mesh);
        assert!(!metadata.visualize_zone);
        assert!(!metadata.mesh_use_embedded_materials);
        assert_eq!(metadata.zone, 0.0);
        assert_eq!(metadata.mesh_type, 10);
        assert_eq!(metadata.override_meshes_dir, None);
        assert_eq!(metadata.mesh_file, None);
        assert_eq!(metadata.mesh_scale, 0.001);
        assert_eq!(
            (
                metadata.mesh_r,
                metadata.mesh_g,
                metadata.mesh_b,
                metadata.mesh_a
            ),
            (1.0, 1.0, 1.0, 1.0)
        );
        assert!(metadata.secondary_transforms.is_empty());
    }

    /// Every scalar field, decoded in one go - this is the table that breaks if
    /// a key is renamed on one side only.
    #[test]
    fn every_scalar_field_is_decoded() {
        let metadata = decode_metadata(&map(vec![
            ("frame_type", text("waypoint")),
            ("visualize_mesh", boolean(true)),
            ("visualize_zone", boolean(true)),
            ("mesh_use_embedded_materials", boolean(true)),
            ("zone", float(0.25)),
            ("mesh_type", int(3)),
            ("mesh_file", text("gripper.stl")),
            ("override_meshes_dir", text("/opt/meshes")),
            ("mesh_scale", float(0.5)),
            ("mesh_r", float(0.1)),
            ("mesh_g", float(0.2)),
            ("mesh_b", float(0.3)),
            ("mesh_a", float(0.4)),
        ]));

        assert_eq!(metadata.frame_type, Some("waypoint".to_string()));
        assert!(metadata.visualize_mesh);
        assert!(metadata.visualize_zone);
        assert!(metadata.mesh_use_embedded_materials);
        assert_eq!(metadata.zone, 0.25);
        assert_eq!(metadata.mesh_type, 3);
        assert_eq!(metadata.mesh_file, Some("gripper.stl".to_string()));
        assert_eq!(metadata.override_meshes_dir, Some("/opt/meshes".to_string()));
        assert_eq!(metadata.mesh_scale, 0.5);
        assert_eq!(metadata.mesh_r, 0.1);
        assert_eq!(metadata.mesh_g, 0.2);
        assert_eq!(metadata.mesh_b, 0.3);
        assert_eq!(metadata.mesh_a, 0.4);
    }

    #[test]
    fn next_frame_collects_the_strings_in_the_array() {
        let metadata = decode_metadata(&map(vec![(
            "next_frame",
            array(vec![text("b"), text("c"), text("b")]),
        )]));

        let frames = metadata.next_frame.expect("next_frame should be set");
        assert_eq!(frames.len(), 2, "it is a set, so the duplicate collapses");
        assert!(frames.contains("b") && frames.contains("c"));
    }

    /// An empty array is treated as "not set" rather than as an empty set, so
    /// downstream code keeps distinguishing "no next frame" from "a next frame
    /// list that happens to be empty".
    #[test]
    fn an_empty_next_frame_array_leaves_it_unset() {
        assert_eq!(decode_metadata(&map(vec![("next_frame", array(vec![]))])).next_frame, None);

        // Same when the array holds nothing that is a string.
        assert_eq!(
            decode_metadata(&map(vec![("next_frame", array(vec![int(1), boolean(true)]))]))
                .next_frame,
            None
        );
    }

    /// `override_meshes_dir` is the one string field with an explicit UNKNOWN
    /// arm: an explicit UNKNOWN clears it rather than leaving the default.
    #[test]
    fn an_unknown_override_meshes_dir_clears_the_field() {
        let metadata = decode_metadata(&map(vec![(
            "override_meshes_dir",
            SPValue::String(StringOrUnknown::UNKNOWN),
        )]));
        assert_eq!(metadata.override_meshes_dir, None);
    }

    /// A field carrying the wrong type must not be decoded, and must not stop
    /// the fields around it from being decoded either - one bad entry in a file
    /// should cost that one entry, not the whole frame.
    #[test]
    fn a_field_of_the_wrong_type_is_skipped_without_affecting_the_rest() {
        let metadata = decode_metadata(&map(vec![
            ("zone", text("not a float")),
            ("mesh_type", float(3.0)),
            ("visualize_mesh", int(1)),
            ("frame_type", int(7)),
            ("mesh_file", boolean(true)),
            ("next_frame", text("not an array")),
            // ... and one good one, after all the bad ones.
            ("mesh_scale", float(0.75)),
        ]));

        let defaults = PotentialTransformMetadata::default();
        assert_eq!(metadata.zone, defaults.zone);
        assert_eq!(metadata.mesh_type, defaults.mesh_type);
        assert_eq!(metadata.visualize_mesh, defaults.visualize_mesh);
        assert_eq!(metadata.frame_type, defaults.frame_type);
        assert_eq!(metadata.mesh_file, defaults.mesh_file);
        assert_eq!(metadata.next_frame, defaults.next_frame);
        assert_eq!(metadata.mesh_scale, 0.75, "the valid field still decodes");
    }

    /// Keys that are not strings, and keys nobody knows, are ignored.
    #[test]
    fn unknown_and_non_string_keys_are_ignored() {
        let entries = vec![
            (int(1), text("a key that is not a string")),
            (key("something_nobody_decodes"), text("value")),
            (key("zone"), float(1.5)),
        ];
        let metadata = decode_metadata(&MapOrUnknown::Map(entries));
        assert_eq!(metadata.zone, 1.5);
    }

    /// `mesh_type` is an `i32` decoded from an `i64`, so a value that does not
    /// fit has to leave the default in place rather than wrap around.
    #[test]
    fn a_mesh_type_that_does_not_fit_in_i32_keeps_the_default() {
        let too_big = decode_metadata(&map(vec![("mesh_type", int(i64::from(i32::MAX) + 1))]));
        assert_eq!(too_big.mesh_type, 10);

        let fits = decode_metadata(&map(vec![("mesh_type", int(i64::from(i32::MAX)))]));
        assert_eq!(fits.mesh_type, i32::MAX);
    }

    #[test]
    fn secondary_transforms_are_decoded() {
        let metadata = decode_metadata(&map(vec![(
            "secondary_transforms",
            array(vec![map_value(vec![(
                "transform",
                transform_value(
                    vec![("x", float(1.0)), ("y", float(2.0)), ("z", float(3.0))],
                    vec![
                        ("x", float(0.0)),
                        ("y", float(0.0)),
                        ("z", float(0.0)),
                        ("w", float(1.0)),
                    ],
                ),
            )])]),
        )]));

        assert_eq!(metadata.secondary_transforms.len(), 1);
        let tf = &metadata.secondary_transforms[0];
        assert_eq!(
            (tf.translation.x.0, tf.translation.y.0, tf.translation.z.0),
            (1.0, 2.0, 3.0)
        );
        assert_eq!(tf.rotation.w.0, 1.0);
    }

    /// A translation or rotation that only lists some of its components takes
    /// the identity for the rest - `(0,0,0)` and `(0,0,0,1)`.
    #[test]
    fn a_partial_transform_falls_back_to_the_identity_components() {
        let metadata = decode_metadata(&map(vec![(
            "secondary_transforms",
            array(vec![map_value(vec![(
                "transform",
                transform_value(vec![("y", float(5.0))], vec![("x", float(0.5))]),
            )])]),
        )]));

        let tf = &metadata.secondary_transforms[0];
        assert_eq!(
            (tf.translation.x.0, tf.translation.y.0, tf.translation.z.0),
            (0.0, 5.0, 0.0)
        );
        assert_eq!(
            (
                tf.rotation.x.0,
                tf.rotation.y.0,
                tf.rotation.z.0,
                tf.rotation.w.0
            ),
            (0.5, 0.0, 0.0, 1.0),
            "w defaults to 1, not 0 - a zero quaternion is not a rotation"
        );
    }

    /// Every way a secondary transform can be malformed drops just that entry.
    #[test]
    fn malformed_secondary_transforms_are_dropped_individually() {
        let cases: Vec<(&str, SPValue)> = vec![
            ("not an array", map_value(vec![])),
            ("an item that is not a map", array(vec![text("nope")])),
            ("an item with no transform key", array(vec![map_value(vec![("other", text("x"))])])),
            (
                "a transform with no rotation",
                array(vec![map_value(vec![(
                    "transform",
                    map_value(vec![("translation", map_value(vec![("x", float(1.0))]))]),
                )])]),
            ),
            (
                "a transform with no translation",
                array(vec![map_value(vec![(
                    "transform",
                    map_value(vec![("rotation", map_value(vec![("w", float(1.0))]))]),
                )])]),
            ),
            (
                "a transform that is not a map",
                array(vec![map_value(vec![("transform", text("nope"))])]),
            ),
        ];

        for (label, value) in cases {
            let metadata = decode_metadata(&map(vec![("secondary_transforms", value)]));
            assert!(
                metadata.secondary_transforms.is_empty(),
                "{label} should have produced no secondary transforms"
            );
        }
    }

    /// A good entry next to a bad one survives.
    #[test]
    fn a_good_secondary_transform_survives_a_bad_neighbour() {
        let good = map_value(vec![(
            "transform",
            transform_value(
                vec![("x", float(9.0))],
                vec![("w", float(1.0))],
            ),
        )]);
        let bad = map_value(vec![("transform", text("nope"))]);

        let metadata = decode_metadata(&map(vec![(
            "secondary_transforms",
            array(vec![bad, good]),
        )]));

        assert_eq!(metadata.secondary_transforms.len(), 1);
        assert_eq!(metadata.secondary_transforms[0].translation.x.0, 9.0);
    }

    /// The struct is `Serialize`/`Deserialize`, and is handed to other
    /// processes that way.
    #[test]
    fn the_decoded_struct_survives_serde() {
        let metadata = decode_metadata(&map(vec![
            ("frame_type", text("tag")),
            ("zone", float(0.3)),
            ("next_frame", array(vec![text("b")])),
        ]));

        let json = serde_json::to_string(&metadata).unwrap();
        let back: PotentialTransformMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(back, metadata);
    }
}
