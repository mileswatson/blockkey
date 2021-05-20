use libp2p::{
    gossipsub::{error::PublishError, GossipsubEvent, IdentTopic as GossipTopic, MessageId},
    mdns::MdnsEvent,
    swarm::SwarmEvent,
    Multiaddr, PeerId, Swarm,
};
use std::error::Error;
mod swarm;

pub struct Network {
    swarm: Swarm<swarm::CustomBehaviour>,
    topic: GossipTopic,
}

impl Network {
    pub async fn next_event(&mut self) -> NetworkEvent {
        loop {
            match self.swarm.next_event().await {
                SwarmEvent::NewListenAddr(addr) => return NetworkEvent::ListeningOn(addr),
                SwarmEvent::Behaviour(internal_event) => match internal_event {
                    InternalEvent::NetworkEvent(e) => return e,
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

#[derive(Debug)]
pub struct Message {
    pub peer: PeerId,
    pub id: MessageId,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub enum NetworkEvent {
    ListeningOn(Multiaddr),
    Received(Message),
}

pub enum InternalEvent {
    NetworkEvent(NetworkEvent),
    Found(Vec<PeerId>),
    Lost(Vec<PeerId>),
    Other,
}

impl From<GossipsubEvent> for InternalEvent {
    fn from(event: GossipsubEvent) -> Self {
        match event {
            GossipsubEvent::Message {
                propagation_source: peer,
                message_id: id,
                message,
            } => InternalEvent::NetworkEvent(NetworkEvent::Received(Message {
                peer,
                id,
                data: message.data,
            })),

            _ => InternalEvent::Other,
        }
    }
}

impl From<MdnsEvent> for InternalEvent {
    fn from(event: MdnsEvent) -> Self {
        match event {
            MdnsEvent::Discovered(list) => {
                InternalEvent::Found(list.map(|(peer, _)| peer).collect())
            }
            MdnsEvent::Expired(list) => InternalEvent::Lost(list.map(|(peer, _)| peer).collect()),
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
