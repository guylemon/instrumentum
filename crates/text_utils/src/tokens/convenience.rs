use std::sync::Arc;
use tokenizers::Tokenizer;

use crate::tokens::{TokenizerCache, Error};

/// Convenience: get a cached tokenizer
///
/// # Errors
/// Relays error from `TokenizerCache.get`. See `TokenizerCache.get` doc comments for more details
pub fn get_tokenizer(model_id: impl AsRef<str>) -> Result<Arc<Tokenizer>, Error> {
    TokenizerCache::global().get(model_id)
}
