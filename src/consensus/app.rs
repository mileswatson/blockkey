use crate::crypto::{contracts::PublicKey, hashing::Hash};

pub trait App<B>: Clone + Default {
    fn proposer(&self, round: u64) -> Hash<PublicKey>;

    fn commit(&mut self, block: B);
}
