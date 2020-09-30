use micro_sp_tools::*;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tokio::prelude::*;
use tokio::time::delay_for;

pub async fn sender(
    kvp: Arc<Mutex<String>>,
    mut send: tokio::sync::mpsc::Sender<std::string::String>,
) -> io::Result<()> {
    let s = kvp.lock().unwrap().clone();
    let des: EnumVariableValue = serde_json::from_str(&s).unwrap();
    loop {
        delay_for(Duration::from_millis(100)).await;
        send.try_send(des.val.to_string()).unwrap_or_default();
    }
}
