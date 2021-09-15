pub mod mock;
pub mod p2p;

use std::error::Error;

use async_trait::async_trait;
use tokio::sync::mpsc::{channel, Receiver, Sender};

#[async_trait(?Send)]
pub trait Network<M> {
    fn new() -> Self;
    async fn create_node(&mut self) -> Result<Box<dyn Node<M>>, Box<dyn Error>>;
}

#[async_trait(?Send)]
pub trait Node<M> {
    async fn run(&mut self, incoming: Sender<M>, mut outgoing: Receiver<M>) -> Result<(), ()>;
}

async fn connect<M>(mut app: impl Node<M>, mut network: impl Node<M>) -> Result<(), ()> {
    let (s1, r1) = channel(10);
    let (s2, r2) = channel(10);

    let (app_res, network_res) = tokio::join!(app.run(s1, r2), network.run(s2, r1));

    if app_res.is_ok() && network_res.is_ok() {
        Ok(())
    } else {
        Err(())
    }
}
