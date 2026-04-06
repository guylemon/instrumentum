use std::{
    collections::HashMap,
    sync::{Arc, OnceLock, RwLock},
};

use tokenizers::Tokenizer;

use crate::tokens::error::Error;

type LoadResult = Result<Tokenizer, tokenizers::Error>;
type TokenizerLoader = dyn Fn(&str) -> LoadResult + Send + Sync;

/// Global singleton; holds tokenizers
pub struct TokenizerCache {
    inner: RwLock<HashMap<String, Arc<Tokenizer>>>,
    loader: Box<TokenizerLoader>,
}

impl TokenizerCache {
    fn with_loader(loader: impl Fn(&str) -> LoadResult + Send + Sync + 'static) -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
            loader: Box::new(loader),
        }
    }

    /// Get the global singleton instance.
    pub fn global() -> &'static Self {
        static GLOBAL: OnceLock<TokenizerCache> = OnceLock::new();
        GLOBAL.get_or_init(|| {
            TokenizerCache::with_loader(|model_id| Tokenizer::from_pretrained(model_id, None))
        })
    }

    /// Get a tokenizer. Will load a tokenizer from the web if not already cached.
    ///
    /// # Errors
    ///
    /// Returns an error if the cache lock is poisoned or if loading the tokenizer
    /// for `model_id` fails.
    pub fn get(&self, model_id: impl AsRef<str>) -> Result<Arc<Tokenizer>, Error> {
        let key = model_id.as_ref().trim().to_lowercase();

        // Try to acquire a read lock and get a cached tokenizer. Prevent deadlock by 
        // keeping the lock in its own scope.
        {
            let guard = self.inner
                .read()
                .map_err(|e| Error::Cache(format!("Read lock poisoned: {e}")))?;

            if let Some(tokenizer) = guard.get(&key) {
                return Ok(tokenizer.clone());
            }
        }

        // Upgrade to write lock if tokenizer not found in cache. Try one last
        // check for the tokenizer before downloading.
        let mut guard = self.inner
            .write()
            .map_err(|e| Error::Cache(format!("Write lock poisoned: {e}")))?;

        if let Some(tokenizer) = guard.get(&key) {
            return Ok(tokenizer.clone());
        }

        // Download the tokenizer if not found in cache
        let tokenizer = (self.loader)(model_id.as_ref())?;
        let tokenizer = Arc::new(tokenizer);

        // Cache the downloaded tokenizer before returning
        guard.insert(key, tokenizer.clone());

        Ok(tokenizer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        sync::{
            atomic::{AtomicUsize, Ordering},
            Barrier, Mutex,
        },
        thread,
    };
    use tokenizers::models::wordlevel::WordLevel;

    fn test_tokenizer() -> Tokenizer {
        Tokenizer::new(WordLevel::default())
    }

    #[test]
    fn global_returns_same_instance() {
        let left = TokenizerCache::global();
        let right = TokenizerCache::global();

        assert!(std::ptr::eq(left, right));
    }

    #[test]
    fn get_normalizes_model_id_for_cache_key() {
        let calls = Arc::new(AtomicUsize::new(0));
        let cache = TokenizerCache::with_loader({
            let calls = Arc::clone(&calls);
            move |_| {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(test_tokenizer())
            }
        });

        let first = cache.get("  BERT-BASE-Uncased  ").unwrap();
        let second = cache.get("bert-base-uncased").unwrap();

        assert!(Arc::ptr_eq(&first, &second));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn cache_hit_returns_same_arc() {
        let calls = Arc::new(AtomicUsize::new(0));
        let cache = TokenizerCache::with_loader({
            let calls = Arc::clone(&calls);
            move |_| {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(test_tokenizer())
            }
        });

        let first = cache.get("model-a").unwrap();
        let second = cache.get("model-a").unwrap();

        assert!(Arc::ptr_eq(&first, &second));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn cache_miss_loads_and_inserts() {
        let calls = Arc::new(AtomicUsize::new(0));
        let cache = TokenizerCache::with_loader({
            let calls = Arc::clone(&calls);
            move |_| {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(test_tokenizer())
            }
        });

        let tokenizer = cache.get("model-a").unwrap();

        assert_eq!(calls.load(Ordering::SeqCst), 1);
        let guard = cache.inner.read().unwrap();
        let cached = guard.get("model-a").unwrap();
        assert!(Arc::ptr_eq(cached, &tokenizer));
    }

    #[test]
    fn different_keys_produce_distinct_cache_entries() {
        let calls = Arc::new(AtomicUsize::new(0));
        let cache = TokenizerCache::with_loader({
            let calls = Arc::clone(&calls);
            move |_| {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(test_tokenizer())
            }
        });

        let first = cache.get("model-a").unwrap();
        let second = cache.get("model-b").unwrap();

        assert!(!Arc::ptr_eq(&first, &second));
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn concurrent_requests_for_same_key_only_load_once() {
        let calls = Arc::new(AtomicUsize::new(0));
        let start = Arc::new(Barrier::new(8));
        let cache = Arc::new(TokenizerCache::with_loader({
            let calls = Arc::clone(&calls);
            move |_| {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(test_tokenizer())
            }
        }));

        let handles: Vec<_> = (0..8)
            .map(|_| {
                let cache = Arc::clone(&cache);
                let start = Arc::clone(&start);
                thread::spawn(move || {
                    start.wait();
                    cache.get("model-a").unwrap()
                })
            })
            .collect();

        let tokenizers: Vec<_> = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect();

        for tokenizer in &tokenizers[1..] {
            assert!(Arc::ptr_eq(&tokenizers[0], tokenizer));
        }
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn concurrent_requests_for_equivalent_normalized_keys_only_load_once() {
        let calls = Arc::new(AtomicUsize::new(0));
        let start = Arc::new(Barrier::new(4));
        let cache = Arc::new(TokenizerCache::with_loader({
            let calls = Arc::clone(&calls);
            move |_| {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(test_tokenizer())
            }
        }));

        let inputs = [
            "bert-base-uncased",
            "  BERT-BASE-Uncased  ",
            "BeRt-BaSe-UnCaSeD",
            " bert-base-uncased ",
        ];

        let handles: Vec<_> = inputs
            .into_iter()
            .map(|input| {
                let cache = Arc::clone(&cache);
                let start = Arc::clone(&start);
                thread::spawn(move || {
                    start.wait();
                    cache.get(input).unwrap()
                })
            })
            .collect();

        let tokenizers: Vec<_> = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect();

        for tokenizer in &tokenizers[1..] {
            assert!(Arc::ptr_eq(&tokenizers[0], tokenizer));
        }
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn loader_error_is_returned_and_not_cached() {
        let calls = Arc::new(AtomicUsize::new(0));
        let cache = TokenizerCache::with_loader({
            let calls = Arc::clone(&calls);
            move |_| {
            calls.fetch_add(1, Ordering::SeqCst);
            Err("boom".into())
            }
        });

        let first = cache.get("model-a").unwrap_err();
        let second = cache.get("model-a").unwrap_err();

        assert!(matches!(first, Error::Tokenizer(_)));
        assert!(matches!(second, Error::Tokenizer(_)));
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        assert!(cache.inner.read().unwrap().is_empty());
    }

    #[test]
    fn success_after_prior_failure_populates_cache() {
        let calls = Arc::new(AtomicUsize::new(0));
        let cache = TokenizerCache::with_loader({
            let calls = Arc::clone(&calls);
            move |_| {
            let call = calls.fetch_add(1, Ordering::SeqCst);
            if call == 0 {
                Err("boom".into())
            } else {
                Ok(test_tokenizer())
            }
            }
        });

        assert!(matches!(cache.get("model-a"), Err(Error::Tokenizer(_))));
        let second = cache.get("model-a").unwrap();
        let third = cache.get("model-a").unwrap();

        assert!(Arc::ptr_eq(&second, &third));
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn poisoned_read_lock_returns_cache_error() {
        let cache = Arc::new(TokenizerCache::with_loader(|_| Ok(test_tokenizer())));
        let cache_for_thread = Arc::clone(&cache);

        let _ = thread::spawn(move || {
            let _guard = cache_for_thread.inner.write().unwrap();
            panic!("poison read lock");
        })
        .join();

        let error = cache.get("model-a").unwrap_err();
        match error {
            Error::Cache(message) => assert!(message.contains("Read lock poisoned")),
            other => panic!("expected cache error, got {other:?}"),
        }
    }

    #[test]
    fn original_model_id_is_passed_to_loader() {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let cache = TokenizerCache::with_loader({
            let seen = Arc::clone(&seen);
            move |model_id| {
                seen.lock().unwrap().push(model_id.to_owned());
                Ok(test_tokenizer())
            }
        });

        let _ = cache.get("  Mixed-Case-Model  ").unwrap();

        assert_eq!(seen.lock().unwrap().as_slice(), &["  Mixed-Case-Model  "]);
    }
}
