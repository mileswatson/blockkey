use itertools::Itertools;

use crate::{
    consensus::{log::Message, App, Error, Tendermint},
    crypto::hashing::Hashable,
};

impl<A: App<B>, B: Hashable + Clone + Eq> Tendermint<A, B> {
    // Advances the round if enough voters have posted messages
    pub async fn line55(&mut self) -> Result<bool, Error> {
        match self.line55_check() {
            Some(round) => {
                self.start_round(round).await?;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    pub fn line55_check(&self) -> Option<u64> {
        // upon <*, h_p, round, *, *>
        self.log
            .get_current()
            .all()
            .map(|m| match m {
                Message::Proposal(c) => (c.content.round, c.signee.hash()),
                Message::Prevote(c) => (c.content.round, c.signee.hash()),
                Message::Precommit(c) => (c.content.round, c.signee.hash()),
            })
            // with round > round_p
            .filter(|(round, _)| round > &self.current.round)
            .group_by(|(round, _)| *round)
            .into_iter()
            .map(|(round, messages)| {
                (
                    round,
                    messages
                        .into_iter()
                        .map(|(_, signee)| signee)
                        .dedup()
                        .map(|signee| self.voting_weight(signee))
                        .sum::<u64>(),
                )
            })
            // upon f+1
            .find(|(_, total)| *total > self.f())
            .map(|(round, _)| round)
    }
}
