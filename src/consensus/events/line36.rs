use crate::{
    consensus::{App, Broadcast, Error, Precommit, Record, Step, Tendermint},
    crypto::hashing::Hashable,
};

impl<A: App<B>, B: Hashable + Clone + Eq> Tendermint<A, B> {
    pub async fn line36(&mut self) -> Result<bool, Error> {
        match self.line36_check().cloned() {
            Some(b) => {
                if self.current.step.is_prevote() {
                    self.locked = Some(Record {
                        round: self.current.round,
                        value: b.clone(),
                    });

                    let vote = Precommit::new(self.height, self.current.round, Some(b.hash()));

                    self.broadcast(Broadcast::Precommit(self.app.sign(vote)))
                        .await?;

                    self.current.step = Step::Precommit;
                }
                self.valid = Some(Record {
                    round: self.current.round,
                    value: b,
                });
                Ok(true)
            }
            None => Ok(false),
        }
    }

    pub fn line36_check(&self) -> Option<&B> {
        // while step_p >= prevote
        if self.current.step.is_propose() {
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
                // upon <..., h_p, round_p, v, *> while valid(v)
                if proposal.height == self.height
                    && proposal.round == self.current.round
                    && self.app.validate_block(&proposal.proposal)
                {
                    Some(&proposal.proposal)
                } else {
                    None
                }
            })
            // AND 2f+1 <prevote, ...>
            .find(|proposal| {
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
                    // <_, h_p, round_p, id(v)>
                    .filter(|(_, prevote)| {
                        prevote.height == self.height
                            && prevote.round == self.current.round
                            && prevote.id == Some(id)
                    })
                    .map(|(weight, _)| weight)
                    .sum::<u64>();
                total_weight > 2 * self.current.voting_third
            })
    }
}
