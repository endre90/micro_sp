use std::sync::Mutex;
use std::sync::Arc;
use tokio::prelude::*;
use tokio::sync::mpsc::channel;
use r2r::*;

pub async fn receiver(mut recv: tokio::sync::mpsc::Receiver<std::string::String>) -> io::Result<()> {
    loop {
        let data = recv.recv().await.unwrap();
        println!("{:?}", data);
    }
}