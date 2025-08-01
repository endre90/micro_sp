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
    pub mesh_file: Option<String>,
    pub mesh_scale: f32,
    pub mesh_r: f32,
    pub mesh_g: f32,
    pub mesh_b: f32,
    pub mesh_a: f32,
    pub secondary_transforms: Vec<SPTransform>,
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
            mesh_file: None,
            mesh_scale: 0.001,
            mesh_r: 1.0,
            mesh_g: 1.0,
            mesh_b: 1.0,
            mesh_a: 1.0,
            secondary_transforms: vec![],
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
