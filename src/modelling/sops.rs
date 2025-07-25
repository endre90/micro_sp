use crate::Operation;
use serde::{Deserialize, Serialize};
use termtree::Tree;

// I look at SOPS as function blocks with a rigid structure, sort of as a high level operation
// Maybe, just maybe, we can also have a "Planned" variant that should use a planner within a certain domain to get a sequence???
#[derive(Debug, PartialEq, Clone, Eq, Hash, Serialize, Deserialize)]
pub enum SOP {
    Operation(Box<Operation>),
    Sequence(Vec<SOP>),
    Parallel(Vec<SOP>),
    Alternative(Vec<SOP>),
    // Planned(Vec<SOP>), ?? Maybe
}

//New, experimental
#[derive(Debug, PartialEq, Clone, Eq, Hash, Serialize, Deserialize)]
pub struct SOPStruct {
    pub id: String,
    pub sop: SOP,
}

impl SOP {
    pub fn get_all_var_keys(&self) -> Vec<String> {
        match self {
            SOP::Operation(op) => op.get_all_var_keys(),
            SOP::Sequence(sops) | SOP::Parallel(sops) | SOP::Alternative(sops) => {
                sops.iter().flat_map(|s| s.get_all_var_keys()).collect()
            }
        }
    }
}

/// Creates a visual representation of a SOP tree and prints it to the console.
///
/// This is the main entry point for visualizing a SOP.
///
/// # Arguments
/// * `root_sop`: The root of the SOP structure you want to visualize.
/// * `title`: A title to print above the tree.
pub fn visualize_sop(root_sop: &SOP) {
    let tree = build_sop_tree(root_sop);
    for line in tree.to_string().lines() {
        // Print the indent string before printing the line from the tree
        println!("       {}", line);
    }
    // println!("{}", tree);
}

fn build_sop_tree(sop: &SOP) -> Tree<String> {
    match sop {
        // A leaf node in the tree
        SOP::Operation(op) => {
            let label = format!("Operation: {}", op.name);
            Tree::new(label)
        }

        // A branch node for sequential operations
        SOP::Sequence(sops) => {
            let mut tree = Tree::new("Sequence".to_string());
            for child_sop in sops {
                tree.push(build_sop_tree(child_sop));
            }
            tree
        }

        // A branch node for parallel operations
        SOP::Parallel(sops) => {
            let mut tree = Tree::new("Parallel".to_string());
            for child_sop in sops {
                tree.push(build_sop_tree(child_sop));
            }
            tree
        }

        // A branch node for alternative operations
        SOP::Alternative(sops) => {
            let mut tree = Tree::new("Alternative".to_string());
            for child_sop in sops {
                tree.push(build_sop_tree(child_sop));
            }
            tree
        }
    }
}

// // To exec the pseudo async
// // Inside your main async loop
// loop {
//     let (response_tx, response_rx) = oneshot::channel();
//     command_sender
//         .send(StateManagement::GetState(response_tx))
//         .await?;
//     let state = response_rx.await?;
//     let mut new_state = state.clone();

//     // Get the SOP runner state (Initial, Executing, etc.)
//     let mut sop_overall_state = SOPState::from_str(&state.get_string_or_default(
//         // ... your key logic
//     ));

//     match sop_overall_state {
//         SOPState::Executing => {
//             // Get the root SOP definition
//             let root_sop = &model
//                 .sops
//                 .iter()
//                 .find(|sop| sop.id == sop_id.to_string()) // assuming sop_id is fetched
//                 .unwrap()
//                 .to_owned();

//             // Fetch the current stack from the state
//             let stack_key = format!("{}_sop_stack", sp_id);
//             let stack_json = state.get_string(&stack_key).unwrap_or_else(|| "[]".to_string());

//             // *** CALL THE NEW TICK FUNCTION ***
//             let (updated_state, new_stack_json) = run_sop_tick(&state, stack_json, root_sop).await;
//             new_state = updated_state;

//             // Update the stack in the new_state object
//             new_state = new_state.update(&stack_key, new_stack_json.to_spvalue());

//             // If the new stack is empty, the SOP is finished
//             if new_stack_json == "[]" {
//                 sop_overall_state = SOPState::Completed;
//             }
//         }
//         // ... other SOPState match arms (Initial, Completed, Failed)
//     }

//     new_state = new_state.update(
//         &format!("{}_sop_state", sp_id),
//         sop_overall_state.to_string().to_spvalue()
//     );

//     // Commit all changes (operation states AND stack state)
//     let modified_state = state.get_diff_partial_state(&new_state);
//     if !modified_state.is_empty() {
//         command_sender
//             .send(StateManagement::SetPartialState(modified_state))
//             .await?;
//     }

//     interval.tick().await;
// }

#[cfg(test)]
mod tests {
    use super::*; // Import everything from the parent module

    #[test]
    fn test_visualize_sop() {
        // 1. Create a complex SOP structure for demonstration.
        let example_sop = SOP::Sequence(vec![
            SOP::Operation(Box::new(Operation {
                name: "StartGripper".to_string(),
                ..Default::default()
            })),
            SOP::Parallel(vec![
                SOP::Operation(Box::new(Operation {
                    name: "MoveToTarget".to_string(),
                    ..Default::default()
                })),
                SOP::Sequence(vec![
                    SOP::Operation(Box::new(Operation {
                        name: "RotateWrist".to_string(),
                        ..Default::default()
                    })),
                    SOP::Operation(Box::new(Operation {
                        name: "CheckPressure".to_string(),
                        ..Default::default()
                    })),
                ]),
            ]),
            SOP::Alternative(vec![
                SOP::Operation(Box::new(Operation {
                    name: "CloseGripperHard".to_string(),
                    ..Default::default()
                })),
                SOP::Operation(Box::new(Operation {
                    name: "CloseGripperSoft".to_string(),
                    ..Default::default()
                })),
            ]),
            SOP::Operation(Box::new(Operation {
                name: "RetractArm".to_string(),
                ..Default::default()
            })),
        ]);

        // 2. Call the visualization function.
        //    When you run `cargo test -- --nocapture`, this tree will be printed.
        visualize_sop(&example_sop);
    }
}
