use crate::crypto::hashing::Hashable;
use tokio::sync::mpsc::{Receiver, Sender};
mod app;
mod types;

pub use app::*;
pub use types::*;

struct Tendermint<A: App<B>, B: Hashable + Clone> {
    height: u64,
    round: u64,
    step: Step,
    locked: Option<(u64, B)>,
    valid: Option<(u64, B)>,
    app: A,
    incoming: Receiver<i32>,
    outgoing: Sender<i32>,
}

impl<A: App<B>, B: Hashable + Clone> Tendermint<A, B> {
    pub fn new(app: A, incoming: Receiver<i32>, outgoing: Sender<i32>) -> Self {
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

    async fn start_round(&mut self, round: u64) -> Result<(), ()> {
        self.round = round;
        self.step = Step::Propose;
        if self.app.proposer(self.round) == self.app.id() {
            let proposal = match self.valid.take() {
                Some((round, value)) => (Some(round), value),
                None => (None, self.app.create_block()),
            };
            self.outgoing.send(1).await.map_err(|_| ())
        } else {
            // Schedule!
            Err(())
        }
    }
}
