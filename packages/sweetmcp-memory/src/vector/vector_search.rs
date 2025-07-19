// src/vector/vector_search.rs
//! Vector search functionality for the vector module.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use surrealdb::sql::Value;

use crate::utils::error::Result;
use crate::vector::embedding_model::EmbeddingModel;
use crate::vector::vector_store::VectorStore;

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// ID of the vector
    pub id: String,
    /// Vector
    pub vector: Vec<f32>,
    /// Similarity score
    pub similarity: f32,
    /// Metadata
    pub metadata: Option<HashMap<String, Value>>,
}

/// Search options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptions {
    /// Maximum number of results to return
    pub limit: Option<usize>,
    /// Minimum similarity threshold (0.0 to 1.0)
    pub min_similarity: Option<f32>,
    /// Filters to apply
    pub filters: Option<HashMap<String, Value>>,
    /// Whether to include vectors in results
    pub include_vectors: Option<bool>,
    /// Whether to include metadata in results
    pub include_metadata: Option<bool>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            limit: Some(10),
            min_similarity: Some(0.7),
            filters: None,
            include_vectors: Some(false),
            include_metadata: Some(true),
        }
    }
}

/// Vector search
pub struct VectorSearch {
    /// Vector store
    store: Arc<dyn VectorStore>,
    /// Embedding model
    embedding_model: Arc<dyn EmbeddingModel>,
}

impl VectorSearch {
    /// Create a new VectorSearch
    pub fn new(store: Arc<dyn VectorStore>, embedding_model: Arc<dyn EmbeddingModel>) -> Self {
        Self {
            store,
            embedding_model,
        }
    }

    /// Search by text
    pub async fn search_by_text(
        &self,
        text: &str,
        options: Option<SearchOptions>,
    ) -> Result<Vec<SearchResult>> {
        // Generate embedding for the text
        let embedding = self.embedding_model.embed(text, Some("search")).await?;

        // Search by embedding
        self.search_by_embedding(&embedding, options).await
    }

    /// Search by embedding
    pub async fn search_by_embedding(
        &self,
        embedding: &[f32],
        options: Option<SearchOptions>,
    ) -> Result<Vec<SearchResult>> {
        let options = options.unwrap_or_default();

        // Prepare search options
        let limit = options.limit;
        let filters = options.filters.clone();

        // Search in vector store
        let results = self.store.search(embedding, limit, filters).await?;

        // Apply minimum similarity threshold if specified
        let filtered_results = if let Some(threshold) = options.min_similarity {
            results
                .into_iter()
                .filter(|(_, _, similarity, _)| *similarity >= threshold)
                .collect::<Vec<_>>()
        } else {
            results
        };

        // Convert to SearchResult format
        let search_results = filtered_results
            .into_iter()
            .map(|(id, vector, similarity, metadata)| {
                let vector = if options.include_vectors.unwrap_or(false) {
                    vector
                } else {
                    Vec::new()
                };

                let metadata = if options.include_metadata.unwrap_or(true) {
                    metadata
                } else {
                    None
                };

                SearchResult {
                    id,
                    vector,
                    similarity,
                    metadata,
                }
            })
            .collect();

        Ok(search_results)
    }

    /// Batch search by texts
    pub async fn batch_search_by_text(
        &self,
        texts: &[String],
        options: Option<SearchOptions>,
    ) -> Result<Vec<Vec<SearchResult>>> {
        // Generate embeddings for all texts
        let embeddings = self
            .embedding_model
            .batch_embed(texts, Some("search"))
            .await?;

        // Search by embeddings
        let mut results = Vec::new();

        for embedding in embeddings {
            let search_results = self
                .search_by_embedding(&embedding, options.clone())
                .await?;
            results.push(search_results);
        }

        Ok(results)
    }

    /// Get the vector store
    pub fn store(&self) -> Arc<dyn VectorStore> {
        Arc::clone(&self.store)
    }

    /// Get the embedding model
    pub fn embedding_model(&self) -> Arc<dyn EmbeddingModel> {
        Arc::clone(&self.embedding_model)
    }
}

