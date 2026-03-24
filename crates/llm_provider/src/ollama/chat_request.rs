use llm_msg::{Message, Role};
use serde::{Deserialize, Serialize};

use crate::ollama::Format;
use crate::ollama::Options;
use crate::ollama::Think;
use crate::ollama::ThinkLevel;
use crate::ollama::Tool;

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    /// Provide a JSON schema to the format field to request structured output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<Format>,

    /// Chat history as an array of messages
    pub messages: Vec<Message>,

    /// Model name
    pub model: String,

    /// Runtime model options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Options>,

    /// Whether to stream model response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// When true, return separate thinking content in addition to message content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub think: Option<Think>,

    /// When true, return separate thinking content in addition to message content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
}

impl ChatRequest {
    /// Build a chat request from model and messages with no options, streaming, or thinking. For
    /// custom options, use `ChatRequestBuilder`.
    pub fn new(model: impl Into<String>, messages: Vec<Message>) -> Self {
        Self {
            format: None,
            messages,
            model: model.into(),
            options: None,
            stream: None,
            think: None,
            tools: None,
        }
    }

    pub fn builder(model: impl Into<String>) -> ChatRequestBuilder {
        ChatRequestBuilder::new(model)
    }
}

#[derive(Debug)]
pub struct ChatRequestBuilder {
    format: Option<Format>,
    messages: Vec<Message>,
    model: String,
    options: Option<Options>,
    stream: Option<bool>,
    think: Option<Think>,
    tools: Option<Vec<Tool>>,
}

impl ChatRequestBuilder {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            format: None,
            messages: Vec::new(),
            model: model.into(),
            options: None,
            stream: None,
            think: None,
            tools: None,
        }
    }
}

impl ChatRequestBuilder {
    /// Append chat messages to chat request
    #[must_use]
    pub fn format(mut self, format: Format) -> Self {
        self.format = Some(format);
        self
    }

    /// Append chat messages to chat request
    #[must_use]
    pub fn messages(mut self, messages: Vec<Message>) -> Self {
        self.messages.extend(messages);
        self
    }

    /// Add a `llm_msg::Message` to the messages array
    #[must_use]
    pub fn add_message(mut self, message: Message) -> Self {
        self.messages.push(message);
        self
    }

    /// Add a `llm_msg::Message` to the messages array with `Role::User`
    #[must_use]
    pub fn add_user_message(mut self, content: impl Into<String>) -> Self {
        let message = Message::new(Role::User, &content.into());
        self.messages.push(message);
        self
    }

    /// Add a `llm_msg::Message` to the messages array with `Role::System`
    #[must_use]
    pub fn add_system_message(mut self, content: impl Into<String>) -> Self {
        let message = Message::new(Role::System, &content.into());
        self.messages.push(message);
        self
    }

    /// Convenience: Set temperature option
    #[must_use]
    pub fn temperature(mut self, temperature: f32) -> Self {
        let opts = self.options.get_or_insert_with(Options::default);
        *opts = opts.clone().temperature(temperature);
        self
    }

    /// Convenience: Set context size option
    #[must_use]
    pub fn num_ctx(mut self, num_ctx: i32) -> Self {
        let opts = self.options.get_or_insert_with(Options::default);
        *opts = opts.clone().num_ctx(num_ctx);
        self
    }

    /// Convenience: Set seed
    #[must_use]
    pub fn seed(mut self, seed: i32) -> Self {
        let opts = self.options.get_or_insert_with(Options::default);
        *opts = opts.clone().seed(seed);
        self
    }

    /// Convenience: Set thinking option
    /// Simple on/off (most common)
    #[must_use]
    pub fn think_enabled(mut self, enabled: bool) -> Self {
        self.think = Some(Think::Bool(enabled));
        self
    }

    /// Convenience: Set thinking option
    /// Fine-grained level (for GPT-OSS etc.)
    #[must_use]
    pub fn think_level(mut self, level: ThinkLevel) -> Self {
        self.think = Some(Think::Level(level));
        self
    }

    /// Convenience: Set thinking option
    /// accept anything that implements Into<Think>
    #[must_use]
    pub fn think<T: Into<Think>>(mut self, value: T) -> Self {
        self.think = Some(value.into());
        self
    }

    /// Include custom model runtime options
    #[must_use]
    pub fn options(mut self, opts: Options) -> Self {
        self.options = Some(opts);
        self
    }

    /// Include custom model runtime options
    #[must_use]
    pub fn stream(mut self, enabled: bool) -> Self {
        self.stream = Some(enabled);
        self
    }

    /// Include custom model runtime options
    #[must_use]
    pub fn tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// ## Errors
    /// Returns error when `ChatRequest.messages` is empty.
    pub fn build(self) -> Result<ChatRequest, Box<dyn std::error::Error>> {
        if self.messages.is_empty() {
            return Err("ChatRequest must contain at least one message"
                .to_string()
                .into());
        }

        Ok(ChatRequest {
            format: self.format,
            messages: self.messages,
            model: self.model,
            options: self.options,
            stream: self.stream,
            think: self.think,
            tools: self.tools,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_request_with_format(format: Format) -> ChatRequest {
        ChatRequest::builder("llama3")
            .add_user_message("Hello")
            .format(format)
            .build()
            .unwrap()
    }

    #[test]
    fn test_builder_with_format_json() {
        let request = build_request_with_format(Format::Json);
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains(r#""format":"json""#));
    }

    #[test]
    fn test_builder_with_format_schema() {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"}
            }
        });
        let request = build_request_with_format(Format::Schema(schema));
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"format\""));
    }
}
