use std::error::Error;
use tokio::io::{self, AsyncBufReadExt};

mod network;
use network::NetworkEvent;

/// Set up the tokio runtime.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut net = network::create("chatter").await?;

    // Read lines from stdin
    let mut stdin = io::BufReader::new(io::stdin()).lines();

    // Start reading stdin and publishing input.
    loop {
        let to_publish = {
            tokio::select! {
                event = net.next_event() => {
                    use NetworkEvent::*;
                    match event {
                        ListeningOn(addr) => println!("Listening on {}", addr),
                        Received(message) => println!("Received {:?} with id {} from {}", String::from_utf8_lossy(&message.data), message.id, message.peer),
                    }
                    None
                }
                line = stdin.next_line() => {
                    let line = line?.expect("stdin closed");
                    Some(line)
                }
            }
        };
        if let Some(line) = to_publish {
            net.broadcast(line.as_bytes()).await.unwrap();
        }
    }
}
