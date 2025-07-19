//! Vector store interface for storing and retrieving vectors

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use surrealdb::sql::Value;

use crate::utils::error::Result;

/// Trait for vector store implementations
#[cfg_attr(test, mockall::automock)]
pub trait VectorStore: Send + Sync {
    /// Add a vector to the store
    fn add_vector(
        &mut self,
        id: &str,
        vector: Vec<f32>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send>>;

    /// Get a vector by ID
    fn get_vector(
        &self,
        id: &str,
    ) -> Pin<Box<dyn Future<Output = Result<(Vec<f32>, Option<HashMap<String, Value>>)>> + Send>>;

    /// Update a vector
    fn update_vector(
        &mut self,
        id: &str,
        vector: Vec<f32>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send>>;

    /// Delete a vector
    fn delete_vector(&mut self, id: &str) -> Pin<Box<dyn Future<Output = Result<()>> + Send>>;

    /// Search for similar vectors
    fn search_similar(
        &self,
        query_vector: &[f32],
        limit: usize,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<String>>> + Send>>;

    /// Search for similar vectors with additional filters
    fn search(
        &self,
        query_vector: &[f32],
        limit: Option<usize>,
        filters: Option<HashMap<String, Value>>,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<Vec<(String, Vec<f32>, f32, Option<HashMap<String, Value>>)>>,
                > + Send,
        >,
    >;

    /// Get the count of vectors in the store
    fn count(&self) -> Pin<Box<dyn Future<Output = Result<usize>> + Send>>;

    /// Clear all vectors from the store
    fn clear(&self) -> Pin<Box<dyn Future<Output = Result<()>> + Send>>;
}
