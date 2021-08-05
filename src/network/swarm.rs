use libp2p::{
    core::{muxing, transport, upgrade},
    dns::TokioDnsConfig,
    gossipsub::{
        self, Gossipsub, GossipsubEvent, GossipsubMessage, MessageAuthenticity, MessageId,
        ValidationMode,
    },
    identity,
    mdns::{Mdns, MdnsEvent},
    mplex, noise,
    ping::{Ping, PingConfig, PingEvent},
    swarm::SwarmBuilder,
    tcp::TokioTcpConfig,
    NetworkBehaviour, PeerId, Swarm, Transport,
};
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::time::Duration;

pub enum InternalEvent {
    Received {
        peer: PeerId,
        id: MessageId,
        message: GossipsubMessage,
    },
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
            } => InternalEvent::Received { peer, id, message },
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

impl From<PingEvent> for InternalEvent {
    fn from(_: PingEvent) -> Self {
        InternalEvent::Other
    }
}

// Create a custom network behaviour that combines Gossipsub and mDNS.
#[derive(NetworkBehaviour)]
#[behaviour(out_event = "InternalEvent", event_process = false)]
pub struct CustomBehaviour {
    pub gossipsub: Gossipsub,
    pub mdns: Mdns,
    pub ping: Ping,
}

async fn create_transport(
    keys: &identity::Keypair,
) -> Result<transport::Boxed<(PeerId, muxing::StreamMuxerBox)>, Box<dyn Error>> {
    // Create a keypair for authenticated encryption of the transport.
    let noise_keys = noise::Keypair::<noise::X25519Spec>::new()
        .into_authentic(keys)
        .expect("Signing libp2p-noise static DH keypair failed.");

    // Base tcp transport
    let tcp = TokioTcpConfig::new().nodelay(true);

    // Add DNS resolver
    let dns_tcp = TokioDnsConfig::system(tcp)?;

    // Upgrade and configure
    let configured = dns_tcp
        .upgrade(upgrade::Version::V1)
        .authenticate(noise::NoiseConfig::xx(noise_keys).into_authenticated())
        .multiplex(mplex::MplexConfig::new())
        .boxed();

    Ok(configured)
}

pub async fn construct() -> Result<Swarm<CustomBehaviour>, Box<dyn Error>> {
    // Generate a random keypair and corresponding ID
    let local_keys = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(local_keys.public());
    println!("Local peer id: {:?}", peer_id);

    // Create a tokio-based TCP transport use noise for authenticated
    // encryption and Mplex for multiplexing of substreams on a TCP stream.
    let transport = create_transport(&local_keys).await?;

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

        let ping = Ping::new(PingConfig::new().with_keep_alive(true));

        let behaviour = CustomBehaviour {
            gossipsub,
            mdns,
            ping,
        };

        SwarmBuilder::new(transport, behaviour, peer_id)
            // Spawn background tasks onto the tokio runtime.
            .executor(Box::new(|fut| {
                tokio::spawn(fut);
            }))
            .build()
    };

    Ok(swarm)
}
