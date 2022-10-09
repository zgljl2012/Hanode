use std::{collections::HashSet};

use libp2p::{Multiaddr};
use serde::{Serialize, Deserialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub id: String,
    pub hostname: String,
    pub host_mac: String,
    pub addrs: HashSet<Multiaddr>,
}
