use std::{collections::HashSet, fmt::Display};

use libp2p::{Multiaddr};
use log::warn;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PeerStatus {
    Connected,
    Disconnected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub id: String,
    pub hostname: String,
    pub host_mac: String,
    pub addrs: HashSet<Multiaddr>,
    pub status: PeerStatus,
}

impl Display for Peer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string(self) {
            Ok(s) => f.write_str(s.as_str()).unwrap(),
            Err(e) => {
                warn!("Failed to serialize peer: {}", e);
                f.write_str("{}").unwrap()
            },
        };
        Ok(())
    }
}

impl From<String> for Peer {
    fn from(id: String) -> Self {
        match serde_json::from_str(&id) {
            Ok(p) => p,
            Err(e) => {
                warn!("Failed to parse peer id: {}", e);
                Peer {
                    id: "".to_string(),
                    hostname: String::new(),
                    host_mac: String::new(),
                    addrs: HashSet::new(),
                    status: PeerStatus::Disconnected,
                }
            }
        }
    }
}
