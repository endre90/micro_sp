use std::sync::Mutex;
use std::sync::Arc;
use tokio::prelude::*;
use lib::{KeyValuePair};
use tokio::sync::mpsc::channel;
use r2r::*;
use std::io::Error;
use arrayvec::ArrayString;

pub async fn receiver(name: String, kvp: Arc<Mutex<KeyValuePair>>, mut recv: tokio::sync::mpsc::Receiver<std::string::String>) -> io::Result<()> {
    loop {
        let data = recv.recv().await.unwrap();
        *kvp.lock().unwrap() = KeyValuePair::new(name.as_str(), &data.to_string());
    }  
}