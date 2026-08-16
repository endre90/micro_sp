use serde_json::Value;
use std::{
    collections::HashMap, fs::{self, File}, io::BufReader, path::Path, time::SystemTime
};

use crate::*;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ErrorMsg {
    info: String,
}

impl ErrorMsg {
    pub fn new(info: &str) -> ErrorMsg {
        ErrorMsg {
            info: info.to_string(),
        }
    }
}

impl fmt::Display for ErrorMsg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.info)
    }
}

impl Error for ErrorMsg {
    fn description(&self) -> &str {
        &self.info
    }
}

pub fn list_frames_in_dir(path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut scenario = vec![];
    match fs::read_dir(path) {
        Ok(dir) => dir.for_each(|file| match file {
            Ok(entry) => match entry.path().to_str() {
                Some(valid) => scenario.push(valid.to_string()),
                None => {
                    log::warn!(target: "r2r_transforms", "Scenario path is not valid unicode.")
                }
            },
            Err(e) => log::warn!(target: "r2r_transforms", "Reading entry failed with '{}'.", e),
        }),
        Err(e) => {
            log::warn!(target: "r2r_transforms",
                "Reading the scenario directory failed with: '{}'.",
                e
            );
            log::warn!(target: "r2r_transforms", "Empty scenario is loaded.");
            return Err(Box::new(ErrorMsg::new(&format!(
                "Reading the scenario directory failed with: '{}'. 
                    Empty scenario is loaded.",
                e
            ))));
        }
    }
    Ok(scenario)
}

fn json_value_to_spvalue(val: &Value) -> Option<SPValue> {
    match val {
        Value::Bool(b) => Some(SPValue::Bool(BoolOrUnknown::Bool(*b))),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Some(SPValue::Int64(IntOrUnknown::Int64(i)))
            } else if let Some(f) = n.as_f64() {
                Some(SPValue::Float64(FloatOrUnknown::Float64(
                    ordered_float::OrderedFloat(f),
                )))
            } else {
                log::error!(target: &&format!("redis_state_manager"), "Cannot represent number as i64 or f64.");
                None
            }
        }
        Value::String(s) => Some(SPValue::String(StringOrUnknown::String(s.clone()))),
        Value::Array(arr) => {
            let items: Vec<SPValue> = arr.iter().filter_map(json_value_to_spvalue).collect();
            Some(SPValue::Array(ArrayOrUnknown::Array(items)))
        }
        Value::Object(obj_map) => {
            let mut entries = Vec::new();
            for (k, v) in obj_map {
                let key_sp = SPValue::String(StringOrUnknown::String(k.clone()));
                if let Some(val_sp) = json_value_to_spvalue(v) {
                    entries.push((key_sp, val_sp));
                } else {
                    log::error!(target: &&format!("redis_state_manager"), "Couldn't convert, skipping.");
                }
            }
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            Some(SPValue::Map(MapOrUnknown::Map(entries)))
        }
        Value::Null => None,
    }
}

fn convert_metadata_value(metadata_val: &Value) -> MapOrUnknown {
    match metadata_val {
        Value::Object(obj_map) => {
            let mut entries = Vec::new();
            for (key_str, value_json) in obj_map {
                let key_sp = SPValue::String(StringOrUnknown::String(key_str.clone()));
                if let Some(value_sp) = json_value_to_spvalue(value_json) {
                    entries.push((key_sp, value_sp));
                } else {
                    log::error!(target: &&format!("redis_state_manager"), "Couldn't convert, skipping.");
                }
            }
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            MapOrUnknown::Map(entries)
        }
        _ => MapOrUnknown::UNKNOWN,
    }
}


fn collect_json_files(path: &Path, file_paths: &mut Vec<String>) {
    if path.is_file() {
        if let Some(path_str) = path.to_str() {
            if path_str.ends_with(".json") {
                file_paths.push(path_str.to_string());
            }
        }
    } else if path.is_dir() {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.filter_map(Result::ok) {
                collect_json_files(&entry.path(), file_paths);
            }
        }
    }
}

