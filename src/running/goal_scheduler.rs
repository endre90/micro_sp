use std::time::SystemTime;

use crate::*;
use tokio::{
    sync::{mpsc, oneshot},
    time::{interval, Duration},
};

// When a goal appears on the goal to be queued variable, this task takes it
// and puts in the queue of goals to be executed. It also clears the variable so that
// new goals can arrive.
pub async fn goal_scheduler(
    name: &str, // micro_sp instance name
    command_sender: mpsc::Sender<StateManagement>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = interval(Duration::from_millis(3000));

    log::info!(target: &&format!("{}_goal_scheduler", name), "Online.");
    // command_sender
    //     .send(StateManagement::Set((
    //         format!("{}_operation_runner_online", name),
    //         SPValue::Bool(BoolOrUnknown::Bool(true)),
    //     )))
    //     .await?;

    loop {
        let (response_tx, response_rx) = oneshot::channel();
        command_sender
            .send(StateManagement::Get((
                format!("{}_incoming_goals", name),
                response_tx,
            )))
            .await?;
        let incoming_goals = response_rx.await?;
        let incoming_goals_and_prios = match incoming_goals {
            SPValue::Map(MapOrUnknown::Map(map)) => map
                .iter()
                .map(|(goal, priority)| {
                    let goal_id = nanoid::nanoid!();
                    let goal_priority = GoalPriority::from_str(&priority.to_string());
                    log::info!(target: &&format!("{}_goal_scheduler", name), 
                        "New goal with id '{}' arrived: '{}'.", goal_id, goal.to_string());
                    let _ = add_goal_to_state(&name, &goal_id, &goal, &goal_priority, &command_sender); // Need also goal from state to remove stuff
                    (goal.clone(), goal_priority.to_int())
                })
                .collect::<Vec<(SPValue, i64)>>(),
            _ => {
                log::error!(target: &&format!("{}_goal_scheduler", name), "Type of incoming_goals has to be a Map.");
                vec![]
            }
        };
        if !incoming_goals_and_prios.is_empty() {
            let (response_tx, response_rx) = oneshot::channel();
            command_sender
                .send(StateManagement::Get((
                    format!("{}_scheduled_goals", name),
                    response_tx,
                )))
                .await?;
            let scheduled_goals = response_rx.await?;
            let mut scheduled_goals_and_prios = match scheduled_goals {
                SPValue::Map(MapOrUnknown::Map(map)) => map
                    .iter()
                    .map(|(goal_id, priority)| {
                        (
                            goal_id.clone(),
                            GoalPriority::from_str(&priority.to_string()).to_int(),
                        )
                    })
                    .collect::<Vec<(SPValue, i64)>>(),
                _ => {
                    log::error!(target: &&format!("{}_goal_scheduler", name), "Type of scheduled_goals has to be a Map.");
                    vec![]
                }
            };
            scheduled_goals_and_prios.extend(incoming_goals_and_prios);
            scheduled_goals_and_prios.sort_by_key(|(_, v)| *v); // Keeps the order of equal elements
            let new_goal_schedule = SPValue::Map(MapOrUnknown::Map(
                scheduled_goals_and_prios
                    .iter()
                    .map(|(k, v)| {
                        (
                            k.clone(),
                            GoalPriority::from_int(v).to_string().to_spvalue(),
                        )
                    })
                    .collect::<Vec<(SPValue, SPValue)>>(),
            ));
            command_sender
                .send(StateManagement::Set((
                    format!("{}_scheduled_goals", name),
                    new_goal_schedule,
                )))
                .await?;
            // Clear the incoming map
            command_sender
                .send(StateManagement::Set((
                    format!("{}_incoming_goals", name),
                    SPValue::Map(MapOrUnknown::Map(vec![])),
                )))
                .await?;
            continue;
        }
        interval.tick().await;
    }
}

