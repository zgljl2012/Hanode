use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::peer::Peer;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeState {
    pub peers: HashMap<String, Peer>
}

impl NodeState {
    pub fn new() -> Self {
        Self {
            peers: HashMap::new(),
        }
    }
}
