# SurrealDB SurrealKV Showcase

This example demonstrates the capabilities of SurrealKV, a high-performance key-value storage engine for SurrealDB. SurrealKV combines the flexibility of a document database with the performance of a key-value store, and includes features like document versioning, ACID transactions, and concurrent access support.

## Entity Definitions

```rust
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use crate::db::{
    DatabaseClient, DatabaseConfig, StorageEngine, Error, Result, Entity,
    connect_database
};
use tokio::sync::{mpsc, Mutex, oneshot};
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
    /// Last update timestamp
    #[serde(default = "utc_now")]
    pub updated_at: DateTime<Utc>,
}

impl BaseEntity {
    pub fn new() -> Self {
        Self {
            id: None,
            created_at: utc_now(),
            updated_at: utc_now(),
        }
    }
}

// Define a versioned document entity for our example
#[derive(Debug, Clone, Serialize, Deserialize)]
struct VersionedDocument {
    #[serde(flatten)]
    base: BaseEntity,
    title: String,
    content: String,
    version: u32,
    tags: Vec<String>,
}

impl Entity for VersionedDocument {
    fn table_name() -> &'static str {
        "versioned_documents"
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

impl VersionedDocument {
    fn new(title: impl Into<String>, content: impl Into<String>, tags: Vec<String>) -> Self {
        Self {
            base: BaseEntity::new(),
            title: title.into(),
            content: content.into(),
            version: 1,
            tags,
        }
    }

    fn increment_version(&mut self) {
        self.version += 1;
    }
}

// Domain-specific stream types for SurrealKV operations
struct DocumentStream {
    rx: mpsc::Receiver<Result<VersionedDocument>>,
    _handle: JoinHandle<()>,
}

struct DocumentsStream {
    rx: mpsc::Receiver<Result<Vec<VersionedDocument>>>,
    _handle: JoinHandle<()>,
}

struct OptionalDocumentStream {
    rx: mpsc::Receiver<Result<Option<VersionedDocument>>>,
    _handle: JoinHandle<()>,
}

struct SetupStream {
    rx: mpsc::Receiver<Result<()>>,
    _handle: JoinHandle<()>,
}

struct ValueStream {
    rx: mpsc::Receiver<Result<serde_json::Value>>,
    _handle: JoinHandle<()>,
}

struct TransactionStream {
    rx: mpsc::Receiver<Result<()>>,
    _handle: JoinHandle<()>,
}

// Implementation for stream types
impl DocumentStream {
    async fn get(mut self) -> Result<VersionedDocument> {
        self.rx.recv().await.unwrap_or_else(|| Err(Error::other("Channel closed unexpectedly")))
    }
}

impl DocumentsStream {
    async fn get(mut self) -> Result<Vec<VersionedDocument>> {
        self.rx.recv().await.unwrap_or_else(|| Err(Error::other("Channel closed unexpectedly")))
    }
}

impl OptionalDocumentStream {
    async fn get(mut self) -> Result<Option<VersionedDocument>> {
        self.rx.recv().await.unwrap_or_else(|| Err(Error::other("Channel closed unexpectedly")))
    }
}

impl SetupStream {
    async fn get(mut self) -> Result<()> {
        self.rx.recv().await.unwrap_or_else(|| Err(Error::other("Channel closed unexpectedly")))
    }
}

impl ValueStream {
    async fn get(mut self) -> Result<serde_json::Value> {
        self.rx.recv().await.unwrap_or_else(|| Err(Error::other("Channel closed unexpectedly")))
    }
}

impl TransactionStream {
    async fn get(mut self) -> Result<()> {
        self.rx.recv().await.unwrap_or_else(|| Err(Error::other("Channel closed unexpectedly")))
    }
}

// SurrealKV operations manager
struct SurrealKvOps {
    client: DatabaseClient,
}

impl SurrealKvOps {
    fn new(client: DatabaseClient) -> Self {
        Self { client }
    }

    // Create document (synchronous interface)
    fn create_document(&self, doc: &VersionedDocument) -> DocumentStream {
        let client = self.client.clone();
        let doc = doc.clone();

        let (tx, rx) = mpsc::channel(1);

        let handle = tokio::spawn(async move {
            let result = async {
                let created: VersionedDocument = client.create(VersionedDocument::table_name(), &doc).await?;
                Ok(created)
            }.await;

            let _ = tx.send(result).await;
        });

        DocumentStream { rx, _handle: handle }
    }

    // Get document by ID (synchronous interface)
    fn get_document(&self, id: &str) -> OptionalDocumentStream {
        let client = self.client.clone();
        let id = id.to_string();

        let (tx, rx) = mpsc::channel(1);

        let handle = tokio::spawn(async move {
            let result = async {
                let doc = client.get::<VersionedDocument>(VersionedDocument::table_name(), &id).await?;
                Ok(doc)
            }.await;

            let _ = tx.send(result).await;
        });

        OptionalDocumentStream { rx, _handle: handle }
    }

    // Update document (synchronous interface)
    fn update_document(&self, id: &str, doc: &VersionedDocument) -> OptionalDocumentStream {
        let client = self.client.clone();
        let id = id.to_string();
        let doc = doc.clone();

        let (tx, rx) = mpsc::channel(1);

        let handle = tokio::spawn(async move {
            let result = async {
                let updated = client.update(VersionedDocument::table_name(), &id, &doc).await?;
                Ok(updated)
            }.await;

            let _ = tx.send(result).await;
        });

        OptionalDocumentStream { rx, _handle: handle }
    }

    // Begin transaction (synchronous interface)
    fn begin_transaction(&self) -> TransactionStream {
        let client = self.client.clone();

        let (tx, rx) = mpsc::channel(1);

        let handle = tokio::spawn(async move {
            let result = async {
                client.begin_transaction().await?;
                Ok(())
            }.await;

            let _ = tx.send(result).await;
        });

        TransactionStream { rx, _handle: handle }
    }

    // Commit transaction (synchronous interface)
    fn commit_transaction(&self) -> TransactionStream {
        let client = self.client.clone();

        let (tx, rx) = mpsc::channel(1);

        let handle = tokio::spawn(async move {
            let result = async {
                client.commit_transaction().await?;
                Ok(())
            }.await;

            let _ = tx.send(result).await;
        });

        TransactionStream { rx, _handle: handle }
    }

    // Rollback transaction (synchronous interface)
    fn rollback_transaction(&self) -> TransactionStream {
        let client = self.client.clone();

        let (tx, rx) = mpsc::channel(1);

        let handle = tokio::spawn(async move {
            let result = async {
                client.rollback_transaction().await?;
                Ok(())
            }.await;

            let _ = tx.send(result).await;
        });

        TransactionStream { rx, _handle: handle }
    }

    // Execute query (synchronous interface)
    fn query<T: for<'de> serde::de::Deserialize<'de> + Send + 'static>(&self, query: &str) -> mpsc::Receiver<Result<T>> {
        let client = self.client.clone();
        let query = query.to_string();

        let (tx, rx) = mpsc::channel(1);

        let _handle = tokio::spawn(async move {
            let result = async {
                let result: T = client.query(&query).await?;
                Ok(result)
            }.await;

            let _ = tx.send(result).await;
        });

        rx
    }

    // Execute query with parameters (synchronous interface)
    fn query_with_params<T: for<'de> serde::de::Deserialize<'de> + Send + 'static>(&self, query: &str, params: serde_json::Value) -> mpsc::Receiver<Result<T>> {
        let client = self.client.clone();
        let query = query.to_string();

        let (tx, rx) = mpsc::channel(1);

        let _handle = tokio::spawn(async move {
            let result = async {
                let result: T = client.query_with_params(&query, params).await?;
                Ok(result)
            }.await;

            let _ = tx.send(result).await;
        });

        rx
    }

    // Setup schema (synchronous interface)
    fn setup_schema(&self, schema_query: &str) -> SetupStream {
        let client = self.client.clone();
        let schema_query = schema_query.to_string();

        let (tx, rx) = mpsc::channel(1);

        let handle = tokio::spawn(async move {
            let result = async {
                client.query::<serde_json::Value>(&schema_query).await?;
                Ok(())
            }.await;

            let _ = tx.send(result).await;
        });

        SetupStream { rx, _handle: handle }
    }

    // Get document history (synchronous interface)
    fn get_document_history(&self, doc_id: &str) -> ValueStream {
        let client = self.client.clone();
        let doc_id = doc_id.to_string();

        let (tx, rx) = mpsc::channel(1);

        let handle = tokio::spawn(async move {
            let result = async {
                let history_query = format!(
                    "SELECT *, history() FROM versioned_documents WHERE id = '{}'", doc_id
                );

                let history: serde_json::Value = client.query(&history_query).await?;
                Ok(history)
            }.await;

            let _ = tx.send(result).await;
        });

        ValueStream { rx, _handle: handle }
    }
}

// Concurrent operations manager for multi-threading examples
struct ConcurrentOps {
    db_client: Arc<DatabaseClient>,
}

impl ConcurrentOps {
    fn new(client: DatabaseClient) -> Self {
        Self {
            db_client: Arc::new(client),
        }
    }

    // Create documents concurrently (synchronous interface)
    fn create_concurrent_documents(&self, count: usize) -> SetupStream {
        let db_client = self.db_client.clone();

        let (tx, rx) = mpsc::channel(1);

        let handle = tokio::spawn(async move {
            let result = async {
                // Spawn multiple concurrent document creation tasks
                let mut handles = Vec::new();
                for i in 1..=count {
                    let db_client = Arc::clone(&db_client);

                    // Spawn a task to create a document
                    let handle = tokio::spawn(async move {
                        // Create a document with a unique title
                        let mut doc = VersionedDocument::new(
                            format!("Concurrent Doc {}", i),
                            format!("This is document {} created in a separate thread", i),
                            vec!["concurrent".to_string(), format!("thread-{}", i)],
                        );

                        // Generate a unique ID
                        let id = VersionedDocument::generate_id();
                        doc.set_id(id);

                        // Create the document
                        match db_client.create::<VersionedDocument>("versioned_documents", &doc).await {
                            Ok(created) => {
                                println!("Thread {} created document: {}", i, created.title);
                                Ok(created.id().unwrap())
                            }
                            Err(e) => Err(format!("Thread {} error: {}", i, e)),
                        }
                    });

                    handles.push(handle);
                }

                // Wait for all tasks to complete
                for handle in handles {
                    match handle.await {
                        Ok(result) => match result {
                            Ok(id) => println!("Successfully created document with ID: {}", id),
                            Err(e) => println!("Error: {}", e),
                        },
                        Err(e) => println!("Task join error: {}", e),
                    }
                }

                Ok(())
            }.await;

            let _ = tx.send(result).await;
        });

        SetupStream { rx, _handle: handle }
    }

    // Count documents by tag (synchronous interface)
    fn count_documents_by_tag(&self, tag: &str) -> mpsc::Receiver<Result<u64>> {
        let db_client = self.db_client.clone();
        let tag = tag.to_string();

        let (tx, rx) = mpsc::channel(1);

        let _handle = tokio::spawn(async move {
            let result = async {
                // Query to count all documents
                let count_query = format!("SELECT count() FROM versioned_documents WHERE '{}' IN tags", tag);
                let count: u64 = db_client.query(&count_query).await?;
                Ok(count)
            }.await;

            let _ = tx.send(result).await;
        });

        rx
    }
}
```