async fn add_goal_to_state(
    name: &str, // micro_sp instance name
    id: &str, // goal_id
    predicate: &SPValue,
    priority: &GoalPriority,
    command_sender: &mpsc::Sender<StateManagement>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut goal_state = State::new();

    let goal_id: SPVariable = v!(&&format!("{}_goal_{}_id", name, id)); // An id to track the goal
    let goal_predicate = v!(&&format!("{}_goal_{}_predicate", name, id)); // The actual goal
    let goal_priority = v!(&&format!("{}_goal_{}_priority", name, id)); // High(1), Normal(2), or Low(3)
    let goal_time_arrived = tv!(&&format!("{}_goal_{}_time_arrived", name, id)); // When did the goal arrive
    let goal_time_started = tv!(&&format!("{}_goal_{}_time_started", name, id)); // Start of the execution time of he goal
    let goal_time_concluded = tv!(&&format!("{}_goal_{}_time_concluded", name, id)); // When was the goal concluded
    let goal_conclusion = v!(&&format!("{}_goal_{}_conclusion", name, id)); // Completed, Failed, Aborted, Timedout
    let goal_nr_of_replans = iv!(&&format!("{}_goal_{}_nr_of_replans", name, id));
    let goal_nr_of_failures = iv!(&&format!("{}_goal_{}_nr_of_failures", name, id));
    let goal_nr_of_timeouts = iv!(&&format!("{}_goal_{}_nr_of_timeouts", name, id));
    let goal_planned_paths = mv!(&&format!("{}_goal_{}_planned_paths", name, id)); // A map of (planned_path(Array), planning_duration(Time))
    let goal_log = mv!(&&format!("{}_goal_{}_log", name, id)); // A map of (goal_id(String), goal_log(Array(GoalLog)))
    let goal_execution_path = av!((&&format!("{}_goal_{}_execution_path", name, id))); // Which operations and autos did we take to reach the goal
    let goal_duration = iv!((&&format!("{}_goal_{}_duration", name, id))); // how many milliseconds did the goal take to conclude

    goal_state = goal_state.add(assign!(goal_id, SPValue::String(StringOrUnknown::UNKNOWN)));
    goal_state = goal_state.add(assign!(goal_predicate, predicate.to_owned()));
    goal_state = goal_state.add(assign!(goal_priority, priority.to_string().to_spvalue()));
    goal_state = goal_state.add(assign!(
        goal_time_arrived,
        SPValue::Time(TimeOrUnknown::Time(SystemTime::now()))
    ));
    goal_state = goal_state.add(assign!(
        goal_time_started,
        SPValue::Time(TimeOrUnknown::UNKNOWN)
    ));
    goal_state = goal_state.add(assign!(
        goal_time_concluded,
        SPValue::Time(TimeOrUnknown::UNKNOWN)
    ));
    goal_state = goal_state.add(assign!(
        goal_conclusion,
        SPValue::String(StringOrUnknown::UNKNOWN)
    ));
    goal_state = goal_state.add(assign!(
        goal_nr_of_replans,
        SPValue::Int64(IntOrUnknown::Int64(0))
    ));
    goal_state = goal_state.add(assign!(
        goal_nr_of_failures,
        SPValue::Int64(IntOrUnknown::Int64(0))
    ));
    goal_state = goal_state.add(assign!(
        goal_nr_of_timeouts,
        SPValue::Int64(IntOrUnknown::Int64(0))
    ));
    goal_state = goal_state.add(assign!(
        goal_planned_paths,
        SPValue::Map(MapOrUnknown::Map(vec!()))
    ));
    goal_state = goal_state.add(assign!(goal_log, SPValue::Map(MapOrUnknown::Map(vec!()))));
    goal_state = goal_state.add(assign!(
        goal_execution_path,
        SPValue::Array(ArrayOrUnknown::Array(vec!()))
    ));
    goal_state = goal_state.add(assign!(
        goal_duration,
        SPValue::Int64(IntOrUnknown::Int64(0))
    ));

    command_sender
        .send(StateManagement::SetPartialState(goal_state))
        .await?;

    Ok(())
}


// #[cfg(test)]
// mod tests {
//     use super::*; // Import items from the outer module (where goal_scheduler is)
//     use crate::*;
//     use std::{collections::HashMap, str::FromStr, sync::Arc}; // Added FromStr for GoalPriority
//     use tokio::sync::{mpsc, oneshot, Mutex};
//     use tokio::time::{sleep, Duration};

//     // Helper to create a Map SPValue easily
//     fn create_map_spvalue(items: Vec<(&str, SPValue)>) -> SPValue {
//         SPValue::Map(MapOrUnknown::Map(
//             items
//                 .into_iter()
//                 .map(|(k, v)| (SPValue::String(StringOrUnknown::String(k.to_string())), v))
//                 .collect(),
//         ))
//     }

