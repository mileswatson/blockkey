use std::collections::HashMap;

use crate::crypto::{
    contracts::PublicKey,
    hashing::{Hash, Hashable},
};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    time::Duration,
};
mod app;
mod log;
mod timeout;
mod types;

pub use app::*;
use log::*;
pub use types::*;

use timeout::TimeoutManager;

struct Record<B> {
    value: B,
    round: u64,
}

enum FunctionCall {
    ProposeTimeout { height: u64, round: u64 },
    PrevoteTimeout { height: u64, round: u64 },
}

struct RoundState {
    round: u64,
    step: Step,
    line34_executed: bool,
    validators: HashMap<Hash<PublicKey>, u64>,
    one_third: u64,
    two_thirds: u64,
}

impl RoundState {
    fn new(round: u64, validators: HashMap<Hash<PublicKey>, u64>) -> RoundState {
        let total: u64 = validators.iter().map(|(_, weight)| *weight).sum();
        RoundState {
            round,
            step: Step::Propose,
            line34_executed: false,
            validators,
            one_third: total / 3,
            two_thirds: (total / 3) * 2,
        }
    }

    fn voting_weight(&self, id: Hash<PublicKey>) -> u64 {
        *self.validators.get(&id).unwrap_or(&0)
    }
}

struct Tendermint<A: App<B>, B: Hashable + Clone> {
    app: A,
    height: u64,
    current: RoundState,
    locked: Option<Record<B>>,
    valid: Option<Record<B>>,
    log: MessageLog<B>,
    incoming: Receiver<Broadcast<B>>,
    outgoing: Sender<Broadcast<B>>,
    timeouts: TimeoutManager<FunctionCall>,
}

impl<A: App<B>, B: Hashable + Clone + Eq> Tendermint<A, B> {
    pub async fn start(
        app: A,
        incoming: Receiver<Broadcast<B>>,
        outgoing: Sender<Broadcast<B>>,
    ) -> Result<(), Error> {
        Tendermint {
            current: RoundState::new(0, app.get_validators()),
            app,
            height: 0,
            locked: None,
            valid: None,
            log: MessageLog::new(),
            incoming,
            outgoing,
            timeouts: TimeoutManager::new(),
        }
        .run()
        .await
    }

    async fn start_round(&mut self, round: u64) -> Result<(), Error> {
        self.current = RoundState::new(round, self.app.get_validators());
        if self.app.proposer(self.height, self.current.round) == self.app.id() {
            let proposal = match self.valid.as_ref() {
                Some(record) => Proposal {
                    height: self.height,
                    round: self.current.round,
                    proposal: record.value.clone(),
                    valid_round: Some(record.round),
                },
                None => Proposal {
                    height: self.height,
                    round: self.current.round,
                    proposal: self.app.create_block(),
                    valid_round: None,
                },
            };
            self.broadcast(Broadcast::Proposal(self.app.sign(proposal)))
                .await
        } else {
            self.timeouts.add(
                FunctionCall::ProposeTimeout {
                    height: self.height,
                    round: self.current.round,
                },
                Duration::from_millis(1000),
            );
            Ok(())
        }
    }

