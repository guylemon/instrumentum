#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("tokenizer error: {0}")]
    Tokenizer(#[from] tokenizers::Error),

    #[error("cache error: {0}")]
    Cache(String),
}
