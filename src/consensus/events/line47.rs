use std::time::Duration;

use crate::{
    consensus::{App, Error, Tendermint, Timeouts},
    crypto::hashing::Hashable,
};

impl<A: App<B>, B: Hashable + Clone + Eq> Tendermint<A, B> {
    // Schedules a timeout foir the precommit stage
    pub fn line47(&mut self) -> Result<bool, Error> {
        if self.line47_check() {
            // schedule OnTimeoutPrecommit(h_p, round_p)
            self.timeouts.add(
                Timeouts::Precommit {
                    height: self.height,
                    round: self.current.round,
                },
                Duration::from_millis(1000),
            );

            // prevent function from triggering again
            self.current.precommit_timeout_scheduled = true;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn line47_check(&self) -> bool {
        // for the first time
        if self.current.precommit_timeout_scheduled {
            return false;
        }

        let total: u64 = self
            .log
            .get_current()
            // upon <precommit, ...>
            .precommits
            .iter()
            // upon <_, h_p, round_p, *>
            .filter_map(|contract| {
                let content = &contract.content;
                if content.height == self.height && content.round == self.current.round {
                    Some(self.current.voting_weight(contract.signee.hash()))
                } else {
                    None
                }
            })
            .sum();

        // 2f+1
        total > 2 * self.current.voting_third
    }
}
