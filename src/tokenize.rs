//! Tokenization and stop-word filtering for BM25 retrieval

/// Tokenize text into lowercase words, filtering stop words and punctuation
pub fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split_whitespace()
        .filter_map(|word| {
            let clean: String = word.chars().filter(|c| c.is_alphanumeric()).collect();
            if clean.len() >= 2 && !is_stop_word(&clean) {
                Some(clean)
            } else {
                None
            }
        })
        .collect()
}

/// Check if a word is a common EN or FR stop word
pub fn is_stop_word(word: &str) -> bool {
    matches!(
        word,
        "the" | "is" | "at" | "which" | "on" | "a" | "an" | "as"
            | "are" | "was" | "were" | "be" | "been" | "being"
            | "have" | "has" | "had" | "do" | "does" | "did"
            | "will" | "would" | "should" | "could" | "may" | "might"
            | "must" | "can" | "shall" | "of" | "to" | "for" | "with"
            | "in" | "and" | "or" | "but" | "not" | "this" | "that"
            | "these" | "those" | "it" | "its"
            | "le" | "la" | "les" | "un" | "une" | "des" | "du" | "de"
            | "et" | "ou" | "est" | "en" | "au" | "aux" | "ce" | "se"
            | "qui" | "que" | "ne" | "pas" | "par" | "sur" | "son"
            | "sa" | "ses" | "dans" | "pour" | "avec" | "il" | "elle"
            | "nous" | "vous" | "ils" | "ont" | "sont" | "mais" | "plus"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_filters_stopwords() {
        let tokens = tokenize("Hello, world! This is a test.");
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
        assert!(tokens.contains(&"test".to_string()));
        assert!(!tokens.contains(&"this".to_string()));
        assert!(!tokens.contains(&"is".to_string()));
    }

    #[test]
    fn test_tokenize_empty() {
        assert!(tokenize("").is_empty());
        assert!(tokenize("   ").is_empty());
    }
}
