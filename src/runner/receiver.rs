use super::*;
use std::io;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::time::Duration;
use tokio::time::Instant;

/// Per measured kind variable, a receiver is spawned that handles 
/// live incomming data from its corresponding ROS2 topic.
pub async fn receiver(
    kvp: Arc<Mutex<(String, Instant)>>,
    mut recv: tokio::sync::mpsc::Receiver<std::string::String>,
) -> io::Result<()> {
    let s = kvp.lock().unwrap().clone();
    let des: EnumValue = serde_json::from_str(&s.0).unwrap();
    loop {
        let looping_now = Instant::now();
        let duration = match looping_now.checked_duration_since(s.1) {
            Some(x) => x,
            None => Duration::new(6, 0),
        };
        let data = recv.recv().await.unwrap_or_default();
        match des.var.domain.contains(&data) {
            true => {
                *kvp.lock().unwrap() = (
                    serde_json::to_string(&EnumValue::new(&des.var, &data, Some(&duration)))
                        .unwrap_or_default(),
                    Instant::now(),
                )
            }
            false => (),
        }
    }
}
