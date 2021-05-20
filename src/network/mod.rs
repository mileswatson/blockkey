mod swarm;

use self::swarm::InternalEvent;
use libp2p::{
    gossipsub::{error::PublishError, IdentTopic as GossipTopic, MessageId},
    swarm::SwarmEvent,
    Multiaddr, PeerId, Swarm,
};
use std::error::Error;

pub struct Network {
    swarm: Swarm<swarm::CustomBehaviour>,
    topic: GossipTopic,
}

pub struct Message {
    pub peer: PeerId,
    pub id: MessageId,
    pub data: Vec<u8>,
}

pub enum NetworkEvent {
    ListeningOn(Multiaddr),
    Received(Message),
}

impl Network {
    pub async fn next_event(&mut self) -> NetworkEvent {
        loop {
            match self.swarm.next_event().await {
                SwarmEvent::NewListenAddr(addr) => return NetworkEvent::ListeningOn(addr),
                SwarmEvent::Behaviour(internal_event) => match internal_event {
                    InternalEvent::Received { peer, id, message } => {
                        return NetworkEvent::Received(Message {
                            peer,
                            id,
                            data: message.data,
                        })
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

    pub async fn broadcast(
        &mut self,
        message: &[u8],
    ) -> std::result::Result<libp2p::gossipsub::MessageId, PublishError> {
        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(self.topic.clone(), message)
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
