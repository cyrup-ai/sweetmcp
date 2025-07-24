//! Vector search module
//!
//! This module provides comprehensive vector search functionality including
//! basic vector search, hybrid search combining vector and keyword search,
//! with zero allocation patterns and blazing-fast performance.

pub mod core;
pub mod vector_search;
pub mod hybrid_search;

// Re-export core types and traits for ergonomic usage
pub use core::{SearchResult, SearchOptions, SearchStats};

pub use vector_search::{VectorSearch, VectorStoreStats};

pub use hybrid_search::{
    HybridSearch, HybridSearchMetrics, CombinationStrategy,
    apply_combination_strategy,
};

// Convenience re-exports for common functionality
pub use vector_search::VectorSearch as Search;
pub use hybrid_search::HybridSearch as Hybrid;

/// Create a new vector search instance
pub fn vector_search(
    store: std::sync::Arc<dyn crate::vector::vector_store::VectorStore>,
    embedding_model: std::sync::Arc<dyn crate::vector::embedding_model::EmbeddingModel>,
) -> VectorSearch {
    VectorSearch::new(store, embedding_model)
}

/// Create a new hybrid search instance
pub fn hybrid_search<F, Fut>(
    vector_search: VectorSearch,
    keyword_search: F,
    vector_weight: Option<f32>,
) -> HybridSearch
where
    F: Fn(String, Option<SearchOptions>) -> Fut + Send + Sync + 'static,
    Fut: futures::Future<Output = crate::utils::error::Result<Vec<SearchResult>>> + Send + 'static,
{
    HybridSearch::new(vector_search, keyword_search, vector_weight)
}