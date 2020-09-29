use tokio::time::{Duration, Instant, delay_for};
use std::sync::Mutex;
use std::sync::Arc;
use tokio::prelude::*;
use micro_sp_tools::*;

// #[tokio::main]
// async fn main() {
//     let now = Instant::now();
//     delay_for(Duration::new(1, 0)).await;
//     let new_now = Instant::now();
//     println!("{:?}", new_now.checked_duration_since(now));
//     println!("{:?}", now.checked_duration_since(new_now)); // None
// }

pub async fn receiver(kvp: Arc<Mutex<(KeyValuePair, Instant)>>, mut recv: tokio::sync::mpsc::Receiver<std::string::String>) -> io::Result<()> {
    let arc = *kvp.lock().unwrap();
    let key_value_pair = arc.0;
    loop {
        let data = recv.recv().await.unwrap();
        *kvp.lock().unwrap() = (KeyValuePair::new(&key_value_pair.key.to_string(), &data.to_string()), Instant::now());
    }  
}