use std::error::Error;

use async_trait::async_trait;
use tokio::sync::broadcast::{channel, Receiver, Sender};
use tokio::sync::mpsc;

use super::{Network, Node, Status};

pub struct MockNetwork<M> {
    sender: Sender<M>,
}

#[async_trait]
impl<M: Clone + Send> Network<M, MockNode<M>> for MockNetwork<M> {
    fn new() -> Self {
        let (sender, _) = channel(100);
        MockNetwork { sender }
    }

    async fn create_node(&mut self) -> Result<MockNode<M>, Box<dyn Error>> {
        Ok(MockNode {
            sender: self.sender.clone(),
            receiver: self.sender.subscribe(),
        })
    }
}

pub struct MockNode<M> {
    sender: Sender<M>,
    receiver: Receiver<M>,
}

#[async_trait]
impl<M: Clone + Send> Node<M> for MockNode<M> {
    async fn run(&mut self, incoming: mpsc::Sender<M>, mut outgoing: mpsc::Receiver<M>) -> Status {
        loop {
            tokio::select! {
                sending = outgoing.recv() => {
                    match sending {
                        None => return Status::Stopped,
                        Some(block) => if self.sender.send(block).is_err() {
                            return Status::Failed
                        }
                    }
                }
                receiving = self.receiver.recv() => {
                    match receiving {
                        Err(_) => return Status::Failed,
                        Ok(block) => if incoming.send(block).await.is_err() {
                            return Status::Stopped
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use futures::future::join_all;

    use super::MockNetwork;
    use crate::network::{connect, Network, Node, Status};

    struct MockApp {
        num: u32,
        complete_on: u32,
    }

    #[async_trait]
    impl Node<u32> for MockApp {
        async fn run(
            &mut self,
            incoming: tokio::sync::mpsc::Sender<u32>,
            mut outgoing: tokio::sync::mpsc::Receiver<u32>,
        ) -> Status {
            if self.num == 0 && incoming.send(self.num).await.is_err() {
                return Status::Stopped;
            }
            loop {
                match outgoing.recv().await {
                    None => return Status::Stopped,
                    Some(block) => {
                        println!("{} received {}", self.num, block);
                        if block == self.complete_on {
                            return Status::Completed;
                        } else if block == self.num && incoming.send(self.num + 1).await.is_err() {
                            return Status::Stopped;
                        }
                    }
                }
            }
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    pub async fn test_valid() {
        let mut network = MockNetwork::<u32>::new();
        let nodes = {
            let mut v = vec![];
            for num in 0..5 {
                v.push((
                    MockApp {
                        num,
                        complete_on: 5,
                    },
                    network.create_node().await.unwrap(),
                ))
            }
            v.into_iter().map(|(a, n)| connect(a, n))
        };
        assert!(join_all(nodes)
            .await
            .iter()
            .all(|x| *x == Status::Completed));
    }
}
