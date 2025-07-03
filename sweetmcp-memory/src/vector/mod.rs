//! Vector operations and storage for memory embeddings

pub mod embedding_model;
pub mod in_memory;
pub mod in_memory_async;
pub mod vector_index;
pub mod vector_repository;
pub mod vector_search;
pub mod vector_store;

// Re-export main types
pub use embedding_model::*;
pub use in_memory_async::InMemoryVectorStore;
pub use vector_index::*;
pub use vector_repository::*;
pub use vector_search::*;

use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::oneshot;

/// Distance metrics for vector comparison
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistanceMetric {
    /// Euclidean distance (L2 norm)
    Euclidean,
    /// Cosine similarity
    Cosine,
    /// Dot product
    DotProduct,
}

/// A pending vector operation
pub struct PendingVectorOp {
    rx: oneshot::Receiver<crate::utils::Result<()>>,
}

impl PendingVectorOp {
    pub fn new(rx: oneshot::Receiver<crate::utils::Result<()>>) -> Self {
        Self { rx }
    }
}

impl Future for PendingVectorOp {
    type Output = crate::utils::Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.rx).poll(cx) {
            Poll::Ready(Ok(result)) => Poll::Ready(result),
            Poll::Ready(Err(_)) => Poll::Ready(Err(crate::utils::error::Error::Internal(
                "Vector operation task failed".to_string(),
            ))),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// A pending vector search
pub struct PendingVectorSearch {
    rx: oneshot::Receiver<crate::utils::Result<Vec<VectorSearchResult>>>,
}

impl PendingVectorSearch {
    pub fn new(rx: oneshot::Receiver<crate::utils::Result<Vec<VectorSearchResult>>>) -> Self {
        Self { rx }
    }
}

impl Future for PendingVectorSearch {
    type Output = crate::utils::Result<Vec<VectorSearchResult>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.rx).poll(cx) {
            Poll::Ready(Ok(result)) => Poll::Ready(result),
            Poll::Ready(Err(_)) => Poll::Ready(Err(crate::utils::error::Error::Internal(
                "Vector search task failed".to_string(),
            ))),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// A pending embedding generation
pub struct PendingEmbedding {
    rx: oneshot::Receiver<crate::utils::Result<Vec<f32>>>,
}

impl PendingEmbedding {
    pub fn new(rx: oneshot::Receiver<crate::utils::Result<Vec<f32>>>) -> Self {
        Self { rx }
    }
}

impl Future for PendingEmbedding {
    type Output = crate::utils::Result<Vec<f32>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.rx).poll(cx) {
            Poll::Ready(Ok(result)) => Poll::Ready(result),
            Poll::Ready(Err(_)) => Poll::Ready(Err(crate::utils::error::Error::Internal(
                "Embedding task failed".to_string(),
            ))),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Vector store trait for different implementations
pub trait VectorStore: Send + Sync {
    /// Add a vector with metadata
    fn add(
        &self,
        id: String,
        embedding: Vec<f32>,
        metadata: Option<serde_json::Value>,
    ) -> PendingVectorOp;

    /// Update a vector
    fn update(
        &self,
        id: String,
        embedding: Vec<f32>,
        metadata: Option<serde_json::Value>,
    ) -> PendingVectorOp;

    /// Delete a vector
    fn delete(&self, id: String) -> PendingVectorOp;

    /// Search for similar vectors
    fn search(
        &self,
        query: Vec<f32>,
        limit: usize,
        filter: Option<crate::memory::filter::MemoryFilter>,
    ) -> PendingVectorSearch;

    /// Generate embedding for text
    fn embed(&self, text: String) -> PendingEmbedding;
}

/// Vector search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSearchResult {
    /// ID of the result
    pub id: String,

    /// Similarity score
    pub score: f32,

    /// Optional metadata
    pub metadata: Option<serde_json::Value>,
}