/// Hybrid search combining vector and keyword search
pub struct HybridSearch {
    /// Vector search
    vector_search: VectorSearch,
    /// Keyword search function
    keyword_search: Box<
        dyn Fn(
                &str,
                Option<SearchOptions>,
            ) -> futures::future::BoxFuture<'static, Result<Vec<SearchResult>>>
            + Send
            + Sync,
    >,
    /// Vector weight (0.0 to 1.0)
    vector_weight: f32,
}

impl HybridSearch {
    /// Create a new HybridSearch
    pub fn new<F, Fut>(
        vector_search: VectorSearch,
        keyword_search: F,
        vector_weight: Option<f32>,
    ) -> Self
    where
        F: Fn(&str, Option<SearchOptions>) -> Fut + Send + Sync + 'static,
        Fut: futures::Future<Output = Result<Vec<SearchResult>>> + Send + 'static,
    {
        let keyword_search_boxed = Box::new(move |text: &str, options: Option<SearchOptions>| {
            let text = text.to_string();
            let fut = keyword_search(&text, options);
            Box::pin(fut) as futures::future::BoxFuture<'static, Result<Vec<SearchResult>>>
        });

        Self {
            vector_search,
            keyword_search: keyword_search_boxed,
            vector_weight: vector_weight.unwrap_or(0.5),
        }
    }

    /// Search by text using hybrid approach
    pub async fn search(
        &self,
        text: &str,
        options: Option<SearchOptions>,
    ) -> Result<Vec<SearchResult>> {
        // Perform both vector and keyword search
        let vector_results = self
            .vector_search
            .search_by_text(text, options.clone())
            .await?;
        let keyword_results = (self.keyword_search)(text, options.clone()).await?;

        // Combine results with weighting
        let combined_results = self.combine_results(vector_results, keyword_results, options);

        Ok(combined_results)
    }

    /// Combine vector and keyword search results
    fn combine_results(
        &self,
        vector_results: Vec<SearchResult>,
        keyword_results: Vec<SearchResult>,
        options: Option<SearchOptions>,
    ) -> Vec<SearchResult> {
        let options = options.unwrap_or_default();
        let limit = options.limit.unwrap_or(10);

        // Create a map of ID to combined result
        let mut combined_map = HashMap::new();

        // Process vector results
        for result in vector_results {
            let weighted_similarity = result.similarity * self.vector_weight;
            combined_map.insert(
                result.id.clone(),
                (result, weighted_similarity, true, false),
            );
        }

        // Process keyword results
        for result in keyword_results {
            let weighted_similarity = result.similarity * (1.0 - self.vector_weight);

            // Check if we already have this result
            let entry_exists = combined_map.contains_key(&result.id);

            if entry_exists {
                // Get the existing values first
                let (existing_result, existing_similarity, _, _) =
                    combined_map.remove(&result.id).unwrap();

                // Update values
                let new_similarity = existing_similarity + weighted_similarity;
                let mut result_clone = existing_result.clone();
                result_clone.similarity = new_similarity;

                // Re-insert with updated values
                combined_map.insert(
                    result.id.clone(),
                    (result_clone, new_similarity, true, true),
                );
            } else {
                // Add new entry
                combined_map.insert(
                    result.id.clone(),
                    (result, weighted_similarity, false, true),
                );
            }
        }

        // Convert map to vector and sort by combined similarity
        let mut combined_results: Vec<_> = combined_map
            .into_iter()
            .map(|(_, (result, similarity, _, _))| SearchResult {
                id: result.id,
                vector: result.vector,
                similarity,
                metadata: result.metadata,
            })
            .collect();

        // Sort by similarity (descending)
        combined_results.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Apply limit
        if combined_results.len() > limit {
            combined_results.truncate(limit);
        }

        combined_results
    }

    /// Set vector weight
    pub fn set_vector_weight(&mut self, weight: f32) {
        let clamped_weight = weight.max(0.0).min(1.0);
        self.vector_weight = clamped_weight;
    }

    /// Get vector weight
    pub fn vector_weight(&self) -> f32 {
        self.vector_weight
    }

    /// Get the vector search
    pub fn vector_search(&self) -> &VectorSearch {
        &self.vector_search
    }
}
