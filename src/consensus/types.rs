use crate::crypto::{
    contracts::Contract,
    hashing::{Hash, Hashable},
};

pub enum Step {
    Propose,
    Prevote,
    Precommit,
}

pub struct Proposal<T: Hashable> {
    height: u64,
    round: u64,
    proposal: T,
    validRound: Option<u64>,
}

pub struct Vote<T> {
    step: Step,
    height: u64,
    round: u64,
    id: Hash<T>,
}

pub enum Broadcast<T: Hashable> {
    Proposal(Contract<Proposal<T>>),
    Vote(Contract<Vote<T>>),
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
        hash![self.height, self.round, self.proposal, self.validRound]
    }
}
impl<T> Hashable for Vote<T> {
    fn hash(&self) -> Hash<Self> {
        hash![self.step, self.height, self.round, self.id]
    }
}
