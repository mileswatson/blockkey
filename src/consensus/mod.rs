use crate::crypto::hashing::Hashable;
use tokio::sync::mpsc::{Receiver, Sender};
mod app;
mod types;

pub use app::*;
pub use types::*;

struct Record<B> {
    value: B,
    round: u64,
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
}

impl<A: App<B>, B: Hashable + Clone> Tendermint<A, B> {
    pub fn new(app: A, incoming: Receiver<Broadcast<B>>, outgoing: Sender<Broadcast<B>>) -> Self {
        Tendermint {
            height: 0,
            round: 0,
            step: Step::Propose,
            locked: None,
            valid: None,
            app,
            incoming,
            outgoing,
        }
    }

    fn reset(&mut self) {
        self.height = 0;
        self.round = 0;
        self.step = Step::Propose;
        self.locked = None;
        self.valid = None;
    }

    pub async fn run(&mut self) {
        self.reset();
        self.start_round(0).await;
    }

    async fn start_round(&mut self, round: u64) -> Result<(), Error> {
        self.round = round;
        self.step = Step::Propose;
        if self.app.proposer(self.round) == self.app.id() {
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
            self.outgoing
                .send(Broadcast::Proposal(self.app.sign(proposal)))
                .await
                .map_err(|_| Error::OutgoingClosed)
        } else {
            // Schedule!
            Err(Error::NotImplemented)
        }
    }
}
