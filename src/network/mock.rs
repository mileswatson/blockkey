use std::{error::Error, fmt::Debug};

use async_trait::async_trait;
use tokio::sync::broadcast::{channel, Receiver, Sender};

use crate::actor::{ActorEvent, Status};

use super::{Actor, Network, Node};

pub struct MockNetwork<M> {
    sender: Sender<MockMessage<M>>,
    nodes: usize,
}

impl<M: Clone> MockNetwork<M> {
    fn new() -> Self {
        let (sender, _) = channel(100);
        MockNetwork { sender, nodes: 0 }
    }
}

#[async_trait]
impl<M: 'static + Clone + Send + Debug> Network<M, MockNode<M>> for MockNetwork<M> {
    async fn create_node(&mut self) -> Result<MockNode<M>, Box<dyn Error>> {
        let node = MockNode {
            sender: self.sender.clone(),
            receiver: self.sender.subscribe(),
        };
        self.nodes += 1;
        Ok(node)
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
impl<M: Clone + Debug + Send> Node for MockNode<M> {
    async fn wait_for_connections(&mut self, num: u32) {
        self.sender.send(MockMessage::NewNode).unwrap();
        for _ in 0..num + 1 {
            loop {
                if let MockMessage::NewNode = self.receiver.recv().await.unwrap() {
                    break;
                }
            }
        }
    }
}

#[async_trait]
impl<M: 'static + Clone + Send + Debug> Actor<M> for MockNode<M> {
    async fn run(
        &mut self,
        mut input: Receiver<ActorEvent<M>>,
        output: Sender<ActorEvent<M>>,
    ) -> Status {
        loop {
            tokio::select! {
                event = input.recv() => {
                    match event.unwrap() {
                        ActorEvent::Send(block) => {
                            if self.sender.send(MockMessage::Message(block.clone())).is_err() {
                                output.send(ActorEvent::Stop).unwrap();
                                return Status::Failed
                            }
                        }
                        ActorEvent::Stop => return Status::Stopped,
                        _ => (),
                    }
                }
                receiving = self.receiver.recv() => {
                    match receiving {
                        Err(_) => {
                            output.send(ActorEvent::Stop).unwrap();
                            return Status::Failed
                        }
                        Ok(MockMessage::Message(block)) => {
                            output.send(ActorEvent::Receive(block)).unwrap();
                        },
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
