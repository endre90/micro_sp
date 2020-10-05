use super::*;
use std::io;
use std::sync::{Arc, Mutex};
use tokio::time::{delay_for, Duration};

pub async fn runner(
    mut prob: PlanningProblem,
    ros_receivers: Vec<(String, tokio::sync::mpsc::Receiver<String>)>,
    ros_senders: Vec<(String, tokio::sync::mpsc::Sender<String>)>,
    state_publisher: (String, tokio::sync::mpsc::Sender<String>)
) -> io::Result<()> {

    let arc = Arc::new(Mutex::new((serde_json::to_string(&State::new()).unwrap(), false)));
    let arc_clone = arc.clone();
    tokio::task::spawn(async {
        let state = state::state(arc, ros_receivers, ros_senders, state_publisher);
        let _res = tokio::try_join!(state);
    });

    let mut i: u32 = 1;
    let mut sink = State::new();
    let mut table = PlanningResultStates {
        plan_found: false,
        plan_length: 0,
        trace: vec!(),
        time_to_solve: Duration::new(0, 0)
    };
    
    loop {

        let arc = arc_clone.lock().unwrap().clone();
        let current_state = serde_json::from_str::<State>(&arc.0).unwrap();

        if arc.1 {
            if sink == State::new() {
                let fresh_prob = refresh_problem(&prob, &current_state);
                let result = incremental(&fresh_prob);
                println!("planner called {:?} times", i);
                i = i + 1;
                pprint_result(&result);
                table = result_to_table(&prob, &result);
            }
            sink = get_sink(&table, &current_state);
            *arc_clone.lock().unwrap() = (serde_json::to_string(&sink).unwrap(), arc.1)
        }

        println!("SINK {:?} :: {:?}", sink, arc.1);

        delay_for(Duration::from_millis(100)).await;
    }
}
