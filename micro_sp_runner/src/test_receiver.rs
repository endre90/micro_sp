use tokio::time::{Duration, Instant, delay_for};
use std::sync::Mutex;
// use tokio::sync::Mutex;
use serde_json::*;
use std::sync::Arc;
use tokio::prelude::*;
use micro_sp_tools::*;

pub async fn test_receiver(kvp: Arc<Mutex<(String, Instant)>>, mut recv: tokio::sync::mpsc::Receiver<std::string::String>) -> io::Result<()> {
    let s = kvp.lock().unwrap().0.clone();
    let deserialized: TestVariable = serde_json::from_str(&s).unwrap();
    loop {
        // *kvp.lock().unwrap() = (String::new(), Instant::now());
        let data = recv.recv().await.unwrap(); 
        println!("{:?}", data);
        *kvp.lock().unwrap() = (
            serde_json::to_string(&TestVariable::new(
                &deserialized.name, 
                &data, 
                &deserialized.r#type, 
                &deserialized.domain.iter().map(|x| x.as_str()).collect(), 
                Some(&deserialized.param), 
                &deserialized.kind)).unwrap(), 
            Instant::now()
        );
    }  
}