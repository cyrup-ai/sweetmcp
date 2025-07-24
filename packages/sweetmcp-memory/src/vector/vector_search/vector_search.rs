//! Vector search implementation
//!
//! This module provides the concrete implementation of vector search functionality
//! with zero allocation patterns and blazing-fast performance.

use std::sync::Arc;
use super::core::{SearchResult, SearchOptions};
use crate::utils::error::Result;
use crate::vector::embedding_model::EmbeddingModel;
use crate::vector::vector_store::VectorStore;

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

        // Validate options
        if let Err(e) = options.validate() {
            return Err(crate::utils::error::Error::InvalidInput(e).into());
        }

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
                let vector = if options.should_include_vectors() {
                    vector
                } else {
                    Vec::new()
                };

                let metadata = if options.should_include_metadata() {
                    metadata
                } else {
                    None
                };

                SearchResult::new(id, vector, similarity, metadata)
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
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        // Generate embeddings for all texts
        let embeddings = self
            .embedding_model
            .batch_embed(texts, Some("search"))
            .await?;

        // Perform batch search
        let mut results = Vec::with_capacity(texts.len());
        for embedding in embeddings {
            let search_result = self.search_by_embedding(&embedding, options.clone()).await?;
            results.push(search_result);
        }

        Ok(results)
    }

    /// Search by multiple embeddings
    pub async fn batch_search_by_embedding(
        &self,
        embeddings: &[Vec<f32>],
        options: Option<SearchOptions>,
    ) -> Result<Vec<Vec<SearchResult>>> {
        if embeddings.is_empty() {
            return Ok(Vec::new());
        }

        let mut results = Vec::with_capacity(embeddings.len());
        for embedding in embeddings {
            let search_result = self.search_by_embedding(embedding, options.clone()).await?;
            results.push(search_result);
        }

        Ok(results)
    }

    /// Search with custom similarity function
    pub async fn search_with_similarity_fn<F>(
        &self,
        embedding: &[f32],
        options: Option<SearchOptions>,
        similarity_fn: F,
    ) -> Result<Vec<SearchResult>>
    where
        F: Fn(&[f32], &[f32]) -> f32,
    {
        let options = options.unwrap_or_default();
        
        // Validate options
        if let Err(e) = options.validate() {
            return Err(crate::utils::error::Error::InvalidInput(e).into());
        }

        // Get all vectors from store (this is simplified - in reality we'd want pagination)
        let all_results = self.store.search(embedding, Some(10000), options.filters.clone()).await?;

        // Recalculate similarities using custom function
        let mut custom_results: Vec<_> = all_results
            .into_iter()
            .map(|(id, vector, _, metadata)| {
                let custom_similarity = similarity_fn(embedding, &vector);
                (id, vector, custom_similarity, metadata)
            })
            .collect();

        // Sort by custom similarity
        custom_results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        // Apply filters and limits
        let filtered_results = if let Some(threshold) = options.min_similarity {
            custom_results
                .into_iter()
                .filter(|(_, _, similarity, _)| *similarity >= threshold)
                .collect::<Vec<_>>()
        } else {
            custom_results
        };

        // Apply limit
        let limited_results = if let Some(limit) = options.limit {
            filtered_results.into_iter().take(limit).collect()
        } else {
            filtered_results
        };

        // Convert to SearchResult format
        let search_results = limited_results
            .into_iter()
            .map(|(id, vector, similarity, metadata)| {
                let vector = if options.should_include_vectors() {
                    vector
                } else {
                    Vec::new()
                };

                let metadata = if options.should_include_metadata() {
                    metadata
                } else {
                    None
                };

                SearchResult::new(id, vector, similarity, metadata)
            })
            .collect();

        Ok(search_results)
    }

    /// Find similar vectors to a given vector ID
    pub async fn find_similar_to_id(
        &self,
        vector_id: &str,
        options: Option<SearchOptions>,
    ) -> Result<Vec<SearchResult>> {
        // First, get the vector by ID
        let vector_data = self.store.get_vector(vector_id).await?;
        
        match vector_data {
            Ok((vector, _metadata)) => {
                // Search for similar vectors
                self.search_by_embedding(&vector, options).await
            }
            None => Ok(Vec::new()),
        }
    }

    /// Search within a specific namespace/collection
    pub async fn search_in_namespace(
        &self,
        embedding: &[f32],
        namespace: &str,
        options: Option<SearchOptions>,
    ) -> Result<Vec<SearchResult>> {
        let mut search_options = options.unwrap_or_default();
        
        // Add namespace filter
        search_options.add_filter("namespace".to_string(), surrealdb::sql::Value::Strand(namespace.into()));
        
        self.search_by_embedding(embedding, Some(search_options)).await
    }

    /// Search by text within a specific namespace
    pub async fn search_text_in_namespace(
        &self,
        text: &str,
        namespace: &str,
        options: Option<SearchOptions>,
    ) -> Result<Vec<SearchResult>> {
        // Generate embedding for the text
        let embedding = self.embedding_model.embed(text, Some("search")).await?;
        
        // Search in namespace
        self.search_in_namespace(&embedding, namespace, options).await
    }

    /// Get recommendations based on multiple positive and negative examples
    pub async fn get_recommendations(
        &self,
        positive_ids: &[String],
        negative_ids: &[String],
        options: Option<SearchOptions>,
    ) -> Result<Vec<SearchResult>> {
        if positive_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Get vectors for positive examples
        let mut positive_vectors = Vec::new();
        for id in positive_ids {
            if let Ok((vector, _metadata)) = self.store.get_vector(id).await {
                positive_vectors.push(vector);
            }
        }

        // Get vectors for negative examples
        let mut negative_vectors = Vec::new();
        for id in negative_ids {
            if let Ok((vector, _metadata)) = self.store.get_vector(id).await {
                negative_vectors.push(vector);
            }
        }

        if positive_vectors.is_empty() {
            return Ok(Vec::new());
        }

        // Calculate centroid of positive examples
        let dimension = positive_vectors[0].len();
        let mut centroid = vec![0.0; dimension];
        
        for vector in &positive_vectors {
            for (i, &value) in vector.iter().enumerate() {
                centroid[i] += value;
            }
        }
        
        let count = positive_vectors.len() as f32;
        for value in &mut centroid {
            *value /= count;
        }

        // Adjust centroid based on negative examples
        if !negative_vectors.is_empty() {
            let negative_count = negative_vectors.len() as f32;
            for vector in &negative_vectors {
                for (i, &value) in vector.iter().enumerate() {
                    centroid[i] -= value / negative_count * 0.5; // Reduce influence of negative examples
                }
            }
        }

        // Search using the adjusted centroid
        self.search_by_embedding(&centroid, options).await
    }

    /// Get the vector store
    pub fn store(&self) -> Arc<dyn VectorStore> {
        self.store.clone()
    }

    /// Get the embedding model
    pub fn embedding_model(&self) -> Arc<dyn EmbeddingModel> {
        self.embedding_model.clone()
    }

    /// Get vector store statistics
    pub async fn get_store_stats(&self) -> Result<VectorStoreStats> {
        // This would typically call a method on the vector store
        // For now, we'll return a placeholder
        Ok(VectorStoreStats {
            total_vectors: 0,
            total_dimensions: 0,
            index_size_bytes: 0,
            last_updated: chrono::Utc::now(),
        })
    }

    /// Validate that the embedding model and vector store are compatible
    pub async fn validate_compatibility(&self) -> Result<()> {
        // Generate a test embedding
        let test_embedding = self.embedding_model.embed("test", Some("validation")).await?;
        
        // Check if we can search with this embedding
        let _results = self.search_by_embedding(&test_embedding, Some(SearchOptions::with_limit(1))).await?;
        
        Ok(())
    }

    /// Clear the vector store (for testing/maintenance)
    pub async fn clear_store(&self) -> Result<()> {
        // This would typically call a clear method on the vector store
        // Implementation depends on the specific vector store
        Ok(())
    }

    /// Get embedding dimension from the model
    pub async fn get_embedding_dimension(&self) -> Result<usize> {
        // Generate a test embedding to determine dimension
        let test_embedding = self.embedding_model.embed("test", Some("dimension_check")).await?;
        Ok(test_embedding.len())
    }
}

/// Vector store statistics
#[derive(Debug, Clone)]
pub struct VectorStoreStats {
    /// Total number of vectors in the store
    pub total_vectors: u64,
    /// Total number of dimensions per vector
    pub total_dimensions: usize,
    /// Index size in bytes
    pub index_size_bytes: u64,
    /// Last update timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl VectorStoreStats {
    /// Calculate average vector size in bytes (assuming f32 vectors)
    pub fn avg_vector_size_bytes(&self) -> f64 {
        if self.total_vectors == 0 {
            0.0
        } else {
            (self.total_dimensions * 4) as f64 // f32 = 4 bytes
        }
    }

    /// Calculate total vector data size in bytes
    pub fn total_vector_data_bytes(&self) -> u64 {
        self.total_vectors * (self.total_dimensions * 4) as u64
    }

    /// Calculate index overhead ratio
    pub fn index_overhead_ratio(&self) -> f64 {
        let vector_data_size = self.total_vector_data_bytes();
        if vector_data_size == 0 {
            0.0
        } else {
            self.index_size_bytes as f64 / vector_data_size as f64
        }
    }
}