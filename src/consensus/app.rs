use std::collections::HashMap;

use crate::crypto::{
    contracts::{Contract, PublicKey},
    hashing::{Hash, Hashable},
};

pub trait App<B: Hashable>: Clone {
    fn id(&self) -> Hash<PublicKey>;

    fn validators(&self) -> HashMap<Hash<PublicKey>, u64>;

    fn total_votes(&self) -> u64;

    fn proposer(&self, round: u64) -> Hash<PublicKey>;

    fn create_block(&self) -> B;

    fn validate_block(&self, block: &B) -> bool;

    fn commit(&mut self, block: B);

    fn sign<T: Hashable>(&self, contract: T) -> Contract<T>;
}
