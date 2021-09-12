pub mod p2p;

use libp2p::{gossipsub::MessageId, swarm::SwarmEvent, PeerId};

#[derive(Debug)]
pub struct Message {
    pub peer: PeerId,
    pub id: MessageId,
    pub data: Vec<u8>,
}
