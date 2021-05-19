use std::error::Error;

mod swarm;

struct Printer {}

impl swarm::Handler for Printer {
    fn receive(&mut self, message: swarm::Message) {
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
    let mut swarm = swarm::construct("chatter").await?;
    swarm::run(&mut swarm, &mut Printer {}).await?;

    Ok(())
}
