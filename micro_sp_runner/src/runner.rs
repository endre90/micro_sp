use super::*;
use std::io;
use std::sync::{Arc, Mutex};
use tokio::time::{delay_for, Duration, Instant};

pub async fn runner(
    prob: PlanningProblem,
    ros_receivers: Vec<(String, tokio::sync::mpsc::Receiver<String>)>,
    ros_senders: Vec<(String, tokio::sync::mpsc::Sender<String>)>,
) -> io::Result<()> {
    let mut measured_list = vec![];
    for r in ros_receivers {
        let past_time = Instant::now().checked_sub(Duration::new(6, 0));
        let amkvp = Arc::new(Mutex::new((r.0.clone(), past_time.unwrap())));
        let amkvp1 = amkvp.clone();
        let amkvp2 = amkvp.clone();
        tokio::task::spawn(async {
            let receiver = receiver::receiver(amkvp2, r.1);
            let _res = tokio::try_join!(receiver);
        });
        measured_list.push(amkvp1);
    }

    let mut command_list = vec![];
    for r in ros_senders {
        let amkvp = Arc::new(Mutex::new(r.0.clone()));
        let amkvp1 = amkvp.clone();
        let amkvp2 = amkvp.clone();
        tokio::task::spawn(async {
            let sender = sender::sender(amkvp2, r.1);
            let _res = tokio::try_join!(sender);
        });
        command_list.push(amkvp1);
    }

    let _result = incremental(&prob);
    // println!("{:?}", result);
    // let mut table = result_to_states(&result);
    // println!("{:?}", table);

    loop {
        let measured_state_string = &measured_list
            .iter()
            .map(|x| x.lock().unwrap().clone())
            .collect::<Vec<(String, Instant)>>();

        let measured_state = &measured_state_string
            .iter()
            .map(|x| (serde_json::from_str(&x.0).unwrap(), x.1))
            .collect::<Vec<(EnumVariableValue, Instant)>>();

        let looping_now = Instant::now();

        for m in measured_state {
            println!("{:?}", looping_now.duration_since(m.1));
            println!("{:?}", m);
        }

        delay_for(Duration::from_millis(10)).await;
    }
}
