mod swarm;

use async_trait::async_trait;
use futures::StreamExt;
use libp2p::{gossipsub::IdentTopic as GossipTopic, swarm::SwarmEvent, Multiaddr, Swarm};
use serde::{de::DeserializeOwned, Serialize};
use std::{convert::TryInto, error::Error, time::Duration};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    time::timeout,
};

use swarm::InternalEvent;

use crate::actor::Status;

use super::{Actor, Network, Node};

pub struct P2PNetwork {
    topic: &'static str,
    nodes: Vec<Multiaddr>,
}

impl P2PNetwork {
    pub fn new(topic: &'static str) -> Self {
        P2PNetwork {
            topic,
            nodes: Vec::new(),
        }
    }
}

#[async_trait]
impl<M> Network<P2PNode, M> for P2PNetwork
where
    M: 'static + Send + Serialize + DeserializeOwned,
{
    async fn create_node(&mut self) -> Result<P2PNode, Box<dyn Error>> {
        let mut node = P2PNode::new(self.topic).await?;
        let address = node.get_listening_address().await;
        println!("Created node with address {}", address);
        node.dial_addresses(&self.nodes);
        self.nodes.push(address);
        Ok(node)
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
        swarm.listen_on("/ip4/127.0.0.1/tcp/0".parse()?)?;

        Ok(P2PNode { swarm, topic })
    }

    pub async fn get_listening_address(&mut self) -> Multiaddr {
        loop {
            if let Some(SwarmEvent::NewListenAddr { address, .. }) = self.swarm.next().await {
                return address;
            }
        }
    }

    pub fn dial_addresses(&mut self, addresses: &[Multiaddr]) {
        addresses
            .iter()
            .for_each(|a| self.swarm.dial_addr(a.clone()).unwrap())
    }
}

#[async_trait]
impl<M> Node<M> for P2PNode
where
    M: 'static + Send + Serialize + DeserializeOwned,
{
    async fn wait_for_connections(&mut self, num: u32) {
        println!("Waiting for {} connections", num);
        loop {
            if let SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } = self.swarm.next().await.unwrap()
            {
                /*self.swarm
                .behaviour_mut()
                .gossipsub
                .add_explicit_peer(&peer_id);*/
                println!(
                    "Established connection with {}@{}",
                    peer_id,
                    endpoint.get_remote_address()
                );
            }
            if self
                .swarm
                .behaviour()
                .gossipsub
                .all_peers()
                .filter(|(_, v)| v.contains(&&self.topic.hash()))
                .count()
                >= num.try_into().unwrap()
            {
                break;
            }
        }

        println!("Found connections. Waiting...");
        timeout(Duration::from_secs(1), async {
            loop {
                self.swarm.next().await.unwrap();
            }
        })
        .await
        .unwrap_err();
        println!("Done waiting.");
    }
}

#[async_trait]
impl<M> Actor<M> for P2PNode
where
    M: 'static + Send + Serialize + DeserializeOwned,
{
    async fn run(&mut self, mut input: Receiver<M>, output: Sender<M>) -> Status {
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
                            if output.send(block).await.is_err() {
                                return Status::Stopped;
                            }
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
    pub async fn test_p2p_network() {
        test_network(P2PNetwork::new("blockkey")).await
    }
}
