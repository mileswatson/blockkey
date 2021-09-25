use async_trait::async_trait;
use tokio::sync::mpsc::{channel, Receiver, Sender};

#[async_trait]
pub trait Actor<Input, Output = Input>: Send + 'static {
    async fn run(&mut self, mut output: Receiver<Input>, input: Sender<Output>) -> Status;
}

#[derive(PartialEq, Eq)]
pub enum Status {
    Completed,
    Stopped,
    Failed,
}

pub async fn connect<A, B>(mut actor_1: impl Actor<A, B>, mut actor_2: impl Actor<B, A>) -> bool {
    let (s1, r1) = channel(10);
    let (s2, r2) = channel(10);

    let results = tokio::join!(actor_1.run(r1, s2), actor_2.run(r2, s1));

    use Status::*;

    matches!(
        results,
        (Completed, Completed) | (Completed, Stopped) | (Stopped, Completed)
    )
}
