use std::error::Error;

use async_trait::async_trait;
use tokio::sync::broadcast::{channel, Receiver, Sender};
use tokio::sync::mpsc;

use crate::actor::Status;

use super::{Actor, Network, Node};

pub struct MockNetwork<M> {
    sender: Sender<MockMessage<M>>,
}

impl<M: Clone> MockNetwork<M> {
    fn new() -> Self {
        let (sender, _) = channel(100);
        MockNetwork { sender }
    }
}

#[async_trait]
impl<M: 'static + Clone + Send + std::fmt::Debug> Network<MockNode<M>, M> for MockNetwork<M> {
    async fn create_node(&mut self) -> Result<MockNode<M>, Box<dyn Error>> {
        Ok(MockNode {
            sender: self.sender.clone(),
            receiver: self.sender.subscribe(),
        })
    }
}

#[derive(Clone, Debug)]
pub enum MockMessage<M> {
    NewNode,
    Message(M),
}

pub struct MockNode<M> {
    sender: Sender<MockMessage<M>>,
    receiver: Receiver<MockMessage<M>>,
}

#[async_trait]
impl<M> Node<M> for MockNode<M>
where
    M: 'static + Send + Clone + std::fmt::Debug,
{
    async fn wait_for_connections(&mut self, num: u32) {
        self.sender.send(MockMessage::NewNode).unwrap();
        for _ in 0..num {
            loop {
                if let MockMessage::NewNode = self.receiver.recv().await.unwrap() {
                    break;
                }
            }
        }
    }
}

#[async_trait]
impl<M: 'static + Clone + Send> Actor<M> for MockNode<M> {
    async fn run(&mut self, mut input: mpsc::Receiver<M>, output: mpsc::Sender<M>) -> Status {
        loop {
            tokio::select! {
                sending = input.recv() => {
                    match sending {
                        None => return Status::Stopped,
                        Some(block) => if self.sender.send(MockMessage::Message(block)).is_err() {
                            return Status::Failed
                        }
                    }
                }
                receiving = self.receiver.recv() => {
                    match receiving {
                        Err(_) => return Status::Failed,
                        Ok(MockMessage::Message(block)) => if output.send(block).await.is_err() {
                            return Status::Stopped
                        }
                        _ => (),
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
