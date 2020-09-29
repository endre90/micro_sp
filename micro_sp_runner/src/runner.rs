use tokio::prelude::*;
use std::{thread, time};
use std::sync::Mutex;
use std::sync::Arc;
use r2r::*;
use tokio::time::delay_for;
use std::time::Duration;
use std::io;
use micro_sp_tools::*;
use arrayvec::ArrayString;

use super::*;

pub async fn runner(prob: PlanningProblem, 
                    ros_receivers: Vec<(KeyValuePair, tokio::sync::mpsc::Receiver<String>)>,
                    ros_senders: Vec<(KeyValuePair, tokio::sync::mpsc::Sender<String>)>) -> io::Result<()> {

    let mut measured_list = vec!();
    for r in ros_receivers {
        let amkvp = Arc::new(Mutex::new(r.0));
        let amkvp1 = amkvp.clone();
        let amkvp2 = amkvp.clone();
        tokio::task::spawn(async{
            let receiver = receiver::receiver(amkvp2, r.1);
            let _res = tokio::try_join!(receiver);
        });
        measured_list.push(amkvp1);
    }

    let mut command_list = vec!();
    for r in ros_senders {
        let amkvp = Arc::new(Mutex::new(r.0));
        let amkvp1 = amkvp.clone();
        let amkvp2 = amkvp.clone();
        tokio::task::spawn(async{
            let sender = sender::sender(amkvp2, r.1);
            let _res = tokio::try_join!(sender);
        });
        command_list.push(amkvp1);
    }

    let mut result = incremental(&prob);
    println!("{:?}", result);
    // let mut table = result_to_states(&result);
    // println!("{:?}", table);

    loop {

        let measured_state = State::new(&measured_list.iter().map(|x| *x.lock().unwrap()).collect());

        // for t in &table {
        //     if t.0.pairs.contains(measured_state.pa) {

        //     }
        // }

        // for c in &command_list {
        //     let name = *c.lock().unwrap();
        //     for t in &table{
                
        //         for v in &t.0.pairs {
        //             if name.key.as_str() == v.key.as_str() {
        //                 *c.lock().unwrap() = KeyValuePair::new(name.key.as_str(), v.value.as_str());
        //             }
        //         }
        //     }
        // }

        // // delay_for(Duration::from_millis(1000)).await;
        // // thread::sleep(time::Duration::from_millis(1000));

        // for c in &command_list {
        //     // let mut v = *c.lock().unwrap();
        //     let name = *c.lock().unwrap();
        //     *c.lock().unwrap() = KeyValuePair::new(name.key.as_str(), "1234");
        // }

        // thread::sleep(time::Duration::from_millis(1000));
        // for c in &command_list {
        //     let mut v = *c.lock().unwrap();
        //     v.value = ArrayString::<[_; 32]>::from("1234").unwrap_or_default();
        //     // thread::sleep(time::Duration::from_millis(1));
        //     // v.value = ArrayString::<[_; 32]>::from("asdf").unwrap_or_default();
        // }
        
        println!("{:?}", measured_state);

        // for t in &table {
        //     if t.0.pairs.iter().filter(|x| x.kind == ControlKind::Measured).map(|x| x.clone()).collect() == current_state {

        //     }
        // }

        // if table.iter().any(|x| x.0 == msrd_state) {
        //     println!("USING OLD PLAN");
        //     for t in &table {
        //         if msrd_state == t.0 {
        //             for s in &ros_senders {
        //                 for v in &t.0.pairs {
        //                     if v.key.to_string() == s.0 {
        //                         s.1.clone().try_send(v.value.to_string()).unwrap_or_default();
        //                     }
        //                 }
        //             }
        //         }
        //     }
        // } else {
        //     println!("PLANNING");
        //     result = incremental(&prob);
        //     // println!("{:?}", result);
        //     table = result_to_states(&result);
        //     // println!("{:?}", table);
        // }
        

        //check stuff here? maybe move some stuff into the async task "planner"

        // compare complete or partial states?
        // have to handle estimated state also somehow??

        // for v in &cmd_vars {
        //     for s in &ros_senders {
        //         if v.name.key.to_string() == s.0 {
        //             s.1.clone().try_send(s.0.to_string()).unwrap_or_default();
        //         }
        //     }
        // }

        delay_for(Duration::from_millis(10)).await;
    }
}