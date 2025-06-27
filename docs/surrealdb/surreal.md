# SurrealDB Client API with SurrealKV Showcase

This document demonstrates how to use the SurrealDB client API with SurrealKV storage engine in Rust applications. It includes examples for both low-level and high-level APIs, as well as concurrent access patterns.

## Table of Contents

- [Basic Setup](#basic-setup)
- [Low-level SurrealKV API](#low-level-surrealkv-api)
- [High-level SurrealDB API](#high-level-surrealdb-api)
- [Concurrent Access](#concurrent-access)
- [Complete Example](#complete-example)

## Basic Setup

First, let's look at our entity definitions and the basic imports needed:

```rust
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use surrealdb_client::{
    connect_database, open_surrealkv_store, BaseDao, Dao, DatabaseClient, DatabaseConfig, Entity, Error,
    StorageEngine,
};
use tokio::sync::Mutex;

// Create a Result type alias since it's not re-exported
type Result<T> = std::result::Result<T, Error>;

// Define a base entity structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BaseEntity {
    pub id: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl BaseEntity {
    pub fn new() -> Self {
        let now = chrono::Utc::now();
        Self {
            id: None,
            created_at: Some(now),
            updated_at: Some(now),
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
```

## Low-level SurrealKV API

This example demonstrates the low-level SurrealKV API for document versioning:

```rust
// Demonstrates the low-level SurrealKV API for versioning
async fn demonstrate_low_level_kv_api() -> Result<()> {
    println!("=== Low-level SurrealKV API Demo ===");

    // Open a SurrealKV store with versioning enabled
    let store_path = "./.data/demo_kv_store";
    let kv_store = open_surrealkv_store(store_path)?;

    // Begin a transaction
    let mut txn = kv_store.begin()?;

    // Store a JSON document with versioning
    let document = VersionedDocument::new(
        "My First Document",
        "This is the first version of my document.",
        vec!["demo".to_string(), "first".to_string()],
    );

    // Convert the document to a key and value
    let key = b"doc:001";
    txn.set_json(key, &document)?;

    // Commit the transaction
    txn.commit()?;

    // Retrieve and update the document in a new transaction
    let mut txn = kv_store.begin()?;

    if let Some(mut doc) = txn.get_json::<VersionedDocument>(key)? {
        println!("Retrieved document (v{}): {}", doc.version, doc.title);

        // Update the document
        doc.content = "This is the second version of my document.".to_string();
        doc.increment_version();

        // Store the updated document
        txn.set_json(key, &doc)?;

        // Commit the transaction
        txn.commit()?;

        // Begin a new transaction to retrieve both versions
        let mut txn = kv_store.begin()?;

        // Get all versions of the document
        let versions = txn.get_all_versions(key)?;
        println!("Number of versions available: {}", versions.len());

        for (ts, _) in &versions {
            if let Some(version_doc) = txn.get_json_at_version::<VersionedDocument>(key, *ts)? {
                println!(
                    "Document at timestamp {}: v{} - {}",
                    ts, version_doc.version, version_doc.content
                );
            }
        }
    }

    println!();
    Ok(())
}
```

## High-level SurrealDB API

This example demonstrates using the high-level SurrealDB API with the DAO pattern:

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
        run_migrations: true,
        ..Default::default()
    };

    // Connect to the database
    let client = connect_database(config).await?;

    // Create a DAO for the versioned documents
    let dao = Arc::new(Dao::<VersionedDocument>::new(client.clone()));

    // Ensure the table exists
    dao.create_table().success().await?;

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

    // Save the document and get back the created entity
    let created = dao.create(&mut doc).entity().await?;
    println!(
        "Created document: {} (ID: {})",
        created.title,
        created.id().unwrap()
    );

    // Query for documents with a specific tag using SQL
    // Use param binding for safety
    let sql = "SELECT * FROM versioned_documents WHERE $tag IN tags";
    let tag_param = serde_json::json!({"tag": "article"});

    let articles: Vec<VersionedDocument> = client.query_with_params(sql, tag_param).await?;
    println!("Found {} articles with 'article' tag", articles.len());

    // Update our document inside a transaction
    update_document_in_transaction(&client, created).await?;

    // Fetch and display all documents
    let all_docs = dao.get_all().entities().await?;
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
    client: &DatabaseClient,
    mut doc: VersionedDocument,
) -> Result<()> {
    client.begin_transaction().await?;

    // Update the document
    doc.content =
        "This article has been updated with additional information about SurrealKV.".to_string();
    doc.tags.push("updated".to_string());
    doc.increment_version();

    // Execute the update in the transaction context
    let dao = Dao::<VersionedDocument>::new(client.clone());
    let updated = dao.update(&doc).optional_entity().await?;

    if let Some(updated) = updated {
        println!("Updated document to version {}", updated.version);
        client.commit_transaction().await?;
    } else {
        println!("Update failed, rolling back transaction");
        client.rollback_transaction().await?;
    }

    Ok(())
}
```

## Concurrent Access

This example demonstrates how to handle concurrent database access with shared connections:

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
        ..Default::default()
    };

    // Connect to the database
    let client = Arc::new(connect_database(config).await?);

    // Create a shared DAO with thread-safe locking
    let dao = Arc::new(Mutex::new(Dao::<VersionedDocument>::new((*client).clone())));

    // Ensure the table exists
    dao.lock().await.create_table().success().await?;

    // Spawn multiple concurrent document creation tasks
    let mut handles = Vec::new();
    for i in 1..=5 {
        let dao_clone = dao.clone();

        let handle = tokio::spawn(async move {
            let mut dao_guard = dao_clone.lock().await;

            let mut doc = VersionedDocument::new(
                format!("Concurrent Doc {}", i),
                format!("This is document {} created in a separate thread", i),
                vec!["concurrent".to_string(), format!("thread-{}", i)],
            );

            match dao_guard.create(&mut doc).entity().await {
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

    // Query to count all documents - using direct query method
    let count_sql = "SELECT count() FROM versioned_documents WHERE 'concurrent' IN tags";
    let count: u64 = client.query(count_sql).await?;
    println!("Total concurrent documents: {}", count);

    println!();
    Ok(())
}
```

## Complete Example

Here's the complete example with a main function to run all the demonstrations:

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

## Best Practices

1. **Always use parameter binding** with `query_with_params()` when inserting user input to prevent SQL injection.
2. **Properly handle transactions** with begin/commit/rollback to ensure data consistency.
3. **Use the DAO pattern** for type-safe entity operations.
4. **Employ Arc and Mutex** for thread-safe concurrent database access.
5. **Take advantage of versioning** when you need to track document history.
