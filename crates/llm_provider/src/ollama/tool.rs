use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Tool {
    pub r#type: String,
    pub function: FunctionDef,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}
