//! Error types for ragrs

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RagrsError {
    #[error("Chunking failed: {0}")]
    Chunking(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Retrieval failed: {0}")]
    Retrieval(String),

    #[error("Verification failed: {0}")]
    Verification(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, RagrsError>;
