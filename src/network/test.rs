use std::time::Duration;

use async_trait::async_trait;
use futures::future::join_all;

use crate::{
    actor::{connect, Actor, Status},
    network::Network,
};

use super::Node;

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
        println!("MockApp {} starting!", self.num);
        if self.num == 0 {
            if output.send(self.num + 1).await.is_ok() {
                println!("{} sent {}", self.num, self.num + 1);
            } else {
                return Status::Stopped;
            }
        }
        loop {
            match input.recv().await {
                None => return Status::Stopped,
                Some(block) => {
                    println!("{} received {}", self.num, block);
                    if block == self.complete_on {
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        return Status::Completed;
                    } else if block == self.num {
                        if output.send(self.num + 1).await.is_ok() {
                            println!("{} sent {}", self.num, self.num + 1);
                        } else {
                            return Status::Stopped;
                        }
                    }
                }
            }
        }
    }
}

pub async fn test_network<A: Node<u32>>(mut network: impl Network<A, u32>) {
    const NUM_NODES: u32 = 5;
    let nodes = {
        let mut v = vec![];
        for num in 0..NUM_NODES {
            v.push((
                MockApp {
                    num,
                    complete_on: NUM_NODES,
                },
                network.create_node().await.unwrap(),
            ))
        }
        println!("Connecting...");
        join_all(
            v.iter_mut()
                .map(|(_, node)| node.wait_for_connections(NUM_NODES - 1)),
        )
        .await;
        println!("Done connecting.");
        println!("Waiting...");
        println!("Done waiting.");
        v.into_iter().map(|(a, n)| tokio::spawn(connect(a, n)))
    };
    assert!(join_all(nodes).await.into_iter().all(|x| x.unwrap()));
}