//     // Helper to create a String SPValue easily
//     fn str_spvalue(s: &str) -> SPValue {
//         SPValue::String(StringOrUnknown::String(s.to_string()))
//     }

//     // Mock State Manager Task
//     async fn mock_state_manager(
//         mut rx: mpsc::Receiver<StateManagement>,
//         state: Arc<Mutex<HashMap<String, SPValue>>>,
//     ) {
//         while let Some(cmd) = rx.recv().await {
//             let mut state_guard = state.lock().await;
//             match cmd {
//                 StateManagement::Get((key, response_tx)) => {
//                     let value = state_guard
//                         .get(&key)
//                         .cloned()
//                         .unwrap_or_else(|| {
//                             // Provide sensible defaults if key not found, mirroring potential real behavior
//                             if key.ends_with("_incoming_goals") || key.ends_with("_scheduled_goals") || key.ends_with("_planned_paths") || key.ends_with("_log") {
//                                 SPValue::Map(MapOrUnknown::Map(vec![]))
//                             } else if key.ends_with("_execution_path") {
//                                 SPValue::Array(ArrayOrUnknown::Array(vec![]))
//                             } else {
//                                 // Default for unknown keys, adjust if needed
//                                 SPValue::String(StringOrUnknown::UNKNOWN)
//                             }
//                         });
//                     let _ = response_tx.send(value); // Ignore error if receiver dropped
//                 }
//                 StateManagement::Set((key, value)) => {
//                     log::debug!("Mock State Set: {} = {:?}", key, value);
//                     state_guard.insert(key, value);
//                 }
//                 StateManagement::SetPartialState(partial_state) => {
//                     log::debug!("Mock State SetPartialState: {} variables", partial_state.state.len());
//                     for (var, value) in partial_state.state {
//                         // Assuming SPVariable has a `path()` method or similar to get the string key
//                         // Adjust this based on your actual SPVariable implementation
//                         let key = var.to_string(); // Needs path() or similar method
//                         log::debug!("  Partial Set: {} = {:?}", key, value);
//                         state_guard.insert(key, value.clone()); // Clone value if needed
//                     }
//                 }
//                  // Add other StateManagement variants if needed
//                 _ => {
//                     log::warn!("Mock State Manager received unhandled command");
//                 }
//             }
//         }
//         log::debug!("Mock State Manager finished.");
//     }

//     #[tokio::test]
//     async fn test_goal_scheduler_adds_and_sorts_goals() {
//         let name = "test_sp";
//         let (command_sender, command_receiver) = mpsc::channel(32); // Channel for state communication

//         // Shared mock state
//         let state_data = Arc::new(Mutex::new(HashMap::new()));

//         // Initialize mock state
//         {
//             let mut state_guard = state_data.lock().await;
//             // Start with empty scheduled goals
//             state_guard.insert(
//                 format!("{}_scheduled_goals", name),
//                 SPValue::Map(MapOrUnknown::Map(vec![])),
//             );
//             // Add some incoming goals
//             let incoming_goals = create_map_spvalue(vec![
//                 ("goal_pred_1", GoalPriority::Normal.to_string().to_spvalue()), // Normal = 2
//                 ("goal_pred_2", GoalPriority::High.to_string().to_spvalue()),   // High = 1
//                 ("goal_pred_3", GoalPriority::Low.to_string().to_spvalue()),    // Low = 3
//             ]);
//             state_guard.insert(format!("{}_incoming_goals", name), incoming_goals);
//         }

//         // Start the mock state manager
//         let state_manager_handle = tokio::spawn(mock_state_manager(command_receiver, state_data.clone()));

//         // Start the goal_scheduler
//         let scheduler_handle = tokio::spawn(goal_scheduler(name, command_sender.clone()));

//         // Give the scheduler time to process (it checks immediately, then waits 3s)
//         sleep(Duration::from_millis(1000)).await; // Should be enough for one cycle

//         // --- Assertions ---

//         let final_state = state_data.lock().await;

//         // 1. Check if incoming_goals is cleared
//         let incoming_goals_final = final_state
//             .get(&format!("{}_incoming_goals", name))
//             .expect("incoming_goals should exist");
//         match incoming_goals_final {
//             SPValue::Map(MapOrUnknown::Map(map)) => {
//                 assert!(map.is_empty(), "Incoming goals map should be empty");
//             }
//             _ => panic!("incoming_goals should be a Map"),
//         }

