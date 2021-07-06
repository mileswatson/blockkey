use crate::crypto::hashing::Hashable;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

mod app;
mod types;

pub use app::*;
pub use types::*;

struct Tendermint<A: App<B>, B: Hashable> {
    height: u64,
    round: u64,
    step: Option<Step>,
    locked: Option<(u64, B)>,
    valid: Option<(u64, B)>,
    app: A,
}

impl<A: App<B>, B: Hashable> Tendermint<A, B> {
    fn new(app: A) -> Self {
        Tendermint {
            height: 0,
            round: 0,
            step: None,
            locked: None,
            valid: None,
            app,
        }
    }

    /*fn start_round(&mut self, round: u64) {
        self.round = round;
        self.step = Step::Propose;
        if self.app.proposer(self.round) == self.app.id() {
            match valid {
                Some((round, value)) =>
            }
        } else {
            // Schedule!
        }
    }*/
}

impl<A: App<B>, B: Hashable> Future for Tendermint<A, B> {
    type Output = Broadcast<B>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Broadcast<B>> {
        todo!()
    }
}
