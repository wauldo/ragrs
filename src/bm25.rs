//! BM25 scoring for text retrieval
//!
//! Pure functions — no I/O, no state, fully deterministic.

use crate::tokenize::tokenize;
use std::collections::HashMap;

const K1: f32 = 1.5;
const B: f32 = 0.75;
const POSITION_BOOST_FACTOR: f32 = 1.3;
const POSITION_THRESHOLD: f32 = 0.2;

/// Cached document statistics for efficient BM25 scoring
#[derive(Clone)]
pub struct DocStats {
    pub tokens: Vec<String>,
    pub length: usize,
    pub term_positions: HashMap<String, Vec<usize>>,
}

/// Compute and cache document statistics
pub fn compute_doc_stats(content: &str) -> DocStats {
    let tokens = tokenize(content);
    let length = tokens.len();

    let mut term_positions: HashMap<String, Vec<usize>> = HashMap::new();
    for (pos, token) in tokens.iter().enumerate() {
        term_positions.entry(token.clone()).or_default().push(pos);
    }

    DocStats {
        tokens,
        length,
        term_positions,
    }
}

/// Compute IDF scores for query terms across a document corpus
pub fn compute_idf(doc_stats: &[DocStats], query: &str) -> HashMap<String, f32> {
    let query_terms = tokenize(query);
    let total_docs = doc_stats.len() as f32;
    let mut idf_scores = HashMap::new();

    for term in query_terms {
        let doc_freq = doc_stats
            .iter()
            .filter(|stats| stats.term_positions.contains_key(&term))
            .count() as f32;

        let idf = ((total_docs - doc_freq + 0.5) / (doc_freq + 0.5) + 1.0).ln();
        idf_scores.insert(term, idf.max(0.0));
    }

    idf_scores
}

/// Compute BM25 score with positional and phrase boosting
pub fn bm25_score(
    doc_stats: &DocStats,
    query: &str,
    idf_scores: &HashMap<String, f32>,
    avg_doc_len: f32,
) -> f32 {
    let query_terms = tokenize(query);
    let doc_len = doc_stats.length as f32;

    if query_terms.is_empty() || doc_len == 0.0 {
        return 0.0;
    }

    let mut term_freq: HashMap<String, f32> = HashMap::new();
    for token in &doc_stats.tokens {
        *term_freq.entry(token.clone()).or_insert(0.0) += 1.0;
    }

    let mut score = 0.0;

    for term in query_terms.iter() {
        let idf = idf_scores.get(term).copied().unwrap_or(0.0);
        let freq = term_freq.get(term).copied().unwrap_or(0.0);

        let numerator = freq * (K1 + 1.0);
        let denominator = freq + K1 * (1.0 - B + B * (doc_len / avg_doc_len));
        let mut term_score = idf * (numerator / denominator);

        // Positional boost: terms in first 20% score higher
        if freq > 0.0 {
            if let Some(positions) = doc_stats.term_positions.get(term) {
                let early_threshold = (doc_len * POSITION_THRESHOLD) as usize;
                if positions.iter().any(|&pos| pos < early_threshold) {
                    term_score *= POSITION_BOOST_FACTOR;
                }
            }
        }

        score += term_score;
    }

    // Phrase matching bonus for consecutive query terms
    if query_terms.len() >= 2 {
        score += compute_phrase_bonus(&query_terms, doc_stats);
    }

    score / query_terms.len() as f32
}

fn compute_phrase_bonus(query_terms: &[String], doc_stats: &DocStats) -> f32 {
    let mut bonus = 0.0;

    for i in 0..query_terms.len().saturating_sub(1) {
        let term1 = &query_terms[i];
        let term2 = &query_terms[i + 1];

        if let (Some(pos1), Some(pos2)) = (
            doc_stats.term_positions.get(term1),
            doc_stats.term_positions.get(term2),
        ) {
            let pos2_set: std::collections::HashSet<&usize> = pos2.iter().collect();
            for &p1 in pos1 {
                if pos2_set.contains(&(p1 + 1)) {
                    bonus += 0.5;
                }
            }
        }
    }

    bonus
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relevant_scores_higher() {
        let docs = vec![
            compute_doc_stats("Rust is a systems programming language"),
            compute_doc_stats("Python is great for data science"),
        ];
        let idf = compute_idf(&docs, "Rust programming");
        let avg_len = docs.iter().map(|d| d.length).sum::<usize>() as f32 / docs.len() as f32;

        let score_rust = bm25_score(&docs[0], "Rust programming", &idf, avg_len);
        let score_python = bm25_score(&docs[1], "Rust programming", &idf, avg_len);

        assert!(score_rust > score_python);
    }

    #[test]
    fn test_empty_query() {
        let doc = compute_doc_stats("some content");
        let idf = HashMap::new();
        assert_eq!(bm25_score(&doc, "", &idf, 10.0), 0.0);
    }
}