//         // 2. Check if scheduled_goals is correctly populated and sorted
//         let scheduled_goals_final = final_state
//             .get(&format!("{}_scheduled_goals", name))
//             .expect("scheduled_goals should exist");

//         let expected_schedule_order = vec![
//             // Expected order: High, Normal, Low
//              // We don't know the exact goal IDs generated by nanoid,
//              // so we check priorities and the count.
//              // Goal IDs will be strings like "goal_pred_2", "goal_pred_1", "goal_pred_3"
//              // *Correction*: The key in scheduled_goals is the *goal predicate* from incoming,
//              // NOT the generated nanoid. The code uses `goal.clone()` which is the predicate.
//             (str_spvalue("goal_pred_2"), GoalPriority::High.to_string().to_spvalue()),
//             (str_spvalue("goal_pred_1"), GoalPriority::Normal.to_string().to_spvalue()),
//             (str_spvalue("goal_pred_3"), GoalPriority::Low.to_string().to_spvalue()),
//         ];
//         let expected_schedule_map = SPValue::Map(MapOrUnknown::Map(expected_schedule_order));

//         assert_eq!(scheduled_goals_final, &expected_schedule_map, "Scheduled goals are not correct or not sorted");

//         // 3. Check if detail state variables were added for each goal
//         // We need to find the keys added by add_goal_to_state. Since IDs are random,
//         // we can list all keys and check if *sets* of goal-related keys exist.
//         let goal_detail_keys: Vec<_> = final_state.keys()
//             .filter(|k| k.starts_with(&format!("{}_goal_", name)) && k.contains("_predicate"))
//             .cloned()
//             .collect();

//         assert_eq!(goal_detail_keys.len(), 3, "Expected detail state for 3 goals");

//         // Optionally, check specific details for one goal (find its ID first)
//         let goal_pred_1_key = format!("{}_goal_{}_predicate", name, "some_id"); // We don't know the ID
//         let goal_pred_1_prio_key = format!("{}_goal_{}_priority", name, "some_id");

//         let mut found_goal_1 = false;
//         for key in final_state.keys() {
//              if key.starts_with(&format!("{}_goal_", name)) && key.ends_with("_predicate") {
//                  if let Some(SPValue::String(StringOrUnknown::Str(pred))) = final_state.get(key) {
//                      if pred == "goal_pred_1" {
//                          // Found the predicate key, now extract the ID part
//                          let parts: Vec<&str> = key.split('_').collect();
//                          if parts.len() >= 4 {
//                             let goal_id = parts[parts.len()-2]; // Assuming format name_goal_ID_predicate
//                             let prio_key = format!("{}_goal_{}_priority", name, goal_id);
//                             let time_key = format!("{}_goal_{}_time_arrived", name, goal_id);

//                             assert_eq!(final_state.get(&prio_key), Some(&GoalPriority::Normal.to_string().to_spvalue()), "Priority mismatch for goal_pred_1");

//                             // Check time arrived is set
//                             match final_state.get(&time_key) {
//                                 Some(SPValue::Time(TimeOrUnknown::Time(_))) => (), // Correct
//                                 _ => panic!("time_arrived is not set correctly for goal_pred_1"),
//                             }
//                             found_goal_1 = true;
//                             break;
//                          }
//                      }
//                  }
//              }
//         }
//         assert!(found_goal_1, "Could not find detailed state for goal_pred_1");


//         // --- Cleanup ---
//         // Stop the scheduler by dropping the sender (causes sends to fail)
//         drop(command_sender);
//         let _ = scheduler_handle.await; // Wait for scheduler task to finish (optional)
//         // Stop the mock state manager (it will exit when the sender is dropped and rx.recv() returns None)
//         let _ = state_manager_handle.await;
//     }

//      #[tokio::test]
//     async fn test_goal_scheduler_merges_with_existing_schedule() {
//         let name = "test_sp_merge";
//         let (command_sender, command_receiver) = mpsc::channel(32);
//         let state_data = Arc::new(Mutex::new(HashMap::new()));

//         // Initialize mock state
//         {
//             let mut state_guard = state_data.lock().await;
//              // Start with existing scheduled goals
//             let existing_schedule = create_map_spvalue(vec![
//                 ("existing_goal_high", GoalPriority::High.to_string().to_spvalue()), // 1
//                 ("existing_goal_low", GoalPriority::Low.to_string().to_spvalue()),   // 3
//             ]);
//             state_guard.insert(format!("{}_scheduled_goals", name), existing_schedule);

