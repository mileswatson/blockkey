mod app;
mod types;

pub use app::*;
pub use types::*;

struct Tendermint<A: App<B>, B> {
    height: u64,
    round: u64,
    step: Step,
    app: A,
    locked: Option<(u64, B)>,
    valid: Option<(u64, B)>,
}

impl<A: App<B>, B> Default for Tendermint<A, B> {
    fn default() -> Self {
        Tendermint {
            height: 0,
            round: 0,
            step: Step::Propose,
            app: A::default(),
            locked: None,
            valid: None,
        }
    }
}
