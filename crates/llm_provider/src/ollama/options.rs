use serde::{Deserialize, Serialize};

/// Runtime model options
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Options {
    /// The temperature of the model. Increasing the temperature makes the model answer more
    /// creatively (higher = more random/diverse; lower = more deterministic/focused). Default:
    /// 0.8
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Sets the size of the context window used to generate the next token (prompt + history the
    /// model can see). Default: 4096
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_ctx: Option<i32>,

    /// Works together with top-k (nucleus sampling). A higher value leads to more diverse text; a
    /// lower value generates more focused and conservative text. Default: 0.9
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// Reduces the probability of generating nonsense. A higher value gives more diverse answers; a lower value is more conservative. Default: 40
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i32>,

    /// Sets the random number seed to use for generation. Setting this to a specific number makes
    /// the model generate the same text for the same prompt. Default: 0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i32>,

    /// Maximum number of tokens to predict when generating text (equivalent to `max_tokens`).
    /// Default: -1 (no limit).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_predict: Option<i32>,

    /// Sets how strongly to penalize repetitions. A higher value penalizes repetitions more
    /// strongly; a lower value is more lenient. Default: 1.1
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeat_penalty: Option<f32>,

    /// Sets the stop sequences to use. When any of these patterns is encountered, the LLM will
    /// stop generating text and return (supports multiple values).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
}

impl Options {
    /// Serializes to an empty options object for default Ollama behavior.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    /// Sets temperature to 0.0, and context size to 64k tokens (reasonable for agents)
    pub fn recommended() -> Self {
        Self::default().temperature(0.0).num_ctx(64000)
    }
}

impl Options {
    #[must_use]
    pub fn temperature(mut self, v: f32) -> Self {
        self.temperature = Some(v);
        self
    }
    #[must_use]
    pub fn num_ctx(mut self, v: i32) -> Self {
        self.num_ctx = Some(v);
        self
    }
    #[must_use]
    pub fn top_p(mut self, v: f32) -> Self {
        self.top_p = Some(v);
        self
    }
    #[must_use]
    pub fn top_k(mut self, v: i32) -> Self {
        self.top_k = Some(v);
        self
    }
    #[must_use]
    pub fn seed(mut self, v: i32) -> Self {
        self.seed = Some(v);
        self
    }
    #[must_use]
    pub fn num_predict(mut self, v: i32) -> Self {
        self.num_predict = Some(v);
        self
    }
    #[must_use]
    pub fn repeat_penalty(mut self, v: f32) -> Self {
        self.repeat_penalty = Some(v);
        self
    }
    #[must_use]
    pub fn stop(mut self, v: Vec<String>) -> Self {
        self.stop = Some(v);
        self
    }
}
