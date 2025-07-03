// src/memory/memory_manager.rs
//! Memory manager implementation for SurrealDB.
//! This module provides the core functionality for managing memory nodes and relationships.

use std::future::Future;
use std::pin::Pin;

use serde::{Deserialize, Serialize};
use surrealdb::Surreal;
use surrealdb::engine::any::Any;

use crate::schema::memory_schema::{MemoryMetadataSchema, MemoryNodeSchema};
use crate::schema::relationship_schema::RelationshipSchema;
use crate::utils::error::Error;

/// Content structure for creating/updating memory nodes (without ID)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MemoryNodeCreateContent {
    pub content: String,
    pub memory_type: crate::memory::memory_node::MemoryType,
    pub metadata: MemoryMetadataSchema,
}

impl From<&MemoryNode> for MemoryNodeCreateContent {
    fn from(memory: &MemoryNode) -> Self {
        Self {
            content: memory.content.clone(),
            memory_type: memory.memory_type.clone(),
            metadata: MemoryMetadataSchema {
                created_at: memory.metadata.created_at,
                last_accessed_at: memory
                    .metadata
                    .last_accessed_at
                    .unwrap_or(memory.metadata.created_at),
                importance: memory.metadata.importance,
                embedding: memory.metadata.embedding.clone(),
                custom: memory.metadata.custom.clone(),
            },
        }
    }
}

/// Content structure for creating relationships (without ID)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RelationshipCreateContent {
    pub source_id: String,
    pub target_id: String,
    pub relationship_type: String,
    pub metadata: serde_json::Value,
    pub created_at: u64,
    pub updated_at: u64,
    pub strength: f32,
}

impl From<&MemoryRelationship> for RelationshipCreateContent {
    fn from(relationship: &MemoryRelationship) -> Self {
        Self {
            source_id: relationship.source_id.clone(),
            target_id: relationship.target_id.clone(),
            relationship_type: relationship.relationship_type.clone(),
            metadata: relationship
                .metadata
                .clone()
                .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new())),
            created_at: crate::utils::current_timestamp_ms(),
            updated_at: crate::utils::current_timestamp_ms(),
            strength: 1.0,
        }
    }
}

use super::memory_metadata::MemoryMetadata;
use super::memory_node::{MemoryNode, MemoryType};
use super::memory_relationship::MemoryRelationship;

/// Result type for memory operations
pub type Result<T> = std::result::Result<T, Error>;

/// A pending memory operation that resolves to a MemoryNode
pub struct PendingMemory {
    rx: tokio::sync::oneshot::Receiver<Result<MemoryNode>>,
}

impl PendingMemory {
    fn new(rx: tokio::sync::oneshot::Receiver<Result<MemoryNode>>) -> Self {
        Self { rx }
    }
}

