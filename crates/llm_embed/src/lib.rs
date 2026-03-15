use std::time::Duration;

use serde::{Deserialize, Serialize};

type EmbeddingInput = Vec<String>;
type EmbeddingOutput = Vec<Vec<f32>>;

#[derive(Clone)]
pub enum Provider {
    Ollama { model: String },
}

#[derive(Serialize)]
pub struct EmbeddingRequest {
    pub model: String,
    pub input: EmbeddingInput,
}

#[derive(Deserialize)]
pub struct EmbeddingResponse {
    pub embeddings: EmbeddingOutput,
}

/// Generate embeddings from input using the specified provider
/// # Errors
/// Returns an error if the LLM request fails or returns an invalid response.
pub fn generate(
    input: EmbeddingInput,
    provider: &Provider,
) -> Result<EmbeddingResponse, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()?;

    match provider {
        Provider::Ollama { model } => ollama_embed(&client, input, model),
    }
}

fn ollama_embed(
    client: &reqwest::blocking::Client,
    input: EmbeddingInput,
    model: &str,
) -> Result<EmbeddingResponse, Box<dyn std::error::Error>> {
    let request = EmbeddingRequest {
        model: model.to_string(),
        input,
    };

    let response = client
        .post("http://localhost:11434/api/embed")
        .json(&request)
        .send()
        .map_err(|e| format!("Request failed: {e}",))?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response.text().unwrap_or_default();
        return Err(format!("Ollama error {status}: {error_text}").into());
    }

    let parsed: EmbeddingResponse = response.json()?;

    Ok(parsed)
}
