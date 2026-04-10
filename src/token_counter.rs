//! Accurate token counting using tiktoken (cl100k_base)

use once_cell::sync::Lazy;
use tiktoken_rs::CoreBPE;

static TOKENIZER: Lazy<CoreBPE> =
    Lazy::new(|| tiktoken_rs::cl100k_base().expect("Failed to load cl100k_base tokenizer"));

/// Count tokens using tiktoken cl100k_base (GPT-4 compatible)
pub fn count_tokens(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    TOKENIZER.encode_with_special_tokens(text).len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        assert_eq!(count_tokens(""), 0);
    }

    #[test]
    fn test_sentence() {
        let n = count_tokens("Hello, world! How are you?");
        assert!(n >= 5 && n <= 10);
    }

    #[test]
    fn test_consistency() {
        let text = "The quick brown fox";
        assert_eq!(count_tokens(text), count_tokens(text));
    }
}
