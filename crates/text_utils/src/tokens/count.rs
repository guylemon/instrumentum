use tokenizers::Tokenizer;

use crate::tokens::{get_tokenizer, Error};

/// Convenience method to count tokens in a string (including special tokens, such as EOS)
///
/// # Errors
/// - Relays errors from `TokenizerCache` if the retrieval process fails.
/// - Relays errors from the encoding process if it fails.
pub fn count_tokens(text: impl AsRef<str>, model_id: impl AsRef<str>) -> Result<usize, Error> {
    let tokenizer = get_tokenizer(model_id)?;
    count_tokens_with_tokenizer(text.as_ref(), tokenizer.as_ref())
}

fn count_tokens_with_tokenizer(text: &str, tokenizer: &Tokenizer) -> Result<usize, Error> {
    if text.is_empty() {
        return Ok(0);
    }

    // Include marker tokens that the model sees. This allows for an accurate accounting of the
    // tokens the model will handle in addition to the raw input text.
    let add_special_tokens = true;
    let encoding = tokenizer.encode(text, add_special_tokens)?;

    Ok(encoding.get_ids().len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokenizers::{
        models::wordlevel::WordLevel,
        pre_tokenizers::whitespace::Whitespace,
        processors::template::TemplateProcessing,
    };

    fn base_tokenizer() -> Tokenizer {
        let model = WordLevel::builder()
            .vocab(
                [
                    ("hello".to_owned(), 0),
                    ("world".to_owned(), 1),
                    ("<unk>".to_owned(), 2),
                ]
                .into(),
            )
            .unk_token("<unk>".to_owned())
            .build()
            .unwrap();

        let mut tokenizer = Tokenizer::new(model);
        tokenizer.with_pre_tokenizer(Some(Whitespace));
        tokenizer
    }

    fn tokenizer_with_special_tokens() -> Tokenizer {
        let model = WordLevel::builder()
            .vocab(
                [
                    ("hello".to_owned(), 0),
                    ("world".to_owned(), 1),
                    ("[CLS]".to_owned(), 2),
                    ("[SEP]".to_owned(), 3),
                    ("<unk>".to_owned(), 4),
                ]
                .into(),
            )
            .unk_token("<unk>".to_owned())
            .build()
            .unwrap();

        let mut tokenizer = Tokenizer::new(model);
        tokenizer.with_pre_tokenizer(Some(Whitespace));
        tokenizer.with_post_processor(Some(
            TemplateProcessing::builder()
                .try_single("[CLS] $0 [SEP]")
                .unwrap()
                .special_tokens(vec![("[CLS]", 2), ("[SEP]", 3)])
                .build()
                .unwrap(),
        ));
        tokenizer
    }

    #[test]
    fn returns_zero_for_empty_input() {
        let tokenizer = base_tokenizer();

        let count = count_tokens_with_tokenizer("", &tokenizer).unwrap();

        assert_eq!(count, 0);
    }

    #[test]
    fn counts_single_token_input() {
        let tokenizer = base_tokenizer();

        let count = count_tokens_with_tokenizer("hello", &tokenizer).unwrap();

        assert_eq!(count, 1);
    }

    #[test]
    fn counts_multiple_tokens_input() {
        let tokenizer = base_tokenizer();

        let count = count_tokens_with_tokenizer("hello world", &tokenizer).unwrap();

        assert_eq!(count, 2);
    }

    #[test]
    fn includes_special_tokens_in_count() {
        let tokenizer = tokenizer_with_special_tokens();

        let count = count_tokens_with_tokenizer("hello world", &tokenizer).unwrap();

        assert_eq!(count, 4);
    }

    #[test]
    fn propagates_encode_errors() {
        let tokenizer = Tokenizer::new(WordLevel::default());
        let error = count_tokens_with_tokenizer("hello", &tokenizer).unwrap_err();

        assert!(matches!(error, Error::Tokenizer(_)));
    }
}
