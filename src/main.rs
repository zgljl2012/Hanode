use p2p::node::{Sender, Receiver};
use p2p::{node::NodeBehaviour, message::Message};
use p2p::message;
use std::error::Error;
use async_std::{io};
use futures::{
    prelude::{stream::StreamExt, *},
    select,
};
use futures::channel::mpsc;

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (mut sender, mut receiver) = mpsc::unbounded::<Message>();
    async fn run<'a>(receiver: &mut Receiver<Message>) -> Result<(), Box<dyn Error>> {
        let mut node = p2p::node::Node::new(receiver).await?;
        match node.start().await {
            Ok(_ok) => print!("Success"),
            Err(err) => print!("Error: {}", err)
        };
        Ok(())
    }
    async fn input(sender: &mut Sender<Message>) -> Result<(), Box<dyn Error>>  {
        // // Read full lines from stdin
        let mut stdin = io::BufReader::new(io::stdin()).lines().fuse();
        loop {
            select! {
                line = stdin.select_next_some() => 
                    sender.send(message::Message::from(line.expect("Stdin not to close").to_string()))
                    .await?,
            }
        }
    }
    let (a_, b_) = futures::join!(run(&mut receiver), input(&mut sender));
    match a_ {
        Ok(_) => println!("Success"),
        Err(err) => println!("Error: {}", err)
    }
    match b_ {
        Ok(_) => println!("Success"),
        Err(err) => println!("Error: {}", err)
    }
    Ok(())
}
