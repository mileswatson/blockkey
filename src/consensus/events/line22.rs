use crate::{
    consensus::{App, Broadcast, Error, Prevote, Step, Tendermint},
    crypto::hashing::Hashable,
};

impl<A: App<B>, B: Hashable + Clone + Eq> Tendermint<A, B> {
    /// Receives a proposal, and votes according to whether the value is valid.
    pub async fn line22(&mut self) -> Result<bool, Error> {
        match self.line22_check() {
            None => Ok(false),
            Some(v) => {
                // if valid(v) ^ (lockedRound_p = -1 || lockedValue_p = v)
                let vote_id = if self.app.validate_block(v)
                    && self.locked.as_ref().map(|x| &x.value == v).unwrap_or(true)
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

    /// Returns a proposal by the current proposer (or None if not found).
    fn line22_check(&self) -> Option<&B> {
        // while Step_p = propose
        if !self.current.step.is_propose() {
            return None;
        }

        let proposer = self.app.proposer(self.current.round);
        self.log
            .get_current()
            // Upon <proposal, ...>
            .proposals
            .iter()
            // From proposer(hp, round)
            .filter(|contract| contract.signee.hash() == proposer)
            .map(|contract| &contract.content)
            // Where <..., h_p, round_p, v, -1>
            .find(|proposal| {
                (proposal.height, proposal.round, proposal.valid_round)
                    == (self.height, self.current.round, None)
            })
            // Return v
            .map(|contract| &contract.proposal)
    }
}
