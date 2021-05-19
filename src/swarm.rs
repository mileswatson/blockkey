use libp2p::{
    core::upgrade,
    gossipsub::{
        self, Gossipsub, GossipsubEvent, GossipsubMessage, IdentTopic as GossipTopic,
        MessageAuthenticity, MessageId, ValidationMode,
    },
    identity,
    mdns::{Mdns, MdnsEvent},
    mplex, noise,
    swarm::{SwarmBuilder, SwarmEvent},
    tcp::TokioTcpConfig,
    Multiaddr, NetworkBehaviour, PeerId, Swarm, Transport,
};
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use tokio::io::{self, AsyncBufReadExt};

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
            MdnsEvent::Discovered(list) => {
                CustomEvent::Found(list.map(|(peer, _)| peer).collect())
                //for (peer, _) in list {
                //    self.gossipsub.add_explicit_peer(&peer);
                //}
            }
            MdnsEvent::Expired(list) => {
                CustomEvent::Lost(list.map(|(peer, _)| peer).collect())
                //for (peer, _) in list {
                //    self.gossipsub.remove_explicit_peer(&peer);
                //}
            }
        }
    }
}

// Create a custom network behaviour that combines Gossipsub and mDNS.
#[derive(NetworkBehaviour)]
#[behaviour(out_event = "CustomEvent", event_process = false)]
pub struct CustomBehaviour {
    gossipsub: Gossipsub,
    mdns: Mdns,
    #[behaviour(ignore)]
    topic: GossipTopic,
}

pub async fn construct(topic: &str) -> Result<Swarm<CustomBehaviour>, Box<dyn Error>> {
    let topic = GossipTopic::new(topic);

    // Generate a random keypair and corresponding ID
    let local_keys = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(local_keys.public());
    println!("Local peer id: {:?}", peer_id);

    // Create a keypair for authenticated encryption of the transport.
    let noise_keys = noise::Keypair::<noise::X25519Spec>::new()
        .into_authentic(&local_keys)
        .expect("Signing libp2p-noise static DH keypair failed.");

    // Create a tokio-based TCP transport use noise for authenticated
    // encryption and Mplex for multiplexing of substreams on a TCP stream.
    let transport = TokioTcpConfig::new()
        .nodelay(true)
        .upgrade(upgrade::Version::V1)
        .authenticate(noise::NoiseConfig::xx(noise_keys).into_authenticated())
        .multiplex(mplex::MplexConfig::new())
        .boxed();

    // Create a Swarm to manage peers and events.
    let swarm = {
        let mdns = Mdns::new(Default::default()).await?;

        // To prevent duplicate messages, we can take the hash of message and use it as an ID.
        let message_id_fn = |message: &GossipsubMessage| {
            let mut s = DefaultHasher::new();
            message.data.hash(&mut s);
            MessageId::from(s.finish().to_string())
        };

        let gossipsub_config = gossipsub::GossipsubConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
            .validation_mode(ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message signing)
            .message_id_fn(message_id_fn) // content-address messages. No two messages of the
            // same content will be propagated.
            .build()
            .expect("Valid config");

        // Create a gossipsub network
        let gossipsub: gossipsub::Gossipsub =
            gossipsub::Gossipsub::new(MessageAuthenticity::Signed(local_keys), gossipsub_config)
                .expect("Correct configuration");

        let mut behaviour = CustomBehaviour {
            gossipsub,
            mdns,
            topic: topic.clone(),
        };

        behaviour.gossipsub.subscribe(&topic).unwrap();

        SwarmBuilder::new(transport, behaviour, peer_id)
            // Spawn background tasks onto the tokio runtime.
            .executor(Box::new(|fut| {
                tokio::spawn(fut);
            }))
            .build()
    };

    Ok(swarm)
}

pub async fn run(
    swarm: &mut Swarm<CustomBehaviour>,
    handler: &mut impl Handler,
) -> Result<(), Box<dyn Error>> {
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
