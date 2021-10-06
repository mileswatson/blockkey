use std::time::Duration;

use async_trait::async_trait;
use futures::future::join_all;
use tokio::sync::broadcast::{Receiver, Sender};

use crate::{
    actor::{connect, Actor, ActorEvent, Status},
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
        mut input: Receiver<ActorEvent<u32>>,
        output: Sender<ActorEvent<u32>>,
    ) -> Status {
        println!("MockApp {} starting!", self.num);
        if self.num == 0 {
            output.send(ActorEvent::Send(self.num + 1)).unwrap();
            println!("{} sent {}", self.num, self.num + 1)
        }
        loop {
            match input.recv().await.unwrap() {
                ActorEvent::Receive(block) => {
                    println!("{} received {}", self.num, block);
                    if block == self.complete_on {
                        output.send(ActorEvent::Stop).unwrap();
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        return Status::Completed;
                    } else if block == self.num {
                        output.send(ActorEvent::Send(self.num + 1)).unwrap();
                        println!("{} sent {}", self.num, self.num + 1);
                    }
                }
                ActorEvent::Stop => return Status::Stopped,
                _ => (),
            }
        }
    }
}

pub async fn test_network<A: Node + Actor<u32>>(mut network: impl Network<u32, A>) {
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
        v.into_iter()
            .map(|(a, n)| tokio::spawn(connect(vec![Box::new(a), Box::new(n)])))
    };
    assert!(join_all(nodes).await.into_iter().all(|x| x.unwrap()));
}
