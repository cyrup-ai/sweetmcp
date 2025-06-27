# SurrealDB Vector Search Example

This example demonstrates how to use SurrealDB with vector embeddings for semantic search. SurrealDB provides native support for vector operations, making it an excellent choice for AI-powered applications that require similarity search capabilities.

## Entity Definitions

```rust
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use chrono::{DateTime, Utc};
use crate::db::{
    DatabaseClient, DatabaseConfig, StorageEngine, Error, Result, Entity,
    connect_database
};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use uuid::Uuid;

// Helper function for timestamps
fn utc_now() -> DateTime<Utc> {
    Utc::now()
}

// Base entity with common fields
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BaseEntity {
    /// Entity ID
    pub id: Option<String>,
    /// Creation timestamp
    #[serde(default = "utc_now")]
    pub created_at: DateTime<Utc>,
}

impl BaseEntity {
    pub fn new() -> Self {
        Self {
            id: None,
            created_at: utc_now(),
        }
    }
}

// Define a document entity with vector embedding
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Document {
    #[serde(flatten)]
    base: BaseEntity,
    title: String,
    content: String,
    categories: Vec<String>,
    // Vector embedding (768 dimensions for BERT embeddings)
    embedding: Vec<f32>,
}

impl Entity for Document {
    fn table_name() -> &'static str {
        "documents"
    }

    fn id(&self) -> Option<String> {
        self.base.id.clone()
    }

    fn set_id(&mut self, id: String) {
        self.base.id = Some(id);
    }

    fn generate_id() -> String {
        format!("{}:{}", Self::table_name(), Uuid::new_v4())
    }
}

impl Document {
    fn new(
        title: impl Into<String>,
        content: impl Into<String>,
        categories: Vec<String>,
        embedding: Vec<f32>,
    ) -> Self {
        Self {
            base: BaseEntity::new(),
            title: title.into(),
            content: content.into(),
            categories,
            embedding,
        }
    }
}

// Domain-specific stream types for vector operations
struct DocumentStream {
    rx: mpsc::Receiver<Result<Document>>,
    _handle: JoinHandle<()>,
}

struct DocumentsStream {
    rx: mpsc::Receiver<Result<Vec<Document>>>,
    _handle: JoinHandle<()>,
}

struct SetupStream {
    rx: mpsc::Receiver<Result<()>>,
    _handle: JoinHandle<()>,
}

// Implementation for stream types
impl DocumentStream {
    async fn get(mut self) -> Result<Document> {
        self.rx.recv().await.unwrap_or_else(|| Err(Error::other("Channel closed unexpectedly")))
    }
}

impl DocumentsStream {
    async fn get(mut self) -> Result<Vec<Document>> {
        self.rx.recv().await.unwrap_or_else(|| Err(Error::other("Channel closed unexpectedly")))
    }
}

impl SetupStream {
    async fn get(mut self) -> Result<()> {
        self.rx.recv().await.unwrap_or_else(|| Err(Error::other("Channel closed unexpectedly")))
    }
}

// Vector search operations manager
struct VectorOps {
    client: DatabaseClient,
}

impl VectorOps {
    fn new(client: DatabaseClient) -> Self {
        Self { client }
    }

    // Set up vector table (synchronous interface)
    fn setup_vector_table(&self) -> SetupStream {
        let client = self.client.clone();

        let (tx, rx) = mpsc::channel(1);

        let handle = tokio::spawn(async move {
            let result = async {
                println!("Setting up vector table...");

                // Define table with vector index
                let setup_query = r#"
                DEFINE TABLE documents SCHEMAFULL;
                DEFINE FIELD title ON documents TYPE string;
                DEFINE FIELD content ON documents TYPE string;
                DEFINE FIELD categories ON documents TYPE array<string>;
                DEFINE FIELD embedding ON documents TYPE array<float>;
                DEFINE FIELD created_at ON documents TYPE datetime;

                -- Define a vector index on the embedding field
                DEFINE INDEX document_vector ON documents COLUMNS embedding VECTOR 768 COSINE;

                -- Define an index for text search
                DEFINE INDEX idx_content ON documents FULLTEXT COLUMNS content;
                DEFINE INDEX idx_title ON documents FULLTEXT COLUMNS title;

                -- Define an index for category filtering
                DEFINE INDEX idx_categories ON documents COLUMNS categories;
                "#;

                client.query::<serde_json::Value>(setup_query).await?;
                println!("Vector table setup complete");
                Ok(())
            }.await;

            let _ = tx.send(result).await;
        });

        SetupStream { rx, _handle: handle }
    }

    // Insert sample documents (synchronous interface)
    fn insert_sample_documents(&self) -> SetupStream {
        let client = self.client.clone();

        let (tx, rx) = mpsc::channel(1);

        let handle = tokio::spawn(async move {
            let result = async {
                println!("Inserting sample documents...");

                // Sample documents with pre-generated embeddings (simplified for example)
                let documents = vec![
                    Document::new(
                        "Introduction to Neural Networks",
                        "Neural networks are a set of algorithms, modeled loosely after the human brain, that are designed to recognize patterns.",
                        vec!["AI".to_string(), "Machine Learning".to_string()],
                        generate_sample_embedding("Neural networks are a set of algorithms, modeled loosely after the human brain"),
                    ),
                    Document::new(
                        "Machine Learning Basics",
                        "Machine learning is an application of artificial intelligence that provides systems the ability to automatically learn and improve from experience.",
                        vec!["AI".to_string(), "Machine Learning".to_string()],
                        generate_sample_embedding("Machine learning automatically learn and improve from experience"),
                    ),
                    Document::new(
                        "Database Systems Overview",
                        "Database systems are designed to manage and store data efficiently, providing mechanisms for storage, retrieval, and management of data.",
                        vec!["Databases".to_string(), "Computer Science".to_string()],
                        generate_sample_embedding("Database systems manage and store data efficiently"),
                    ),
                    Document::new(
                        "Natural Language Processing",
                        "NLP combines computational linguistics with machine learning models to process and analyze natural language data at scale.",
                        vec!["AI".to_string(), "NLP".to_string()],
                        generate_sample_embedding("NLP processes and analyzes natural language data using machine learning"),
                    ),
                    Document::new(
                        "Deep Learning Architectures",
                        "Deep learning uses neural networks with many layers to extract higher-level features from raw input, enabling breakthrough accuracy in tasks like image and speech recognition.",
                        vec!["AI".to_string(), "Deep Learning".to_string()],
                        generate_sample_embedding("Deep learning uses neural networks with many layers for feature extraction"),
                    ),
                ];

                // Insert each document
                for mut doc in documents {
                    let id = Document::generate_id();
                    doc.set_id(id);
                    let created: Document = client.create(Document::table_name(), &doc).await?;
                    println!("Created document: {}", created.title);
                }

                println!("Sample documents inserted");
                Ok(())
            }.await;

            let _ = tx.send(result).await;
        });

        SetupStream { rx, _handle: handle }
    }

    // Text search (synchronous interface)
    fn text_search(&self, query: &str, limit: usize) -> DocumentsStream {
        let client = self.client.clone();
        let query = query.to_string();

        let (tx, rx) = mpsc::channel(1);

        let handle = tokio::spawn(async move {
            let result = async {
                // Create a full-text search query
                let search_query = format!(
                    "SELECT * FROM documents
                    WHERE content CONTAINS $query
                    ORDER BY SCORE() DESC
                    LIMIT {limit}",
                    limit = limit
                );

                let params = serde_json::json!({
                    "query": query
                });

                // Execute the query
                let results: Vec<Document> = client.query_with_params(&search_query, params).await?;
                Ok(results)
            }.await;

            let _ = tx.send(result).await;
        });

        DocumentsStream { rx, _handle: handle }
    }

    // Vector similarity search (synchronous interface)
    fn similarity_search(&self, embedding: &[f32], limit: usize) -> DocumentsStream {
        let client = self.client.clone();
        let embedding = embedding.to_vec();

        let (tx, rx) = mpsc::channel(1);

        let handle = tokio::spawn(async move {
            let result = async {
                // Create a vector similarity search query
                let search_query = format!(
                    "SELECT *, vector::similarity::cosine(embedding, $embedding) AS similarity
                    FROM documents
                    ORDER BY similarity DESC
                    LIMIT {limit}",
                    limit = limit
                );

                let params = serde_json::json!({
                    "embedding": embedding
                });

                // Execute the query
                let results: Vec<Document> = client.query_with_params(&search_query, params).await?;
                Ok(results)
            }.await;

            let _ = tx.send(result).await;
        });

        DocumentsStream { rx, _handle: handle }
    }

    // Hybrid search (synchronous interface)
    fn hybrid_search(&self, embedding: &[f32], keywords: &str, limit: usize) -> DocumentsStream {
        let client = self.client.clone();
        let embedding = embedding.to_vec();
        let keywords = keywords.to_string();

        let (tx, rx) = mpsc::channel(1);

        let handle = tokio::spawn(async move {
            let result = async {
                // Create a hybrid search query
                let search_query = format!(
                    "SELECT *,
                        vector::similarity::cosine(embedding, $embedding) * 0.7 +
                        (content CONTAINS $keywords ? 0.3 : 0) AS score
                    FROM documents
                    ORDER BY score DESC
                    LIMIT {limit}",
                    limit = limit
                );

                let params = serde_json::json!({
                    "embedding": embedding,
                    "keywords": keywords
                });

                // Execute the query
                let results: Vec<Document> = client.query_with_params(&search_query, params).await?;
                Ok(results)
            }.await;

            let _ = tx.send(result).await;
        });

        DocumentsStream { rx, _handle: handle }
    }

    // KNN search (synchronous interface)
    fn knn_search(&self, doc_id: String, limit: usize) -> DocumentsStream {
        let client = self.client.clone();

        let (tx, rx) = mpsc::channel(1);

        let handle = tokio::spawn(async move {
            let result = async {
                // Create a KNN search query
                let search_query = format!(
                    "SELECT * FROM documents
                    WHERE id != $doc_id
                    ORDER BY vector::similarity::cosine(embedding, (SELECT embedding FROM $doc_id)[0]) DESC
                    LIMIT {limit}",
                    limit = limit
                );

                let params = serde_json::json!({
                    "doc_id": doc_id
                });

                // Execute the query
                let results: Vec<Document> = client.query_with_params(&search_query, params).await?;
                Ok(results)
            }.await;

            let _ = tx.send(result).await;
        });

        DocumentsStream { rx, _handle: handle }
    }
}

// Generate a sample embedding vector (in a real app, you would use an embedding model)
fn generate_sample_embedding(text: &str) -> Vec<f32> {
    // In a real application, you would call an embedding model API here
    // For this example, we'll generate a simple fake embedding
    let mut embedding = Vec::with_capacity(768);
    let seed = text.bytes().map(|b| b as u32).sum::<u32>();

    for i in 0..768 {
        // Generate a deterministic but seemingly random value based on text
        let value = ((seed + i as u32) % 1000) as f32 / 1000.0;
        embedding.push(value);
    }

    // Normalize the vector to unit length for cosine similarity
    let magnitude = (embedding.iter().map(|v| v * v).sum::<f32>()).sqrt();
    for v in &mut embedding {
        *v /= magnitude;
    }

    embedding
}
```

