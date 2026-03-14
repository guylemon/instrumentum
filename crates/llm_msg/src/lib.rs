use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Assistant,
    System,
    Tool,
    User,
}

impl FromStr for Role {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "assistant" => Ok(Role::Assistant),
            "system" => Ok(Role::System),
            "tool" => Ok(Role::Tool),
            "user" => Ok(Role::User),
            other => Err(format!("Invalid LLM message role: {other}").into()),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Message {
    pub role: Role,
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
    pub fn new(role: Role, content: &str) -> Self {
        Message {
            role,
            content: content.to_string(),
            tool_calls: None,
            tool_call_id: None,
            tool_name: None,
        }
    }
}
