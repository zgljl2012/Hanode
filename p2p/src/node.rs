use libp2p::{
    core,
    PeerId,
    identity
};

pub struct Node {
    pub key: core::identity::Keypair,
    pub peer_id: PeerId,
}

impl Node {
    pub fn new() -> Node {
        // Create a random PeerId
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());
        Node {
            key: local_key,
            peer_id: local_peer_id,
        }
    }
}