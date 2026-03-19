use serde::{Deserialize, Serialize};

/// Fine-grained thinking levels
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThinkLevel {
    Low,
    Medium,
    High,
}

/// The `think` parameter — accepts whatever the model expects
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Think {
    /// Boolean on/off (works for Qwen3, most Ollama models)
    Bool(bool),
    /// Level-based control (used by GPT-OSS, some others)
    Level(ThinkLevel),
}
