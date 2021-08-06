use crate::crypto::{
    contracts::Contract,
    hashing::{Hash, Hashable},
};

#[derive(PartialEq, Eq)]
pub enum Step {
    Propose,
    Prevote,
    Precommit,
}

pub struct Proposal<T: Hashable> {
    pub height: u64,
    pub round: u64,
    pub proposal: T,
    pub valid_round: Option<u64>,
}

pub struct Prevote<T> {
    height: u64,
    round: u64,
    id: Option<Hash<T>>,
}

impl<T> Prevote<T> {
    pub fn new(height: u64, round: u64, id: Option<Hash<T>>) -> Prevote<T> {
        Prevote { height, round, id }
    }
}
pub struct Precommit<T> {
    height: u64,
    round: u64,
    id: Option<Hash<T>>,
}

impl<T> Precommit<T> {
    pub fn new(height: u64, round: u64, id: Option<Hash<T>>) -> Precommit<T> {
        Precommit { height, round, id }
    }
}

#[allow(clippy::large_enum_variant)]
pub enum Broadcast<B: Hashable> {
    Proposal(Contract<Proposal<B>>),
    Prevote(Contract<Prevote<B>>),
    Precommit(Contract<Precommit<B>>),
}

impl Hashable for Step {
    fn hash(&self) -> Hash<Self> {
        hash![match self {
            Step::Propose => 0,
            Step::Prevote => 1,
            Step::Precommit => 2,
        }]
    }
}

impl<T: Hashable> Hashable for Proposal<T> {
    fn hash(&self) -> Hash<Self> {
        hash![self.height, self.round, self.proposal, self.valid_round]
    }
}
impl<T> Hashable for Prevote<T> {
    fn hash(&self) -> Hash<Self> {
        hash![self.height, self.round, self.id]
    }
}
impl<T> Hashable for Precommit<T> {
    fn hash(&self) -> Hash<Self> {
        hash![self.height, self.round, self.id]
    }
}

pub enum Error {
    NotImplemented,
    OutgoingClosed,
    IncomingClosed,
}
