// lib
mod ollama;

pub use ollama::ChatRequest;
pub use ollama::ChatRequestBuilder;
pub use ollama::Config;
pub use ollama::FunctionDef;
pub use ollama::Options;
pub use ollama::Tool;

#[derive(Clone)]
pub enum Provider {
    Ollama(ollama::Config),
    Xai { api_key: String, model: String },
}
