use async_trait::async_trait;
use async_std::{task};
use log::{warn, info, error, debug};
use futures::{
    prelude::{stream::StreamExt},
    select,
};
use libp2p::{
    core,
    floodsub::{self, Floodsub, FloodsubEvent},
    mdns::{Mdns, MdnsConfig, MdnsEvent},
    ping::{Ping, PingConfig, self},
    swarm::SwarmEvent,
    identity,
    NetworkBehaviour, Swarm, PeerId, Multiaddr,
};
use std::{error::Error, time::Duration, collections::HashSet};
use crate::{message::{Message, MessageType}, lifecycle::NodeLifecycleHooks, peer::{Peer, PeerStatus}};
use futures::channel::mpsc;

pub type Sender<T> = mpsc::UnboundedSender<T>;
pub type Receiver<T> = mpsc::UnboundedReceiver<T>;

/// Node
/// Test node: libp2p-lookup direct --address /ip4/127.0.0.1/tcp/32000
pub struct Node {
    pub key: core::identity::Keypair,
    pub peer_id: PeerId,
    db: sled::Db,
    port: Option<u16>,
    bootnode: Option<String>,
    swarm: Swarm<MyBehaviour>,
    floodsub_topic: floodsub::Topic,
    message_receiver:Box<Receiver<Message>>,
    hooks: Box<dyn NodeLifecycleHooks + Send + Sync>,
}

#[derive(Debug, Clone)]
pub struct NodeBehaviourOptions {
    pub port: Option<u16>,
    pub bootnode: Option<String>
}

// NodeBehaviour
#[async_trait]
pub trait NodeBehaviour {
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
    ping: ping::Behaviour,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
enum OutEvent {
    Floodsub(FloodsubEvent),
    Mdns(MdnsEvent),
    Ping(ping::Event),
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

impl From<ping::Event> for OutEvent {
    fn from(v: ping::Event) -> Self {
        Self::Ping(v)
    }
}


#[derive(strum_macros::Display)]
pub enum NodeStateKey {
    NodeLocalKey,
    NodePeersKey,
}

fn peer_db_key(id: &PeerId) -> String {
    format!("{}${}", NodeStateKey::NodePeersKey, id.to_base58())
}

impl Node {
    fn get_peer(&self, id: &PeerId) -> Option<Peer> {
        match self.db.get(peer_db_key(id)) {
            Ok(v) => match v {
                Some(v) => match String::from_utf8(v.to_vec()) {
                    Ok(v) => Some(Peer::from(v)),
                    Err(_) => None,
                },
                None => None
            },
            Err(_) => None
        }
    }

    fn peer_connected(&mut self, id: PeerId, addr: Multiaddr) {
        // hooks
        self.hooks.on_peer_connection(id, addr.clone());
        // create peer
        let mut peer = match self.get_peer(&id) {
            Some(peer) => peer,
            None => Peer{
                id: id.to_base58(),
                hostname: "".to_string(),
                host_mac: "".to_string(),
                addrs: HashSet::new(),
                status: PeerStatus::Connected,
            }
        };
        peer.status = PeerStatus::Connected;
        peer.addrs.insert(addr);
        // add to floodsub
        self.swarm
            .behaviour_mut()
            .floodsub
            .add_node_to_partial_view(id);
        match self.db.insert(peer_db_key(&id), peer.to_string().as_bytes()) {
            Ok(_) => {},
            Err(e) => error!("Failed to insert peer: {:?}", e)
        };
    }

    fn peer_disconnected(&mut self, id: PeerId) {
        match self.get_peer(&id) {
            Some(p) => {
                let mut p = p.clone();
                p.status = PeerStatus::Disconnected;
                match self.db.insert(peer_db_key(&id), p.to_string().as_bytes()) {
                    Ok(_) => {},
                    Err(e) => error!("Failed to set peer's status to disconnected: {:?}", e)
                }
            },
            None => return
        };
    }

