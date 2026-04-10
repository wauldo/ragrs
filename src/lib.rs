//! ragrs — Fast local RAG in Rust
//!
//! Index documents, query with BM25, verify with Wauldo.
//!
//! ```no_run
//! use ragrs::RagrsEngine;
//!
//! #[tokio::main]
//! async fn main() -> ragrs::Result<()> {
//!     let engine = RagrsEngine::new(".ragrs/index.db").await?;
//!     engine.index_directory(std::path::Path::new("./docs")).await?;
//!     let result = engine.query("What is X?", 5).await?;
//!     for source in &result.sources {
//!         println!("[{:.2}] {}: {}", source.score, source.document, source.content);
//!     }
//!     Ok(())
//! }
//! ```

pub mod bm25;
pub mod chunking;
pub mod cli;
pub mod engine;
pub mod error;
pub mod retriever;
pub mod store;
pub mod token_counter;
pub mod tokenize;
pub mod verify;
pub mod types;

pub use engine::{IndexResult, QueryResult, RagrsEngine, Source};
pub use error::{RagrsError, Result};
pub use types::Chunk;
