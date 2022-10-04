
#[derive(Debug,Clone)]
pub struct Message {
    pub message: String
}

impl Message {
    pub fn from(s: String) -> Message {
        Message {
            message: s
        }
    }
}
