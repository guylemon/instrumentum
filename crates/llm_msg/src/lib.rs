use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Message {
    pub role: String,
    pub content: String,
}

impl Message {
    #[must_use]
    pub fn new(role: &str, content: &str) -> Self {
        Message {
            role: role.to_string(),
            content: content.to_string(),
        }
    }
}
