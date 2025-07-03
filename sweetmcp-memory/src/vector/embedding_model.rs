//! Embedding model interface for generating vector embeddings

use std::future::Future;
use std::pin::Pin;

use crate::utils::error::Result;

/// Trait for embedding model implementations
#[cfg_attr(test, mockall::automock)]
pub trait EmbeddingModel: Send + Sync {
    /// Generate an embedding for text
    fn embed<'a>(
        &self,
        text: &str,
        task: Option<&'a str>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<f32>>> + Send>>;

    /// Generate embeddings for multiple texts
    fn batch_embed<'a>(
        &self,
        texts: &[String],
        task: Option<&'a str>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Vec<f32>>>> + Send>>;

    /// Get the dimension of the embedding vectors
    fn dimension(&self) -> usize;

    /// Get the name of the embedding model
    fn name(&self) -> &str;
}
