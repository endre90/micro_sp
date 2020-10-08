use micro_sp_tools::*;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::prelude::*;
use tokio::time::Instant;
use tokio::time::Duration;

pub async fn receiver(
    kvp: Arc<Mutex<(String, Instant)>>,
    mut recv: tokio::sync::mpsc::Receiver<std::string::String>,
) -> io::Result<()> {
    let s = kvp.lock().unwrap().clone();
    let des: EnumVariableValue = serde_json::from_str(&s.0).unwrap();
    loop {
        let looping_now = Instant::now();
        let duration = match looping_now.checked_duration_since(s.1){
            Some(x) => x,
            None => Duration::new(6, 0)
        };
        let data = recv.recv().await.unwrap_or_default();
        *kvp.lock().unwrap() = (
            serde_json::to_string(&EnumVariableValue::timed(&des.var, &data, duration)).unwrap(),
            Instant::now(),
        );
    }
}
