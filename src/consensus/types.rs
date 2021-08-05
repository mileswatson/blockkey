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

pub struct Vote<T> {
    step: Step,
    height: u64,
    round: u64,
    id: Option<Hash<T>>,
}

impl<T> Vote<T> {
    pub fn new(step: Step, height: u64, round: u64, id: Option<Hash<T>>) -> Vote<T> {
        Vote {
            step,
            height,
            round,
            id,
        }
    }
}

#[allow(clippy::large_enum_variant)]
pub enum Broadcast<B: Hashable> {
    Proposal(Contract<Proposal<B>>),
    Vote(Contract<Vote<B>>),
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
impl<T> Hashable for Vote<T> {
    fn hash(&self) -> Hash<Self> {
        hash![self.step, self.height, self.round, self.id]
    }
}

pub enum Error {
    NotImplemented,
    OutgoingClosed,
}
