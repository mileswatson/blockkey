use std::error::Error;
use std::sync::mpsc::{channel, SendError};
use tokio::io::{self, AsyncBufReadExt};

use blockkey::network;

/// Set up the tokio runtime.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut net = network::create("chatter").await?;

    // Read lines from stdin
    let mut stdin = io::BufReader::new(io::stdin()).lines();

    let (sender, receiver) = channel::<network::Message>();

    let process = tokio::task::spawn_blocking(move || {
        while let Ok(message) = receiver.recv() {
            let data = String::from_utf8_lossy(&message.data);
            println!(
                "Received {:?} with id {} from {}",
                data, message.id, message.peer
            );
            let mut total: i64 = 0;
            for i in 0..100000000 {
                total = total.wrapping_add(i);
            }
        }
    });

    // Start reading stdin and publishing input.
    loop {
        let to_publish = {
            tokio::select! {
                event = net.next_event() => {
                    use network::NetworkEvent::*;
                    match event {
                        ListeningOn(addr) => println!("Listening on {}", addr),
                        Received(message) => {
                            match sender.send(message) {
                                Ok(_) => (),
                                Err(SendError(e)) => println!("Failed to send {:?}", e),
                            }
                        }
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
            if line == "exit" {
                drop(sender);
                break;
            }
            net.broadcast(line.as_bytes()).await.unwrap();
        }
    }

    process.await?;

    Ok(())
}
