use super::*;
use std::io;
use std::sync::{Arc, Mutex};
use tokio::time::{delay_for, Duration, Instant};

pub fn make_measured(ros_receivers: Vec<(String, tokio::sync::mpsc::Receiver<String>)>) -> Vec<Arc<Mutex<(String, Instant)>>> {
    
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
    measured_list
}

pub fn make_command(ros_senders: Vec<(String, tokio::sync::mpsc::Sender<String>)>) -> Vec<Arc<Mutex<(String, Instant)>>> {
    
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