pub fn load_new_scenario(scenario: &Vec<String>) -> HashMap<String, SPTransformStamped> {
    let mut transforms_stamped = HashMap::new();
    let mut all_paths = Vec::new();

    for item in scenario {
        collect_json_files(Path::new(item), &mut all_paths);
    }

    for path in all_paths {
        let json = match load_json_from_file(&path) {
            Some(json) => json,
            None => continue,
        };

        let child_frame_id = match extract_string_field(&json, "child_frame_id") {
            Some(id) => id,
            None => continue,
        };

        let parent_frame_id = match extract_string_field(&json, "parent_frame_id") {
            Some(id) => id,
            None => continue,
        };

        let transform = match extract_transform(&json) {
            Some(transform) => transform,
            None => continue,
        };

        let metadata = json["metadata"].clone();

        let active_transform = if let Some(Value::Bool(val)) = metadata.get("active_transform") {
            *val
        } else {
            println!("active_transform not found or not a bool. Defaulting to true.");
            true
        };

        let enable_transform = if let Some(Value::Bool(val)) = metadata.get("enable_transform") {
            *val
        } else {
            println!("enable_transform not found or not a bool. Defaulting to true.");
            true
        };

        if enable_transform {
            transforms_stamped.insert(
                child_frame_id.clone(),
                SPTransformStamped {
                    active_transform,
                    enable_transform,
                    time_stamp: SystemTime::now(),
                    child_frame_id,
                    parent_frame_id,
                    transform,
                    metadata: convert_metadata_value(&metadata),
                },
            );
        }
    }

    transforms_stamped
}

pub fn load_new_scenario_no_check(scenario: &Vec<String>) -> HashMap<String, SPTransformStamped> {
    let mut transforms_stamped = HashMap::new();

    for path in scenario {
        let json = match load_json_from_file(path) {
            Some(json) => json,
            None => continue,
        };

        let child_frame_id = match extract_string_field(&json, "child_frame_id") {
            Some(id) => id,
            None => continue,
        };

        let parent_frame_id = match extract_string_field(&json, "parent_frame_id") {
            Some(id) => id,
            None => continue,
        };

        let transform = match extract_transform(&json) {
            Some(transform) => transform,
            None => continue,
        };

        let metadata = json["metadata"].clone();

        let active_transform = if let Some(Value::Bool(val)) = metadata.get("active_transform") {
            *val
        } else {
            println!("active_transform not found or not a bool. Defaulting to true.");
            true
        };

        let enable_transform = if let Some(Value::Bool(val)) = metadata.get("enable_transform") {
            *val
        } else {
            println!("enable_transform not found or not a bool. Defaulting to true.");
            true
        };

        transforms_stamped.insert(
            child_frame_id.clone(),
            SPTransformStamped {
                active_transform,
                enable_transform,
                time_stamp: SystemTime::now(),
                child_frame_id,
                parent_frame_id,
                transform,
                metadata: convert_metadata_value(&metadata),
            },
        );
    }

    transforms_stamped
}

fn load_json_from_file(path: &str) -> Option<Value> {
    match File::open(path) {
        Ok(file) => {
            let reader = BufReader::new(file);
            match serde_json::from_reader(reader) {
                Ok(json) => Some(json),
                Err(e) => {
                    log::warn!(target: "r2r_transforms",
                        concat!(
                            "Deserialization failed with: '{}'. ",
                            "The JSON file may be malformed or contain ",
                            "unexpected data."
                        ),
                        e
                    );
                    None
                }
            }
        }
        Err(e) => {
            log::warn!(target: "r2r_transforms",
                concat!(
                    "Opening json file failed with: '{}'. ",
                    "Please check if the file path is correct and ",
                    "you have sufficient permissions."
                ),
                e
            );
            None
        }
    }
}

fn extract_string_field(json: &Value, field: &str) -> Option<String> {
    match json.get(field).and_then(|v| v.as_str()) {
        Some(value) => Some(value.to_string()),
        None => {
            log::warn!(target: "r2r_transforms",
                concat!(
                    "Invalid or missing '{}'. ",
                    "Ensure the '{}' field is present and ",
                    "is a valid string."
                ),
                field, field
            );
            None
        }
    }
}

fn extract_transform(json: &Value) -> Option<SPTransform> {
    match json.get("transform") {
        Some(value) => match serde_json::from_value(value.clone()) {
            Ok(transform) => Some(transform),
            Err(e) => {
                log::warn!(target: "r2r_transforms",
                    concat!(
                        "Failed to deserialize 'transform' field: '{}'. ",
                        "Ensure the 'transform' field is correctly formatted."
                    ),
                    e
                );
                None
            }
        },
        None => {
            log::warn!(target: "",
                concat!(
                    "Missing 'transform' field. ",
                    "Ensure the 'transform' field is present in the JSON."
                )
            );
            None
        }
    }
}

