use crate::crypto::{
    contracts::{Contract, PublicKey},
    hashing::{Hash, Hashable},
};

pub trait App<B: Hashable>: Clone {
    fn id(&self) -> Hash<PublicKey>;

    fn proposer(&self, round: u64) -> Hash<PublicKey>;

    fn create_block(&self) -> B;

    fn commit(&mut self, block: B);

    fn sign<T: Hashable>(&self, contract: T) -> Contract<T>;
}
