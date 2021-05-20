use libp2p::{
    gossipsub::{Gossipsub, GossipsubEvent, IdentTopic as GossipTopic, MessageId},
    mdns::{Mdns, MdnsEvent},
    swarm::SwarmEvent,
    Multiaddr, NetworkBehaviour, PeerId, Swarm,
};
use std::error::Error;
use tokio::io::{self, AsyncBufReadExt};

mod swarm;

pub trait Handler: Sync {
    fn receive(&mut self, message: Message);
}

#[derive(Debug)]
pub struct Message {
    pub peer: PeerId,
    pub id: MessageId,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub enum CustomEvent {
    Received(Message),
    Found(Vec<PeerId>),
    Lost(Vec<PeerId>),
    Other,
}

impl From<GossipsubEvent> for CustomEvent {
    fn from(event: GossipsubEvent) -> Self {
        match event {
            GossipsubEvent::Message {
                propagation_source: peer,
                message_id: id,
                message,
            } => CustomEvent::Received(Message {
                peer,
                id,
                data: message.data,
            }),

            _ => CustomEvent::Other,
        }
    }
}

impl From<MdnsEvent> for CustomEvent {
    fn from(event: MdnsEvent) -> Self {
        match event {
            MdnsEvent::Discovered(list) => CustomEvent::Found(list.map(|(peer, _)| peer).collect()),
            MdnsEvent::Expired(list) => CustomEvent::Lost(list.map(|(peer, _)| peer).collect()),
        }
    }
}

pub async fn run(topic: &str, handler: &mut impl Handler) -> Result<(), Box<dyn Error>> {
    let mut swarm = swarm::construct(topic).await?;

    // Reach out to another node if specified in args
    if let Some(to_dial) = std::env::args().nth(1) {
        let addr: Multiaddr = to_dial.parse()?;
        swarm.dial_addr(addr)?;
        println!("Dialed {:?}", to_dial)
    }

    // Read lines from stdin
    let mut stdin = io::BufReader::new(io::stdin()).lines();

    // Listen on all interfaces and whatever port the OS assigns
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    // Start reading stdin and publishing input.
    loop {
        let to_publish = {
            tokio::select! {
                line = stdin.next_line() => {
                    let line = line?.expect("stdin closed");
                    Some((swarm.behaviour().topic.clone(), line))
                }
                event = swarm.next_event() => match event {
                    SwarmEvent::NewListenAddr(addr) => {
                        println!("Now listening on {:?}", addr);
                        None
                    }
                    SwarmEvent::Behaviour(e) => match e {
                            CustomEvent::Received(m) => {
                                handler.receive(m);
                                None
                            }
                            CustomEvent::Found(peers) => {
                                for peer in peers {
                                    swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer);
                                }
                                None
                            }
                            CustomEvent::Lost(peers) => {
                                for peer in peers {
                                    swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer);
                                }
                                None
                            }
                            CustomEvent::Other => None
                        }
                    _ => None
                }
            }
        };
        if let Some((topic, line)) = to_publish {
            swarm
                .behaviour_mut()
                .gossipsub
                .publish(topic, line.as_bytes())
                .unwrap();
        }
    }
}
