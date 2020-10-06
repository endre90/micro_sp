use super::*;
use micro_sp_tools::*;
use std::io;
use std::sync::{Arc, Mutex};
use tokio::time::{delay_for, Duration, Instant};

pub async fn mstate(
    arc: Arc<Mutex<String>>,
    ros_receivers: Vec<(String, tokio::sync::mpsc::Receiver<String>)>,
) -> io::Result<()> {

    let mut measured_list = vec![];
    for r in ros_receivers {
        let past_time = Instant::now().checked_sub(Duration::new(6, 0));
        let amkvp = Arc::new(Mutex::new((r.0.clone(), past_time.unwrap())));
        let amkvp1 = amkvp.clone();
        tokio::task::spawn(async {
            let receiver = receiver::receiver(amkvp, r.1);
            let _res = tokio::try_join!(receiver);
        });
        measured_list.push(amkvp1);
    }

    loop {
        let looping_now = Instant::now();
        let measured_vec = &measured_list
            .iter()
            .map(|x| {
                let des: EnumVariableValue = serde_json::from_str(&x.lock().unwrap().0).unwrap();
                let duration = match looping_now.checked_duration_since(x.lock().unwrap().1){
                    Some(x) => x,
                    None => Duration::new(6, 0)
                };
                EnumVariableValue::timed(
                    &des.var,
                    &des.val,
                    duration,
                )
            })
            .collect::<Vec<EnumVariableValue>>();
        let measured_state = State::from(&measured_vec, &ControlKind::Measured);

        *arc.lock().unwrap() = serde_json::to_string(&measured_state).unwrap();
        delay_for(Duration::from_millis(10)).await;
    }
}

// pub async fn measured_state(
//     arc: Arc<Mutex<(String, bool)>>,
//     ros_receivers: Vec<(String, tokio::sync::mpsc::Receiver<String>)>
// )

// pub fn make_measured(
//     ros_receivers: Vec<(String, tokio::sync::mpsc::Receiver<String>)>,
// ) -> Vec<Arc<Mutex<(String, Instant)>>> {
//     let mut measured_list = vec![];
//     for r in ros_receivers {
//         let past_time = Instant::now().checked_sub(Duration::new(6, 0));
//         let amkvp = Arc::new(Mutex::new((r.0.clone(), past_time.unwrap())));
//         let amkvp1 = amkvp.clone();
//         tokio::task::spawn(async {
//             let receiver = receiver::receiver(amkvp, r.1);
//             let _res = tokio::try_join!(receiver);
//         });
//         measured_list.push(amkvp1);
//     }
//     measured_list
// }

// pub fn make_command(
//     ros_senders: Vec<(String, tokio::sync::mpsc::Sender<String>)>,
// ) -> Vec<Arc<Mutex<(String, Instant)>>> {
//     let mut command_list = vec![];
//     for r in ros_senders {
//         let past_time = Instant::now().checked_sub(Duration::new(6, 0));
//         let amkvp = Arc::new(Mutex::new((r.0.clone(), past_time.unwrap())));
//         let amkvp1 = amkvp.clone();
//         tokio::task::spawn(async {
//             let sender = sender::sender(amkvp, r.1);
//             let _res = tokio::try_join!(sender);
//         });
//         command_list.push(amkvp1);
//     }
//     command_list
// }

// pub async fn state(
//     arc: Arc<Mutex<String>>,
//     ros_receivers: Vec<(String, tokio::sync::mpsc::Receiver<String>)>,
//     ros_senders: Vec<(String, tokio::sync::mpsc::Sender<String>)>,
//     state_publisher: (String, tokio::sync::mpsc::Sender<String>),
// ) -> io::Result<()> {
//     let measured_list = state::make_measured(ros_receivers);
//     let command_list = state::make_command(ros_senders);

//     loop {
//         let looping_now = Instant::now();
//         let measured_vec = &measured_list
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

//         let command_vec = &command_list
//             .iter()
//             .map(|x| serde_json::from_str(&x.lock().unwrap().0).unwrap())
//             .collect::<Vec<EnumVariableValue>>();

//         *arc.lock().unwrap() =
//             serde_json::to_string(&State::from_lists(measured_vec, command_vec, &vec![])).unwrap();

//         delay_for(Duration::from_millis(10)).await;
//     }
// }
