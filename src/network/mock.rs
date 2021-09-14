use std::error::Error;

use async_channel::{unbounded, Receiver, Sender};
use async_trait::async_trait;
use tokio::sync::mpsc;

use super::{Network, Node};

pub struct MockNetwork<M> {
    sender: Sender<M>,
    receiver: Receiver<M>,
}

#[async_trait(?Send)]
impl<M: 'static> Network<M> for MockNetwork<M> {
    fn new() -> Self {
        let (sender, receiver) = unbounded();
        MockNetwork { sender, receiver }
    }

    async fn create_node(&mut self) -> Result<Box<dyn Node<M>>, Box<dyn Error>> {
        Ok(Box::new(MockNode {
            sender: self.sender.clone(),
            receiver: self.receiver.clone(),
        }))
    }
}

pub struct MockNode<M> {
    sender: Sender<M>,
    receiver: Receiver<M>,
}

#[async_trait(?Send)]
impl<M> Node<M> for MockNode<M> {
    async fn run(
        &mut self,
        incoming: mpsc::Sender<M>,
        mut outgoing: mpsc::Receiver<M>,
    ) -> Result<(), ()> {
        loop {
            tokio::select! {
                sending = outgoing.recv() => {
                    match sending {
                        None => return Err(()),
                        Some(block) => self.sender.send(block).await.map_err(|_| {})?
                    }
                }
                receiving = self.receiver.recv() => {
                    match receiving {
                        Err(_) => return Err(()),
                        Ok(block) => incoming.send(block).await.map_err(|_| {})?
                    }
                }
            }
        }
    }
}