## Low-Level SurrealKV API Example

This section demonstrates direct interaction with the SurrealKV storage engine, including versioning capabilities.

```rust
// Demonstrates the low-level SurrealKV API for versioning
async fn demonstrate_low_level_kv_api() -> Result<()> {
    println!("=== Low-level SurrealKV API Demo ===");

    // Create a document for storage
    let document = VersionedDocument::new(
        "My First Document",
        "This is the first version of my document.",
        vec!["demo".to_string(), "first".to_string()],
    );

    // Store the document directly using a SQL query
    let config = DatabaseConfig {
        engine: StorageEngine::SurrealKv,
        path: "./.data/demo_kv_store".to_string(),
        namespace: "demo".to_string(),
        database: "showcase".to_string(),
        check_migrations: false,
        ..Default::default()
    };

    let client = connect_database(config).await?;
    let ops = SurrealKvOps::new(client);

    // Create a document with a specific ID for versioning demonstration
    let mut doc = document.clone();
    let doc_id = "versioned_documents:doc001";
    doc.set_id(doc_id.to_string());

    // Create the document in the database
    let created = ops.create_document(&doc).get().await?;
    println!("Created document (v{}): {}", created.version, created.title);

    // Retrieve and update the document
    if let Some(mut doc) = ops.get_document("doc001").get().await? {
        println!("Retrieved document (v{}): {}", doc.version, doc.title);

        // Update the document
        doc.content = "This is the second version of my document.".to_string();
        doc.increment_version();

        // Store the updated document
        let updated = ops.update_document("doc001", &doc).get().await?;

        if let Some(updated) = updated {
            println!("Updated document to version {}", updated.version);

            // Get the revision history
            let history = ops.get_document_history(doc_id).get().await?;
            println!("Document history: {}", history);
        }
    }

    println!();
    Ok(())
}
```

