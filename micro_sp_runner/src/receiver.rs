use std::sync::Mutex;
use std::sync::Arc;
use tokio::prelude::*;
use micro_sp_tools::*;

pub async fn receiver(kvp: Arc<Mutex<KeyValuePair>>, mut recv: tokio::sync::mpsc::Receiver<std::string::String>) -> io::Result<()> {
    let key_value_pair = *kvp.lock().unwrap();
    loop {
        let data = recv.recv().await.unwrap();
        *kvp.lock().unwrap() = KeyValuePair::new(&key_value_pair.key.to_string(), &data.to_string());
    }  
}