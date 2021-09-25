pub mod mock;
pub mod p2p;

use crate::actor::Actor;
use async_trait::async_trait;
use std::error::Error;

#[async_trait]
pub trait Network<Node: Actor<AppOutput, AppInput>, AppOutput, AppInput = AppOutput> {
    fn new() -> Self;
    async fn create_node(&mut self) -> Result<Node, Box<dyn Error>>;
}
