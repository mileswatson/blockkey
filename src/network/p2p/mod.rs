mod swarm;

use libp2p::{gossipsub::IdentTopic as GossipTopic, Multiaddr, Swarm};
use std::error::Error;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::network::Message;

use self::swarm::InternalEvent;

pub struct Network {
    swarm: Swarm<swarm::CustomBehaviour>,
    topic: GossipTopic,
}

impl Network {
    pub async fn run(
        &mut self,
        incoming: Sender<Message>,
        mut outgoing: Receiver<Vec<u8>>,
    ) -> Result<(), ()> {
        loop {
            tokio::select! {
                data = outgoing.recv() => {
                    match data {
                        None => {
                            return Err(())
                        },
                        Some(data) => {
                            self.swarm
                                .behaviour_mut()
                                .gossipsub
                                .publish(self.topic.clone(), data).map_err(|_| {})?;
                        }
                    }
                }
                event = self.swarm.next_event() => {
                    use super::SwarmEvent::*;
                    match event {
                        NewListenAddr(addr) => println!("Listening on {}", addr),
                        Behaviour(internal_event) => match internal_event {
                            InternalEvent::Received { peer, id, message } => {
                                incoming
                                    .send(Message {
                                        peer,
                                        id,
                                        data: message.data,
                                    })
                                    .await
                                    .map_err(|_| {})?;
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

pub async fn create(topic: &str) -> Result<Network, Box<dyn Error>> {
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

    Ok(Network { swarm, topic })
}
