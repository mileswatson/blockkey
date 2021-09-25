use async_trait::async_trait;
use futures::future::join_all;

use crate::{
    actor::{connect, Actor, Status},
    network::Network,
};

struct MockApp {
    num: u32,
    complete_on: u32,
}

#[async_trait]
impl Actor<u32> for MockApp {
    async fn run(
        &mut self,
        mut input: tokio::sync::mpsc::Receiver<u32>,
        output: tokio::sync::mpsc::Sender<u32>,
    ) -> Status {
        if self.num == 0 && output.send(self.num).await.is_err() {
            return Status::Stopped;
        }
        loop {
            match input.recv().await {
                None => return Status::Stopped,
                Some(block) => {
                    println!("{} received {}", self.num, block);
                    if block == self.complete_on {
                        return Status::Completed;
                    } else if block == self.num && output.send(self.num + 1).await.is_err() {
                        return Status::Stopped;
                    }
                }
            }
        }
    }
}

pub async fn test_network<A: Actor<u32>>(mut network: impl Network<A, u32>) {
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
        println!("connecting!");
        v.into_iter().map(|(a, n)| tokio::spawn(connect(a, n)))
    };
    assert!(join_all(nodes).await.into_iter().all(|x| x.unwrap()));
}
