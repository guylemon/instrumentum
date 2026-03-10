use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "name")]
    pub tool_name: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ToolCall {
    pub id: Option<String>,
    pub function: Function,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Function {
    pub name: String,
    pub arguments: serde_json::Value,
}

impl Message {
    #[must_use]
    pub fn new(role: &str, content: &str) -> Self {
        Message {
            role: role.to_string(),
            content: content.to_string(),
            tool_calls: None,
            tool_call_id: None,
            tool_name: None,
        }
    }
}
