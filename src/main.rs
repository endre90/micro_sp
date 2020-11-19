use lib::*;
use std::io;
mod models;
use std::sync::Arc;
use std::sync::Mutex;
mod runner;
use r2r::*;
use tokio::time::{Duration, Instant, timeout_at};
use tokio::runtime;
use tokio_timer::Timeout;
use futures::Future;
use std::process;
use tokio::prelude::*;

// use std::process;

// process::exit(0x0100);


#[tokio::main]
async fn main() -> io::Result<()> {
    let ha = handle_args();
    // let mut runtime = runtime::Runtime::new().expect("failed to start new Runtime");

    // let modclone = Arc::new(Mutex::new(unparam(&ha.model)));
    // let modclone_clone = modclone.clone();
    // let model_clone = ha.model.clone();
    // let timeout_clone = ha.timeout.clone();
    // let max_steps_clone = ha.max_steps.clone();
    // let _res = runtime.spawn_blocking(move ||{
    //     incremental(&unparam(&model_clone), timeout_clone, max_steps_clone)
    //     // async_incremental(modclone_clone)
    // });
    // // let handle = tokio::task::spawn(async {
    // //     let du = async_incremental(modclone_clone);
    // //     let _res = tokio::try_join!(du);
    // // });

    // let now = Instant::now();

    // loop {
    //     println!("elapsed {:?}", now.elapsed());
    //     // println!("timeout {:?}", Duration::from_secs(ha.timeout));
    //     if now.elapsed() > Duration::from_secs(ha.timeout) {
    //         break;
    //         // handle.join();
    //         // drop(handle);
    //         // break;
    //         // // assert!(handle.await.unwrap_err().is_cancelled());
    //     }
    //     // assert!(handle.await.unwrap_err().is_cancelled());
    // }
    // runtime.shutdown_timeout(Duration::from_millis(100));

    // // let unparam = unparam(&copmod);
    // tokio::task::spawn(async {
    //     let copmod = ha.model.clone();
    //     let unparam = unparam(&copmod);
    //     let du = async_incremental(&unparam);
    //     let _res = tokio::try_join!(du);
    // });

    // Wrap the future with a `Timeout` set to expire in 10 milliseconds.
    // let prob = async_incremental(&unparam(&ha.model));
    // let timeout = Timeout::new(async_incremental(unparam(&ha.model)), Duration::from_secs(ha.timeout));

    // if let Err(_) = timeout_at(Instant::now() + Duration::from_millis(2000), async_incremental(&unparam(&ha.model))).await {
    //     println!("did not receive value within 10 ms");
    // }

    // println!("{}", ha.timeout);
    // let mut result = String::from("initial");
    // if let Ok(async_res) = timeout_at(Instant::now() + Duration::from_secs(ha.timeout), async_incremental(&unparam(&ha.model))).await {
    //     result = async_res.name.clone();
    // } else {
    //     result = String::from("timeout");
    // }

    // println!("{}", result);

    let result = match ha.alg.as_str() {
        "seq" => sequential(&unparam(&ha.model), ha.timeout, ha.max_steps),
        "inc" => incremental(&unparam(&ha.model), ha.timeout, ha.max_steps),
        "seqexp" => seqexponential(&unparam(&ha.model), ha.timeout, ha.max_steps),
        "incexp" => incexponential(&unparam(&ha.model), ha.timeout, ha.max_steps),
        "comp" => compositional(&ha.model, ha.timeout, ha.max_steps),
        "seqsub" => subgoaling(&ha.model, "seq", ha.timeout, ha.max_steps),
        "incsub" => subgoaling(&ha.model, "inc", ha.timeout, ha.max_steps),
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
