use serde::{Deserialize, Serialize};

use crate::Operation;

// I look a SOPS as function blocks with a rigid structure, sort of as a high level operation
// Maybe, just maybe, we can also have a "Planned" variant that should use a planner within a certain domain to get a sequence???
// #[derive(Debug, PartialEq, Clone, Eq, Hash, Serialize, Deserialize)]
// pub enum SOP {
//     Operation(Box<Operation>),
//     Sequence(Vec<SOP>),
//     // Parallel(Vec<SOP>),
//     // Alternative(Vec<SOP>),
//     // Planned(Vec<SOP>), ?? Maybe
// }

// #[derive(Debug, PartialEq, Clone, Eq, Hash, Serialize, Deserialize)]
// pub struct SOPStruct {
//     pub id: String,
//     pub sop: Vec<String>,
// }

#[derive(Debug, PartialEq, Clone, Eq, Hash, Serialize, Deserialize)]
pub struct SOPStruct {
    pub id: String,
    pub sop: Vec<Operation>,
}

// There is a way to extract all predicates and take actions in sp-rust,
// but for now I will try only to extend the operation with a optional SOP field
// If SOP, do this execute thing, otherwise, execute as a reqular operation
// EVENTUALLY: Should change everything to sop, i.e. wrap the operation to SOP
// Then I have solved the hierarchies => Automatic hierarchical planning and execution
// For now, have a SOP field in the operation

// Have to think about this, we can't execute sops like this
// impl SOP {
//     #[async_recursion]
//     pub async fn execute_sop(
//         &self,
//         sp_id: &str,
//         command_sender: mpsc::Sender<StateManagement>,
//     ) -> Result<OperationState, Box<dyn std::error::Error>> {
//         match self {
//             SOP::Operation(op) => {
//                 cycle_operation(sp_id, *op.clone(), command_sender).await
//             }
//             SOP::Sequence(seq) => {
//                 for sop_item in seq {
//                     match sop_item.execute_sop(sp_id, command_sender.clone()).await {
//                         Ok(state) => match state {
//                             OperationState::Completed => {}
//                             _ => return Ok(OperationState::Failed),
//                         },
//                         Err(e) => {
//                             return Err(e);
//                         }
//                     }
//                 }
//                 Ok(OperationState::Completed)
//             }
//         }
//     }
// }

// For now, convert a sop to a lan and execute with plan runner

// impl SOP {
//     pub fn convert_to_plan(&self) -> Vec<Operation> {
//         let mut plan = vec![];
//         match self {
//             SOP::Operation(op) => {
//                 plan.push(op);
//             }
//             SOP::Sequence(seq) => {
//                 for sop_item in seq {
//                     match sop_item.convert_to_plan() {
//                         Ok(state) => match state {
//                             OperationState::Completed => {}
//                             _ => return Ok(OperationState::Failed),
//                         },
//                         Err(e) => {
//                             return Err(e);
//                         }
//                     }
//                 }
//                 Ok(OperationState::Completed)
//             }
//         }
//     }
// }
