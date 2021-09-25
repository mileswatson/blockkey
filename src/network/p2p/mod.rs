mod swarm;

use async_trait::async_trait;
use futures::StreamExt;
use libp2p::{gossipsub::IdentTopic as GossipTopic, swarm::SwarmEvent, Swarm};
use serde::{de::DeserializeOwned, Serialize};
use std::error::Error;
use tokio::sync::mpsc::{Receiver, Sender};

use swarm::InternalEvent;

use crate::actor::Status;

use super::{Actor, Network, Node};

pub struct P2PNetwork {
    topic: &'static str,
}

impl P2PNetwork {
    pub fn new(topic: &'static str) -> Self {
        P2PNetwork { topic }
    }
}

#[async_trait]
impl<M> Network<P2PNode, M> for P2PNetwork
where
    M: 'static + Send + Serialize + DeserializeOwned,
{
    async fn create_node(&mut self) -> Result<P2PNode, Box<dyn Error>> {
        Ok(P2PNode::new(self.topic).await?)
    }
}

pub struct P2PNode {
    swarm: Swarm<swarm::CustomBehaviour>,
    topic: GossipTopic,
}

impl P2PNode {
    pub async fn new(topic: &str) -> Result<P2PNode, Box<dyn Error>> {
        let mut swarm = swarm::construct().await?;

        let topic = GossipTopic::new(topic);

        swarm.behaviour_mut().gossipsub.subscribe(&topic).unwrap();

        // Listen on all interfaces and whatever port the OS assigns
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

        Ok(P2PNode { swarm, topic })
    }
}

#[async_trait]
impl<NetOutput, NetInput> Node<NetInput, NetOutput> for P2PNode
where
    NetInput: 'static + Send + Serialize,
    NetOutput: 'static + Send + DeserializeOwned,
{
    async fn wait_for_connections(&mut self, num: u32) {
        for _ in 0..num - 1 {
            loop {
                if let SwarmEvent::ConnectionEstablished { .. } = self.swarm.next().await.unwrap() {
                    println!("Connection established!");
                    break;
                }
            }
        }
    }
}

#[async_trait]
impl<NetOutput, NetInput> Actor<NetInput, NetOutput> for P2PNode
where
    NetOutput: 'static + DeserializeOwned + Send,
    NetInput: 'static + Serialize + Send,
{
    async fn run(&mut self, mut input: Receiver<NetInput>, output: Sender<NetOutput>) -> Status {
        loop {
            tokio::select! {
                block = input.recv() => {
                    match block {
                        None => {
                            return Status::Stopped
                        },
                        Some(block) => {
                            let bytes = match serde_json::to_vec(&block) {
                                Ok(bytes) => bytes,
                                Err(_) => return Status::Failed,
                            };
                            if self.swarm
                                .behaviour_mut()
                                .gossipsub
                                .publish(self.topic.clone(), bytes).is_err() { return Status::Failed }
                        }
                    }
                }
                event = self.swarm.next() => {
                    let event = event.unwrap();
                    use SwarmEvent::*;
                    match event {
                        NewListenAddr { address, .. } => println!("Listening on {}", address),
                        Behaviour(internal_event) => match internal_event {
                            InternalEvent::Received { message, .. } => {
                                match serde_json::from_slice(&message.data) {
                                    Ok(b) => if output.send(b).await.is_err() {
                                        return Status::Stopped
                                    },
                                    Err(e) => println!("Failed to deserialize! {:?}", e)
                                }
                            }
                            InternalEvent::Found(peers) => {
                                for peer in peers {
                                    self.swarm
                                        .behaviour_mut()
                                        .gossipsub
                                        .add_explicit_peer(&peer)
                                }
                            }
                            InternalEvent::Lost(peers) => {
                                for peer in peers {
                                    self.swarm
                                        .behaviour_mut()
                                        .gossipsub
                                        .remove_explicit_peer(&peer)
                                }
                            }
                            InternalEvent::Other => (),
                        },
                        _ => (),
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::P2PNetwork;
    use crate::network::test::test_network;

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    pub async fn test_mock_network() {
        test_network(P2PNetwork::new("blockkey")).await
    }
}
