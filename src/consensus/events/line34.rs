use std::time::Duration;

use crate::{
    consensus::{App, Error, FunctionCall, Step, Tendermint},
    crypto::hashing::Hashable,
};

impl<A: App<B>, B: Hashable + Clone + Eq> Tendermint<A, B> {
    // Schedules a timeout foir the prevote stage
    pub fn line34(&mut self) -> Result<bool, Error> {
        if self.line34_check() {
            // schedule OnTimeoutPrevote(h_p, round_p)
            self.timeouts.add(
                FunctionCall::PrevoteTimeout {
                    height: self.height,
                    round: self.current.round,
                },
                Duration::from_millis(1000),
            );

            // prevent function from triggering again
            self.current.step = Step::Prevote {
                timeout_scheduled: true,
            };
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn line34_check(&self) -> bool {
        // while step_p = prevote for the first time
        if !matches!(
            self.current.step,
            Step::Prevote {
                timeout_scheduled: false
            }
        ) {
            return false;
        }

        let total: u64 = self
            .log
            .get_current()
            // upon <Prevote, ...>
            .prevotes
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
