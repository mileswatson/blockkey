mod swarm;

use async_trait::async_trait;
use libp2p::{
    gossipsub::IdentTopic as GossipTopic, gossipsub::MessageId, swarm::SwarmEvent, Multiaddr,
    PeerId, Swarm,
};
use serde::{de::DeserializeOwned, Serialize};
use std::error::Error;
use tokio::sync::mpsc::{Receiver, Sender};

use swarm::InternalEvent;

use super::{Network, Node};

#[derive(Debug)]
pub struct Message {
    pub peer: PeerId,
    pub id: MessageId,
    pub data: Vec<u8>,
}

pub struct P2PNetwork {}

#[async_trait]
impl<B: 'static + Serialize + DeserializeOwned> Network<B> for P2PNetwork {
    fn new() -> Self {
        P2PNetwork {}
    }

    async fn create_node(&mut self) -> Result<Box<dyn super::Node<B>>, Box<dyn Error>> {
        Ok(Box::new(P2PNode::new("blockkey").await?))
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

        // Reach out to another node if specified in args
        if let Some(to_dial) = std::env::args().nth(1) {
            let addr: Multiaddr = to_dial.parse()?;
            swarm.dial_addr(addr)?;
            println!("Dialed {:?}", to_dial)
        }

        // Listen on all interfaces and whatever port the OS assigns
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

        Ok(P2PNode { swarm, topic })
    }
}

#[async_trait(?Send)]
impl<B: 'static + Serialize + DeserializeOwned> Node<B> for P2PNode {
    async fn run(&mut self, incoming: Sender<B>, mut outgoing: Receiver<B>) -> Result<(), ()> {
        loop {
            tokio::select! {
                block = outgoing.recv() => {
                    match block {
                        None => {
                            return Err(())
                        },
                        Some(block) => {
                            let bytes = serde_json::to_vec(&block).map_err(|_| {})?;
                            self.swarm
                                .behaviour_mut()
                                .gossipsub
                                .publish(self.topic.clone(), bytes).map_err(|_| {})?;
                        }
                    }
                }
                event = self.swarm.next_event() => {
                    use SwarmEvent::*;
                    match event {
                        NewListenAddr(addr) => println!("Listening on {}", addr),
                        Behaviour(internal_event) => match internal_event {
                            InternalEvent::Received { message, .. } => {
                                match serde_json::from_slice(&message.data) {
                                    Ok(b) => incoming.send(b).await.map_err(|_| {})?,
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
