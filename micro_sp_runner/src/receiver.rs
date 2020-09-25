use std::sync::Mutex;
use std::sync::Arc;
use tokio::prelude::*;
use super::{KeyValuePair};

pub async fn receiver(name: String, kvp: Arc<Mutex<KeyValuePair>>, mut recv: tokio::sync::mpsc::Receiver<std::string::String>) -> io::Result<()> {
    loop {
        let data = recv.recv().await.unwrap();
        *kvp.lock().unwrap() = KeyValuePair::new(name.as_str(), &data.to_string());
    }  
}