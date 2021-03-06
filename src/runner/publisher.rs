use r2r::*;
use std::time::Duration;
use tokio::prelude::*;
use tokio::time::delay_for;

/// Per command kind variable, a publisher is spawned that pushes the current 
/// value of that variable to a ROS2 topic. 
pub async fn publisher(
    publisher: Publisher<std_msgs::msg::String>,
    mut recv: tokio::sync::mpsc::Receiver<std::string::String>,
) -> io::Result<()> {
    loop {
        delay_for(Duration::from_millis(100)).await;
        let to_pub = recv.recv().await.unwrap_or_default();
        let to_send = std_msgs::msg::String {
            data: to_pub.to_owned(),
        };
        publisher.publish(&to_send).unwrap_or_default();
    }
}
