use super::*;
use std::io;
use std::sync::{Arc, Mutex};
use tokio::time::{interval, Duration};

/// The runner gets the current measured state, calls the planner when
/// needed and sets the corresponding sink state from the plan as command.
pub async fn ticker(
    prob: PlanningProblem,
    ros_receivers: Vec<(String, tokio::sync::mpsc::Receiver<String>)>,
    // ros_handshakers: Vec<(String, tokio::sync::mpsc::Receiver<String>)>,
    ros_senders: Vec<(String, tokio::sync::mpsc::Sender<String>)>,
    state_sender: Option<(String, tokio::sync::mpsc::Sender<String>)>,
) -> io::Result<()> {
    let measured_arc = Arc::new(Mutex::new(
        serde_json::to_string(&State::new(&vec![], &Kind::Measured)).unwrap(),
    ));
    // let handshake_arc = Arc::new(Mutex::new(
    //     serde_json::to_string(&State::new(&vec![], &Kind::Handshake)).unwrap(),
    // ));
    let command_arc = Arc::new(Mutex::new((
        serde_json::to_string(&State::new(&vec![], &Kind::Command)).unwrap(),
        false,
    )));
    let measured_arc_clone = measured_arc.clone();
    // let handshake_arc_clone = handshake_arc.clone();
    let command_arc_clone = command_arc.clone();
    tokio::task::spawn(async {
        let state = state::state(
            measured_arc,
            // handshake_arc,
            command_arc,
            ros_receivers,
            // ros_handshakers,
            ros_senders,
            state_sender,
        );
        let _res = tokio::try_join!(state);
    });

    let mut sink = State::new(&vec![], &Kind::Command);
    let mut result = PlanningResult {
        plan_found: false,
        plan_length: 0,
        trace: vec![],
        time_to_solve: Duration::new(0, 0),
    };

    loop {
        let measured_arc_clone_clone = measured_arc_clone.lock().unwrap().clone();
        let command_arc_clone_clone = command_arc_clone.lock().unwrap().clone();
        // let handshake_arc_clone_clone = handshake_arc_clone.lock().unwrap().clone();
        let current_measured_state: State =
            serde_json::from_str(&measured_arc_clone_clone).unwrap();
        let current_command_state: State =
            serde_json::from_str(&command_arc_clone_clone.0).unwrap();
        // let current_handshake_state: State =
        //     serde_json::from_str(&handshake_arc_clone_clone).unwrap();

        let fresh_measurement = match current_measured_state.vec.len() > 0 {
            true => current_measured_state
                .vec
                .iter()
                .all(|x| x.lifetime < Duration::from_millis(5000)),
            false => false,
        };

        let not_dummy_measurement = match current_measured_state.vec.len() > 0 {
            true => current_measured_state
                .vec
                .iter()
                .all(|x| x.val != "dummy_value"),
            false => false,
        };

        // let fresh_handshake = match current_handshake_state.vec.len() > 0 {
        //     true => current_handshake_state
        //         .vec
        //         .iter()
        //         .all(|x| x.lifetime < Duration::from_millis(5000)),
        //     false => false,
        // };

        // let fresh = fresh_measurement && fresh_handshake;

        if fresh_measurement && not_dummy_measurement {
            if sink == State::new(&vec![], &Kind::Command) {
                result = incremental(&refresh_problem(
                    &prob,
                    &current_measured_state,
                    &current_command_state
                ));
                pprint_result(&result);
            }
            sink = get_sink(&result, &current_measured_state, &current_command_state).command; //, &current_handshake_state).command;
        } else {
            sink = State::new(&vec![], &Kind::Command);
        }

        println!("SINK {:?} :: {:?}", sink, fresh_measurement);

        *command_arc_clone.lock().unwrap() = (serde_json::to_string(&sink).unwrap(), fresh_measurement);

        let mut interval = interval(Duration::from_millis(100));
        interval.tick().await;
        interval.tick().await;
    }
}
