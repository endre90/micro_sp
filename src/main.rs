use lib::*;
use std::io;
mod models;
mod runner;
use r2r::*;
use tokio::time::{Duration, delay_for, Instant, timeout_at};

#[tokio::main]
async fn main() -> io::Result<()> {
    let ha = handle_args();

    // "async" => async_incremental(&unparam(&ha.model), ha.timeout, ha.max_steps),

    // let result = timeout(Duration::from_secs(ha.timeout), async_incremental(&unparam(&ha.model))).await.ok();
    
    if let Err(_) = timeout_at(Instant::now() + Duration::from_millis(2000), async_incremental(&unparam(&ha.model))).await {
        println!("did not receive value within 10 ms");
    }

    // println!("{}", ha.timeout);
    // let mut result = String::from("initial");
    // if let Ok(async_res) = timeout_at(Instant::now() + Duration::from_secs(ha.timeout), async_incremental(&unparam(&ha.model))).await {
    //     result = async_res.name.clone();
    // } else {
    //     result = String::from("timeout");
    // }

    // println!("{}", result);

    // let result = match ha.alg.as_str() {
    //     "seq" => sequential(&unparam(&ha.model), ha.timeout, ha.max_steps),
    //     "inc" => incremental(&unparam(&ha.model), ha.timeout, ha.max_steps),
    //     "seqexp" => seqexponential(&unparam(&ha.model), ha.timeout, ha.max_steps),
    //     "incexp" => incexponential(&unparam(&ha.model), ha.timeout, ha.max_steps),
    //     "comp" => unimplemented!(),
    //     "seqsub" => subgoaling(&ha.model, "seq", ha.timeout, ha.max_steps),
    //     "incsub" => subgoaling(&ha.model, "inc", ha.timeout, ha.max_steps),
    //     "compsub" => unimplemented!(),
    //     _ => panic!("nonexistent algorithm"),
    // };

    // match result {
    //     Some(x) => {
    //         match ha.print {
    //             true => pprint_result(&x),
    //             false => pprint_result_trans_only(&x),
    //         }
    //     },
    //     None => panic!("future failed!")
    // }
    

    // match ha.filesave {
    //     true => pprint_result_to_file(&result),
    //     false => (),
    // }

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
