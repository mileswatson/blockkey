use std::collections::HashMap;

use crate::crypto::{contracts::Contract, hashing::Hashable};

use super::{Broadcast, Precommit, Prevote, Proposal};

const LIMIT: u64 = 5;

pub struct Messages<B: Hashable> {
    pub proposals: Vec<Contract<Proposal<B>>>,
    pub prevotes: Vec<Contract<Prevote<B>>>,
    pub precommits: Vec<Contract<Precommit<B>>>,
}

impl<B: Hashable> Messages<B> {
    pub fn new() -> Messages<B> {
        Messages {
            proposals: Vec::new(),
            prevotes: Vec::new(),
            precommits: Vec::new(),
        }
    }
}

pub struct MessageLog<B: Hashable> {
    height: u64,
    messages: HashMap<u64, Messages<B>>,
}

impl<B: Hashable> MessageLog<B> {
    pub fn new() -> MessageLog<B> {
        let mut messages = HashMap::new();
        for i in 0..LIMIT {
            messages.insert(i, Messages::new());
        }
        MessageLog {
            height: 0,
            messages,
        }
    }

    pub fn increment_height(&mut self) {
        self.messages.remove(&self.height);
        self.height += 1;
        self.messages.insert(self.height + LIMIT, Messages::new());
    }

    pub fn add(&mut self, broadcast: Broadcast<B>) {
        match broadcast {
            Broadcast::Proposal(contract) => {
                if let Some(m) = self.messages.get_mut(&contract.content.height) {
                    m.proposals.push(contract)
                }
            }
            Broadcast::Prevote(contract) => {
                if let Some(m) = self.messages.get_mut(&contract.content.height) {
                    m.prevotes.push(contract)
                }
            }
            Broadcast::Precommit(contract) => {
                if let Some(m) = self.messages.get_mut(&contract.content.height) {
                    m.precommits.push(contract)
                }
            }
        };
    }

    pub fn get_current(&self) -> &Messages<B> {
        self.messages.get(&self.height).unwrap()
    }
}
