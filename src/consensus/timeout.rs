use futures::future::select_all;
use futures::FutureExt;
use tokio::time::Instant;

use tokio::time::sleep_until;

struct Timeout<A> {
    time: Instant,
    function: Box<dyn FnOnce(&mut A)>,
}

pub struct TimeoutManager<A> {
    timeouts: Vec<Timeout<A>>,
}

impl<A> TimeoutManager<A> {
    pub async fn get_next(&mut self) -> Box<dyn FnOnce(&mut A)> {
        let futures = self
            .timeouts
            .iter()
            .map(|timeout| sleep_until(timeout.time).boxed());
        let (_, idx, _) = select_all(futures).await;
        self.timeouts.remove(idx).function
    }
}
