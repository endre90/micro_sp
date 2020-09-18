use std::sync::Mutex;
use std::sync::Arc;
use tokio::prelude::*;
use r2r::*;

fn read_f64(slice: &[u8]) -> f64 {
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(slice);
    f64::from_be_bytes(bytes)
}


pub async fn ur_writer(program_running: Arc<Mutex<bool>>,
                       mut recv: tokio::sync::mpsc::Receiver<std::string::String>,
                       ur_address: String) -> io::Result<()> {
    loop {
        let data = recv.recv().await.unwrap();
        *program_running.lock().unwrap() = true;

        let mut stream = tokio::net::TcpStream::connect(&ur_address).await?;
        stream.write_all(data.as_bytes()).await?;
        stream.flush().await?;
    }
}
