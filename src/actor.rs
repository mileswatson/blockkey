use async_trait::async_trait;
use futures::future::join_all;
use tokio::sync::broadcast::*;

#[async_trait]
pub trait Actor<M>: Send + 'static {
    async fn run(
        &mut self,
        mut input: Receiver<ActorEvent<M>>,
        output: Sender<ActorEvent<M>>,
    ) -> Status;
}

#[derive(Clone, Debug)]
pub enum ActorEvent<Message> {
    Send(Message),
    Receive(Message),
    Stop,
}

#[derive(PartialEq, Eq)]
pub enum Status {
    Completed,
    Stopped,
    Failed,
}

pub async fn connect<M: Clone + 'static>(mut actors: Vec<Box<dyn Actor<M>>>) -> bool {
    let (sender, _) = channel(1000);

    let running = actors
        .iter_mut()
        .map(|x| (x, sender.subscribe()))
        .collect::<Vec<_>>()
        .into_iter()
        .map(|(x, input)| x.run(input, sender.clone()));

    let results = join_all(running).await;

    results.contains(&Status::Completed) && !results.contains(&Status::Failed)
}
