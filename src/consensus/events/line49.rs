use crate::{
    consensus::{App, Error, Tendermint},
    crypto::hashing::Hashable,
};

impl<A: App<B>, B: Hashable + Clone + Eq> Tendermint<A, B> {
    pub async fn line49(&mut self) -> Result<bool, Error> {
        match self.line49_check().cloned() {
            Some(b) => {
                // h_p <- h_p + 1
                self.new_height(self.height + 1, Some(b)).await?;

                Ok(true)
            }
            None => Ok(false),
        }
    }

    pub fn line49_check(&self) -> Option<&B> {
        // while decision_p[h_p] = nil is redundant, because if it wasn't nil then h_p would have been incremented

        let messages = self.log.get_current();
        messages
            // upon <proposal, _, r, ...>
            .proposals
            .iter()
            // from proposer(h_p, r)
            .filter(|contract| contract.signee.hash() == self.app.proposer(contract.content.round))
            .map(|contract| &contract.content)
            .filter_map(|proposal| {
                // upon <_, h_p, _, v, *>
                // if valid(v)
                if proposal.height == self.height && self.app.validate_block(&proposal.proposal) {
                    Some((proposal.round, &proposal.proposal))
                } else {
                    None
                }
            })
            // AND 2f+1 <precommit, ...>
            .find(|(r, v)| {
                let id = v.hash();
                let total_weight = messages
                    .precommits
                    .iter()
                    .map(|contract| {
                        (
                            self.voting_weight(contract.signee.hash()),
                            &contract.content,
                        )
                    })
                    // <_, h_p, r, id(v)>
                    .filter(|(_, prevote)| {
                        prevote.height == self.height
                            && prevote.round == *r
                            && prevote.id == Some(id)
                    })
                    .map(|(weight, _)| weight)
                    .sum::<u64>();
                total_weight > self.two_f()
            })
            .map(|(_, v)| v)
    }
}
