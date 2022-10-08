use std::{collections::HashSet};

use libp2p::{PeerId, Multiaddr};


#[derive(Debug, Clone)]
pub struct Peer {
    pub id: PeerId,
    pub hostname: String,
    pub host_mac: String,
    pub addrs: HashSet<Multiaddr>,
}
