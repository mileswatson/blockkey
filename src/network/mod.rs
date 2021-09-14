pub mod p2p;

use std::error::Error;

use async_trait::async_trait;
use tokio::sync::mpsc::{Receiver, Sender};

#[async_trait]
pub trait Network<B> {
    fn new() -> Self;
    async fn create_node(&mut self) -> Result<Box<dyn Node<B>>, Box<dyn Error>>;
}

#[async_trait(?Send)]
pub trait Node<B> {
    async fn run(&mut self, incoming: Sender<B>, mut outgoing: Receiver<B>) -> Result<(), ()>;
}
