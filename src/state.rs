use super::*;
use std::io;
use std::sync::{Arc, Mutex};
use tokio::time::{delay_for, Duration, Instant};

pub async fn state(
    measured_arc: Arc<Mutex<String>>,
    command_arc: Arc<Mutex<String>>,
    ros_receivers: Vec<(String, tokio::sync::mpsc::Receiver<String>)>,
    ros_senders: Vec<(String, tokio::sync::mpsc::Sender<String>)>,
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

    loop {
        let looping_now = Instant::now();

        let measured_vec = &measured_list
            .iter()
            .map(|x| {
                let des: EnumVariableValue = serde_json::from_str(&x.lock().unwrap().0).unwrap();
                let duration = match looping_now.checked_duration_since(x.lock().unwrap().1) {
                    Some(x) => x,
                    None => Duration::new(6, 0),
                };
                EnumVariableValue::timed(&des.var, &des.val, duration)
            })
            .collect::<Vec<EnumVariableValue>>();
        let measured_state = State::from(&measured_vec, &ControlKind::Measured);

        *measured_arc.lock().unwrap() = serde_json::to_string(&measured_state).unwrap();
        delay_for(Duration::from_millis(10)).await;

        let sink: State = serde_json::from_str(&command_arc.lock().unwrap()).unwrap();
        let _command_vec = &command_list
            .iter()
            .map(|x| {
                let des: EnumVariableValue = serde_json::from_str(&x.lock().unwrap().0).unwrap();
                let dummy = EnumVariableValue::new(&des.var, &des.val);
                let update: &EnumVariableValue =
                    sink.vec.iter().find(|x| x.var == des.var).unwrap_or(&dummy);
                *x.lock().unwrap() = (serde_json::to_string(&update).unwrap(), Instant::now());
                EnumVariableValue::timed(
                    &des.var,
                    &update.val,
                    looping_now.saturating_duration_since(x.lock().unwrap().1),
                )
            })
            .collect::<Vec<EnumVariableValue>>();

        delay_for(Duration::from_millis(10)).await;
    }
}
