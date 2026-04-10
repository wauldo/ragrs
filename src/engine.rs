//! RagrsEngine — unified entry point for indexing and querying

use std::path::Path;

use crate::chunking::ChunkingEngine;
use crate::error::{RagrsError, Result};
use crate::retriever::{retrieve, ScoredChunk};
use crate::store::SqliteStore;
use crate::types::Chunk;

/// Main engine for ragrs — index documents and query with BM25
pub struct RagrsEngine {
    store: SqliteStore,
    chunking: ChunkingEngine,
}

/// Result of a query operation
#[derive(Debug)]
pub struct QueryResult {
    pub sources: Vec<Source>,
    pub latency_ms: u64,
}

/// A ranked source from retrieval
#[derive(Debug, Clone)]
pub struct Source {
    pub content: String,
    pub document: String,
    pub score: f32,
}

/// Result of indexing a file
#[derive(Debug)]
pub struct IndexResult {
    pub file: String,
    pub chunks: usize,
}

impl RagrsEngine {
    /// Create a new engine with SQLite store at the given path
    pub async fn new(db_path: &str) -> Result<Self> {
        let store = SqliteStore::open(db_path).await?;
        Ok(Self {
            store,
            chunking: ChunkingEngine::default(),
        })
    }

    /// Index a single file (.md or .txt)
    pub async fn index_file(&self, path: &Path) -> Result<IndexResult> {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(RagrsError::Io)?;

        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let chunks = self.chunking.chunk_text(&content, filename)?;
        let count = self.store.insert_chunks(&chunks).await?;

        Ok(IndexResult {
            file: filename.to_string(),
            chunks: count,
        })
    }

    /// Index all .md and .txt files in a directory
    pub async fn index_directory(&self, dir: &Path) -> Result<Vec<IndexResult>> {
        let mut results = Vec::new();
        let mut entries = tokio::fs::read_dir(dir)
            .await
            .map_err(RagrsError::Io)?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(RagrsError::Io)?
        {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if matches!(ext, "md" | "txt" | "markdown") {
                        match self.index_file(&path).await {
                            Ok(result) => results.push(result),
                            Err(e) => eprintln!("Warning: skipping {}: {e}", path.display()),
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    /// Query indexed documents using BM25
    pub async fn query(&self, question: &str, top_k: usize) -> Result<QueryResult> {
        let start = std::time::Instant::now();

        let chunks = self.store.get_all().await?;
        let scored = retrieve(&chunks, question, top_k);

        let sources = scored
            .into_iter()
            .map(|ScoredChunk { chunk, score }| Source {
                content: chunk.content,
                document: chunk.metadata.source,
                score,
            })
            .collect();

        Ok(QueryResult {
            sources,
            latency_ms: start.elapsed().as_millis() as u64,
        })
    }

    /// Get stats about the index
    pub async fn stats(&self) -> Result<(usize, usize)> {
        let chunks = self.store.count().await?;
        let docs = self.store.document_count().await?;
        Ok((docs, chunks))
    }

    /// Delete all indexed data
    pub async fn reset(&self) -> Result<()> {
        self.store.delete_all().await
    }

    /// Get all chunks (for verify to access raw data)
    pub async fn get_chunks(&self) -> Result<Vec<Chunk>> {
        self.store.get_all().await
    }
}
