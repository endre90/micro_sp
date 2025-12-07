use crate::{Operation, OperationState, SOPState};
use serde::{Deserialize, Serialize};
use termtree::Tree;

// I look at SOPS as function blocks with a rigid structure, sort of as a high level operation
// Maybe, just maybe, we can also have a "Planned" variant that should use a planner within a certain domain to get a sequence???
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum SOP {
    Operation(Box<Operation>),
    Sequence(Vec<SOP>),
    Parallel(Vec<SOP>),
    Alternative(Vec<SOP>),
    // Planned(Vec<SOP>), ?? Maybe
}

// // I look at SOPS as function blocks with a rigid structure, sort of as a high level operation
// // Maybe, just maybe, we can also have a "Planned" variant that should use a planner within a certain domain to get a sequence???
// #[derive(Debug, PartialEq, Clone, Eq, Hash, Serialize, Deserialize)]
// pub enum SOP {
//     Operation(String, Box<Operation>),
//     Sequence(String, Vec<SOP>),
//     Parallel(String, Vec<SOP>),
//     Alternative(String, Vec<SOP>),
// }

// #[derive(Debug, PartialEq, Clone, Eq, Hash, Serialize, Deserialize)]
// pub struct SOPStruct {
//     pub id: String,
//     pub sop: SOP,
// }

// impl SOP {
//     pub fn get_all_var_keys(&self) -> Vec<String> {
//         match self {
//             SOP::Operation(_, op) => op.get_all_var_keys(),
//             SOP::Sequence(_, sops) | SOP::Parallel(_, sops) | SOP::Alternative(_, sops) => {
//                 sops.iter().flat_map(|s| s.get_all_var_keys()).collect()
//             }
//         }
//     }
//     pub fn get_all_operation_names(&self) -> Vec<String> {
//         let mut operations: Vec<String> = vec![];
//         match self {
//             SOP::Operation(_, op) => operations.push(op.name.clone()),
//             SOP::Sequence(_, sops) | SOP::Parallel(_, sops) | SOP::Alternative(_, sops) => {
//                 sops.iter().for_each(|s| {
//                     s.get_all_operation_names();
//                 });
//             }
//         };
//         operations
//     }
// }

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
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
    
    pub fn get_all_operation_names(&self) -> Vec<String> {
        let mut operations: Vec<String> = vec![];
        match self {
            SOP::Operation(op) => operations.push(op.name.clone()),
            SOP::Sequence(sops) | SOP::Parallel(sops) | SOP::Alternative(sops) => {
                sops.iter().for_each(|s| {
                    s.get_all_operation_names();
                });
            }
        };
        operations
    }

   pub fn get_state(&self) -> SOPState {
        match self {
            SOP::Operation(op) => match op.state {
                OperationState::Initial => SOPState::Initial,
                OperationState::Disabled => SOPState::Executing,
                OperationState::Executing => SOPState::Executing,
                OperationState::Timedout => SOPState::Executing,
                OperationState::Failed => SOPState::Executing,
                OperationState::Bypassed => SOPState::Completed,
                OperationState::Completed => SOPState::Completed,
                OperationState::Fatal => SOPState::Fatal,
                OperationState::Cancelled => SOPState::Cancelled,
                OperationState::UNKNOWN => SOPState::UNKNOWN,
            },
            SOP::Sequence(sops) => {
                if sops.is_empty() {
                    return SOPState::Completed;
                }

                let states: Vec<SOPState> = sops.iter().map(|s| s.get_state()).collect();

                let any_fatal = states.iter().any(|s| *s == SOPState::Fatal);
                let any_cancelled = states.iter().any(|s| *s == SOPState::Cancelled);
                let all_initial = states.iter().all(|s| *s == SOPState::Initial);
                let all_completed = states.iter().all(|s| *s == SOPState::Completed);
                let any_not_initial = states.iter().any(|s| *s != SOPState::Initial);

                if any_fatal {
                    return SOPState::Fatal;
                }
                
                if any_cancelled {
                    return SOPState::Cancelled;
                }

                if all_initial {
                    return SOPState::Initial;
                }

                if all_completed {
                    return SOPState::Completed;
                }

                if !all_completed && any_not_initial && !any_fatal && !any_cancelled {
                    return SOPState::Executing;
                }

                SOPState::UNKNOWN
            },
            SOP::Parallel(_) => todo!(),
            SOP::Alternative(_) => todo!(),
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

// fn build_sop_tree(sop: &SOP) -> Tree<String> {
//     match sop {
//         // A leaf node in the tree
//         SOP::Operation(id, op) => {
//             let label = format!("Operation {id}: {}", op.name);
//             Tree::new(label)
//         }

//         // A branch node for sequential operations
//         SOP::Sequence(id, sops) => {
//             let mut tree = Tree::new(format!("Sequence {id}"));
//             for child_sop in sops {
//                 tree.push(build_sop_tree(child_sop));
//             }
//             tree
//         }

//         // A branch node for parallel operations
//         SOP::Parallel(id, sops) => {
//             let mut tree = Tree::new(format!("Parallel {id}"));
//             for child_sop in sops {
//                 tree.push(build_sop_tree(child_sop));
//             }
//             tree
//         }

//         // A branch node for alternative operations
//         SOP::Alternative(id, sops) => {
//             let mut tree = Tree::new(format!("Alternative {id}"));
//             for child_sop in sops {
//                 tree.push(build_sop_tree(child_sop));
//             }
//             tree
//         }
//     }
// }

fn build_sop_tree(sop: &SOP) -> Tree<String> {
    match sop {
        // A leaf node in the tree
        SOP::Operation(op) => {
            let label = format!("Operation: {}", op.name);
            Tree::new(label)
        }

        // A branch node for sequential operations
        SOP::Sequence(sops) => {
            let mut tree = Tree::new(format!("Sequence:"));
            for child_sop in sops {
                tree.push(build_sop_tree(child_sop));
            }
            tree
        }

        // A branch node for parallel operations
        SOP::Parallel(sops) => {
            let mut tree = Tree::new(format!("Parallel:"));
            for child_sop in sops {
                tree.push(build_sop_tree(child_sop));
            }
            tree
        }

        // A branch node for alternative operations
        SOP::Alternative(sops) => {
            let mut tree = Tree::new(format!("Alternative:"));
            for child_sop in sops {
                tree.push(build_sop_tree(child_sop));
            }
            tree
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*; // Import everything from the parent module
    // use nanoid::nanoid;

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
                    SOP::Operation(Box::new(Operation {
                        name: "CheckPressure".to_string(),
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
