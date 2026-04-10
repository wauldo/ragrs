//! Sentence-aware chunking engine

use crate::error::Result;
use crate::token_counter::count_tokens;
use crate::types::Chunk;

/// Splits content into chunks preserving sentence boundaries
pub struct ChunkingEngine {
    chunk_size: usize,
    overlap: usize,
}

impl ChunkingEngine {
    pub fn new(chunk_size: usize, overlap: usize) -> Self {
        Self {
            chunk_size,
            overlap,
        }
    }

    /// Split text into sentence-aware chunks with overlap
    pub fn chunk_text(&self, text: &str, source: &str) -> Result<Vec<Chunk>> {
        let mut chunks = Vec::new();
        let sentences = split_sentences(text);
        let mut current_chunk = String::new();
        let mut current_tokens = 0;
        let mut chunk_idx = 0;

        for sentence in sentences {
            let sentence_tokens = count_tokens(&sentence);

            if current_tokens + sentence_tokens > self.chunk_size && !current_chunk.is_empty() {
                let source_prefix = source.replace(['.', '/'], "_");
                chunks.push(Chunk::new(
                    format!("{source_prefix}_{chunk_idx}"),
                    current_chunk.trim().to_string(),
                    source.to_string(),
                    chunk_idx,
                ));
                chunk_idx += 1;

                let overlap_text = self.get_overlap(&current_chunk);
                current_chunk = overlap_text;
                current_tokens = count_tokens(&current_chunk);
            }

            current_chunk.push_str(&sentence);
            current_chunk.push(' ');
            current_tokens += sentence_tokens;
        }

        if !current_chunk.trim().is_empty() {
            let source_prefix = source.replace(['.', '/'], "_");
            chunks.push(Chunk::new(
                format!("{source_prefix}_{chunk_idx}"),
                current_chunk.trim().to_string(),
                source.to_string(),
                chunk_idx,
            ));
        }

        let total = chunks.len();
        for chunk in &mut chunks {
            chunk.metadata.total_chunks = total;
        }

        Ok(chunks)
    }

    /// Get overlap text from end of chunk (sentence-aware)
    fn get_overlap(&self, text: &str) -> String {
        let sentences = split_sentences(text);
        if sentences.is_empty() {
            return String::new();
        }

        let mut parts: Vec<&str> = Vec::new();
        let mut token_count = 0;

        for sentence in sentences.iter().rev() {
            let tokens = count_tokens(sentence);
            if token_count + tokens > self.overlap && !parts.is_empty() {
                break;
            }
            parts.push(sentence);
            token_count += tokens;
        }

        parts.reverse();
        parts.join(" ")
    }
}

impl Default for ChunkingEngine {
    fn default() -> Self {
        Self::new(200, 30)
    }
}

fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();

    for ch in text.chars() {
        current.push(ch);
        if ch == '.' || ch == '!' || ch == '?' {
            sentences.push(current.trim().to_string());
            current = String::new();
        }
    }

    if !current.trim().is_empty() {
        sentences.push(current.trim().to_string());
    }

    sentences
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_text_produces_chunks() {
        let engine = ChunkingEngine::new(50, 10);
        let text = "This is sentence one. This is sentence two. This is sentence three.";
        let chunks = engine.chunk_text(text, "test.md").unwrap();
        assert!(!chunks.is_empty());
        for chunk in &chunks {
            assert!(!chunk.content.is_empty());
            assert_eq!(chunk.metadata.source, "test.md");
        }
    }

    #[test]
    fn test_empty_text() {
        let engine = ChunkingEngine::default();
        let chunks = engine.chunk_text("", "test.md").unwrap();
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_sentence_splitting() {
        let sentences = split_sentences("First. Second! Third?");
        assert_eq!(sentences.len(), 3);
    }
}
