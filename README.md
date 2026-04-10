# ragrs

Fast local RAG in Rust. Index, query, verify.

> Your RAG works. Until it lies.

## Install

```bash
cargo install ragrs
```

## 30-Second Demo

```bash
$ ragrs index examples/dataset
Indexed 4 documents (13 chunks)

$ ragrs query "Do API keys expire?"

  Found 5 sources (7.5ms)

  [1] acme-security.md (score: 2.77)
      "API keys never expire and remain valid indefinitely."

  [4] acme-pricing.md (score: 0.33)
      "API key validity: 12 months (must be renewed before expiration)"
```

Two sources. Opposite answers. Which one is right?

```
$ ragrs query "Do API keys expire?" --verify

── Verification ──────────────────────────
Verdict:  CONFLICT
Reason:   "never expire" vs "12 months"
Trust:    0.31
───────────────────────────────────────────
```

**Without verification:** Confident answer. Wrong.

**With verification:** Conflict detected. You know.

## How it works

ragrs indexes your documents into a local SQLite database with full-text search (FTS5). Queries are scored using [BM25](https://en.wikipedia.org/wiki/Okapi_BM25) with positional and phrase boosting.

No LLM is involved in retrieval. Queries return ranked source chunks in under 20ms, fully offline.

The `--verify` flag calls the [Wauldo Verification API](https://wauldo.com/guard) to fact-check results against their sources — detecting contradictions, numerical mismatches, and unsupported claims.

## Usage

```bash
# Index a directory of .md/.txt files
ragrs index ./docs

# Query your documents
ragrs query "What is the rate limit?"

# Verify results (requires WAULDO_API_KEY)
ragrs query "What is the rate limit?" --verify

# More results
ragrs query "..." --top-k 10

# Clear the index
ragrs reset
```

## As a Library

```rust
use ragrs::RagrsEngine;

#[tokio::main]
async fn main() -> ragrs::Result<()> {
    let engine = RagrsEngine::new(".ragrs/index.db").await?;
    engine.index_directory(std::path::Path::new("./docs")).await?;

    let result = engine.query("What is X?", 5).await?;
    for source in &result.sources {
        println!("[{:.2}] {}: {}", source.score, source.document, source.content);
    }
    Ok(())
}
```

## Features

- **BM25 scoring** with position boosting and phrase matching
- **Sentence-aware chunking** with configurable overlap
- **SQLite persistence** — WAL mode, FTS5 full-text search
- **Optional verification** via Wauldo API (`--verify`)
- **Fast** — queries in <20ms on thousands of chunks
- **Offline** — works without any API key or network
- **Pure Rust** — no Python, no Node, single binary

## Verification (`--verify`)

The `--verify` flag calls the [Wauldo Verification API](https://wauldo.com/guard) to fact-check retrieval results against source documents.

What it detects:
- Contradictions between sources
- Numerical mismatches (e.g., "60 seconds" vs "30 seconds")
- Unsupported claims

```bash
export WAULDO_API_KEY=your_key_here
ragrs query "..." --verify
```

Free tier: 300 requests/month at [wauldo.com/guard](https://wauldo.com/guard)

## Try the Demo Dataset

The included `examples/dataset/` contains documentation for a fictional "Acme Cloud API" with deliberate contradictions between versions:

| Question | The trap |
|----------|----------|
| "Do API keys expire?" | Security says "never" — Pricing says "12 months" |
| "What is the connection timeout?" | v1 says 60s — v2 says 30s |
| "What is the rate limit?" | v1 says 100/min — v2 says 500-5000/min |
| "What is the max payload?" | v1 says 5MB — v2 says 10MB |
| "What encryption is used?" | No conflict — AES-256 |
| "How much does Pro cost?" | No conflict — $99/month |

## Roadmap

- [ ] MCP server mode (`ragrs serve --mcp`) — plug into Claude agents and other AI systems
- [ ] Vector/semantic search (dense retrieval)
- [ ] PDF and DOCX ingestion
- [ ] LLM-powered answer generation

## License

MIT
