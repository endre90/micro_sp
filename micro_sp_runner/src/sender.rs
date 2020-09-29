use std::sync::Mutex;
use std::sync::Arc;
use tokio::prelude::*;
use micro_sp_tools::*;
use tokio::time::delay_for;
use std::time::Duration;

pub async fn sender(kvp: Arc<Mutex<KeyValuePair>>, mut send: tokio::sync::mpsc::Sender<std::string::String>) -> io::Result<()> {
    let key_value_pair = *kvp.lock().unwrap();
    loop {
        delay_for(Duration::from_millis(100)).await;
        send.try_send(key_value_pair.value.to_string()).unwrap_or_default();
    }  
}