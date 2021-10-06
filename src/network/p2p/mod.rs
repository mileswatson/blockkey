mod swarm;

use async_trait::async_trait;
use futures::StreamExt;
use libp2p::{gossipsub::IdentTopic as GossipTopic, swarm::SwarmEvent, Multiaddr, Swarm};
use rand::random;
use serde::{de::DeserializeOwned, Serialize};
use std::{convert::TryInto, error::Error, fmt::Debug, time::Duration};
use tokio::{
    sync::broadcast::{Receiver, Sender},
    time::timeout,
};

use swarm::InternalEvent;

use crate::actor::{ActorEvent, Status};

use super::{Actor, Network, Node};

pub struct P2PNetwork {
    topic: String,
    nodes: Vec<Multiaddr>,
    public: bool,
}

impl P2PNetwork {
    /// Generates a random topic if None.
    pub fn new(topic: Option<String>, public: bool) -> Self {
        P2PNetwork {
            topic: topic.unwrap_or_else(|| random::<u64>().to_string()),
            nodes: Vec::new(),
            public,
        }
    }
}

#[async_trait]
impl<M: Clone + 'static> Network<M, P2PNode> for P2PNetwork
where
    M: 'static + Send + Serialize + DeserializeOwned + Debug,
{
    async fn create_node(&mut self) -> Result<P2PNode, Box<dyn Error>> {
        let mut node = P2PNode::new(self.topic.clone(), self.public).await?;
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
    pub async fn new(topic: String, public: bool) -> Result<P2PNode, Box<dyn Error>> {
        let mut swarm = swarm::construct().await?;

        let topic = GossipTopic::new(topic);

        swarm.behaviour_mut().gossipsub.subscribe(&topic).unwrap();

        let addr = format!(
            "/ip4/{}/tcp/0",
            if public { "0.0.0.0" } else { "127.0.0.1" }
        )
        .parse()?;

        // Listen on all interfaces and whatever port the OS assigns
        swarm.listen_on(addr)?;

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
impl Node for P2PNode {
    async fn wait_for_connections(&mut self, num: u32) {
        println!("Waiting for {} connections", num);
        loop {
            if let SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } = self.swarm.next().await.unwrap()
            {
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
    M: 'static + Send + Serialize + DeserializeOwned + Clone + Debug,
{
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
                            let bytes = match serde_json::to_vec(&block) {
                                Ok(bytes) => bytes,
                                Err(_) => {
                                    output.send(ActorEvent::Stop).unwrap();
                                    return Status::Failed
                                }
                            };
                            if self.swarm
                                .behaviour_mut()
                                .gossipsub
                                .publish(self.topic.clone(), bytes).is_err() {
                                    output.send(ActorEvent::Stop).unwrap();
                                    return Status::Failed
                                }
                            output.send(ActorEvent::Receive(block)).unwrap();
                        }
                        ActorEvent::Stop => return Status::Stopped,
                        _ => (),
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
                                    Ok(b) => { output.send(ActorEvent::Receive(b)).unwrap(); },
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
        test_network(P2PNetwork::new(None, false)).await
    }
}
