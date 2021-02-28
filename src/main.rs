use lib::*;
use std::io;
mod models;
// use std::sync::Arc;
// use std::sync::Mutex;
mod runner;
// use r2r::*;
// use tokio::time::{Duration, Instant, timeout_at};
// use tokio::runtime;
// use tokio_timer::Timeout;
// use futures::Future;
// use std::process;
// use tokio::prelude::*;

// use std::process;

// process::exit(0x0100);


#[tokio::main]
async fn main() -> io::Result<()> {
    let ha = handle_args();

    let result = match ha.alg.as_str() {
        "seq" => sequential(&unparam(&ha.model), ha.logic.as_str(), ha.timeout, ha.max_steps),
        "inc" => incremental(&unparam(&ha.model), ha.logic.as_str(), ha.timeout, ha.max_steps),
        "sub" => subgoaling(&ha.model, ha.logic.as_str(), ha.timeout, ha.max_steps),
        "skpc3" => skipping_c3(&unparam(&ha.model), ha.logic.as_str(), ha.timeout, ha.max_steps),
        "skpc5" => skipping_c5(&unparam(&ha.model), ha.logic.as_str(), ha.timeout, ha.max_steps),
        "skpc10" => skipping_c10(&unparam(&ha.model), ha.logic.as_str(), ha.timeout, ha.max_steps),
        "skpq" => skipping_q(&unparam(&ha.model), ha.logic.as_str(), ha.timeout, ha.max_steps),
        "skpe" => skipping_e(&unparam(&ha.model), ha.logic.as_str(), ha.timeout, ha.max_steps),
        "seqskpc3" => seq_skipping_c3(&unparam(&ha.model), ha.logic.as_str(), ha.timeout, ha.max_steps),
        "seqskpc5" => seq_skipping_c5(&unparam(&ha.model), ha.logic.as_str(), ha.timeout, ha.max_steps),
        "seqskpc10" => seq_skipping_c10(&unparam(&ha.model), ha.logic.as_str(), ha.timeout, ha.max_steps),
        "seqskpq" => seq_skipping_q(&unparam(&ha.model), ha.logic.as_str(), ha.timeout, ha.max_steps),
        "seqskpe" => seq_skipping_e(&unparam(&ha.model), ha.logic.as_str(), ha.timeout, ha.max_steps),
        "sss1" => subgoaling_seq_skp1(&ha.model, ha.logic.as_str(), ha.timeout, ha.max_steps),
        "sss3" => subgoaling_seq_skp3(&ha.model, ha.logic.as_str(), ha.timeout, ha.max_steps),
        "sss5" => subgoaling_seq_skp5(&ha.model, ha.logic.as_str(), ha.timeout, ha.max_steps),
        "sss10" => subgoaling_seq_skp10(&ha.model, ha.logic.as_str(), ha.timeout, ha.max_steps),
        "sssq" => subgoaling_seq_skpq(&ha.model, ha.logic.as_str(), ha.timeout, ha.max_steps),
        "ssse" => subgoaling_seq_skpe(&ha.model, ha.logic.as_str(), ha.timeout, ha.max_steps),
        "comp" => unimplemented!(),
        "compsub" => unimplemented!(),
        _ => panic!("nonexistent algorithm"),
    };

    match ha.print {
        true => pprint_result(&result),
        false => pprint_result_trans_only(&result),
    }

    match ha.filesave {
        true => pprint_result_to_file(&result),
        false => (),
    }

    Ok(())
}

// #[tokio::main]
// async fn main() -> io::Result<()> {
//     let ha = handle_args();
//     match ha.run {
//         false => match ha.print {
//             true => {
//                 let result = incremental(&ha.problem);
//                 pprint_result(&result);
//             }
//             false => {
//                 let result = incremental(&ha.problem);
//                 pprint_result_trans_only(&result);
//             }
//         },
//         true => {
//             let ros_ctx = Context::create()
//                 .expect("Error 3357ef39-2674-46c8-9841-bd126e70e059: Creating ros context failed.");
//             let mut node = runner::node::node(&ros_ctx, &ha.problem).await;
//             match ha.dummy {
//                 true => {
//                     let mut dummy = runner::dummy::raar_dummy(&ros_ctx, &ha.problem).await;
//                     loop {
//                         node.spin_once(std::time::Duration::from_millis(10));
//                         dummy.spin_once(std::time::Duration::from_millis(10));
//                     }
//                 }
//                 false => loop {
//                     node.spin_once(std::time::Duration::from_millis(10));
//                 },
//             }
//         }
//     }

//     Ok(())
// }
