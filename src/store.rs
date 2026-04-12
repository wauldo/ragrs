//! SQLite chunk store with FTS5 full-text search

use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Row, SqlitePool};

use crate::error::{RagrsError, Result};
use crate::types::{Chunk, ChunkMetadata};

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS chunks (
    id           TEXT PRIMARY KEY,
    document_id  TEXT NOT NULL,
    content      TEXT NOT NULL,
    tokens       INTEGER NOT NULL,
    position     INTEGER NOT NULL,
    total_chunks INTEGER NOT NULL,
    created_at   TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_chunks_document ON chunks(document_id);
"#;

const FTS5_SCHEMA: &str = r#"
CREATE VIRTUAL TABLE IF NOT EXISTS chunks_fts USING fts5(
    content,
    content='chunks',
    content_rowid='rowid',
    tokenize='unicode61 remove_diacritics 2'
);
"#;

const FTS5_TRIGGER_INSERT: &str = r#"
CREATE TRIGGER IF NOT EXISTS chunks_fts_ai AFTER INSERT ON chunks BEGIN
    INSERT INTO chunks_fts(rowid, content) VALUES (new.rowid, new.content);
END;
"#;

const FTS5_TRIGGER_DELETE: &str = r#"
CREATE TRIGGER IF NOT EXISTS chunks_fts_ad AFTER DELETE ON chunks BEGIN
    INSERT INTO chunks_fts(chunks_fts, rowid, content)
    VALUES ('delete', old.rowid, old.content);
END;
"#;

const PRAGMA_TUNING: &[&str] = &[
    "PRAGMA journal_mode = WAL",
    "PRAGMA synchronous = NORMAL",
    "PRAGMA cache_size = -32000",
    "PRAGMA temp_store = MEMORY",
    "PRAGMA foreign_keys = ON",
];

/// SQLite-backed chunk store
pub struct SqliteStore {
    pool: SqlitePool,
}

impl SqliteStore {
    /// Open or create a SQLite database
    pub async fn open(path: &str) -> Result<Self> {
        let url = if path == ":memory:" {
            "sqlite::memory:".to_string()
        } else {
            format!("sqlite://{}?mode=rwc", path)
        };

        let pool = SqlitePoolOptions::new()
            .max_connections(2)
            .connect(&url)
            .await
            .map_err(|e| RagrsError::Storage(format!("connect: {e}")))?;

        let store = Self { pool };
        store.init_schema().await?;
        Ok(store)
    }

    async fn init_schema(&self) -> Result<()> {
        for pragma in PRAGMA_TUNING {
            sqlx::query(pragma)
                .execute(&self.pool)
                .await
                .map_err(|e| RagrsError::Storage(format!("PRAGMA: {e}")))?;
        }

        sqlx::query(SCHEMA)
            .execute(&self.pool)
            .await
            .map_err(|e| RagrsError::Storage(format!("schema: {e}")))?;

        for sql in [FTS5_SCHEMA, FTS5_TRIGGER_INSERT, FTS5_TRIGGER_DELETE] {
            sqlx::query(sql)
                .execute(&self.pool)
                .await
                .map_err(|e| RagrsError::Storage(format!("FTS5: {e}")))?;
        }

        Ok(())
    }

    /// Insert chunks into the store
    pub async fn insert_chunks(&self, chunks: &[Chunk]) -> Result<usize> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| RagrsError::Storage(format!("begin: {e}")))?;

        let mut count = 0;
        for chunk in chunks {
            sqlx::query(
                "INSERT OR REPLACE INTO chunks \
                 (id, document_id, content, tokens, position, total_chunks) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )
            .bind(&chunk.id)
            .bind(&chunk.metadata.source)
            .bind(&chunk.content)
            .bind(chunk.tokens as i32)
            .bind(chunk.metadata.position as i32)
            .bind(chunk.metadata.total_chunks as i32)
            .execute(&mut *tx)
            .await
            .map_err(|e| RagrsError::Storage(format!("insert: {e}")))?;
            count += 1;
        }

        tx.commit()
            .await
            .map_err(|e| RagrsError::Storage(format!("commit: {e}")))?;
        Ok(count)
    }

    /// Get all chunks
    pub async fn get_all(&self) -> Result<Vec<Chunk>> {
        let rows = sqlx::query("SELECT * FROM chunks ORDER BY document_id, position")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| RagrsError::Storage(format!("get_all: {e}")))?;
        Ok(rows.iter().map(Self::row_to_chunk).collect())
    }

    /// Search chunks using FTS5
    pub async fn search_fts(&self, query: &str, limit: usize) -> Result<Vec<Chunk>> {
        let rows = sqlx::query(
            "SELECT c.* FROM chunks c \
             INNER JOIN chunks_fts f ON c.rowid = f.rowid \
             WHERE chunks_fts MATCH ?1 \
             ORDER BY f.rank LIMIT ?2",
        )
        .bind(query)
        .bind(limit as i32)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RagrsError::Storage(format!("fts_search: {e}")))?;
        Ok(rows.iter().map(Self::row_to_chunk).collect())
    }

    /// Count total chunks
    pub async fn count(&self) -> Result<usize> {
        let row = sqlx::query("SELECT COUNT(*) as cnt FROM chunks")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RagrsError::Storage(format!("count: {e}")))?;
        Ok(row.get::<i32, _>("cnt") as usize)
    }

    /// Count distinct documents
    pub async fn document_count(&self) -> Result<usize> {
        let row = sqlx::query("SELECT COUNT(DISTINCT document_id) as cnt FROM chunks")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RagrsError::Storage(format!("doc_count: {e}")))?;
        Ok(row.get::<i32, _>("cnt") as usize)
    }

    /// Delete all chunks
    pub async fn delete_all(&self) -> Result<()> {
        sqlx::query("DELETE FROM chunks")
            .execute(&self.pool)
            .await
            .map_err(|e| RagrsError::Storage(format!("delete_all: {e}")))?;
        Ok(())
    }

    fn row_to_chunk(row: &sqlx::sqlite::SqliteRow) -> Chunk {
        Chunk {
            id: row.get("id"),
            content: row.get("content"),
            tokens: row.get::<i32, _>("tokens") as usize,
            metadata: ChunkMetadata {
                source: row.get("document_id"),
                position: row.get::<i32, _>("position") as usize,
                total_chunks: row.get::<i32, _>("total_chunks") as usize,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_insert_and_get() {
        let store = SqliteStore::open(":memory:").await.unwrap();
        let chunks = vec![
            Chunk::new("c1".into(), "Hello world".into(), "doc1.md".into(), 0),
            Chunk::new("c2".into(), "Rust programming".into(), "doc1.md".into(), 1),
        ];
        let count = store.insert_chunks(&chunks).await.unwrap();
        assert_eq!(count, 2);

        let all = store.get_all().await.unwrap();
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn test_fts_search() {
        let store = SqliteStore::open(":memory:").await.unwrap();
        let chunks = vec![
            Chunk::new(
                "c1".into(),
                "Rust is a systems programming language".into(),
                "doc1.md".into(),
                0,
            ),
            Chunk::new(
                "c2".into(),
                "Python is great for data science".into(),
                "doc2.md".into(),
                0,
            ),
        ];
        store.insert_chunks(&chunks).await.unwrap();

        let results = store.search_fts("Rust programming", 10).await.unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].id, "c1");
    }

    #[tokio::test]
    async fn test_delete_all() {
        let store = SqliteStore::open(":memory:").await.unwrap();
        let chunks = vec![Chunk::new("c1".into(), "test".into(), "doc.md".into(), 0)];
        store.insert_chunks(&chunks).await.unwrap();
        assert_eq!(store.count().await.unwrap(), 1);

        store.delete_all().await.unwrap();
        assert_eq!(store.count().await.unwrap(), 0);
    }
}
