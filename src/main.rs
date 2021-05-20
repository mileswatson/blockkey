use std::error::Error;

mod network;

struct Printer {}

impl network::Handler for Printer {
    fn receive(&mut self, message: network::Message) {
        println!(
            "Got message: {} with id: {} from peer: {:?}",
            String::from_utf8_lossy(&message.data),
            message.id,
            message.peer
        );
    }
}

/// Set up the tokio runtime.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    network::run("chatter", &mut Printer {}).await?;
    Ok(())
}
