use std::sync::Mutex;
use std::sync::Arc;
use tokio::prelude::*;
use std::{thread, time};
use r2r::*;

pub async fn emmiter(publisher: Publisher<std_msgs::msg::String>,
    mut recv: tokio::sync::mpsc::Receiver<std::string::String>) -> io::Result<()> {

    loop {
        thread::sleep(time::Duration::from_millis(100));
        let to_pub = recv.recv().await.unwrap();
        let to_send = std_msgs::msg::String { data: to_pub.to_owned()};
        publisher.publish(&to_send).unwrap();
    }
}