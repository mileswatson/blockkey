#[cfg(test)]
pub mod mock;
pub mod p2p;
#[cfg(test)]
mod test;

use crate::actor::Actor;
use async_trait::async_trait;
use std::error::Error;

#[async_trait]
pub trait Network<Node: Actor<NetInput, NetOutput>, NetInput, NetOutput = NetInput> {
    async fn create_node(&mut self) -> Result<Node, Box<dyn Error>>;
}
