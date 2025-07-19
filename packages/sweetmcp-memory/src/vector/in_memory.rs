//! In-memory vector store implementation

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use surrealdb::sql::Value;

use super::vector_store::VectorStore;
use crate::utils::error::{Error, Result};

/// In-memory vector store implementation
pub struct InMemoryVectorStore {
    vectors: HashMap<String, Vec<f32>>,
    metadata: HashMap<String, HashMap<String, Value>>,
}

impl InMemoryVectorStore {
    /// Create a new in-memory vector store
    pub fn new() -> Self {
        Self {
            vectors: HashMap::new(),
            metadata: HashMap::new(),
        }
    }
}

impl VectorStore for InMemoryVectorStore {
    fn add_vector(
        &mut self,
        id: &str,
        vector: Vec<f32>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> {
        self.vectors.insert(id.to_string(), vector);
        Box::pin(async { Ok(()) })
    }

    fn get_vector(
        &self,
        id: &str,
    ) -> Pin<Box<dyn Future<Output = Result<(Vec<f32>, Option<HashMap<String, Value>>)>> + Send>>
    {
        let result = if let Some(vector) = self.vectors.get(id) {
            let metadata = self.metadata.get(id).cloned();
            Ok((vector.clone(), metadata))
        } else {
            Err(Error::NotFound(format!("Vector with id {} not found", id)))
        };
        Box::pin(async move { result })
    }

    fn update_vector(
        &mut self,
        id: &str,
        vector: Vec<f32>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> {
        if self.vectors.contains_key(id) {
            self.vectors.insert(id.to_string(), vector);
            Box::pin(async { Ok(()) })
        } else {
            let id = id.to_string();
            Box::pin(
                async move { Err(Error::NotFound(format!("Vector with id {} not found", id))) },
            )
        }
    }

    fn delete_vector(&mut self, id: &str) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> {
        self.vectors.remove(id);
        self.metadata.remove(id);
        Box::pin(async { Ok(()) })
    }

    fn search_similar(
        &self,
        query_vector: &[f32],
        limit: usize,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<String>>> + Send>> {
        let mut results: Vec<(String, f32)> = Vec::new();

        // Simple cosine similarity search
        for (id, vector) in &self.vectors {
            let similarity = cosine_similarity(query_vector, vector);
            results.push((id.clone(), similarity));
        }

        // Sort by similarity (descending)
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top k results
        let top_k: Vec<String> = results.into_iter().take(limit).map(|(id, _)| id).collect();

        Box::pin(async move { Ok(top_k) })
    }

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
    > {
        let mut results: Vec<(String, Vec<f32>, f32, Option<HashMap<String, Value>>)> = Vec::new();

        // Simple cosine similarity search
        for (id, vector) in &self.vectors {
            // Apply filters if any
            if let Some(ref filters) = filters {
                if let Some(metadata) = self.metadata.get(id) {
                    let mut matches = true;
                    for (key, value) in filters {
                        if metadata.get(key) != Some(value) {
                            matches = false;
                            break;
                        }
                    }
                    if !matches {
                        continue;
                    }
                } else {
                    continue; // No metadata, skip if filters are present
                }
            }

            let similarity = cosine_similarity(query_vector, vector);
            let metadata = self.metadata.get(id).cloned();
            results.push((id.clone(), vector.clone(), similarity, metadata));
        }

        // Sort by similarity (descending)
        results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        // Take top k results
        if let Some(limit) = limit {
            results.truncate(limit);
        }

        Box::pin(async move { Ok(results) })
    }

    fn count(&self) -> Pin<Box<dyn Future<Output = Result<usize>> + Send>> {
        let count = self.vectors.len();
        Box::pin(async move { Ok(count) })
    }

    fn clear(&self) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> {
        // Note: This requires mutable access, but the trait signature doesn't allow it
        // For now, we return an error
        Box::pin(async {
            Err(Error::Other(
                "Clear operation requires mutable access".to_string(),
            ))
        })
    }
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}