/// Loading a scenario off disk.
///
/// This is the crate's only file-system input path: a directory of JSON frame
/// definitions written by hand or by a scene editor, turned into the transform
/// buffer the whole TF subsystem then works from. It is therefore also the place
/// where a typo in somebody's scene file gets its first and only validation.
///
/// The rules that matter, and that these tests pin, are all about *partial*
/// failure: one malformed file must cost that one frame and not the scenario,
/// and a frame with `enable_transform: false` must be left out entirely rather
/// than loaded in a disabled state.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    /// A scratch directory that cleans itself up. `tempfile` is not a
    /// dependency, and adding one for this would be more machinery than the
    /// three tests that need it are worth.
    struct ScratchDir {
        path: std::path::PathBuf,
    }

    impl ScratchDir {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "micro_sp_loading_{}",
                nanoid::nanoid!(10, &NANOID_ALPHABET)
            ));
            fs::create_dir_all(&path).unwrap();
            ScratchDir { path }
        }

        fn as_str(&self) -> &str {
            self.path.to_str().unwrap()
        }

        fn write(&self, name: &str, contents: &str) -> String {
            let file = self.path.join(name);
            if let Some(parent) = file.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            std::fs::write(&file, contents).unwrap();
            file.to_str().unwrap().to_string()
        }
    }

    impl Drop for ScratchDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn frame_json(child: &str, parent: &str, x: f64, metadata: &str) -> String {
        format!(
            r#"{{
                "parent_frame_id": "{parent}",
                "child_frame_id": "{child}",
                "transform": {{
                    "translation": {{ "x": {x}, "y": 0.0, "z": 0.0 }},
                    "rotation": {{ "x": 0.0, "y": 0.0, "z": 0.0, "w": 1.0 }}
                }},
                "metadata": {metadata}
            }}"#
        )
    }

    /// The bundled example scenario, which is what a consumer copies as a
    /// starting point - so it has to keep loading.
    #[test]
    fn the_bundled_example_scenario_loads() {
        let manifest_dir =
            std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");
        let path = format!("{}/src/transforms/examples/data", manifest_dir);

        let frames = list_frames_in_dir(&path).expect("the example data should be readable");
        assert!(!frames.is_empty(), "the example directory should not be empty");

        let scenario = load_new_scenario(&frames);
        assert!(!scenario.is_empty(), "the example scenario should load frames");

        // Every loaded frame is keyed by its own child id, and the metadata in
        // the files is carried through rather than dropped.
        for (key, transform) in &scenario {
            assert_eq!(key, &transform.child_frame_id);
            assert!(!transform.parent_frame_id.is_empty());
        }
        assert!(
            scenario.contains_key("chair"),
            "expected the example 'chair' frame, got {:?}",
            scenario.keys()
        );
        // 'chair' is declared with `active_transform: false` in the data.
        assert!(!scenario["chair"].active_transform);
    }

    #[test]
    fn listing_a_directory_that_does_not_exist_is_an_error() {
        let result = list_frames_in_dir("/definitely/not/a/real/directory");
        assert!(result.is_err());
        let message = result.unwrap_err().to_string();
        assert!(
            message.contains("Empty scenario is loaded"),
            "unexpected message: {message}"
        );
    }

    #[test]
    fn listing_an_empty_directory_yields_no_frames() {
        let dir = ScratchDir::new();
        assert_eq!(list_frames_in_dir(dir.as_str()).unwrap(), Vec::<String>::new());
    }

    /// `.json` files are collected recursively, and anything else is left
    /// alone - a README or a `.bak` next to the scene files must not break it.
    #[test]
    fn json_files_are_collected_recursively_and_others_ignored() {
        let dir = ScratchDir::new();
        dir.write("a.json", &frame_json("a", "world", 1.0, "{}"));
        dir.write("nested/b.json", &frame_json("b", "a", 2.0, "{}"));
        dir.write("nested/deeper/c.json", &frame_json("c", "b", 3.0, "{}"));
        dir.write("notes.txt", "not a frame");
        dir.write("nested/a.json.bak", "not a frame either");

        let scenario = load_new_scenario(&vec![dir.as_str().to_string()]);

        assert_eq!(scenario.len(), 3, "got {:?}", scenario.keys());
        for child in ["a", "b", "c"] {
            assert!(scenario.contains_key(child), "{child} missing");
        }
    }

    /// `enable_transform: false` means "do not load this frame at all", which is
    /// how a scene file is commented out. Its absence from the buffer is the
    /// whole point.
    #[test]
    fn a_disabled_frame_is_not_loaded() {
        let dir = ScratchDir::new();
        dir.write(
            "on.json",
            &frame_json("on", "world", 1.0, r#"{"enable_transform": true}"#),
        );
        dir.write(
            "off.json",
            &frame_json("off", "world", 1.0, r#"{"enable_transform": false}"#),
        );

        let scenario = load_new_scenario(&vec![dir.as_str().to_string()]);

        assert!(scenario.contains_key("on"));
        assert!(!scenario.contains_key("off"), "a disabled frame must be skipped");
    }

    /// Both metadata flags default to true when absent, so a minimal scene file
    /// with no metadata at all still loads as an active, enabled frame.
    #[test]
    fn the_metadata_flags_default_to_true() {
        let dir = ScratchDir::new();
        dir.write("plain.json", &frame_json("plain", "world", 1.0, "{}"));

        let scenario = load_new_scenario(&vec![dir.as_str().to_string()]);

        let frame = &scenario["plain"];
        assert!(frame.active_transform);
        assert!(frame.enable_transform);
    }

    /// Every way a file can be unusable. Each costs that file and nothing else,
    /// which is what lets a scenario survive one bad edit.
    #[test]
    fn a_malformed_file_is_skipped_without_losing_the_others() {
        let dir = ScratchDir::new();
        dir.write("good.json", &frame_json("good", "world", 1.0, "{}"));
        dir.write("not_json.json", "{ this is not json,,, }");
        dir.write(
            "no_child.json",
            r#"{"parent_frame_id": "world", "transform": {"translation": {"x":0.0,"y":0.0,"z":0.0}, "rotation": {"x":0.0,"y":0.0,"z":0.0,"w":1.0}}, "metadata": {}}"#,
        );
        dir.write(
            "no_parent.json",
            r#"{"child_frame_id": "x", "transform": {"translation": {"x":0.0,"y":0.0,"z":0.0}, "rotation": {"x":0.0,"y":0.0,"z":0.0,"w":1.0}}, "metadata": {}}"#,
        );
        dir.write(
            "no_transform.json",
            r#"{"parent_frame_id": "world", "child_frame_id": "y", "metadata": {}}"#,
        );
        dir.write(
            "bad_transform.json",
            r#"{"parent_frame_id": "world", "child_frame_id": "z", "transform": "not a transform", "metadata": {}}"#,
        );
        dir.write(
            "wrong_type_child.json",
            r#"{"parent_frame_id": "world", "child_frame_id": 42, "transform": {"translation": {"x":0.0,"y":0.0,"z":0.0}, "rotation": {"x":0.0,"y":0.0,"z":0.0,"w":1.0}}, "metadata": {}}"#,
        );

        let scenario = load_new_scenario(&vec![dir.as_str().to_string()]);

        assert_eq!(
            scenario.len(),
            1,
            "only the good frame should have loaded, got {:?}",
            scenario.keys()
        );
        assert!(scenario.contains_key("good"));
    }

    /// Metadata is converted into the crate's own `SPValue` map, with the keys
    /// sorted so the same file always produces the same value - which is what
    /// keeps a reload from showing up as a state change.
    #[test]
    fn metadata_is_converted_and_ordered_deterministically() {
        let dir = ScratchDir::new();
        dir.write(
            "meta.json",
            &frame_json(
                "meta",
                "world",
                1.0,
                r#"{"zone": 0.5, "frame_type": "waypoint", "visualize_mesh": true, "next_frame": ["b", "c"], "mesh_type": 3, "nested": {"k": "v"}, "nothing": null}"#,
            ),
        );

        let first = load_new_scenario(&vec![dir.as_str().to_string()]);
        let second = load_new_scenario(&vec![dir.as_str().to_string()]);
        assert_eq!(
            first["meta"].metadata, second["meta"].metadata,
            "the same file must convert to the same metadata every time"
        );

        let MapOrUnknown::Map(entries) = &first["meta"].metadata else {
            panic!("expected a metadata map");
        };
        let keys: Vec<String> = entries.iter().map(|(k, _)| k.to_string()).collect();
        let mut sorted = keys.clone();
        sorted.sort();
        assert_eq!(keys, sorted, "metadata keys must be sorted");

        // A JSON null carries no type, so it is dropped rather than guessed at.
        assert!(!keys.contains(&"nothing".to_string()));

        // And the whole thing decodes into the typed metadata struct.
        let decoded = decode_metadata(&first["meta"].metadata);
        assert_eq!(decoded.frame_type, Some("waypoint".to_string()));
        assert_eq!(decoded.zone, 0.5);
        assert_eq!(decoded.mesh_type, 3);
        assert!(decoded.visualize_mesh);
        assert_eq!(
            decoded.next_frame.map(|f| f.len()),
            Some(2),
            "the next_frame array should have survived the round trip"
        );
    }

    /// Metadata that is not an object at all cannot be converted, and becomes
    /// UNKNOWN rather than an empty map - so a consumer can tell "no metadata"
    /// from "metadata that made no sense".
    #[test]
    fn non_object_metadata_becomes_unknown() {
        let dir = ScratchDir::new();
        dir.write("weird.json", &frame_json("weird", "world", 1.0, r#""a string""#));

        let scenario = load_new_scenario(&vec![dir.as_str().to_string()]);
        assert_eq!(scenario["weird"].metadata, MapOrUnknown::UNKNOWN);
    }

    /// The `_no_check` variant takes explicit file paths rather than walking
    /// directories, and - as its name says - loads disabled frames too.
    #[test]
    fn load_new_scenario_no_check_takes_paths_and_keeps_disabled_frames() {
        let dir = ScratchDir::new();
        let on = dir.write(
            "on.json",
            &frame_json("on", "world", 1.0, r#"{"enable_transform": true}"#),
        );
        let off = dir.write(
            "off.json",
            &frame_json("off", "world", 2.0, r#"{"enable_transform": false}"#),
        );

        let scenario = load_new_scenario_no_check(&vec![on, off]);

        assert_eq!(scenario.len(), 2);
        assert!(scenario["on"].enable_transform);
        assert!(
            !scenario["off"].enable_transform,
            "the disabled frame is loaded, but still marked disabled"
        );

        // It is a plain path list, so a directory or a missing file is skipped.
        let mixed = load_new_scenario_no_check(&vec![
            dir.as_str().to_string(),
            "/no/such/file.json".to_string(),
        ]);
        assert!(mixed.is_empty());
    }

    /// A later file wins when two declare the same child frame - the buffer is
    /// keyed by child id, so one frame can only have one parent.
    #[test]
    fn a_duplicate_child_frame_is_overwritten_rather_than_duplicated() {
        let dir = ScratchDir::new();
        let first = dir.write("first.json", &frame_json("dup", "world", 1.0, "{}"));
        let second = dir.write("second.json", &frame_json("dup", "other", 2.0, "{}"));

        let scenario = load_new_scenario_no_check(&vec![first, second]);

        assert_eq!(scenario.len(), 1);
        assert_eq!(scenario["dup"].parent_frame_id, "other");
        assert_eq!(scenario["dup"].transform.translation.x.0, 2.0);
    }

    #[test]
    fn the_error_type_carries_its_message() {
        let error = ErrorMsg::new("something went wrong");
        assert_eq!(error.to_string(), "something went wrong");
        let boxed: Box<dyn std::error::Error> = Box::new(error);
        assert_eq!(boxed.to_string(), "something went wrong");
    }
}

// pub fn load_overlay_scenario

// pub async fn reload_scenario(
//     message: &r2r::scene_manipulation_msgs::srv::ManipulateExtras::Request,
//     broadcasted_frames: &Arc<Mutex<HashMap<String, FrameData>>>,
//     node_id: &str,
// ) -> ManipulateExtras::Response {
//     match list_frames_in_dir(&message.scenario_path, node_id).await {
//         Ok(scenario) => {
//             let loaded = load_scenario(&scenario, node_id);
//             let mut local_broadcasted_frames = broadcasted_frames.lock().unwrap().clone();
//             for x in &loaded {
//                 local_broadcasted_frames.insert(x.0.clone(), x.1.clone());
//             }
//             *broadcasted_frames.lock().unwrap() = local_broadcasted_frames;
//             extra_success_response(&format!(
//                 "Reloaded frames in the scene: '{:?}'.",
//                 loaded.keys()
//             ))
//         }
//         Err(e) => extra_error_response(&format!("Reloading the scenario failed with: '{:?}'.", e)),
//     }
// }

// async fn persist_frame_change(path: &str, frame: FrameData) -> bool {
//     match fs::read_dir(path) {
//         Ok(dir) => dir.for_each(|file| match file {
//             Ok(entry) => match entry.path().to_str() {
//                 Some(valid) => match valid.to_string() == format!("{}{}", path, frame.child_frame_id.clone()) {
//                     true => {
//                         println!("Changing existing frame {} permanently", frame.child_frame_id.clone());
//                         match File::open(valid.clone()) {
//                             Ok(file) =>
//                         }
//                         let writer = BufWriter::;
//                     // }
//                     },
//                     false => {}
//                 }
//                 None => r2r::log_warn!(NODE_ID, "Path is not valid unicode."),
//             },
//             Err(e) => r2r::log_warn!(NODE_ID, "Reading entry failed with '{}'.", e),
//         }),
//         Err(e) => {
//             r2r::log_warn!(
//                 NODE_ID,
//                 "Reading the scenario directory failed with: '{}'.",
//                 e
//             );
//             r2r::log_warn!(NODE_ID, "Empty scenario is loaded/reloaded.");
//             return false
//         }
//     }
//     true
// }
