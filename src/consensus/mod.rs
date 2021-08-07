use crate::crypto::hashing::Hashable;
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
}

struct Tendermint<A: App<B>, B: Hashable + Clone> {
    height: u64,
    round: u64,
    step: Step,
    locked: Option<Record<B>>,
    valid: Option<Record<B>>,
    app: A,
    incoming: Receiver<Broadcast<B>>,
    outgoing: Sender<Broadcast<B>>,
    timeouts: TimeoutManager<FunctionCall>,
    log: MessageLog<B>,
}

impl<A: App<B>, B: Hashable + Clone + Eq> Tendermint<A, B> {
    pub async fn start(
        app: A,
        incoming: Receiver<Broadcast<B>>,
        outgoing: Sender<Broadcast<B>>,
    ) -> Result<(), Error> {
        Tendermint {
            height: 0,
            round: 0,
            step: Step::Propose,
            locked: None,
            valid: None,
            app,
            incoming,
            outgoing,
            timeouts: TimeoutManager::new(),
            log: MessageLog::new(),
        }
        .run()
        .await
    }

    async fn start_round(&mut self, round: u64) -> Result<(), Error> {
        self.round = round;
        self.step = Step::Propose;
        if self.app.proposer(self.height, self.round) == self.app.id() {
            let proposal = match self.valid.as_ref() {
                Some(record) => Proposal {
                    height: self.height,
                    round: self.round,
                    proposal: record.value.clone(),
                    valid_round: Some(record.round),
                },
                None => Proposal {
                    height: self.height,
                    round: self.round,
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
                    round: self.round,
                },
                Duration::from_millis(1000),
            );
            Ok(())
        }
    }

    async fn line22(&mut self) -> Result<bool, Error> {
        if self.step != Step::Propose {
            return Ok(false);
        }

        let proposer = self.app.proposer(self.height, self.round);
        let broadcast = self
            .log
            .get_current()
            .proposals
            .iter()
            .filter(|contract| contract.signee.hash() == proposer)
            .map(|contract| &contract.content)
            .find(|proposal| {
                (proposal.height, proposal.round, proposal.valid_round)
                    == (self.height, self.round, None)
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

        if (height, round, valid_round) == (&self.height, &self.round, &None) {
            let vote_id = if self.app.validate_block(v) && self.locked.is_none()
                || self.locked.as_ref().map(|x| &x.value == v).unwrap_or(false)
            {
                Some(v.hash())
            } else {
                None
            };

            let prevote = Prevote::new(self.height, self.round, vote_id);
            self.broadcast(Broadcast::Prevote(self.app.sign(prevote)))
                .await?;

            self.step = Step::Prevote;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn line28(&mut self) -> Result<bool, Error> {
        if self.step == Step::Propose {
            return Ok(false);
        }

        let proposer = self.app.proposer(self.height, self.round);
        let messages = self.log.get_current();
        let broadcast = messages
            .proposals
            .iter()
            .filter(|contract| contract.signee.hash() == proposer)
            .map(|contract| &contract.content)
            .filter(|proposal| {
                if let Some(vr) = proposal.valid_round {
                    vr < self.round
                        && proposal.height == self.height
                        && proposal.round == self.round
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
                            self.app.get_voting_weight(contract.signee.hash()),
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

                total_weight > (self.app.total_voting_weight() + 2) / 3 + 1
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

        if (height, round, valid_round) == (&self.height, &self.round, &None) {
            let vote_id = if self.app.validate_block(v) && self.locked.is_none()
                || self.locked.as_ref().map(|x| &x.value == v).unwrap_or(false)
            {
                Some(v.hash())
            } else {
                None
            };

            let prevote = Prevote::new(self.height, self.round, vote_id);
            self.broadcast(Broadcast::Prevote(self.app.sign(prevote)))
                .await?;

            self.step = Step::Prevote;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        self.start_round(0).await?;

        loop {
            tokio::select! {
                function_call = self.timeouts.get_next() => {
                    match function_call {
                        FunctionCall::ProposeTimeout {height, round} => self.propose_timeout(height, round).await?,
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
                            self.line28().await?
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
        if self.height == height && self.round == round && self.step == Step::Prevote {
            let vote = Prevote::new(height, round, None);
            self.broadcast(Broadcast::Prevote(self.app.sign(vote)))
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
