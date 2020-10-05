use super::*;
use std::io;
use std::sync::{Arc, Mutex};
use tokio::time::{delay_for, Duration};

pub async fn runner(
    mut prob: PlanningProblem,
    ros_receivers: Vec<(String, tokio::sync::mpsc::Receiver<String>)>,
    ros_senders: Vec<(String, tokio::sync::mpsc::Sender<String>)>,
) -> io::Result<()> {

    let arc = Arc::new(Mutex::new((serde_json::to_string(&State::new()).unwrap(), false)));
    let arc_clone = arc.clone();
    tokio::task::spawn(async {
        let state = state::state(arc, ros_receivers, ros_senders);
        let _res = tokio::try_join!(state);
    });
    
    let result = incremental(&prob);
    pprint_result(&result);  

    let table = result_to_table(&prob, &result);
    // println!("{:#?}", table);

    loop {

        let arc = arc_clone.lock().unwrap().clone();
        let state = serde_json::from_str::<State>(&arc.0).unwrap();
        let sink = get_sink(&table, &state);

        println!("{:?} :: {:?}", sink, arc.1);

        delay_for(Duration::from_millis(100)).await;
    }
}
