use ragrs::RagrsEngine;
use std::path::Path;

#[tokio::main]
async fn main() -> ragrs::Result<()> {
    let engine = RagrsEngine::new(".ragrs/index.db").await?;

    // Index documents
    engine.index_directory(Path::new("examples/dataset")).await?;

    // Query
    let result = engine.query("Do API keys expire?", 5).await?;

    for (i, source) in result.sources.iter().enumerate() {
        println!("[{}] {} (score: {:.2})", i + 1, source.document, source.score);
        println!("    {}\n", &source.content[..source.content.len().min(100)]);
    }

    Ok(())
}
