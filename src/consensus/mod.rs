use crate::crypto::hashing::Hashable;
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
}

impl<A: App<B>, B: Hashable + Clone> Tendermint<A, B> {
    fn new(app: A) -> Self {
        Tendermint {
            height: 0,
            round: 0,
            step: Step::Propose,
            locked: None,
            valid: None,
            app,
        }
    }

    async fn run(&mut self) {
        self.start_round(0);
    }

    async fn start_round(&mut self, round: u64) {
        self.round = round;
        self.step = Step::Propose;
        if self.app.proposer(self.round) == self.app.id() {
            let proposal = match self.valid.take() {
                Some((round, value)) => (Some(round), value),
                None => (None, self.app.create_block()),
            };
        } else {
            // Schedule!
        }
    }
}
