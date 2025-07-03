//! In-memory vector store implementation with async trait

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

use super::{
    PendingEmbedding, PendingVectorOp, PendingVectorSearch, VectorSearchResult, VectorStore,
};
use crate::memory::filter::MemoryFilter;
use crate::utils::error::Error;

/// In-memory vector store implementation
pub struct InMemoryVectorStore {
    vectors: Arc<Mutex<HashMap<String, Vec<f32>>>>,
    metadata: Arc<Mutex<HashMap<String, serde_json::Value>>>,
}

impl InMemoryVectorStore {
    /// Create a new in-memory vector store
    pub fn new() -> Self {
        Self {
            vectors: Arc::new(Mutex::new(HashMap::new())),
            metadata: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl VectorStore for InMemoryVectorStore {
    fn add(
        &self,
        id: String,
        embedding: Vec<f32>,
        metadata: Option<serde_json::Value>,
    ) -> PendingVectorOp {
        let (tx, rx) = oneshot::channel();
        let vectors = self.vectors.clone();
        let metadata_store = self.metadata.clone();

        tokio::spawn(async move {
            vectors.lock().unwrap().insert(id.clone(), embedding);

            if let Some(meta) = metadata {
                metadata_store.lock().unwrap().insert(id, meta);
            }

            let _ = tx.send(Ok(()));
        });

        PendingVectorOp::new(rx)
    }

    fn update(
        &self,
        id: String,
        embedding: Vec<f32>,
        metadata: Option<serde_json::Value>,
    ) -> PendingVectorOp {
        let (tx, rx) = oneshot::channel();
        let vectors = self.vectors.clone();
        let metadata_store = self.metadata.clone();

        tokio::spawn(async move {
            let result = {
                let vectors_lock = vectors.lock().unwrap();
                if !vectors_lock.contains_key(&id) {
                    Err(Error::NotFound(format!("Vector with id {} not found", id)))
                } else {
                    drop(vectors_lock);
                    vectors.lock().unwrap().insert(id.clone(), embedding);

                    if let Some(meta) = metadata {
                        metadata_store.lock().unwrap().insert(id, meta);
                    }

                    Ok(())
                }
            };

            let _ = tx.send(result);
        });

        PendingVectorOp::new(rx)
    }

    fn delete(&self, id: String) -> PendingVectorOp {
        let (tx, rx) = oneshot::channel();
        let vectors = self.vectors.clone();
        let metadata_store = self.metadata.clone();

        tokio::spawn(async move {
            vectors.lock().unwrap().remove(&id);
            metadata_store.lock().unwrap().remove(&id);
            let _ = tx.send(Ok(()));
        });

        PendingVectorOp::new(rx)
    }

    fn search(
        &self,
        query: Vec<f32>,
        limit: usize,
        filter: Option<MemoryFilter>,
    ) -> PendingVectorSearch {
        let (tx, rx) = oneshot::channel();
        let vectors = self.vectors.clone();
        let metadata_store = self.metadata.clone();

        tokio::spawn(async move {
            let vectors_lock = vectors.lock().unwrap();
            let metadata_lock = metadata_store.lock().unwrap();

            let mut results: Vec<(String, f32, Option<serde_json::Value>)> = Vec::new();

            // Simple cosine similarity search
            for (id, vector) in vectors_lock.iter() {
                // Apply filters if any
                if let Some(_filter) = &filter {
                    // TODO: Implement filter logic based on MemoryFilter
                    // For now, skip filtering
                }

                let similarity = cosine_similarity(&query, vector);
                let meta = metadata_lock.get(id).cloned();
                results.push((id.clone(), similarity, meta));
            }

            // Sort by similarity (descending)
            results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            // Take top k results
            let search_results: Vec<VectorSearchResult> = results
                .into_iter()
                .take(limit)
                .map(|(id, score, metadata)| VectorSearchResult {
                    id,
                    score,
                    metadata,
                })
                .collect();

            let _ = tx.send(Ok(search_results));
        });

        PendingVectorSearch::new(rx)
    }

    fn embed(&self, text: String) -> PendingEmbedding {
        let (tx, rx) = oneshot::channel();

        tokio::spawn(async move {
            // Simple mock embedding - in a real implementation, this would call an embedding model
            // For now, just generate a random vector based on text hash
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};

            let mut hasher = DefaultHasher::new();
            text.hash(&mut hasher);
            let hash = hasher.finish();

            // Generate a 384-dimensional vector (like sentence-transformers)
            let mut embedding = Vec::with_capacity(384);
            for i in 0..384 {
                let value = ((hash.wrapping_add(i as u64) % 1000) as f32) / 1000.0;
                embedding.push(value);
            }

            let _ = tx.send(Ok(embedding));
        });

        PendingEmbedding::new(rx)
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
