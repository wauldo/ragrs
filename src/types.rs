//! Core data types for ragrs

use crate::token_counter::count_tokens;
use serde::{Deserialize, Serialize};

/// A chunk of indexed content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub id: String,
    pub content: String,
    pub tokens: usize,
    pub metadata: ChunkMetadata,
}

/// Metadata attached to each chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMetadata {
    pub source: String,
    pub position: usize,
    pub total_chunks: usize,
}

impl Chunk {
    pub fn new(id: String, content: String, source: String, position: usize) -> Self {
        let tokens = count_tokens(&content);
        Self {
            id,
            content,
            tokens,
            metadata: ChunkMetadata {
                source,
                position,
                total_chunks: 0,
            },
        }
    }

    pub fn recalculate_tokens(&mut self) {
        self.tokens = count_tokens(&self.content);
    }
}
