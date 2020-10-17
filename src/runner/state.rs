use super::*;
use std::io;
use std::sync::{Arc, Mutex};
use tokio::time::{delay_for, interval, Duration, Instant};

/// Collects the measured values to a current measured state and
/// decomposes the current command state to be sent to corresponding
/// publishers.
pub async fn state(
    measured_arc: Arc<Mutex<String>>,
    command_arc: Arc<Mutex<(String, bool)>>,
    ros_receivers: Vec<(String, tokio::sync::mpsc::Receiver<String>)>,
    ros_senders: Vec<(String, tokio::sync::mpsc::Sender<String>)>,
    state_sender: Option<(String, tokio::sync::mpsc::Sender<String>)>,
) -> io::Result<()> {
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

    let state_amkvp = Arc::new(Mutex::new((
        serde_json::to_string(&CompleteState::empty())?,
        Instant::now(),
    )));
    let state_amkvp_clone = state_amkvp.clone();
    match state_sender {
        Some(x) => {    
            tokio::task::spawn(async {
                let sender = sender::complete_state_sender(state_amkvp, x.1);
                let _res = tokio::try_join!(sender);
            });
        },
        None => ()
    }
    

    loop {
        let looping_now = Instant::now();

        let measured_vec = &measured_list
            .iter()
            .map(|x| {
                let des: EnumValue = serde_json::from_str(&x.lock().unwrap().0).unwrap();
                let duration = match looping_now.checked_duration_since(x.lock().unwrap().1) {
                    Some(x) => x,
                    None => Duration::new(6, 0),
                };
                EnumValue::new(&des.var, &des.val, Some(&duration))
            })
            .collect::<Vec<EnumValue>>();
        let measured_state = State::new(&measured_vec, &Kind::Measured);
        // let measured_state = CompleteState::from_vec(&measured_vec);

        *measured_arc.lock().unwrap() = serde_json::to_string(&measured_state)?;
        delay_for(Duration::from_millis(10)).await;

        let sink: State = serde_json::from_str(&command_arc.lock().unwrap().0)?;
        let fresh: bool = command_arc.lock().unwrap().1;

        *state_amkvp_clone.lock().unwrap() = (
            serde_json::to_string(&CompleteState::from_states(
                // &measured_state.measured,
                // &sink,
                // &measured_state.estimated,
                &measured_state,
                &sink,
                &State::new(&vec![], &Kind::Estimated),
            ))?,
            Instant::now(),
        ); 

        let _command_vec = &command_list
            .iter()
            .map(|x| {
                let des: EnumValue = serde_json::from_str(&x.lock().unwrap().0).unwrap();
                let dummy = EnumValue::new(&des.var, "dummy_value", None);
                let update: &EnumValue =
                    sink.vec.iter().find(|x| x.var == des.var).unwrap_or(&dummy);
                match fresh {
                    true => {
                        *x.lock().unwrap() =
                            (serde_json::to_string(&update).unwrap(), Instant::now())
                    }
                    false => {
                        *x.lock().unwrap() =
                            (serde_json::to_string(&dummy).unwrap(), Instant::now())
                    }
                }
                EnumValue::new(
                    &des.var,
                    &update.val,
                    Some(&looping_now.saturating_duration_since(x.lock().unwrap().1)),
                )
            })
            .collect::<Vec<EnumValue>>();

        let mut interval = interval(Duration::from_millis(100));
        interval.tick().await;
        interval.tick().await;
    }
}