    fn list_peers(&self) -> Vec<Peer> {
        let mut r = self.db.scan_prefix(NodeStateKey::NodePeersKey.to_string());
        let mut peers: Vec<Peer> = Vec::new();
        while let Some(cur) = r.next() {
            match cur {
                Ok((_, value)) => {
                    let v = Peer::from(String::from_utf8(value.to_vec()).unwrap());
                    peers.push(v);
                },
                Err(_) => {},
            }
        }
        peers
    }

    async fn dial(&mut self, peer: &Peer) {
        for to_dial in peer.addrs.iter() {
            match self.swarm.dial(to_dial.clone()) {
                Ok(_) => {debug!("Dialed {} {:?}", peer.id, to_dial)}
                Err(e) => { error!("Error connecting: {:?}", e) }
            }
        }
    }

    pub async fn new(receiver: Box<Receiver<Message>>, hooks: Box<dyn NodeLifecycleHooks + Send + Sync>, db: sled::Db, opts: NodeBehaviourOptions) -> Result<Node, Box<dyn Error>> {
        // Create or load a random secret key
        let mut secret = identity::secp256k1::SecretKey::generate();
        // Check if exists local key in database
        if let Some(k) = db.get(NodeStateKey::NodeLocalKey.to_string() as String)? {
            secret = identity::secp256k1::SecretKey::from_bytes(k)?;
            debug!("Found local key in database");
        } else {
            // Store the local key in database
            db.insert(NodeStateKey::NodeLocalKey.to_string() as String, &secret.to_bytes())?;
        }
        let local_key = identity::Keypair::Secp256k1(identity::secp256k1::Keypair::from(secret));
        // let local_key = identity::Keypair::generate_ed25519();
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
                ping: Ping::new(PingConfig::new().with_interval(Duration::from_secs(5)).with_keep_alive(true)),
            };
            behaviour.floodsub.subscribe(floodsub_topic.clone());
            Swarm::new(transport, behaviour, local_peer_id)
        };
        Ok(Node {
            swarm,
            db: db,
            port: opts.port,
            key: local_key,
            peer_id: local_peer_id,
            floodsub_topic,
            message_receiver: receiver,
            bootnode: opts.bootnode,
            hooks,
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
            }
        }
        // dial to all peers
        let peers = self.list_peers();
        for peer in peers {
            self.dial(&peer).await;
        }
        // Listen on all interfaces and whatever port the OS assigns
        let port = match self.port {
            Some(p) => p,
            None => 0 as u16
        };
        match self.swarm.listen_on(format!("/ip4/0.0.0.0/tcp/{}", port).parse()?) {
            Ok(listener) => {
                info!("Listening on {:?}", listener);
            },
            Err(e) => {
                panic!("Error listening: {:?}", e);
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
                    SwarmEvent::Behaviour(OutEvent::Ping(
                        event
                    )) => {
                        debug!("Received ping from {:?}", event.peer.to_base58());
                    }
                    SwarmEvent::Behaviour(OutEvent::Mdns(
                        MdnsEvent::Discovered(list)
                    )) => {
                        for (peer, addr) in list {
                            // save peer
                            self.peer_connected(peer, addr.clone());
                            info!("Discovered {:?}", peer);
                        }
                    }
                    SwarmEvent::Behaviour(OutEvent::Mdns(MdnsEvent::Expired(
                        list
                    ))) => {
                        for (peer, _) in list {
                            // save peer
                            self.peer_disconnected(peer);
                            // remove peer
                            if !self.swarm.behaviour_mut().mdns.has_node(&peer) {
                                self.swarm
                                    .behaviour_mut()
                                    .floodsub
                                    .remove_node_from_partial_view(&peer);
                            }
                        }
                    },
                    SwarmEvent::ConnectionEstablished{
                        peer_id,
                        endpoint,
                        num_established: _,
                        concurrent_dial_errors: _,
                    } => {
                        let remote_addr = endpoint.get_remote_address();
                        self.peer_connected(peer_id, remote_addr.clone());
                        info!("Connection established: {:?} {:?}", peer_id, remote_addr);
                    }
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
}