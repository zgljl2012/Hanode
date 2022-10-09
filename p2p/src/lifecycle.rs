use std::{collections::{HashSet}, process, sync::{Arc, RwLock}};

use libp2p::{Multiaddr, PeerId};
use log::debug;

use crate::{peer::Peer, state::NodeState};


pub trait NodeLifecycleHooks {
    // on_peer_connection
    fn on_peer_connection(&mut self, peer_id: PeerId, addr: Multiaddr);
    // Trigger this function when the node is destroyed.
    fn on_stopped(&self);
}

#[derive(Debug, Clone)]
pub struct NodeLifecycle {
    state: Arc<RwLock<NodeState>>
}

impl NodeLifecycle {
    pub fn new(state: Arc<RwLock<NodeState>>) -> Box<dyn NodeLifecycleHooks + Send + Sync> {
        Box::new(NodeLifecycle {
            state: state.clone(),
        })
    }
}

impl NodeLifecycleHooks for NodeLifecycle {
    fn on_stopped(&self) {
        debug!("NodeLifecycleHooks on_stopped()");
        // Shutdown when the node is stopped
        process::exit(0);
    }
    fn on_peer_connection(&mut self, peer_id: PeerId, addr: Multiaddr) {
        debug!("NodeLifecycleHooks on_peer_connection({:?})", peer_id);
        let peer_id_ = peer_id.to_base58();
        if !(*self.state.read().unwrap()).peers.contains_key(&peer_id_) {
            (*self.state.write().unwrap()).peers.insert(peer_id_.clone(), Peer {
                id: peer_id_.clone(),
                hostname: "".to_string(),
                host_mac: "".to_string(),
                addrs: HashSet::new(),
            });
        }
        let mut p = (*self.state.read().unwrap()).peers.get(&peer_id_).unwrap().clone();
        let mut addrs = p.addrs.clone();
        addrs.insert(addr.clone());
        p.addrs = addrs;
        (*self.state.write().unwrap()).peers.insert(peer_id_, p);
    }
}
