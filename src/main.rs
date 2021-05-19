use libp2p::{
    core::upgrade,
    gossipsub::{
        self, Gossipsub, GossipsubEvent, GossipsubMessage, IdentTopic as GossipTopic,
        MessageAuthenticity, MessageId, ValidationMode,
    },
    identity,
    mdns::{Mdns, MdnsEvent},
    mplex, noise,
    swarm::{NetworkBehaviourEventProcess, SwarmBuilder, SwarmEvent},
    tcp::TokioTcpConfig,
    Multiaddr, NetworkBehaviour, PeerId, Transport,
};
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use tokio::io::{self, AsyncBufReadExt};

/// Set up the tokio runtime.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
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

    // Creates a Gossipsub topic.
    let chat_topic = GossipTopic::new("chat");

    // Create a custom network behaviour that combines Gossipsub and mDNS.
    // NetworkBehaviour can be derived if NetworkBehaviourEventProcess is implemented
    // for each possible event.
    #[derive(NetworkBehaviour)]
    struct CustomBehaviour {
        gossipsub: Gossipsub,
        mdns: Mdns,
    }

    // Handles incoming Gossipsub messages.
    impl NetworkBehaviourEventProcess<GossipsubEvent> for CustomBehaviour {
        fn inject_event(&mut self, message: GossipsubEvent) {
            if let GossipsubEvent::Message {
                propagation_source: peer_id,
                message_id: id,
                message,
            } = message
            {
                println!(
                    "Got message: {} with id: {} from peer: {:?}",
                    String::from_utf8_lossy(&message.data),
                    id,
                    peer_id
                );
            }
        }
    }

    // Create a Swarm to manage peers and events.
    let mut swarm = {
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

        let mut behaviour = CustomBehaviour { gossipsub, mdns };

        behaviour.gossipsub.subscribe(&chat_topic).unwrap();

        SwarmBuilder::new(transport, behaviour, peer_id)
            // Spawn background tasks onto the tokio runtime.
            .executor(Box::new(|fut| {
                tokio::spawn(fut);
            }))
            .build()
    };

    // Handles mDNS events to allow discovery of nodes on the same LAN.
    impl NetworkBehaviourEventProcess<MdnsEvent> for CustomBehaviour {
        fn inject_event(&mut self, event: MdnsEvent) {
            match event {
                MdnsEvent::Discovered(list) => {
                    for (peer, _) in list {
                        self.gossipsub.add_explicit_peer(&peer);
                    }
                }
                MdnsEvent::Expired(list) => {
                    for (peer, _) in list {
                        self.gossipsub.remove_explicit_peer(&peer);
                    }
                }
            }
        }
    }

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
                    Some((chat_topic.clone(), line))
                }
                event = swarm.next_event() => match event {
                    SwarmEvent::NewListenAddr(addr) => {
                        println!("Now listening on {:?}", addr);
                        None
                    }
                    SwarmEvent::Behaviour(e) => {
                        // All behaviour events should be handled by
                        // NetworkBehaviourEventProcess implementations.
                        panic!("Unexpected event: {:?}", e);
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
