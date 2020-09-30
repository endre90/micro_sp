use tokio::prelude::*;
use std::{thread, time};
// use tokio::sync::Mutex;
use std::sync::Mutex;
use std::sync::Arc;
use r2r::*;
use tokio::time::{Duration, Instant, delay_for};
use std::io;
use micro_sp_tools::*;

use super::*;

pub async fn runner(prob: PlanningProblem, 
                    ros_receivers: Vec<(KeyValuePair, tokio::sync::mpsc::Receiver<String>)>,
                    ros_senders: Vec<(KeyValuePair, tokio::sync::mpsc::Sender<String>)>,
                    test_ros_receivers: Vec<(String, tokio::sync::mpsc::Receiver<String>)>) -> io::Result<()> {

    // let mut measured_list = vec!();
    // for r in ros_receivers {
    //     let past_time = Instant::now().checked_sub(Duration::new(6, 0));
    //     let amkvp = Arc::new(Mutex::new((r.0, past_time.unwrap())));
    //     let amkvp1 = amkvp.clone();
    //     let amkvp2 = amkvp.clone();
    //     tokio::task::spawn(async{
    //         let receiver = receiver::receiver(amkvp2, r.1);
    //         let _res = tokio::try_join!(receiver);
    //     });
    //     measured_list.push(amkvp1);
    // }

    let mut test_measured_list = vec!();
    for r in test_ros_receivers {
        let past_time = Instant::now().checked_sub(Duration::new(6, 0));
        let amkvp: Arc<Mutex<(String, Instant)>> = Arc::new(Mutex::new((r.0.to_string(), past_time.unwrap())));
        let amkvp1 = amkvp.clone();
        let amkvp2 = amkvp.clone();
        tokio::task::spawn(async{
            let test_receiver = test_receiver::test_receiver(amkvp2, r.1);
            let _res = tokio::try_join!(test_receiver);
        });
        test_measured_list.push(amkvp1);
    }

    // let mut command_list = vec!();
    // for r in ros_senders {
    //     let amkvp = Arc::new(Mutex::new(r.0));
    //     let amkvp1 = amkvp.clone();
    //     let amkvp2 = amkvp.clone();
    //     tokio::task::spawn(async{
    //         let sender = sender::sender(amkvp2, r.1);
    //         let _res = tokio::try_join!(sender);
    //     });
    //     command_list.push(amkvp1);
    // }

    let mut result = incremental(&prob);
    // println!("{:?}", result);
    // let mut table = result_to_states(&result);
    // println!("{:?}", table);

    loop {

        let measured_state_string = &test_measured_list.iter().map(|x| x.lock().unwrap().clone()).collect::<Vec<(String, Instant)>>();
        let measured_state = &measured_state_string.iter().map(|x| (serde_json::from_str(&x.0).unwrap(), x.1)).collect::<Vec<(TestVariable, Instant)>>();
        
        let looping_now = Instant::now();

        println!("{:?}", looping_now.duration_since(measured_state[0].1));
        
        println!("{:?}", measured_state);

        delay_for(Duration::from_millis(10)).await;
    }
}