use crate::crypto::{
    contracts::PublicKey,
    hashing::{Hash, Hashable},
};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    time::Duration,
};
mod app;
mod events;
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

enum Timeouts {
    Propose { height: u64, round: u64 },
    Prevote { height: u64, round: u64 },
    Precommit { height: u64, round: u64 },
}

struct RoundState {
    round: u64,
    step: Step,
    precommit_timeout_scheduled: bool,
}

impl RoundState {
    fn new(round: u64) -> RoundState {
        RoundState {
            round,
            step: Step::Propose,
            precommit_timeout_scheduled: false,
        }
    }
}

pub struct Tendermint<A: App<B>, B: Hashable + Clone> {
    app: A,
    height: u64,
    current: RoundState,
    locked: Option<Record<B>>,
    valid: Option<Record<B>>,
    log: MessageLog<B>,
    incoming: Receiver<Broadcast<B>>,
    outgoing: Sender<Broadcast<B>>,
    timeouts: TimeoutManager<Timeouts>,
}

impl<A: App<B>, B: Hashable + Clone + Eq> Tendermint<A, B> {
    pub async fn start(
        app: A,
        incoming: Receiver<Broadcast<B>>,
        outgoing: Sender<Broadcast<B>>,
    ) -> Result<(), Error> {
        Tendermint {
            current: RoundState::new(0),
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
        self.current = RoundState::new(round);
        if self.app.proposer(self.current.round) == self.app.id() {
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
                Timeouts::Propose {
                    height: self.height,
                    round: self.current.round,
                },
                Duration::from_millis(1000),
            );
            Ok(())
        }
    }

    async fn new_height(&mut self, height: u64, block: Option<B>) -> Result<(), Error> {
        if let Some(b) = block {
            // decision_p[h_p] = v
            self.app.commit(b)
        }

        self.height = height;

        // resets locked, valid, and empties message log.
        self.locked = None;
        self.valid = None;
        self.log.increment_height();

        // StartRound(0)
        self.start_round(0).await
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        self.new_height(0, None).await?;

        loop {
            tokio::select! {
                function_call = self.timeouts.get_next() => {
                    match function_call {
                        Timeouts::Propose {height, round} => self.propose_timeout(height, round).await?,
                        Timeouts::Prevote {height, round} => self.prevote_timeout(height, round).await?,
                        Timeouts::Precommit {height, round} => self.precommit_timeout(height, round).await?,
                    }
                }
                incoming = self.incoming.recv() => {
                    match incoming {
                        Some(b) => self.log.add(b),
                        None => return Err(Error::IncomingClosed),
                    };

                    loop {
                        #[allow(clippy::eval_order_dependence)]
                        let changed = [
                            self.line22().await?,
                            self.line28().await?,
                            self.line34()?,
                            self.line36().await?,
                            self.line44().await?,
                            self.line47()?,
                            self.line49().await?,
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

    async fn precommit_timeout(&mut self, height: u64, round: u64) -> Result<(), Error> {
        if height == self.height && round == self.current.round {
            self.start_round(self.current.round + 1).await?
        }
        Ok(())
    }

    async fn broadcast(&self, msg: Broadcast<B>) -> Result<(), Error> {
        self.outgoing
            .send(msg)
            .await
            .map_err(|_| Error::OutgoingClosed)
    }

    fn voting_weight(&self, id: Hash<PublicKey>) -> u64 {
        *self.app.validators().get(&id).unwrap_or(&0)
    }

    /// Returns the maximum voting weight of faulty processes in the network.
    /// If votes > one_f, then at least one correct process agrees.
    fn f(&self) -> u64 {
        self.app.total_votes() / 3
    }

    /// Returns twice the maximum voting weight of adversaries in the network.
    /// If votes > two_f, then the majority of correct processes agree.
    fn two_f(&self) -> u64 {
        self.f() * 2
    }
}
