//! Search and query operations for memory nodes
//!
//! This module implements content-based and vector-based search operations
//! with blazing-fast performance and efficient result streaming.

use super::core::SurrealDBMemoryManager;
use super::trait_def::MemoryManager;
use crate::memory::memory_stream::MemoryStream;
use crate::schema::memory_schema::MemoryNodeSchema;
use crate::utils::error::Error;

impl MemoryManager for SurrealDBMemoryManager {
    /// Search memory nodes by content using full-text search
    /// 
    /// This method performs efficient full-text search across memory node content
    /// with relevance scoring and result limiting.
    fn search_by_content(&self, query: &str, limit: usize) -> MemoryStream {
        let db = self.db.clone();
        let query = query.to_string();

        let (tx, rx) = tokio::sync::mpsc::channel(100);

        tokio::spawn(async move {
            if query.trim().is_empty() {
                let _ = tx.send(Err(Error::ValidationError("Search query cannot be empty".to_string()))).await;
                return;
            }

            if limit == 0 {
                let _ = tx.send(Err(Error::ValidationError("Search limit must be greater than 0".to_string()))).await;
                return;
            }

            // Use SurrealDB's full-text search capabilities with relevance scoring
            let sql_query = format!(
                "SELECT *, search::score(1) AS relevance_score 
                FROM memory 
                WHERE content @@ $query 
                ORDER BY relevance_score DESC 
                LIMIT {};",
                limit
            );

            // Execute query with parameterized search term
            match db.query(&sql_query).bind(("query", query)).await {
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

    /// Search memory nodes by vector similarity using cosine similarity
    /// 
    /// This method performs efficient vector similarity search using SurrealDB's
    /// native vector operations with cosine similarity scoring.
    fn search_by_vector(&self, vector: Vec<f32>, limit: usize) -> MemoryStream {
        let db = self.db.clone();

        let (tx, rx) = tokio::sync::mpsc::channel(100);

        tokio::spawn(async move {
            if vector.is_empty() {
                let _ = tx.send(Err(Error::ValidationError("Search vector cannot be empty".to_string()))).await;
                return;
            }

            if limit == 0 {
                let _ = tx.send(Err(Error::ValidationError("Search limit must be greater than 0".to_string()))).await;
                return;
            }

            // Convert vector to JSON array format for SurrealDB
            let vector_json = match serde_json::to_string(&vector) {
                Ok(json) => json,
                Err(_) => {
                    let _ = tx.send(Err(Error::ValidationError("Failed to serialize search vector".to_string()))).await;
                    return;
                }
            };

            // Use SurrealDB's native vector similarity search with cosine similarity
            let sql_query = format!(
                "SELECT *, vector::similarity::cosine(metadata.embedding, {}) AS similarity_score 
                FROM memory 
                WHERE metadata.embedding != NULL 
                ORDER BY similarity_score DESC 
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

impl SurrealDBMemoryManager {
    /// Advanced search combining content and vector similarity
    /// 
    /// This method performs hybrid search using both content matching and vector similarity
    /// with weighted scoring for optimal relevance ranking.
    /// 
    /// # Arguments
    /// * `content_query` - Text query for content search
    /// * `vector_query` - Vector for similarity search
    /// * `content_weight` - Weight for content relevance (0.0 to 1.0)
    /// * `vector_weight` - Weight for vector similarity (0.0 to 1.0)
    /// * `limit` - Maximum number of results to return
    /// 
    /// # Returns
    /// MemoryStream of search results ordered by combined relevance score
    pub fn hybrid_search(
        &self,
        content_query: &str,
        vector_query: Vec<f32>,
        content_weight: f32,
        vector_weight: f32,
        limit: usize,
    ) -> MemoryStream {
        let db = self.db.clone();
        let content_query = content_query.to_string();

        let (tx, rx) = tokio::sync::mpsc::channel(100);

        tokio::spawn(async move {
            // Validate inputs
            if content_query.trim().is_empty() && vector_query.is_empty() {
                let _ = tx.send(Err(Error::ValidationError("At least one search criterion must be provided".to_string()))).await;
                return;
            }

            if limit == 0 {
                let _ = tx.send(Err(Error::ValidationError("Search limit must be greater than 0".to_string()))).await;
                return;
            }

            if content_weight < 0.0 || content_weight > 1.0 || vector_weight < 0.0 || vector_weight > 1.0 {
                let _ = tx.send(Err(Error::ValidationError("Weights must be between 0.0 and 1.0".to_string()))).await;
                return;
            }

            // Convert vector to JSON for SurrealDB
            let vector_json = match serde_json::to_string(&vector_query) {
                Ok(json) => json,
                Err(_) => {
                    let _ = tx.send(Err(Error::ValidationError("Failed to serialize search vector".to_string()))).await;
                    return;
                }
            };

            // Build hybrid search query with weighted scoring
            let sql_query = format!(
                "SELECT *, 
                ({} * search::score(1)) + ({} * vector::similarity::cosine(metadata.embedding, {})) AS combined_score
                FROM memory 
                WHERE (content @@ $content_query OR metadata.embedding != NULL)
                ORDER BY combined_score DESC 
                LIMIT {};",
                content_weight, vector_weight, vector_json, limit
            );

            match db.query(&sql_query).bind(("content_query", content_query)).await {
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

    /// Search memory nodes by metadata attributes
    /// 
    /// This method allows searching memory nodes based on custom metadata attributes
    /// with flexible filtering and sorting options.
    /// 
    /// # Arguments
    /// * `metadata_filter` - JSON object representing metadata filter criteria
    /// * `limit` - Maximum number of results to return
    /// 
    /// # Returns
    /// MemoryStream of memory nodes matching the metadata criteria
    pub fn search_by_metadata(&self, metadata_filter: serde_json::Value, limit: usize) -> MemoryStream {
        let db = self.db.clone();

        let (tx, rx) = tokio::sync::mpsc::channel(100);

        tokio::spawn(async move {
            if limit == 0 {
                let _ = tx.send(Err(Error::ValidationError("Search limit must be greater than 0".to_string()))).await;
                return;
            }

            // Convert metadata filter to SQL conditions
            let filter_json = match serde_json::to_string(&metadata_filter) {
                Ok(json) => json,
                Err(_) => {
                    let _ = tx.send(Err(Error::ValidationError("Invalid metadata filter format".to_string()))).await;
                    return;
                }
            };

            let sql_query = format!(
                "SELECT * FROM memory 
                WHERE metadata.custom CONTAINS $metadata_filter 
                ORDER BY metadata.importance DESC 
                LIMIT {};",
                limit
            );

            match db.query(&sql_query).bind(("metadata_filter", filter_json)).await {
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