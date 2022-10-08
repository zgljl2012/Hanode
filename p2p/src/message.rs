use std::fmt::{Display, self};

#[derive(Debug,Clone)]
pub enum MessageType {
    Text,
    ListPeers,
    Stop
}

impl Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MessageType::Text => write!(f, "text"),
            MessageType::ListPeers => write!(f, "listpeers"),
            MessageType::Stop => write!(f, "stop")
        }
    }
}

#[derive(Debug,Clone)]
pub struct Message {
    pub type_: MessageType,
    pub message: String
}

impl Message {
    pub fn from(s: String) -> Message {
        Message {
            type_: MessageType::Text,
            message: s
        }
    }

    pub fn stop_message() -> Message {
        Message {
            type_: MessageType::Stop,
            message: String::new()
        }
    }

    pub fn list_peers_message() -> Message {
        Message {
            type_: MessageType::ListPeers,
            message: String::new()
        }
    }
}