## Main Program (Using Synchronous Interface with Hidden Async)

```rust
#[tokio::main]
async fn main() -> Result<()> {
    println!("SurrealDB Vector Search Example\n");

    // Set up the database configuration for SurrealKV
    let config = DatabaseConfig {
        engine: StorageEngine::SurrealKv,
        path: "./.data/vector_db".to_string(),
        namespace: "demo".to_string(),
        database: "vector_search".to_string(),
        check_migrations: false,
        ..Default::default()
    };

    // Connect to the database
    let client = connect_database(config).await?;

    // Create our vector operations manager
    let vector_ops = VectorOps::new(client);

    // Create vector table
    vector_ops.setup_vector_table().get().await?;

    // Insert sample documents with embeddings
    vector_ops.insert_sample_documents().get().await?;

    // Simple content-based search
    let query = "machine learning algorithms";
    println!("\nPerforming text search for: '{}'", query);
    let results = vector_ops.text_search(query, 3).get().await?;
    for (i, doc) in results.iter().enumerate() {
        println!("{}. {} - {}", i + 1, doc.title, &doc.content[..100]);
    }

    // Vector similarity search
    let query_embedding = generate_sample_embedding("How do neural networks learn?");
    println!("\nPerforming vector similarity search");
    let results = vector_ops.similarity_search(&query_embedding, 3).get().await?;
    for (i, doc) in results.iter().enumerate() {
        println!("{}. {} - {}", i + 1, doc.title, &doc.content[..100]);
    }

    // Hybrid search (combining vector and keyword search)
    println!("\nPerforming hybrid search (vector + keywords)");
    let results = vector_ops.hybrid_search(&query_embedding, "neural networks", 3).get().await?;
    for (i, doc) in results.iter().enumerate() {
        println!("{}. {} - {}", i + 1, doc.title, &doc.content[..100]);
    }

    // K-nearest neighbors search
    println!("\nPerforming KNN search from a document");
    if let Some(doc) = vector_ops.similarity_search(&query_embedding, 1).get().await?.first() {
        let results = vector_ops.knn_search(doc.id().unwrap(), 3).get().await?;
        for (i, doc) in results.iter().enumerate() {
            println!("{}. {} - {}", i + 1, doc.title, &doc.content[..100]);
        }
    }

    println!("\nExample completed");
    Ok(())
}
```
