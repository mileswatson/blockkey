use async_trait::async_trait;
use futures::future::join_all;
use tokio::sync::broadcast::*;

#[async_trait]
pub trait Actor<M>: Send + 'static {
    async fn run(&mut self, mut input: Receiver<M>, output: Sender<M>) -> Status;
}

#[derive(PartialEq, Eq)]
pub enum Status {
    Completed,
    Stopped,
    Failed,
}

pub async fn connect<M: Clone + 'static>(mut actors: Vec<Box<dyn Actor<M>>>) -> bool {
    let (output, _) = channel(100);

    let running = actors
        .iter_mut()
        .map(|x| x.run(output.subscribe(), output.clone()));

    let results = join_all(running).await;

    results.contains(&Status::Completed) && !results.contains(&Status::Failed)
}
