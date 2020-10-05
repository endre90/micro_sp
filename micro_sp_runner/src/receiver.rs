use micro_sp_tools::*;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::prelude::*;
use tokio::time::Instant;

pub async fn receiver(
    kvp: Arc<Mutex<(String, Instant)>>,
    mut recv: tokio::sync::mpsc::Receiver<std::string::String>,
) -> io::Result<()> {
    let s = kvp.lock().unwrap().0.clone();
    let des: EnumVariableValue = serde_json::from_str(&s).unwrap();
    loop {
        let data = recv.recv().await.unwrap_or_default();
        *kvp.lock().unwrap() = (
            serde_json::to_string(&EnumVariableValue::new(&des.var, &data)).unwrap(),
            Instant::now(),
        );
    }
}
