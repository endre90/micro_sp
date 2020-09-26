use tokio::prelude::*;
use std::{thread, time};
use std::sync::Mutex;
use std::sync::Arc;
use r2r::*;
use tokio::time::delay_for;
use std::time::Duration;
use std::io;
use micro_sp_tools::*;
use super::*;

pub async fn runner(prob: PlanningProblem, 
                    ros_receivers: Vec<(String, KeyValuePair, tokio::sync::mpsc::Receiver<String>)>,
                    ros_senders: Vec<(String, tokio::sync::mpsc::Sender<String>)>) -> io::Result<()> {
    
    let vars = GetProblemVars::new(&prob);
    let msr_vars: Vec<EnumVariable> = vars.iter().filter(|x| x.kind == ControlKind::Measured).map(|x| x.clone()).collect();
    let cmd_vars: Vec<EnumVariable> = vars.iter().filter(|x| x.kind == ControlKind::Command).map(|x| x.clone()).collect();

    let measured_values = msr_vars.iter().map(|x| KeyValuePair::new(x.name.key.as_str(), "dummy_value")).collect();
    let measured_state = State::new(&measured_values);

    let mut measured_list = vec!();
    for r in ros_receivers {
        let kvp = measured_values.iter().find(|x| x.key == r.1.key).unwrap();
        let amkvp = Arc::new(Mutex::new(*kvp));
        let amkvp1 = amkvp.clone();
        let amkvp2 = amkvp.clone();
        tokio::task::spawn(async{
            let receiver = receiver::receiver(r.0, amkvp2, r.2);
            let _res = tokio::try_join!(receiver);
        });
        measured_list.push(amkvp1);
    }

    loop {

        for t in &measured_list {
            println!("{:?}", *t.lock().unwrap());
        }

        for v in &cmd_vars {
            for s in &ros_senders {
                if v.name.key.to_string() == s.0 {
                    s.1.clone().try_send(s.0.to_string()).unwrap_or_default();
                }
            }
        }

        delay_for(Duration::from_millis(10)).await;
    }
}