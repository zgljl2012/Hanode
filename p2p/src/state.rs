use std::collections::HashMap;
use libp2p::PeerId;
use crate::peer::Peer;


#[derive(Debug, Clone)]
pub struct NodeState {
    pub peers: HashMap<PeerId, Peer>
}

impl NodeState {
    pub fn new() -> Self {
        Self {
            peers: HashMap::new(),
        }
    }
}
