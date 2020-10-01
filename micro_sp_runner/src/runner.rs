use super::*;
use std::io;
use std::sync::{Arc, Mutex};
use tokio::time::{delay_for, Duration, Instant};

pub async fn runner(
    prob: PlanningProblem,
    ros_receivers: Vec<(String, tokio::sync::mpsc::Receiver<String>)>,
    ros_senders: Vec<(String, tokio::sync::mpsc::Sender<String>)>,
) -> io::Result<()> {

    let measured_list = state::make_measured(ros_receivers);
    let command_list = state::make_command(ros_senders);
    
    let result = incremental(&prob);
    pprint_result(&result);  

    let table = result_to_table(&prob, &result);
    println!("{:#?}", table);

    loop {

        let measured_state_strings = &measured_list
            .iter()
            .map(|x| x.lock().unwrap().clone())
            .collect::<Vec<(String, Instant)>>();

        let measured_state = &measured_state_strings
            .iter()
            .map(|x| (serde_json::from_str(&x.0).unwrap(), x.1))
            .collect::<Vec<(EnumVariableValue, Instant)>>();

        let command_state_strings = &command_list
            .iter()
            .map(|x| x.lock().unwrap().clone())
            .collect::<Vec<(String, Instant)>>();

        let command_state = &command_state_strings
            .iter()
            .map(|x| (serde_json::from_str(&x.0).unwrap(), x.1))
            .collect::<Vec<(EnumVariableValue, Instant)>>();

        let looping_now = Instant::now();

        for m in measured_state {
            println!("message lifetime: {:?}", looping_now.duration_since(m.1));
            println!("{:?}", m);
        }
        println!("-------------------------");

        for c in command_state {
            println!("message lifetime: {:?}", looping_now.duration_since(c.1));
            println!("{:?}", c);
        }
        println!("-------------------------");

        delay_for(Duration::from_millis(10)).await;
    }
}
