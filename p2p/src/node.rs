use async_trait::async_trait;
use async_std::{task};
use futures::{
    prelude::{stream::StreamExt},
    select,
};
use libp2p::{
    core,
    floodsub::{self, Floodsub, FloodsubEvent},
    mdns::{Mdns, MdnsConfig, MdnsEvent},
    swarm::SwarmEvent,
    identity,
    NetworkBehaviour, Swarm, PeerId, Multiaddr,
};
use std::error::Error;
use crate::message::Message;
use futures::channel::mpsc;

pub type Sender<T> = mpsc::UnboundedSender<T>;
pub type Receiver<T> = mpsc::UnboundedReceiver<T>;

// Node
pub struct Node<'a> {
    pub key: core::identity::Keypair,
    pub peer_id: PeerId,
    bootnode: Option<String>,
    swarm: Swarm<MyBehaviour>,
    floodsub_topic: floodsub::Topic,
    message_receiver: &'a mut Receiver<Message>
}

#[derive(Debug, Clone)]
pub struct NodeBehaviourOptions {
    pub bootnode: Option<String>
}

// NodeBehaviour
#[async_trait]
pub trait NodeBehaviour {
    // Create a new node
    async fn new(receiver: &mut Receiver<Message>, opts: NodeBehaviourOptions) -> Result<Node, Box<dyn Error>>;
    // Start listening
    async fn start(&mut self) -> Result<(), Box<dyn Error>>;
}

// We create a custom network behaviour that combines floodsub and mDNS.
// Use the derive to generate delegating NetworkBehaviour impl.
#[derive(NetworkBehaviour)]
#[behaviour(out_event = "OutEvent")]
struct MyBehaviour {
    floodsub: Floodsub,
    mdns: Mdns,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
enum OutEvent {
    Floodsub(FloodsubEvent),
    Mdns(MdnsEvent),
}

impl From<MdnsEvent> for OutEvent {
    fn from(v: MdnsEvent) -> Self {
        Self::Mdns(v)
    }
}

impl From<FloodsubEvent> for OutEvent {
    fn from(v: FloodsubEvent) -> Self {
        Self::Floodsub(v)
    }
}

#[async_trait]
impl<'a> NodeBehaviour for Node<'a> {
    async fn new(receiver: &mut Receiver<Message>, opts: NodeBehaviourOptions) -> Result<Node, Box<dyn Error>> {
        // Create a random PeerId
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());
        // Set up an encrypted DNS-enabled TCP Transport over the Mplex and Yamux protocols
        let k2 = local_key.clone();
        let transport = libp2p::development_transport(k2).await?;

        // Create a Floodsub topic
        let floodsub_topic = floodsub::Topic::new("chat");
        // Create a Swarm to manage peers and events
        let swarm = {
            let mdns = task::block_on(Mdns::new(MdnsConfig::default()))?;
            let mut behaviour = MyBehaviour {
                floodsub: Floodsub::new(local_peer_id),
                mdns,
            };
            behaviour.floodsub.subscribe(floodsub_topic.clone());
            Swarm::new(transport, behaviour, local_peer_id)
        };
        Ok(Node {
            swarm: swarm,
            key: local_key,
            peer_id: local_peer_id,
            floodsub_topic: floodsub_topic,
            message_receiver: receiver,
            bootnode: opts.bootnode,
        })
    }

    async fn start(&mut self) -> Result<(), Box<dyn Error>> {
        env_logger::init();
        println!("Local peer id: {:?}", self.peer_id);

        // Reach out to another node if specified
        match self.bootnode {
            Some(ref bootnode) => {
                let to_dial = bootnode;
                let addr: Multiaddr = to_dial.parse()?;
                match self.swarm.dial(addr) {
                    Ok(_) => {println!("Dialed {:?}", to_dial)}
                    Err(e) => { println!("Error connecting: {:?}", e) }
                };
            },
            None => {}, // skip this
        }

        // Listen on all interfaces and whatever port the OS assigns
        match self.swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?) {
            Ok(listener) => {
                println!("Listening on {:?}", listener);
            },
            Err(e) => {
                println!("Error listening: {:?}", e);
            },
        };

        // Kick it off
        loop {
            select! {
                msg = self.message_receiver.next() => match msg {
                    Some(msg) => {
                        println!("You input message: {:?}, send to everyone", msg.message);
                        self.swarm.behaviour_mut()
                            .floodsub
                            .publish(self.floodsub_topic.clone(), msg.message.clone().as_bytes());
                    },
                    None => {
                        println!("Error input: None");
                    }
                },
                event = self.swarm.select_next_some() => match event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Listening on {:?}", address);
                    }
                    SwarmEvent::Behaviour(OutEvent::Floodsub(
                        FloodsubEvent::Message(message)
                    )) => {
                        println!(
                            "Received: '{:?}' from {:?}",
                            String::from_utf8_lossy(&message.data),
                            message.source
                        );
                    }
                    SwarmEvent::Behaviour(OutEvent::Mdns(
                        MdnsEvent::Discovered(list)
                    )) => {
                        for (peer, _) in list {
                            println!("Discovered {:?}", peer);
                            self.swarm
                                .behaviour_mut()
                                .floodsub
                                .add_node_to_partial_view(peer);
                        }
                    }
                    SwarmEvent::Behaviour(OutEvent::Mdns(MdnsEvent::Expired(
                        list
                    ))) => {
                        for (peer, _) in list {
                            if !self.swarm.behaviour_mut().mdns.has_node(&peer) {
                                self.swarm
                                    .behaviour_mut()
                                    .floodsub
                                    .remove_node_from_partial_view(&peer);
                            }
                        }
                    },
                    _ => {}
                }
            }
        }
    }
}