pub mod mock;
pub mod p2p;

use std::error::Error;

use async_trait::async_trait;
use tokio::sync::mpsc::{Receiver, Sender};

#[async_trait(?Send)]
pub trait Network<M> {
    fn new() -> Self;
    async fn create_node(&mut self) -> Result<Box<dyn Node<M>>, Box<dyn Error>>;
}

#[async_trait(?Send)]
pub trait Node<M> {
    async fn run(&mut self, incoming: Sender<M>, mut outgoing: Receiver<M>) -> Result<(), ()>;
}
