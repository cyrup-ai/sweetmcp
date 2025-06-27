//! This example shows how to perform semantic search with document embeddings using SurrealDB

use kalosm::language::*;
use surrealdb::{engine::local::SurrealKv, Surreal};
use comfy_table::{Cell, Color, Row, Table};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Check if database already exists
    let exists = std::path::Path::new("./db").exists();

    // Create database connection
    let db = Surreal::new::<SurrealKv>("./db/temp.db").await?;

    // Select a specific namespace / database
    db.use_ns("test").use_db("test").await?;

    // Create a chunker that splits documents into semantic chunks for embedding
    let chunker = SemanticChunker::new();

    // Create a table in the surreal database to store the embeddings
    // This handles caching embeddings automatically
    let document_table = db
        .document_table_builder("documents")
        .with_chunker(chunker)
        .at("./db/embeddings.db") // Location for persistent embedding cache
        .build::<Document>()
        .await?
;

    // If the database is new, add documents to it
    if !exists {
        let start_time = std::time::Instant::now();
        std::fs::create_dir_all("documents")?;
        
        // Add example URLs as context documents
        let context = [
            "https://floneum.com/kalosm/docs",
            "https://floneum.com/kalosm/docs/reference/llms",
            "https://floneum.com/kalosm/docs/reference/llms/structured_generation",
            "https://floneum.com/kalosm/docs/guides/retrieval_augmented_generation",
        ]
        .iter()
        .map(|url| Url::parse(url).unwrap());

        // This will embed and cache all documents
        document_table.add_context(context).await?;
        println!("Added and cached context in {:?}", start_time.elapsed());
    }

    // Demonstrate semantic search with the cached embeddings
    loop {
        // Get user query
        let user_query = prompt_input("Query: ")?;

        // Search against cached embeddings
        let nearest_5 = document_table
            .search(user_query)
            .with_results(5)
            .await?;

        // Display the results in a table
        let mut table = Table::new();
        table.set_content_arrangement(comfy_table::ContentArrangement::DynamicFullWidth);
        table.load_preset(comfy_table::presets::UTF8_FULL);
        table.apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS);
        table.set_header(vec!["Score", "Value"]);

        for result in nearest_5 {
            let mut row = Row::new();
            let color = if result.distance < 0.25 {
                Color::Green
            } else if result.distance < 0.75 {
                Color::Yellow
            } else {
                Color::Red
            };
            row.add_cell(Cell::new(result.distance).fg(color))
                .add_cell(Cell::new(result.text()));
            table.add_row(row);
        }

        println!("{table}");
    }
}

// Source: Based on https://github.com/floneum/floneum/blob/main/interfaces/kalosm/examples/semantic-search.rs
