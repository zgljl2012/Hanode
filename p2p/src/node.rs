use async_trait::async_trait;
use async_std::{task};
use log::{warn, info, error};
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
use std::{error::Error, collections::{HashMap, HashSet}};
use crate::{message::{Message, MessageType}, lifecycle::NodeLifecycleHooks, peer::Peer};
use futures::channel::mpsc;

pub type Sender<T> = mpsc::UnboundedSender<T>;
pub type Receiver<T> = mpsc::UnboundedReceiver<T>;

// Node
pub struct Node {
    pub key: core::identity::Keypair,
    pub peer_id: PeerId,
    bootnode: Option<String>,
    swarm: Swarm<MyBehaviour>,
    floodsub_topic: floodsub::Topic,
    message_receiver:Box<Receiver<Message>>,
    hooks: Box<dyn NodeLifecycleHooks + Send + Sync>,
    peers: HashMap<PeerId, Peer>
}

#[derive(Debug, Clone)]
pub struct NodeBehaviourOptions {
    pub bootnode: Option<String>
}

// NodeBehaviour
#[async_trait]
pub trait NodeBehaviour {
    fn peers(&self) -> Vec<Peer>;
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

impl Node {
    pub async fn new(receiver: Box<Receiver<Message>>, hooks: Box<dyn NodeLifecycleHooks + Send + Sync>, opts: NodeBehaviourOptions) -> Result<Node, Box<dyn Error>> {
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
            swarm,
            key: local_key,
            peer_id: local_peer_id,
            floodsub_topic,
            message_receiver: receiver,
            bootnode: opts.bootnode,
            hooks,
            peers: HashMap::new(),
        })
    }
}

#[async_trait]
impl NodeBehaviour for Node {
    async fn start(&mut self) -> Result<(), Box<dyn Error>> {
        info!("Local peer id: {:?}", self.peer_id);

        // Reach out to another node if specified
        if let Some(ref bootnode) = self.bootnode {
            let to_dial = bootnode;
            let addr: Multiaddr = to_dial.parse()?;
            match self.swarm.dial(addr) {
                Ok(_) => {info!("Dialed {:?}", to_dial)}
                Err(e) => { error!("Error connecting: {:?}", e) }
            };
        }

        // Listen on all interfaces and whatever port the OS assigns
        match self.swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?) {
            Ok(listener) => {
                info!("Listening on {:?}", listener);
            },
            Err(e) => {
                error!("Error listening: {:?}", e);
            },
        };

        // Kick it off
        loop {
            let mut stop_flag = false;
            select! {
                msg = self.message_receiver.next() => match msg {
                    Some(msg) => {
                        info!("You input message: {:?}, send to everyone", msg.message);
                        match msg.type_ {
                            MessageType::Text => {
                                self.swarm.behaviour_mut()
                                    .floodsub
                                    .publish(self.floodsub_topic.clone(), msg.message.clone().as_bytes());
                            },
                            MessageType::Stop => {
                                warn!("Stopping p2p node...");
                                stop_flag = true;
                            },
                            MessageType::ListPeers => {
                                info!("{:?}", self.peers());
                            },
                            // _ => warn!("Unknown message type: {:?}", msg.type_.to_string())
                        }
                    },
                    None => {
                        error!("Error input: None");
                    }
                },
                event = self.swarm.select_next_some() => match event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        info!("Listening on {:?}", address);
                    }
                    SwarmEvent::Behaviour(OutEvent::Floodsub(
                        FloodsubEvent::Message(message)
                    )) => {
                        info!(
                            "Received: '{:?}' from {:?}",
                            String::from_utf8_lossy(&message.data),
                            message.source
                        );
                    }
                    SwarmEvent::Behaviour(OutEvent::Mdns(
                        MdnsEvent::Discovered(list)
                    )) => {
                        for (peer, addr) in list {
                            self.hooks.on_peer_connection(peer, addr.clone());
                            if !self.peers.contains_key(&peer) {
                                self.peers.insert(peer, Peer {
                                    id: peer,
                                    hostname: "".to_string(),
                                    host_mac: "".to_string(),
                                    addrs: HashSet::new(),
                                });
                            }
                            let mut p = self.peers.get(&peer).unwrap().clone();
                            let mut addrs = p.addrs.clone();
                            addrs.insert(addr.clone());
                            p.addrs = addrs;
                            self.peers.insert(p.id, p);
                            info!("Discovered {:?}", peer);
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
            if stop_flag {
                break;
            }
        }
        info!("Stopped");
        self.hooks.on_stopped();
        Ok(())
    }

    fn peers(&self) -> Vec<Peer> {
        self.peers.clone().into_values().collect()
    }
}