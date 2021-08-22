use crate::{
    consensus::{App, Broadcast, Error, Prevote, Step, Tendermint},
    crypto::hashing::Hashable,
};

impl<A: App<B>, B: Hashable + Clone + Eq> Tendermint<A, B> {
    /// Recieves a proposal with 2f+1 prevotes and prevotes according to whether the value is valid.
    pub async fn line28(&mut self) -> Result<bool, Error> {
        match self.line28_check() {
            None => Ok(false),
            Some((v, valid_round)) => {
                // if valid(v) & (lockedRound <= vr || lockedValue = v)
                let vote_id = if self.app.validate_block(v)
                    && self
                        .locked
                        .as_ref()
                        .map(|x| x.round <= valid_round || &x.value == v)
                        .unwrap_or(true)
                {
                    // id(v)
                    Some(v.hash())
                } else {
                    // nil
                    None
                };

                // broadcast <prevote, h_p, round_p, _>
                let prevote = Prevote::new(self.height, self.current.round, vote_id);
                self.broadcast(Broadcast::Prevote(self.app.sign(prevote)))
                    .await?;

                // step_p <- prevote
                self.current.step = Step::prevote();

                Ok(true)
            }
        }
    }

    fn line28_check(&self) -> Option<(&B, u64)> {
        // while step_p = propose
        if !self.current.step.is_propose() {
            return None;
        }

        let proposer = self.app.proposer(self.height, self.current.round);

        let messages = self.log.get_current();
        messages
            // upon <proposal, ...>
            .proposals
            .iter()
            // from proposer(h_p, round_p)
            .filter(|contract| contract.signee.hash() == proposer)
            .map(|contract| &contract.content)
            .filter_map(|proposal| {
                // upon <..., h_p, round_p, v, vr> where (vr >= 0 ^ vr < round_p)
                proposal.valid_round.and_then(|vr| {
                    if vr < self.current.round
                        && proposal.height == self.height
                        && proposal.round == self.current.round
                    {
                        Some((&proposal.proposal, vr))
                    } else {
                        None
                    }
                })
            })
            // AND 2f+1 <prevote, ...>
            .find(|(proposal, valid_round)| {
                let id = proposal.hash();
                let total_weight = messages
                    .prevotes
                    .iter()
                    .map(|contract| {
                        (
                            self.current.voting_weight(contract.signee.hash()),
                            &contract.content,
                        )
                    })
                    // <_, h_p, vr, id(v)>
                    .filter(|(_, prevote)| {
                        prevote.height == self.height
                            && prevote.round == *valid_round
                            && prevote.id == Some(id)
                    })
                    .map(|(weight, _)| weight)
                    .sum::<u64>();
                total_weight > 2 * self.current.voting_third
            })
    }
}
