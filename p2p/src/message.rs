
#[derive(Debug,Clone)]
pub enum MessageType {
    TEXT,
    STOP
}

#[derive(Debug,Clone)]
pub struct Message {
    pub type_: MessageType,
    pub message: String
}

impl Message {
    pub fn from(s: String) -> Message {
        Message {
            type_: MessageType::TEXT,
            message: s
        }
    }

    pub fn stop_message() -> Message {
        Message {
            type_: MessageType::STOP,
            message: String::new()
        }
    }
}
