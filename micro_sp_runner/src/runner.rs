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

    // let command_state = cstate::cstate(ros_senders);
    // let command_arc = Arc::new(Mutex::new(serde_json::to_string(&State::new(&ControlKind::Command)).unwrap()));
    // let command_arc_clone = command_arc.clone();
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
    // tokio::task::spawn(async {
    //     let command_state = cstate::cstate(ros_senders);
    //     let _res = tokio::try_join!(command_state);
    // });

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

        let command_vec = &command_list
            .iter()
            .map(|x| {
                let des: EnumVariableValue = serde_json::from_str(&x.lock().unwrap().0).unwrap();
                let dummy = EnumVariableValue::new(&des.var, &des.val);
                let update = sink.vec.iter().find(|x| x.var == des.var).unwrap_or(&dummy);
                println!("UPDATE {:?}", update);
                *x.lock().unwrap() = (serde_json::to_string(&update).unwrap(), Instant::now());
                EnumVariableValue::timed(
                    &des.var,
                    &update.val,
                    looping_now.saturating_duration_since(x.lock().unwrap().1),
                )
            })
            .collect::<Vec<EnumVariableValue>>();

        // *arc.lock().unwrap() = serde_json::to_string(&command_vec).unwrap();

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
