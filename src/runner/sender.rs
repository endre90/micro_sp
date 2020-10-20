use std::sync::Arc;
use std::sync::Mutex;
// use std::time::Duration;
use tokio::time::{delay_for, interval, Instant, Duration};
// use tokio::time::Instant;
use std::io;
use super::*;

/// Per command kind variable, a sender is spawned that updates the
/// value of the variable.
pub async fn sender(
    kvp: Arc<Mutex<(String, Instant)>>,
    mut send: tokio::sync::mpsc::Sender<std::string::String>,
    delay: Arc<Mutex<u64>>
) -> io::Result<()> {
    let past_time = Instant::now().checked_sub(Duration::new(6, 0));
    let des: EnumValue = serde_json::from_str(&kvp.lock().unwrap().0).unwrap();
    let dummy = EnumValue::new(&des.var, "dummy_value", None);
    let kvp_delayed = Arc::new(Mutex::new((serde_json::to_string(&dummy).unwrap(), past_time.unwrap())));
    let kvp_delayed_clone = kvp_delayed.clone();
    let kvp_clone = kvp.clone();
    tokio::task::spawn(async {
        let du = delayed_update(kvp_clone, kvp_delayed, delay);
        let _res = tokio::try_join!(du);
    });
    loop {
        let looping_now = Instant::now();
        let s = kvp_delayed_clone.lock().unwrap().0.clone();
        let des: EnumValue = serde_json::from_str(&s)?;
        send.try_send(des.val.to_string()).unwrap_or_default();
        let mut interval = interval(Duration::from_millis(100));
        interval.tick().await;
        interval.tick().await;
    }
}

async fn delayed_update(
    kvp: Arc<Mutex<(String, Instant)>>,
    kvp_delayed: Arc<Mutex<(String, Instant)>>,
    delay: Arc<Mutex<u64>>
) -> io::Result<()> {
    let d = delay.lock().unwrap().clone();
    loop {
        delay_for(Duration::from_millis(d)).await;
        *kvp_delayed.lock().unwrap() = kvp.lock().unwrap().clone();
    }
}

/// Send out the complete state as a json string.
pub async fn complete_state_sender(
    kvp: Arc<Mutex<(String, Instant)>>,
    mut send: tokio::sync::mpsc::Sender<std::string::String>,
) -> io::Result<()> {
    loop {
        let s = kvp.lock().unwrap().0.clone();
        delay_for(Duration::from_millis(100)).await;
        send.try_send(s).unwrap_or_default();
    }
}