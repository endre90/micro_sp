use micro_sp_tools::*;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tokio::prelude::*;
use tokio::time::delay_for;
use tokio::time::Instant;

pub async fn sender(
    kvp: Arc<Mutex<(String, Instant)>>,
    mut send: tokio::sync::mpsc::Sender<std::string::String>,
) -> io::Result<()> {
    loop {
        let s = kvp.lock().unwrap().0.clone();
        let des: EnumVariableValue = serde_json::from_str(&s).unwrap();
        println!("SENDER {:?}", des);
        delay_for(Duration::from_millis(100)).await;
        // *kvp.lock().unwrap() = (
        //     serde_json::to_string(&EnumVariableValue::new(&des.var, &des.val)).unwrap(),
        //     Instant::now(),
        // );
        send.try_send(des.val.to_string()).unwrap_or_default();
    }
}

// pub async fn state_sender(
//     kvp: Arc<Mutex<String>>,
//     mut send: tokio::sync::mpsc::Sender<std::string::String>,
// ) -> io::Result<()> {
//     let s = kvp.lock().unwrap().clone();
//     let des: State = serde_json::from_str(&s).unwrap();
//     loop {
//         delay_for(Duration::from_millis(100)).await;
//         *kvp.lock().unwrap() = serde_json::to_string(&State::from_lists(&des.measured, &des.command, &des.estimated)).unwrap();
//         send.try_send(des.val.to_string()).unwrap_or_default();
//     }
// }
