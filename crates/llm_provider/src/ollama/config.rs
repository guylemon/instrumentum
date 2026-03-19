const OLLAMA_DEFAULT_BASE_URL: &str = "http://localhost:11434/api";

#[derive(Clone)]
pub struct Config {
    /// The base url serving the Ollama api
    /// Example: <http://localhost:11434/api>
    pub base_url: String,
}

impl Config {
    #[must_use]
    pub fn new(base_url: Option<&str>) -> Self {
        Self {
            base_url: base_url.unwrap_or(OLLAMA_DEFAULT_BASE_URL).to_string(),
        }
    }
}
