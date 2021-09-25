pub mod mock;
pub mod p2p;

use std::error::Error;

use async_trait::async_trait;
use tokio::sync::mpsc::{channel, Receiver, Sender};

#[async_trait]
pub trait Network<M, N> {
    fn new() -> Self;
    async fn create_node(&mut self) -> Result<N, Box<dyn Error>>;
}

#[async_trait]
pub trait Node<M> {
    /// Returns true if successful exit, otherwise false.
    async fn run(&mut self, incoming: Sender<M>, mut outgoing: Receiver<M>) -> Status;
}

#[derive(PartialEq, Eq)]
pub enum Status {
    Completed,
    Stopped,
    Failed,
}

pub async fn connect<M>(mut app: impl Node<M>, mut network: impl Node<M>) -> Status {
    let (s1, r1) = channel(10);
    let (s2, r2) = channel(10);

    let results = tokio::join!(app.run(s1, r2), network.run(s2, r1));

    use Status::*;

    match results {
        (Completed, Completed) | (Completed, Stopped) | (Stopped, Completed) => Status::Completed,
        (Stopped, Stopped) => Stopped,
        (Failed, _) | (_, Failed) => Failed,
    }
}
