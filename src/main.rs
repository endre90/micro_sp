use lib::*;
use std::io;
mod models;
mod runner;
use r2r::*;

#[tokio::main]
async fn main() -> io::Result<()> {

    let timeout = 1200;
    let max_steps = 50;

    let ha = handle_args();
    match ha.comp {
        true => {
            let result = compositional(&ha.problem, timeout, max_steps);
            match ha.print {
                true => pprint_result(&result),
                false => pprint_result_trans_only(&result)
            }
        },
        false => {
            let result = parameterized(&activate_all_in_problem(&ha.problem), timeout, max_steps);
            match ha.print {
                true => pprint_result(&result),
                false => pprint_result_trans_only(&result)
            }
        }
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