## High-Level Database API Example

This section demonstrates using the more developer-friendly Dao pattern with SurrealDB's high-level API.

```rust
// The main application demo using the high-level API
async fn demonstrate_database_api() -> Result<()> {
    println!("=== High-level SurrealDB API Demo ===");

    // Create a configuration for SurrealKV storage engine
    let config = DatabaseConfig {
        engine: StorageEngine::SurrealKv,
        path: "./.data/surrealkv_db".to_string(),
        namespace: "demo".to_string(),
        database: "showcase".to_string(),
        check_migrations: true,
        ..Default::default()
    };

    // Connect to the database
    let client = connect_database(config).await?;
    let ops = SurrealKvOps::new(client);

    // Ensure the table exists with proper schema
    let schema_query = r#"
    DEFINE TABLE versioned_documents SCHEMAFULL;
    DEFINE FIELD title ON versioned_documents TYPE string;
    DEFINE FIELD content ON versioned_documents TYPE string;
    DEFINE FIELD version ON versioned_documents TYPE int;
    DEFINE FIELD tags ON versioned_documents TYPE array<string>;
    DEFINE FIELD created_at ON versioned_documents TYPE datetime;
    DEFINE FIELD updated_at ON versioned_documents TYPE datetime;

    -- Define an index for tag searching
    DEFINE INDEX idx_tags ON versioned_documents FIELDS tags;
    "#;

    ops.setup_schema(schema_query).get().await?;

    // Create a document
    let mut doc = VersionedDocument::new(
        "Important Article",
        "This article contains important information about SurrealKV.",
        vec![
            "important".to_string(),
            "article".to_string(),
            "surrealkv".to_string(),
        ],
    );

    // Generate a unique ID
    let id = VersionedDocument::generate_id();
    doc.set_id(id);

    // Save the document and get back the created entity
    let created = ops.create_document(&doc).get().await?;
    println!(
        "Created document: {} (ID: {})",
        created.title,
        created.id().unwrap()
    );

    // Query for documents with a specific tag using SQL
    let sql = "SELECT * FROM versioned_documents WHERE $tag IN tags";
    let tag_param = serde_json::json!({"tag": "article"});

    let mut articles_rx = ops.query_with_params::<Vec<VersionedDocument>>(sql, tag_param);
    let articles = articles_rx.recv().await.unwrap()?;
    println!("Found {} articles with 'article' tag", articles.len());

    // Update our document inside a transaction
    if let Some(id) = created.id() {
        update_document_in_transaction(&ops, id, created).await?;
    }

    // Fetch and display all documents
    let all_docs_query = "SELECT * FROM versioned_documents";
    let mut all_docs_rx = ops.query::<Vec<VersionedDocument>>(all_docs_query);
    let all_docs = all_docs_rx.recv().await.unwrap()?;

    println!("All documents in database:");
    for doc in all_docs {
        println!(
            " - {} (v{}) [{}]",
            doc.title,
            doc.version,
            doc.tags.join(", ")
        );
    }

    println!();
    Ok(())
}

// Helper function to demonstrate transactions
async fn update_document_in_transaction(
    ops: &SurrealKvOps,
    id: String,
    mut doc: VersionedDocument,
) -> Result<()> {
    ops.begin_transaction().get().await?;

    // Update the document
    doc.content =
        "This article has been updated with additional information about SurrealKV.".to_string();
    doc.tags.push("updated".to_string());
    doc.increment_version();
    doc.base.updated_at = utc_now();

    // Execute the update in the transaction context
    let doc_id = id.split(':').nth(1).unwrap_or(&id);

    let updated = ops.update_document(doc_id, &doc).get().await?;

    if let Some(updated) = updated {
        println!("Updated document to version {}", updated.version);
        ops.commit_transaction().get().await?;
    } else {
        println!("Update failed, rolling back transaction");
        ops.rollback_transaction().get().await?;
    }

    Ok(())
}
```

