use async_trait::async_trait;
use tokio::sync::mpsc::{channel, Receiver, Sender};

#[async_trait]
pub trait Actor<Input, Output = Input> {
    async fn run(&mut self, mut output: Receiver<Input>, input: Sender<Output>) -> Status;
}

#[derive(PartialEq, Eq)]
pub enum Status {
    Completed,
    Stopped,
    Failed,
}

pub async fn connect<AppInput, AppOutput>(
    mut app: impl Actor<AppInput, AppOutput>,
    mut network: impl Actor<AppOutput, AppInput>,
) -> Status {
    let (s1, r1) = channel(10);
    let (s2, r2) = channel(10);

    let results = tokio::join!(app.run(r1, s2), network.run(r2, s1));

    use Status::*;

    match results {
        (Completed, Completed) | (Completed, Stopped) | (Stopped, Completed) => Status::Completed,
        (Stopped, Stopped) => Stopped,
        (Failed, _) | (_, Failed) => Failed,
    }
}
