use crate::{
    consensus::{App, Broadcast, Error, Precommit, Step, Tendermint},
    crypto::hashing::Hashable,
};

impl<A: App<B>, B: Hashable + Clone + Eq> Tendermint<A, B> {
    pub async fn line44(&mut self) -> Result<bool, Error> {
        if self.line44_check() {
            // broadcast <precommit, h_p, round_p, nil>
            let vote = Precommit::new(self.height, self.current.round, None);
            self.broadcast(Broadcast::Precommit(self.app.sign(vote)))
                .await?;

            // step_p <- precommit
            self.current.step = Step::Precommit;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn line44_check(&self) -> bool {
        // while step_p = prevote
        if !self.current.step.is_prevote() {
            return false;
        }

        let total: u64 = self
            .log
            .get_current()
            // upon <Prevote, ...>
            .prevotes
            .iter()
            // upon <_, h_p, round_p, nil>
            .filter_map(|contract| {
                let content = &contract.content;
                if content.height == self.height
                    && content.round == self.current.round
                    && content.id == None
                {
                    Some(self.voting_weight(contract.signee.hash()))
                } else {
                    None
                }
            })
            .sum();

        // 2f+1
        total > self.two_f()
    }
}
