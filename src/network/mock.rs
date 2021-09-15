use std::error::Error;

use async_trait::async_trait;
use tokio::sync::broadcast::{channel, Receiver, Sender};
use tokio::sync::mpsc;

use super::{Network, Node};

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
                        Some(block) => { self.sender.send(block).map_err(|_| {})?; }
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