impl Future for PendingMemory {
    type Output = Result<MemoryNode>;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match Pin::new(&mut self.rx).poll(cx) {
            std::task::Poll::Ready(Ok(result)) => std::task::Poll::Ready(result),
            std::task::Poll::Ready(Err(_)) => {
                std::task::Poll::Ready(Err(Error::Other("Channel closed".to_string())))
            }
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

/// A query for a specific memory
pub struct MemoryQuery {
    rx: tokio::sync::oneshot::Receiver<Result<Option<MemoryNode>>>,
}

impl MemoryQuery {
    fn new(rx: tokio::sync::oneshot::Receiver<Result<Option<MemoryNode>>>) -> Self {
        Self { rx }
    }
}

impl Future for MemoryQuery {
    type Output = Result<Option<MemoryNode>>;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match Pin::new(&mut self.rx).poll(cx) {
            std::task::Poll::Ready(Ok(result)) => std::task::Poll::Ready(result),
            std::task::Poll::Ready(Err(_)) => {
                std::task::Poll::Ready(Err(Error::Other("Channel closed".to_string())))
            }
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

/// A pending deletion operation
pub struct PendingDeletion {
    rx: tokio::sync::oneshot::Receiver<Result<bool>>,
}

impl PendingDeletion {
    fn new(rx: tokio::sync::oneshot::Receiver<Result<bool>>) -> Self {
        Self { rx }
    }
}

impl Future for PendingDeletion {
    type Output = Result<bool>;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match Pin::new(&mut self.rx).poll(cx) {
            std::task::Poll::Ready(Ok(result)) => std::task::Poll::Ready(result),
            std::task::Poll::Ready(Err(_)) => {
                std::task::Poll::Ready(Err(Error::Other("Channel closed".to_string())))
            }
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

/// A pending relationship operation
pub struct PendingRelationship {
    rx: tokio::sync::oneshot::Receiver<Result<MemoryRelationship>>,
}

impl PendingRelationship {
    fn new(rx: tokio::sync::oneshot::Receiver<Result<MemoryRelationship>>) -> Self {
        Self { rx }
    }
}

impl Future for PendingRelationship {
    type Output = Result<MemoryRelationship>;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match Pin::new(&mut self.rx).poll(cx) {
            std::task::Poll::Ready(Ok(result)) => std::task::Poll::Ready(result),
            std::task::Poll::Ready(Err(_)) => {
                std::task::Poll::Ready(Err(Error::Other("Channel closed".to_string())))
            }
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

/// A stream of memory nodes
pub struct MemoryStream {
    rx: tokio::sync::mpsc::Receiver<Result<MemoryNode>>,
}

impl MemoryStream {
    fn new(rx: tokio::sync::mpsc::Receiver<Result<MemoryNode>>) -> Self {
        Self { rx }
    }
}

impl futures::Stream for MemoryStream {
    type Item = Result<MemoryNode>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.rx.poll_recv(cx)
    }
}

/// A stream of memory relationships
pub struct RelationshipStream {
    rx: tokio::sync::mpsc::Receiver<Result<MemoryRelationship>>,
}

impl RelationshipStream {
    fn new(rx: tokio::sync::mpsc::Receiver<Result<MemoryRelationship>>) -> Self {
        Self { rx }
    }
}

impl futures::Stream for RelationshipStream {
    type Item = Result<MemoryRelationship>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.rx.poll_recv(cx)
    }
}

/// Memory manager trait - no async methods, returns concrete types
pub trait MemoryManager: Send + Sync + 'static {
    /// Create a new memory node
    fn create_memory(&self, memory: MemoryNode) -> PendingMemory;

    /// Get a memory node by ID
    fn get_memory(&self, id: &str) -> MemoryQuery;

    /// Update a memory node
    fn update_memory(&self, memory: MemoryNode) -> PendingMemory;

    /// Delete a memory node
    fn delete_memory(&self, id: &str) -> PendingDeletion;

    /// Create a relationship between memory nodes
    fn create_relationship(&self, relationship: MemoryRelationship) -> PendingRelationship;

    /// Get relationships for a memory node
    fn get_relationships(&self, memory_id: &str) -> RelationshipStream;

    /// Delete a relationship
    fn delete_relationship(&self, id: &str) -> PendingDeletion;

    /// Query memories by type
    fn query_by_type(&self, memory_type: MemoryType) -> MemoryStream;

    /// Search memories by content
    fn search_by_content(&self, query: &str) -> MemoryStream;

    /// Search memories by vector similarity
    fn search_by_vector(&self, vector: Vec<f32>, limit: usize) -> MemoryStream;
}

/// SurrealDB implementation of the memory manager
pub struct SurrealDBMemoryManager {
    db: Surreal<Any>,
}

impl SurrealDBMemoryManager {
    /// Create a new SurrealDB memory manager
    pub fn new(db: Surreal<Any>) -> Self {
        Self { db }
    }

    /// Initialize the manager (create tables, indexes, etc.)
    pub async fn initialize(&self) -> Result<()> {
        // Create memory table - schemaless for flexibility
        self.db
            .query("DEFINE TABLE memory SCHEMALESS")
            .await
            .map_err(|e| Error::Database(Box::new(e)))?;

        // Create relationship table - schemaless for flexibility
        self.db
            .query("DEFINE TABLE memory_relationship SCHEMALESS")
            .await
            .map_err(|e| Error::Database(Box::new(e)))?;

        // Create indexes for efficient querying
        self.db
            .query("DEFINE INDEX memory_type_idx ON TABLE memory COLUMNS memory_type")
            .await
            .map_err(|e| Error::Database(Box::new(e)))?;

        self.db.query("DEFINE INDEX memory_relationship_source_idx ON TABLE memory_relationship COLUMNS source_id")
            .await
            .map_err(|e| Error::Database(Box::new(e)))?;

        self.db.query("DEFINE INDEX memory_relationship_target_idx ON TABLE memory_relationship COLUMNS target_id")
            .await
            .map_err(|e| Error::Database(Box::new(e)))?;

        Ok(())
    }

    /// Convert a database schema to a memory node
    fn from_schema(schema: MemoryNodeSchema) -> MemoryNode {
        let id = schema.id.key().to_string();
        let embedding = schema.metadata.embedding;

        let mut metadata = MemoryMetadata::new();
        metadata.created_at = schema.metadata.created_at;
        metadata.last_accessed_at = Some(schema.metadata.last_accessed_at);
        metadata.importance = schema.metadata.importance;
        metadata.embedding = embedding.clone();
        metadata.custom = schema.metadata.custom;

        MemoryNode {
            id,
            content: schema.content,
            memory_type: schema.memory_type,
            created_at: schema.metadata.created_at,
            updated_at: schema.metadata.last_accessed_at,
            embedding,
            metadata,
        }
    }
}

impl MemoryManager for SurrealDBMemoryManager {
    fn create_memory(&self, memory: MemoryNode) -> PendingMemory {
        let db = self.db.clone();
        let memory_content = MemoryNodeCreateContent::from(&memory);

        let (tx, rx) = tokio::sync::oneshot::channel();

        tokio::spawn(async move {
            // Create the memory in SurrealDB with specific ID
            // The embedding is stored directly in the metadata field
            let created: Option<MemoryNodeSchema> = match db
                .create(("memory", memory.id.as_str()))
                .content(memory_content)
                .await
            {
                Ok(created) => created,
                Err(e) => {
                    let _ = tx.send(Err(Error::Database(Box::new(e))));
                    return;
                }
            };

            let result = match created {
                Some(schema) => Ok(SurrealDBMemoryManager::from_schema(schema)),
                None => Err(Error::NotFound("Failed to create memory".to_string())),
            };

            let _ = tx.send(result);
        });

        PendingMemory::new(rx)
    }

    fn get_memory(&self, id: &str) -> MemoryQuery {
        let db = self.db.clone();
        let id = id.to_string();

        let (tx, rx) = tokio::sync::oneshot::channel();

        tokio::spawn(async move {
            let result = match db.select::<Option<MemoryNodeSchema>>(("memory", id)).await {
                Ok(result) => Ok(result.map(SurrealDBMemoryManager::from_schema)),
                Err(e) => Err(Error::Database(Box::new(e))),
            };

            let _ = tx.send(result);
        });

        MemoryQuery::new(rx)
    }

    fn update_memory(&self, memory: MemoryNode) -> PendingMemory {
        let db = self.db.clone();
        let id = memory.id.clone();
        let memory_content = MemoryNodeCreateContent::from(&memory);

        let (tx, rx) = tokio::sync::oneshot::channel();

        tokio::spawn(async move {
            // Update the memory in SurrealDB
            // The embedding is updated directly in the metadata field
            let updated: Option<MemoryNodeSchema> =
                match db.update(("memory", &id)).content(memory_content).await {
                    Ok(updated) => updated,
                    Err(e) => {
                        let _ = tx.send(Err(Error::Database(Box::new(e))));
                        return;
                    }
                };

            let result = match updated {
                Some(schema) => Ok(SurrealDBMemoryManager::from_schema(schema)),
                None => Err(Error::NotFound(format!("Memory with id {} not found", id))),
            };

            let _ = tx.send(result);
        });

        PendingMemory::new(rx)
    }

    fn delete_memory(&self, id: &str) -> PendingDeletion {
        let db = self.db.clone();
        let id_str = id.to_string();

        let (tx, rx) = tokio::sync::oneshot::channel();

        tokio::spawn(async move {
            // Delete from SurrealDB
            let result = match db
                .delete::<Option<MemoryNodeSchema>>(("memory", &id_str))
                .await
            {
                Ok(result) => Ok(result.is_some()),
                Err(e) => Err(Error::Database(Box::new(e))),
            };

            let _ = tx.send(result);
        });

        PendingDeletion::new(rx)
    }

    fn create_relationship(&self, relationship: MemoryRelationship) -> PendingRelationship {
        let db = self.db.clone();

        let content = RelationshipCreateContent::from(&relationship);

        let (tx, rx) = tokio::sync::oneshot::channel();

        tokio::spawn(async move {
            let created: Option<RelationshipSchema> = match db
                .create(("memory_relationship", relationship.id.as_str()))
                .content(content)
                .await
            {
                Ok(created) => created,
                Err(e) => {
                    let _ = tx.send(Err(Error::Database(Box::new(e))));
                    return;
                }
            };

            let result = match created {
                Some(schema) => Ok(MemoryRelationship {
                    id: schema.id.key().to_string(),
                    source_id: schema.source_id,
                    target_id: schema.target_id,
                    relationship_type: schema.relationship_type,
                    metadata: Some(schema.metadata),
                }),
                None => Err(Error::NotFound("Failed to create relationship".to_string())),
            };

            let _ = tx.send(result);
        });

        PendingRelationship::new(rx)
    }

    fn get_relationships(&self, memory_id: &str) -> RelationshipStream {
        let db = self.db.clone();
        let memory_id = memory_id.to_string();

        let (tx, rx) = tokio::sync::mpsc::channel(100);

        tokio::spawn(async move {
            let sql_query = "SELECT * FROM memory_relationship WHERE source_id = $memory_id OR target_id = $memory_id";

            match db.query(sql_query).bind(("memory_id", memory_id)).await {
                Ok(mut response) => {
                    let results: Vec<RelationshipSchema> = response.take(0).unwrap_or_default();

                    for schema in results {
                        let relationship = MemoryRelationship {
                            id: schema.id.key().to_string(),
                            source_id: schema.source_id,
                            target_id: schema.target_id,
                            relationship_type: schema.relationship_type,
                            metadata: Some(schema.metadata),
                        };

                        if tx.send(Ok(relationship)).await.is_err() {
                            break;
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(Error::Database(Box::new(e)))).await;
                }
            }
        });

        RelationshipStream::new(rx)
    }

    fn delete_relationship(&self, id: &str) -> PendingDeletion {
        let db = self.db.clone();
        let id_str = id.to_string();

        let (tx, rx) = tokio::sync::oneshot::channel();

        tokio::spawn(async move {
            let result = match db
                .delete::<Option<RelationshipSchema>>(("memory_relationship", &id_str))
                .await
            {
                Ok(result) => Ok(result.is_some()),
                Err(e) => Err(Error::Database(Box::new(e))),
            };

            let _ = tx.send(result);
        });

        PendingDeletion::new(rx)
    }

    fn query_by_type(&self, memory_type: MemoryType) -> MemoryStream {
        let db = self.db.clone();
        // Use the serialized format which is capitalized
        let memory_type_str = match &memory_type {
            MemoryType::Episodic => "Episodic".to_string(),
            MemoryType::Semantic => "Semantic".to_string(),
            MemoryType::Procedural => "Procedural".to_string(),
            MemoryType::Custom(name) => format!("Custom(\"{}\")", name),
        };

        let (tx, rx) = tokio::sync::mpsc::channel(100);

        tokio::spawn(async move {
            let sql_query = "SELECT * FROM memory WHERE memory_type = $memory_type";

            match db
                .query(sql_query)
                .bind(("memory_type", memory_type_str))
                .await
            {
                Ok(mut response) => {
                    let results: Vec<MemoryNodeSchema> = response.take(0).unwrap_or_default();

                    for schema in results {
                        let memory = SurrealDBMemoryManager::from_schema(schema);
                        if tx.send(Ok(memory)).await.is_err() {
                            break;
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(Error::Database(Box::new(e)))).await;
                }
            }
        });

        MemoryStream::new(rx)
    }

    fn search_by_content(&self, query: &str) -> MemoryStream {
        let db = self.db.clone();
        let query = query.to_string();

        let (tx, rx) = tokio::sync::mpsc::channel(100);

        tokio::spawn(async move {
            let sql_query = "SELECT * FROM memory WHERE content CONTAINS $query";

            match db.query(sql_query).bind(("query", query)).await {
                Ok(mut response) => {
                    let results: Vec<MemoryNodeSchema> = response.take(0).unwrap_or_default();

                    for schema in results {
                        let memory = SurrealDBMemoryManager::from_schema(schema);
                        if tx.send(Ok(memory)).await.is_err() {
                            break;
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(Error::Database(Box::new(e)))).await;
                }
            }
        });

        MemoryStream::new(rx)
    }

    fn search_by_vector(&self, vector: Vec<f32>, limit: usize) -> MemoryStream {
        let db = self.db.clone();

        let (tx, rx) = tokio::sync::mpsc::channel(100);

        tokio::spawn(async move {
            // Convert vector to JSON array format for SurrealDB
            let vector_json = serde_json::to_string(&vector).unwrap_or_else(|_| "[]".to_string());

            // Use SurrealDB's native vector similarity search
            let sql_query = format!(
                "SELECT *, vector::similarity::cosine(metadata.embedding, {}) AS score 
                FROM memory 
                WHERE metadata.embedding != NULL 
                ORDER BY score DESC 
                LIMIT {};",
                vector_json, limit
            );

            match db.query(&sql_query).await {
                Ok(mut response) => {
                    let results: Vec<MemoryNodeSchema> = response.take(0).unwrap_or_default();

                    for schema in results {
                        let memory = SurrealDBMemoryManager::from_schema(schema);
                        if tx.send(Ok(memory)).await.is_err() {
                            break;
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(Error::Database(Box::new(e)))).await;
                }
            }
        });

        MemoryStream::new(rx)
    }
}
