//! BM25 retriever — scores and ranks chunks against a query

use crate::bm25::{bm25_score, compute_doc_stats, compute_idf, DocStats};
use crate::types::Chunk;

/// A scored search result
#[derive(Debug, Clone)]
pub struct ScoredChunk {
    pub chunk: Chunk,
    pub score: f32,
}

/// Score and rank chunks using BM25
pub fn retrieve(chunks: &[Chunk], query: &str, top_k: usize) -> Vec<ScoredChunk> {
    if chunks.is_empty() || query.trim().is_empty() {
        return Vec::new();
    }

    let doc_stats: Vec<DocStats> = chunks
        .iter()
        .map(|c| compute_doc_stats(&c.content))
        .collect();

    let idf_scores = compute_idf(&doc_stats, query);
    let avg_doc_len =
        doc_stats.iter().map(|d| d.length).sum::<usize>() as f32 / doc_stats.len().max(1) as f32;

    let mut scored: Vec<ScoredChunk> = chunks
        .iter()
        .zip(doc_stats.iter())
        .map(|(chunk, stats)| {
            let score = bm25_score(stats, query, &idf_scores, avg_doc_len);
            ScoredChunk {
                chunk: chunk.clone(),
                score,
            }
        })
        .collect();

    scored.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    scored.truncate(top_k);
    scored
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retrieve_ranks_correctly() {
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
            Chunk::new(
                "c3".into(),
                "Cooking recipes for pasta".into(),
                "doc3.md".into(),
                0,
            ),
        ];

        let results = retrieve(&chunks, "Rust programming", 5);
        assert!(!results.is_empty());
        assert_eq!(results[0].chunk.id, "c1");
        assert!(results[0].score > results[1].score);
    }

    #[test]
    fn test_retrieve_empty_query() {
        let chunks = vec![Chunk::new("c1".into(), "test".into(), "doc.md".into(), 0)];
        let results = retrieve(&chunks, "", 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_retrieve_top_k() {
        let chunks: Vec<Chunk> = (0..10)
            .map(|i| {
                Chunk::new(
                    format!("c{i}"),
                    format!("document number {i} about testing"),
                    "doc.md".into(),
                    i,
                )
            })
            .collect();
        let results = retrieve(&chunks, "testing", 3);
        assert_eq!(results.len(), 3);
    }
}