//             // Add new incoming goals
//             let incoming_goals = create_map_spvalue(vec![
//                 ("new_goal_normal", GoalPriority::Normal.to_string().to_spvalue()), // 2
//                 ("new_goal_high", GoalPriority::High.to_string().to_spvalue()),     // 1
//             ]);
//             state_guard.insert(format!("{}_incoming_goals", name), incoming_goals);
//         }

//         let state_manager_handle = tokio::spawn(mock_state_manager(command_receiver, state_data.clone()));
//         let scheduler_handle = tokio::spawn(goal_scheduler(name, command_sender.clone()));

//         sleep(Duration::from_millis(1000)).await;

//         let final_state = state_data.lock().await;

//         // Check final schedule - should be sorted: high, high, normal, low
//         // Stable sort: existing_high before new_high
//         let scheduled_goals_final = final_state
//             .get(&format!("{}_scheduled_goals", name))
//             .expect("scheduled_goals should exist");

//          let expected_schedule_order = vec![
//              // Expected order: High, High, Normal, Low
//              // Keys are predicates. Order of equal priorities is preserved (existing first)
//             (str_spvalue("existing_goal_high"), GoalPriority::High.to_string().to_spvalue()),
//             (str_spvalue("new_goal_high"), GoalPriority::High.to_string().to_spvalue()),
//             (str_spvalue("new_goal_normal"), GoalPriority::Normal.to_string().to_spvalue()),
//             (str_spvalue("existing_goal_low"), GoalPriority::Low.to_string().to_spvalue()),
//         ];
//         let expected_schedule_map = SPValue::Map(MapOrUnknown::Map(expected_schedule_order));

//         assert_eq!(scheduled_goals_final, &expected_schedule_map, "Merged scheduled goals are not correct or not sorted");

//          // Check incoming is cleared
//          let incoming_goals_final = final_state
//             .get(&format!("{}_incoming_goals", name))
//             .expect("incoming_goals should exist");
//         match incoming_goals_final {
//             SPValue::Map(MapOrUnknown::Map(map)) => {
//                 assert!(map.is_empty(), "Incoming goals map should be empty after merge");
//             }
//             _ => panic!("incoming_goals should be a Map"),
//         }

//          // Check detail state exists for the *new* goals
//         let goal_detail_keys: Vec<_> = final_state.keys()
//             .filter(|k| k.starts_with(&format!("{}_goal_", name)) && k.contains("_predicate"))
//             .cloned()
//             .collect();

//         // We expect details only for the 2 *new* goals processed in this run
//         assert_eq!(goal_detail_keys.len(), 2, "Expected detail state for 2 new goals");


//         drop(command_sender);
//         let _ = scheduler_handle.await;
//         let _ = state_manager_handle.await;
//     }

//     // Add more tests:
//     // - Test with empty incoming_goals (should do nothing to schedule)
//     // - Test error handling if state variables are not Maps (though the code logs errors)
// }

