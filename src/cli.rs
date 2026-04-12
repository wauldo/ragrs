//! CLI commands for ragrs

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use colored::*;

use crate::engine::RagrsEngine;
use crate::retriever::retrieve;
use crate::verify::VerifyClient;

#[derive(Parser)]
#[command(name = "ragrs", version, about = "Fast local RAG in Rust")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Index documents from a file or directory
    Index {
        /// Path to a file or directory
        path: PathBuf,
    },
    /// Query indexed documents
    Query {
        /// The question to ask
        question: String,
        /// Number of results to return
        #[arg(long, default_value = "5")]
        top_k: usize,
        /// Verify results using Wauldo API
        #[arg(long)]
        verify: bool,
    },
    /// Delete all indexed data
    Reset,
}

fn db_path() -> String {
    let dir = PathBuf::from(".ragrs");
    if !dir.exists() {
        std::fs::create_dir_all(&dir).ok();
    }
    dir.join("index.db").to_string_lossy().to_string()
}

pub async fn run(cli: Cli) -> crate::error::Result<()> {
    match cli.command {
        Commands::Index { path } => cmd_index(path).await,
        Commands::Query {
            question,
            top_k,
            verify,
        } => cmd_query(question, top_k, verify).await,
        Commands::Reset => cmd_reset().await,
    }
}

async fn cmd_index(path: PathBuf) -> crate::error::Result<()> {
    let engine = RagrsEngine::new(&db_path()).await?;

    if path.is_dir() {
        let results = engine.index_directory(&path).await?;
        let total_chunks: usize = results.iter().map(|r| r.chunks).sum();
        println!(
            "  {} {} documents ({} chunks)",
            "Indexed".green().bold(),
            results.len(),
            total_chunks,
        );
        for r in &results {
            println!("    {} — {} chunks", r.file, r.chunks);
        }
    } else if path.is_file() {
        let result = engine.index_file(&path).await?;
        println!(
            "  {} {} ({} chunks)",
            "Indexed".green().bold(),
            result.file,
            result.chunks,
        );
    } else {
        eprintln!(
            "  {} Path not found: {}",
            "Error".red().bold(),
            path.display()
        );
    }

    Ok(())
}

async fn cmd_query(question: String, top_k: usize, verify: bool) -> crate::error::Result<()> {
    let engine = RagrsEngine::new(&db_path()).await?;

    let chunks = engine.get_chunks().await?;
    if chunks.is_empty() {
        println!(
            "  {} No documents indexed. Run `ragrs index <path>` first.",
            "Error".red().bold()
        );
        return Ok(());
    }

    let start = std::time::Instant::now();
    let scored = retrieve(&chunks, &question, top_k);
    let latency = start.elapsed();

    if scored.is_empty() {
        println!("  No relevant results found.");
        return Ok(());
    }

    println!(
        "\n  Found {} sources ({:.1}ms)\n",
        scored.len(),
        latency.as_secs_f64() * 1000.0,
    );

    for (i, result) in scored.iter().enumerate() {
        let snippet = extract_relevant_snippet(&result.chunk.content, &question, 200);
        println!(
            "  {} {} (score: {:.2})",
            format!("[{}]", i + 1).cyan().bold(),
            result.chunk.metadata.source.dimmed(),
            result.score,
        );
        println!("      \"{}\"\n", snippet.dimmed());
    }

    if verify {
        run_verification(&scored, &question).await;
    }

    Ok(())
}

async fn run_verification(scored: &[crate::retriever::ScoredChunk], question: &str) {
    let api_key = match std::env::var("WAULDO_API_KEY") {
        Ok(key) if !key.is_empty() => key,
        _ => {
            println!(
                "  {} Set {} to enable verification.",
                "Note".yellow().bold(),
                "WAULDO_API_KEY".bold(),
            );
            println!("  Get a free key at {}\n", "wauldo.com/guard".underline());
            return;
        }
    };

    let client = VerifyClient::new(&api_key);

    // Use top result as claim, concatenate top sources as evidence
    let claim = format!(
        "Regarding \"{}\": {}",
        question,
        extract_relevant_snippet(&scored[0].chunk.content, question, 300),
    );
    let source_text: String = scored
        .iter()
        .take(3)
        .enumerate()
        .map(|(i, s)| {
            format!(
                "[Source {}] {}: {}",
                i + 1,
                s.chunk.metadata.source,
                s.chunk.content
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    match client.verify(&claim, &source_text).await {
        Ok(result) => {
            let verdict_colored = match result.verdict.as_str() {
                "verified" => result.verdict.green().bold(),
                "rejected" => result.verdict.red().bold(),
                _ => result.verdict.yellow().bold(),
            };

            println!(
                "  {}",
                "── Verification ──────────────────────────".dimmed()
            );
            println!("  Verdict:  {}", verdict_colored);
            if let Some(reason) = &result.reason {
                println!("  Reason:   {}", reason);
            }
            println!("  Trust:    {:.2}", result.confidence);
            println!(
                "  {}",
                "───────────────────────────────────────────".dimmed()
            );
            println!(
                "  Verify your RAG {} {}\n",
                "→".dimmed(),
                "wauldo.com/guard".underline()
            );
        }
        Err(e) => {
            println!("  {} Verification failed: {}\n", "Error".red().bold(), e);
        }
    }
}

async fn cmd_reset() -> crate::error::Result<()> {
    let engine = RagrsEngine::new(&db_path()).await?;
    engine.reset().await?;
    println!("  {} Index cleared.", "Done".green().bold());
    Ok(())
}

/// Strip markdown formatting from text
fn strip_markdown(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '#' => {
                // Skip markdown headers (# ## ###)
                while chars.peek() == Some(&'#') {
                    chars.next();
                }
                if chars.peek() == Some(&' ') {
                    chars.next();
                }
            }
            '*' => {
                // Skip bold/italic markers
                if chars.peek() == Some(&'*') {
                    chars.next();
                }
            }
            '`' => {
                // Keep content inside backticks but strip the backticks
            }
            _ => result.push(ch),
        }
    }

    result
}

/// Extract the most relevant snippet from content based on query terms
fn extract_relevant_snippet(content: &str, query: &str, max_chars: usize) -> String {
    let clean = strip_markdown(content);
    let clean: String = clean
        .chars()
        .filter(|c| !c.is_control() || *c == ' ')
        .collect();

    // Split into sentences
    let sentences: Vec<&str> = clean
        .split(['.', '!', '?'])
        .map(|s| s.trim())
        .filter(|s| s.len() > 10)
        .collect();

    if sentences.is_empty() {
        return truncate(&clean, max_chars);
    }

    // Tokenize query for matching
    let query_terms: Vec<String> = crate::tokenize::tokenize(query);

    // Score each sentence by query term overlap
    let mut best_idx = 0;
    let mut best_score = 0;
    for (i, sentence) in sentences.iter().enumerate() {
        let lower = sentence.to_lowercase();
        let score: usize = query_terms
            .iter()
            .filter(|t| lower.contains(t.as_str()))
            .count();
        if score > best_score {
            best_score = score;
            best_idx = i;
        }
    }

    // Build snippet: best sentence + neighbors for context
    let start = best_idx.saturating_sub(1);
    let end = (best_idx + 2).min(sentences.len());
    let snippet: String = sentences[start..end].join(". ");

    truncate(&snippet, max_chars)
}

fn truncate(text: &str, max_chars: usize) -> String {
    if text.len() <= max_chars {
        text.to_string()
    } else {
        format!("{}...", &text[..max_chars])
    }
}
