use ragrs::bm25::{bm25_score, compute_doc_stats, compute_idf};
use ragrs::chunking::ChunkingEngine;
use ragrs::retriever::retrieve;
use ragrs::store::SqliteStore;
use ragrs::types::Chunk;
use ragrs::RagrsEngine;
use std::path::Path;

#[test]
fn test_chunking_produces_valid_chunks() {
    let engine = ChunkingEngine::new(50, 10);
    let text = "First sentence here. Second sentence follows. Third sentence ends.";
    let chunks = engine.chunk_text(text, "test.md").unwrap();

    assert!(!chunks.is_empty());
    for chunk in &chunks {
        assert!(!chunk.content.is_empty());
        assert_eq!(chunk.metadata.source, "test.md");
        assert!(chunk.tokens > 0);
    }
}

#[test]
fn test_bm25_relevant_scores_higher() {
    let docs = vec![
        compute_doc_stats("API keys never expire and remain valid indefinitely"),
        compute_doc_stats("The connection timeout is 30 seconds configurable"),
        compute_doc_stats("Cooking recipes for pasta and Italian food"),
    ];
    let idf = compute_idf(&docs, "API keys expire");
    let avg_len = docs.iter().map(|d| d.length).sum::<usize>() as f32 / docs.len() as f32;

    let score_relevant = bm25_score(&docs[0], "API keys expire", &idf, avg_len);
    let score_irrelevant = bm25_score(&docs[2], "API keys expire", &idf, avg_len);

    assert!(
        score_relevant > score_irrelevant,
        "Relevant doc should score higher"
    );
}

#[tokio::test]
async fn test_sqlite_store_insert_and_search() {
    let store = SqliteStore::open(":memory:").await.unwrap();
    let chunks = vec![
        Chunk::new(
            "c1".into(),
            "API keys never expire and remain valid".into(),
            "security.md".into(),
            0,
        ),
        Chunk::new(
            "c2".into(),
            "API key validity 12 months renewed".into(),
            "pricing.md".into(),
            0,
        ),
        Chunk::new(
            "c3".into(),
            "Connection timeout is 30 seconds".into(),
            "api-v2.md".into(),
            0,
        ),
    ];
    store.insert_chunks(&chunks).await.unwrap();

    let results = store.search_fts("API keys expire", 10).await.unwrap();
    assert!(!results.is_empty(), "FTS should find API key chunks");
}

#[tokio::test]
async fn test_engine_index_and_query() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/dataset");
    if !dir.exists() {
        return; // skip if dataset not present
    }

    let engine = RagrsEngine::new(":memory:").await.unwrap();
    let results = engine.index_directory(&dir).await.unwrap();
    assert_eq!(results.len(), 4, "Should index 4 markdown files");

    let total_chunks: usize = results.iter().map(|r| r.chunks).sum();
    assert!(total_chunks > 0, "Should produce chunks");

    // Verify we can query and get results back
    let query_result = engine.query("connection timeout seconds", 5).await.unwrap();
    assert!(
        !query_result.sources.is_empty(),
        "Should find relevant sources"
    );
    eprintln!("Query returned {} sources", query_result.sources.len());
    for s in &query_result.sources {
        eprintln!(
            "  [{:.2}] {} — {}...",
            s.score,
            s.document,
            &s.content[..s.content.len().min(60)]
        );
    }

    // At least one result should mention timeout
    let has_timeout = query_result
        .sources
        .iter()
        .any(|s| s.content.to_lowercase().contains("timeout"));
    assert!(has_timeout, "Should find timeout-related content");
}

#[tokio::test]
async fn test_engine_conflicting_sources() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/dataset");
    if !dir.exists() {
        return;
    }

    let engine = RagrsEngine::new(":memory:").await.unwrap();
    engine.index_directory(&dir).await.unwrap();

    let result = engine
        .query("API keys expire validity months", 10)
        .await
        .unwrap();
    eprintln!("Conflict query returned {} sources", result.sources.len());
    for s in &result.sources {
        eprintln!(
            "  [{:.2}] {} — {}...",
            s.score,
            s.document,
            &s.content[..s.content.len().min(80)]
        );
    }
    assert!(result.sources.len() >= 2, "Should find multiple sources");

    // Should find API key related content
    let has_key_content = result.sources.iter().any(|s| {
        let lower = s.content.to_lowercase();
        lower.contains("api key") || lower.contains("expire") || lower.contains("validity")
    });
    assert!(has_key_content, "Should find API key related content");
}

#[test]
fn test_query_no_results_for_unrelated() {
    let chunks = vec![
        Chunk::new(
            "c1".into(),
            "API documentation for cloud services".into(),
            "doc.md".into(),
            0,
        ),
        Chunk::new(
            "c2".into(),
            "Rate limiting and authentication".into(),
            "doc.md".into(),
            1,
        ),
    ];
    let results = retrieve(&chunks, "quantum physics entanglement", 5);

    // Unrelated query should produce low scores
    if !results.is_empty() {
        assert!(
            results[0].score < 0.5,
            "Unrelated query should have low scores"
        );
    }
}
