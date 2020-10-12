use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tokio::time::delay_for;
use tokio::time::Instant;
use std::io;
use super::*;

/// Per command kind variable, a sender is spawned that updates the
/// value of the variable.
pub async fn sender(
    kvp: Arc<Mutex<(String, Instant)>>,
    mut send: tokio::sync::mpsc::Sender<std::string::String>,
) -> io::Result<()> {
    loop {
        let s = kvp.lock().unwrap().0.clone();
        let des: EnumValue = serde_json::from_str(&s)?;
        delay_for(Duration::from_millis(100)).await;
        send.try_send(des.val.to_string()).unwrap_or_default();
    }
}