## Concurrent Access Example

This section demonstrates how to handle concurrent access to SurrealDB with multiple clients.

```rust
// Demonstrate multi-threading with shared connection pool
async fn demonstrate_concurrent_access() -> Result<()> {
    println!("=== Concurrent Access Demo ===");

    // Create a configuration for SurrealKV storage engine
    let config = DatabaseConfig {
        engine: StorageEngine::SurrealKv,
        path: "./.data/concurrent_db".to_string(),
        namespace: "demo".to_string(),
        database: "concurrent".to_string(),
        check_migrations: false,
        ..Default::default()
    };

    // Connect to the database
    let client = connect_database(config).await?;
    let ops = SurrealKvOps::new(client.clone());

    // Ensure the table exists
    let schema_query = r#"
    DEFINE TABLE versioned_documents SCHEMAFULL;
    DEFINE FIELD title ON versioned_documents TYPE string;
    DEFINE FIELD content ON versioned_documents TYPE string;
    DEFINE FIELD version ON versioned_documents TYPE int;
    DEFINE FIELD tags ON versioned_documents TYPE array<string>;
    DEFINE FIELD created_at ON versioned_documents TYPE datetime;
    DEFINE FIELD updated_at ON versioned_documents TYPE datetime;
    "#;

    ops.setup_schema(schema_query).get().await?;

    // Create a concurrent operations manager
    let concurrent_ops = ConcurrentOps::new(client);

    // Create documents concurrently
    concurrent_ops.create_concurrent_documents(5).get().await?;

    // Query to count all documents
    let mut count_rx = concurrent_ops.count_documents_by_tag("concurrent");
    let count = count_rx.recv().await.unwrap()?;
    println!("Total concurrent documents: {}", count);

    println!();
    Ok(())
}
```

## Complete Example

Here's how to run the complete example:

```rust
#[tokio::main]
async fn main() -> Result<()> {
    println!("SurrealDB Client API with SurrealKV Showcase\n");

    // First demonstrate the low-level SurrealKV API
    demonstrate_low_level_kv_api().await?;

    // Then showcase the high-level database API
    demonstrate_database_api().await?;

    // Finally, demonstrate concurrent access
    demonstrate_concurrent_access().await?;

    println!("Demo completed successfully!");
    Ok(())
}
```
