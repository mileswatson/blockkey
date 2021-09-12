use futures::future::select_all;
use futures::FutureExt;
use tokio::time::Instant;

use tokio::time::{sleep_until, Duration};

struct Timeout<T> {
    time: Instant,
    value: T,
}

pub struct TimeoutManager<T> {
    timeouts: Vec<Timeout<T>>,
}

impl<T> TimeoutManager<T> {
    pub fn new() -> TimeoutManager<T> {
        TimeoutManager {
            timeouts: Vec::new(),
        }
    }

    pub fn add(&mut self, value: T, delay: Duration) {
        self.timeouts.push(Timeout {
            value,
            time: Instant::now() + delay,
        });
    }

    pub async fn get_next(&mut self) -> T {
        let futures = self
            .timeouts
            .iter()
            .map(|timeout| sleep_until(timeout.time).boxed());
        let (_, idx, _) = select_all(futures).await;
        self.timeouts.remove(idx).value
    }
}
