use std::error::Error;

use async_trait::async_trait;
use tokio::sync::broadcast::{channel, Receiver, Sender};
use tokio::sync::mpsc;

use crate::actor::Status;

use super::{Actor, Network};

pub struct MockNetwork<M> {
    sender: Sender<M>,
}

impl<M: Clone> MockNetwork<M> {
    fn new() -> Self {
        let (sender, _) = channel(100);
        MockNetwork { sender }
    }
}

#[async_trait]
impl<M: 'static + Clone + Send> Network<MockNode<M>, M> for MockNetwork<M> {
    async fn create_node(&mut self) -> Result<MockNode<M>, Box<dyn Error>> {
        Ok(MockNode {
            sender: self.sender.clone(),
            receiver: self.sender.subscribe(),
        })
    }
}

pub struct MockNode<M> {
    sender: Sender<M>,
    receiver: Receiver<M>,
}

#[async_trait]
impl<M: 'static + Clone + Send> Actor<M> for MockNode<M> {
    async fn run(&mut self, mut input: mpsc::Receiver<M>, output: mpsc::Sender<M>) -> Status {
        loop {
            tokio::select! {
                sending = input.recv() => {
                    match sending {
                        None => return Status::Stopped,
                        Some(block) => if self.sender.send(block).is_err() {
                            return Status::Failed
                        }
                    }
                }
                receiving = self.receiver.recv() => {
                    match receiving {
                        Err(_) => return Status::Failed,
                        Ok(block) => if output.send(block).await.is_err() {
                            return Status::Stopped
                        }
                    }
                }
            }
        }
    }
}

use crate::network::test::test_network;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
pub async fn test_mock_network() {
    test_network(MockNetwork::new()).await
}
