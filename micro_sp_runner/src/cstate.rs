use super::*;
use micro_sp_tools::*;
use std::io;
use std::sync::{Arc, Mutex};
use tokio::time::{delay_for, Duration, Instant};

pub fn cstate(
    // arc: Arc<Mutex<String>>,
    ros_senders: Vec<(String, tokio::sync::mpsc::Sender<String>)>,
) -> Vec<Arc<Mutex<(String, Instant)>>> {
    let mut command_list = vec![];
    for r in ros_senders {
        let past_time = Instant::now().checked_sub(Duration::new(6, 0));
        let amkvp = Arc::new(Mutex::new((r.0.clone(), past_time.unwrap())));
        let amkvp1 = amkvp.clone();
        tokio::task::spawn(async {
            let sender = sender::sender(amkvp, r.1);
            let _res = tokio::try_join!(sender);
        });
        command_list.push(amkvp1);
    }
    command_list
}

//     loop {
//         let looping_now = Instant::now();
//         let command_vec = &command_list
//             .iter()
//             .map(|x| {
//                 let des: EnumVariableValue = serde_json::from_str(&x.lock().unwrap().0).unwrap();
//                 EnumVariableValue::timed(
//                     &des.var,
//                     &des.val,
//                     looping_now.saturating_duration_since(x.lock().unwrap().1),
//                 )
//             })
//             .collect::<Vec<EnumVariableValue>>();

//         *arc.lock().unwrap() = serde_json::to_string(&command_vec).unwrap();
//         delay_for(Duration::from_millis(10)).await;
//     }
// }
