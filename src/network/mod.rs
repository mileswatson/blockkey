#[cfg(test)]
pub mod mock;
pub mod p2p;
#[cfg(test)]
mod test;

use crate::actor::Actor;
use async_trait::async_trait;
use std::error::Error;

#[async_trait]
pub trait Node {
    async fn wait_for_connections(&mut self, num: u32);
}

#[async_trait]
pub trait Network<M: Clone + 'static, N: Node + Actor<M>> {
    async fn create_node(&mut self) -> Result<N, Box<dyn Error>>;
}
