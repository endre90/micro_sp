use super::*;
use std::io;
use std::sync::{Arc, Mutex};
use tokio::time::{interval, Duration};

pub async fn runner(
    prob: PlanningProblem,
    ros_receivers: Vec<(String, tokio::sync::mpsc::Receiver<String>)>,
    ros_senders: Vec<(String, tokio::sync::mpsc::Sender<String>)>,
    // state_publisher: (String, tokio::sync::mpsc::Sender<String>),
) -> io::Result<()> {
    
    let measured_arc = Arc::new(Mutex::new(serde_json::to_string(&State::new(&vec!(), &Kind::Measured)).unwrap()));
    let command_arc = Arc::new(Mutex::new(serde_json::to_string(&State::new(&vec!(), &Kind::Command)).unwrap()));
    let measured_arc_clone = measured_arc.clone();
    let command_arc_clone = command_arc.clone();
    tokio::task::spawn(async {
        let state = state::state(measured_arc, command_arc, ros_receivers, ros_senders);
        let _res = tokio::try_join!(state);
    });
    
    // let mut i: u32 = 1;
    // // let mut fresh = false;
    // let mut sink = State::new(&vec!(), &Kind::Command);
    // let mut table = PlanningResultStates {
    //     plan_found: false,
    //     plan_length: 0,
    //     trace: vec![],
    //     time_to_solve: Duration::new(0, 0),
    // };

    loop {

        // let measured_arc_clone_clone = measured_arc_clone.lock().unwrap().clone();
        // let current_measured_state: State = serde_json::from_str(&measured_arc_clone_clone).unwrap();

        // let fresh = match current_measured_state.vec.len() > 0 {
        //     true => current_measured_state
        //         .vec
        //         .iter()
        //         .all(|x| x.lifetime < Duration::from_millis(5000)),
        //     false => false
        // };

        // if fresh {
        //     if sink == State::new(&vec!(), &Kind::Command) {
        //         let fresh_prob = refresh_problem(&prob, &current_measured_state);
        //         let result = incremental(&fresh_prob);
        //         println!("planner called {:?} times", i);
        //         i = i + 1;
        //         pprint_result(&result);
        //         table = result_to_table(&prob, &result);
        //     }
        //     sink = get_sink(&table, &current_measured_state).command;
        // } else {
        //     sink = State::new(&vec!(), &Kind::Command);
        // }

        // *command_arc_clone.lock().unwrap() = serde_json::to_string(&sink).unwrap();

        let mut interval = interval(Duration::from_millis(100));
        interval.tick().await;
        interval.tick().await;
    }
}