// Add necessary stubs or imports for your crate types if not available globally
// Example stub structure (replace with your actual definitions)
/*
mod crate {
    use std::{collections::HashMap, str::FromStr, time::SystemTime};
    use tokio::sync::{mpsc, oneshot};

    #[derive(Debug, Clone, PartialEq)]
    pub enum SPValue {
        Map(MapOrUnknown),
        String(StringOrUnknown),
        Int64(IntOrUnknown),
        Time(TimeOrUnknown),
        Array(ArrayOrUnknown),
        Bool(BoolOrUnknown),
        // Add others like Float64, Duration etc. if used
    }

    #[derive(Debug, Clone, PartialEq)] pub enum MapOrUnknown { Map(Vec<(SPValue, SPValue)>), UNKNOWN }
    #[derive(Debug, Clone, PartialEq)] pub enum StringOrUnknown { Str(String), UNKNOWN }
    #[derive(Debug, Clone, PartialEq)] pub enum IntOrUnknown { Int64(i64), UNKNOWN }
    #[derive(Debug, Clone, PartialEq)] pub enum TimeOrUnknown { Time(SystemTime), UNKNOWN }
    #[derive(Debug, Clone, PartialEq)] pub enum ArrayOrUnknown { Array(Vec<SPValue>), UNKNOWN }
    #[derive(Debug, Clone, PartialEq)] pub enum BoolOrUnknown { Bool(bool), UNKNOWN }


    // Basic SPValue methods stubbed
    impl SPValue {
        pub fn to_string(&self) -> String {
             match self {
                SPValue::String(StringOrUnknown::Str(s)) => s.clone(),
                SPValue::Map(_) => "map_value".to_string(), // Placeholder
                _ => "some_value".to_string(), // Placeholder
            }
        }
    }

    // Basic String extension trait stub
    pub trait ToSPValue { fn to_spvalue(self) -> SPValue; }
    impl ToSPValue for String { fn to_spvalue(self) -> SPValue { SPValue::String(StringOrUnknown::Str(self)) } }
    impl ToSPValue for &str { fn to_spvalue(self) -> SPValue { SPValue::String(StringOrUnknown::Str(self.to_string())) } }


    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)] // Added Ord for sorting
    pub enum GoalPriority { High = 1, Normal = 2, Low = 3 }

    impl GoalPriority {
        pub fn to_int(&self) -> i64 { *self as i64 }
        pub fn from_int(v: &i64) -> Self { // Changed signature slightly
             match v { 1 => GoalPriority::High, 3 => GoalPriority::Low, _ => GoalPriority::Normal }
        }
         pub fn to_string(&self) -> String {
             match self {
                 GoalPriority::High => "High".to_string(),
                 GoalPriority::Normal => "Normal".to_string(),
                 GoalPriority::Low => "Low".to_string(),
             }
         }
    }
     impl FromStr for GoalPriority { // Needed for parsing from SPValue string
         type Err = (); // Simple error type for stub
         fn from_str(s: &str) -> Result<Self, Self::Err> {
             match s {
                 "High" => Ok(GoalPriority::High),
                 "Normal" => Ok(GoalPriority::Normal),
                 "Low" => Ok(GoalPriority::Low),
                 _ => Err(()), // Or default to Normal? Err is safer.
             }
         }
     }


    #[derive(Debug)]
    pub enum StateManagement {
        Get((String, oneshot::Sender<SPValue>)),
        Set((String, SPValue)),
        SetPartialState(State),
        // Add others if needed
    }

    #[derive(Debug, Clone, Default)] // Added Default
    pub struct State {
        // Stub implementation - needs real fields/methods used by add_goal_to_state
        assignments: Vec<(SPVariable, SPValue)>,
    }

    impl State {
        pub fn new() -> Self { State::default() }
        pub fn add(mut self, assignment: (SPVariable, SPValue)) -> Self {
            self.assignments.push(assignment);
            self
        }
        // Method used in mock state manager
        pub fn projection(&self) -> &Vec<(SPVariable, SPValue)> {
             &self.assignments
        }
    }

    #[derive(Debug, Clone, PartialEq, Hash, Eq)] // Added Hash, Eq for HashMap key
    pub struct SPVariable{ path: String } // Stub

    impl SPVariable {
        // Stub method used in mock state manager
        pub fn path(&self) -> &str { &self.path }
    }

    // Stubs for v!, assign! etc. (assuming they create SPVariable and assignments)
    #[macro_export]
    macro_rules! v { ($path:expr) => { SPVariable { path: $path.to_string() } }; }
    #[macro_export]
    macro_rules! tv { ($path:expr) => { SPVariable { path: $path.to_string() } }; } // Treat Time vars same way for stub
    #[macro_export]
    macro_rules! iv { ($path:expr) => { SPVariable { path: $path.to_string() } }; } // Treat Int vars same way for stub
     #[macro_export]
    macro_rules! mv { ($path:expr) => { SPVariable { path: $path.to_string() } }; } // Treat Map vars same way for stub
     #[macro_export]
    macro_rules! av { ($path:expr) => { SPVariable { path: $path.to_string() } }; } // Treat Array vars same way for stub

    #[macro_export]
    macro_rules! assign { ($var:expr, $val:expr) => { ($var, $val) }; }

}

// Ensure nanoid is a dependency in Cargo.toml
// [dev-dependencies]
// nanoid = "0.4" // Or your specific version

// Ensure log facade and an implementation (like env_logger or tracing-subscriber) are set up for logging capture
// Example setup (call this at the start of your test or globally)
fn setup_logger() {
    let _ = env_logger::builder().is_test(true).try_init();
}
*/