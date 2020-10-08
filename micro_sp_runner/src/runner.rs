use super::*;
use std::io;
use std::sync::{Arc, Mutex};
use tokio::time::{Instant, interval, Duration};

pub async fn runner(
    mut prob: PlanningProblem,
    ros_receivers: Vec<(String, tokio::sync::mpsc::Receiver<String>)>,
    ros_senders: Vec<(String, tokio::sync::mpsc::Sender<String>)>,
    // state_publisher: (String, tokio::sync::mpsc::Sender<String>),
) -> io::Result<()> {
    
    let measured_arc = Arc::new(Mutex::new(serde_json::to_string(&State::new(&ControlKind::Measured)).unwrap()));
    let measured_arc_clone = measured_arc.clone();
    tokio::task::spawn(async {
        let measured_state = mstate::mstate(measured_arc, ros_receivers);
        let _res = tokio::try_join!(measured_state);
    });

    let command_arc = Arc::new(Mutex::new(serde_json::to_string(&State::new(&ControlKind::Command)).unwrap()));
    let command_arc_clone = command_arc.clone();
    tokio::task::spawn(async {
        let command_state = cstate::cstate(command_arc, ros_senders);
        let _res = tokio::try_join!(command_state);
    });

    let mut i: u32 = 1;
    // let mut fresh = false;
    let mut sink = State::new(&ControlKind::Command);
    let mut table = PlanningResultStates {
        plan_found: false,
        plan_length: 0,
        trace: vec![],
        time_to_solve: Duration::new(0, 0),
    };

    loop {

        let measured_arc_clone_clone = measured_arc_clone.lock().unwrap().clone();
        let current_measured_state: State = serde_json::from_str(&measured_arc_clone_clone).unwrap();

        let fresh = match current_measured_state.vec.len() > 0 {
            true => current_measured_state
                .vec
                .iter()
                .all(|x| x.lifetime < Duration::from_millis(5000)),
            false => false
        };

        let looping_now = Instant::now();

        if fresh {
            if sink == State::new(&ControlKind::Command) {
                let fresh_prob = refresh_problem(&prob, &current_measured_state);
                let result = incremental(&fresh_prob);
                println!("planner called {:?} times", i);
                i = i + 1;
                pprint_result(&result);
                table = result_to_table(&prob, &result);
            }
            sink = get_sink(&table, &current_measured_state).command;
        } else {
            sink = State::new(&ControlKind::Command);
        }

        *command_arc_clone.lock().unwrap() = serde_json::to_string(&sink).unwrap();
        

        // println!("asdfjpoij");
        // println!("MEASURED {:?}", current_measured_state);
        // println!("COMMAND {:?}", command_vec);
        // println!("TABLE {:?}", table);     
        println!("FRESH {:?}", fresh);

        let mut interval = interval(Duration::from_millis(100));
        interval.tick().await;
        interval.tick().await;
    }
}