    async fn line22(&mut self) -> Result<bool, Error> {
        if !self.current.step.is_propose() {
            return Ok(false);
        }

        let proposer = self.app.proposer(self.height, self.current.round);
        let broadcast = self
            .log
            .get_current()
            .proposals
            .iter()
            .filter(|contract| contract.signee.hash() == proposer)
            .map(|contract| &contract.content)
            .find(|proposal| {
                (proposal.height, proposal.round, proposal.valid_round)
                    == (self.height, self.current.round, None)
            });

        let proposal = match broadcast {
            Some(contract) => contract,
            None => return Ok(false),
        };

        let Proposal {
            height,
            round,
            proposal: v,
            valid_round,
        } = proposal;

        if (height, round, valid_round) == (&self.height, &self.current.round, &None) {
            let vote_id = if self.app.validate_block(v)
                && self.locked.as_ref().map(|x| &x.value == v).unwrap_or(true)
            {
                Some(v.hash())
            } else {
                None
            };

            let prevote = Prevote::new(self.height, self.current.round, vote_id);
            self.broadcast(Broadcast::Prevote(self.app.sign(prevote)))
                .await?;

            self.current.step = Step::prevote();

            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn line28(&mut self) -> Result<bool, Error> {
        if !self.current.step.is_propose() {
            return Ok(false);
        }

        let proposer = self.app.proposer(self.height, self.current.round);
        let messages = self.log.get_current();
        let broadcast = messages
            .proposals
            .iter()
            .filter(|contract| contract.signee.hash() == proposer)
            .map(|contract| &contract.content)
            .filter(|proposal| {
                if let Some(vr) = proposal.valid_round {
                    vr < self.current.round
                        && proposal.height == self.height
                        && proposal.round == self.current.round
                } else {
                    false
                }
            })
            .find(|proposal| {
                let id = proposal.proposal.hash();
                let total_weight = messages
                    .prevotes
                    .iter()
                    .map(|contract| {
                        (
                            self.current.voting_weight(contract.signee.hash()),
                            &contract.content,
                        )
                    })
                    .filter(|(_, prevote)| {
                        prevote.height == self.height
                            && prevote.round == proposal.round
                            && prevote.id == Some(id)
                    })
                    .map(|(weight, _)| weight)
                    .sum::<u64>();

                total_weight > self.current.two_thirds
            });

        let proposal = match broadcast {
            Some(contract) => contract,
            None => return Ok(false),
        };

        let Proposal {
            height,
            round,
            proposal: ref v,
            valid_round,
        } = proposal;

        let valid_round = match valid_round {
            Some(x) => x,
            None => return Ok(false),
        };

        if (height, round) == (&self.height, &self.current.round) {
            let vote_id = if self.app.validate_block(v)
                && self
                    .locked
                    .as_ref()
                    .map(|x| &x.round <= valid_round || &x.value == v)
                    .unwrap_or(true)
            {
                Some(v.hash())
            } else {
                None
            };

            let prevote = Prevote::new(self.height, self.current.round, vote_id);
            self.broadcast(Broadcast::Prevote(self.app.sign(prevote)))
                .await?;

            self.current.step = Step::prevote();

            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn line34(&mut self) -> Result<bool, Error> {
        if !matches!(
            self.current.step,
            Step::Prevote {
                timeout_scheduled: false
            }
        ) {
            return Ok(false);
        }

        let messages = self.log.get_current();

        let total: u64 = messages
            .prevotes
            .iter()
            .filter_map(|contract| {
                let content = &contract.content;
                if content.height == self.height && content.round == self.current.round {
                    Some(self.current.voting_weight(contract.signee.hash()))
                } else {
                    None
                }
            })
            .sum();

        if total <= self.current.two_thirds {
            return Ok(false);
        }

        self.timeouts.add(
            FunctionCall::PrevoteTimeout {
                height: self.height,
                round: self.current.round,
            },
            Duration::from_millis(1000),
        );

        // False, as no state has been modified, therefore no rechecks are needed
        Ok(false)
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        self.start_round(0).await?;

        loop {
            tokio::select! {
                function_call = self.timeouts.get_next() => {
                    match function_call {
                        FunctionCall::ProposeTimeout {height, round} => self.propose_timeout(height, round).await?,
                        FunctionCall::PrevoteTimeout {height, round} => self.prevote_timeout(height, round).await?,
                    }
                }
                incoming = self.incoming.recv() => {
                    match incoming {
                        Some(b) => self.log.add(b),
                        None => return Err(Error::IncomingClosed),
                    };

                    loop {
                        let changed = [
                            self.line22().await?,
                            self.line28().await?,
                            self.line34().await?
                        ];
                        if !changed.iter().any(|x| *x) {
                            break
                        }
                    }
                }
            }
        }
    }

    async fn propose_timeout(&mut self, height: u64, round: u64) -> Result<(), Error> {
        if self.height == height && self.current.round == round && self.current.step.is_propose() {
            let vote = Prevote::new(height, round, None);
            self.broadcast(Broadcast::Prevote(self.app.sign(vote)))
                .await?
        }
        Ok(())
    }

    async fn prevote_timeout(&mut self, height: u64, round: u64) -> Result<(), Error> {
        if height == self.height && round == self.current.round && self.current.step.is_prevote() {
            let vote = Precommit::new(height, round, None);
            self.broadcast(Broadcast::Precommit(self.app.sign(vote)))
                .await?
        }
        Ok(())
    }

    async fn broadcast(&self, msg: Broadcast<B>) -> Result<(), Error> {
        self.outgoing
            .send(msg)
            .await
            .map_err(|_| Error::OutgoingClosed)
    }
}
