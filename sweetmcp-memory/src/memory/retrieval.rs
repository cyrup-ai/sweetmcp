//! Memory retrieval strategies and algorithms

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::oneshot;

use crate::memory::filter::MemoryFilter;
use crate::utils::Result;
use crate::vector::VectorStore;

/// Retrieval method used to find the memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetrievalMethod {
    VectorSimilarity,
    Semantic,
    Temporal,
    Keyword,
    Hybrid,
}

/// A pending retrieval operation
pub struct PendingRetrieval {
    rx: oneshot::Receiver<Result<Vec<RetrievalResult>>>,
}

impl PendingRetrieval {
    pub fn new(rx: oneshot::Receiver<Result<Vec<RetrievalResult>>>) -> Self {
        Self { rx }
    }
}

impl Future for PendingRetrieval {
    type Output = Result<Vec<RetrievalResult>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.rx).poll(cx) {
            Poll::Ready(Ok(result)) => Poll::Ready(result),
            Poll::Ready(Err(_)) => Poll::Ready(Err(crate::utils::error::Error::Internal(
                "Retrieval task failed".to_string(),
            ))),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Memory retrieval strategy trait
pub trait RetrievalStrategy: Send + Sync {
    /// Retrieve memories based on the strategy
    fn retrieve(
        &self,
        query: String,
        limit: usize,
        filter: Option<MemoryFilter>,
    ) -> PendingRetrieval;

    /// Get strategy name
    fn name(&self) -> &str;
}

/// Result from memory retrieval
#[derive(Debug, Clone)]
pub struct RetrievalResult {
    /// Memory ID
    pub id: String,

    /// Relevance score
    pub score: f32,

    /// Retrieval method used
    pub method: RetrievalMethod,

    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Hybrid retrieval strategy combining multiple approaches
pub struct HybridRetrieval<V: VectorStore> {
    vector_store: V,
    strategies: std::sync::Arc<Vec<std::sync::Arc<dyn RetrievalStrategy>>>,
    weights: std::sync::Arc<HashMap<String, f32>>,
}

impl<V: VectorStore> HybridRetrieval<V> {
    /// Create a new hybrid retrieval strategy
    pub fn new(vector_store: V) -> Self {
        let mut weights = HashMap::new();
        weights.insert("semantic".to_string(), 0.6);
        weights.insert("keyword".to_string(), 0.2);
        weights.insert("temporal".to_string(), 0.2);

        Self {
            vector_store,
            strategies: std::sync::Arc::new(Vec::new()),
            weights: std::sync::Arc::new(weights),
        }
    }

    /// Add a retrieval strategy
    pub fn add_strategy(mut self, strategy: std::sync::Arc<dyn RetrievalStrategy>) -> Self {
        std::sync::Arc::make_mut(&mut self.strategies).push(strategy);
        self
    }

    /// Set weight for a strategy
    pub fn set_weight(mut self, strategy_name: &str, weight: f32) -> Self {
        std::sync::Arc::make_mut(&mut self.weights).insert(strategy_name.to_string(), weight);
        self
    }

    /// Get vector similarity results from the vector store
    pub async fn get_vector_similarity(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<RetrievalResult>> {
        let filter = crate::memory::filter::MemoryFilter::new();
        let results = self
            .vector_store
            .search(query_vector, limit, Some(filter))
            .await?;
        let retrieval_results = results
            .into_iter()
            .map(|result| RetrievalResult {
                id: result.id,
                method: RetrievalMethod::VectorSimilarity,
                score: result.score,
                metadata: HashMap::new(),
            })
            .collect();
        Ok(retrieval_results)
    }
}

impl<V: VectorStore + Send + Sync + 'static> RetrievalStrategy for HybridRetrieval<V> {
    fn retrieve(
        &self,
        query: String,
        limit: usize,
        filter: Option<MemoryFilter>,
    ) -> PendingRetrieval {
        let (tx, rx) = oneshot::channel();
        let strategies = self.strategies.clone();
        let weights = self.weights.clone();

        tokio::spawn(async move {
            let result: Result<Vec<RetrievalResult>> = (async {
                let mut all_results: HashMap<String, (f32, RetrievalResult)> = HashMap::new();

                // Get results from each strategy
                for strategy in &*strategies {
                    let results = strategy
                        .retrieve(query.clone(), limit * 2, filter.clone())
                        .await?;
                    let weight = weights.get(strategy.name()).unwrap_or(&1.0);

                    for result in results {
                        let weighted_score = result.score * weight;

                        all_results
                            .entry(result.id.clone())
                            .and_modify(|(score, _)| *score += weighted_score)
                            .or_insert((weighted_score, result));
                    }
                }

                // Sort by combined score and take top results
                let mut sorted_results: Vec<_> = all_results
                    .into_iter()
                    .map(|(_, (score, mut result))| {
                        result.score = score;
                        result
                    })
                    .collect();

                sorted_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
                sorted_results.truncate(limit);

                Ok(sorted_results)
            })
            .await;

            let _ = tx.send(result);
        });

        PendingRetrieval::new(rx)
    }

    fn name(&self) -> &str {
        "hybrid"
    }
}

/// Semantic similarity retrieval using vector embeddings
pub struct SemanticRetrieval<V: VectorStore> {
    vector_store: std::sync::Arc<V>,
}

impl<V: VectorStore> SemanticRetrieval<V> {
    pub fn new(vector_store: V) -> Self {
        Self {
            vector_store: std::sync::Arc::new(vector_store),
        }
    }
}

impl<V: VectorStore + Send + Sync + 'static> RetrievalStrategy for SemanticRetrieval<V> {
    fn retrieve(
        &self,
        query: String,
        limit: usize,
        filter: Option<MemoryFilter>,
    ) -> PendingRetrieval {
        let (tx, rx) = oneshot::channel();
        let vector_store = self.vector_store.clone();

        tokio::spawn(async move {
            let result: Result<Vec<RetrievalResult>> = (async {
                // Generate query embedding
                let query_embedding = vector_store.embed(query).await?;

                // Search in vector store
                let results = vector_store.search(query_embedding, limit, filter).await?;

                let retrieval_results = results
                    .into_iter()
                    .map(|r| RetrievalResult {
                        id: r.id,
                        score: r.score,
                        method: RetrievalMethod::Semantic,
                        metadata: HashMap::new(), // VectorSearchResult doesn't include metadata
                    })
                    .collect();

                Ok(retrieval_results)
            })
            .await;

            let _ = tx.send(result);
        });

        PendingRetrieval::new(rx)
    }

    fn name(&self) -> &str {
        "semantic"
    }
}

/// Temporal proximity retrieval
pub struct TemporalRetrieval {
    time_decay_factor: f32,
}

impl TemporalRetrieval {
    pub fn new(time_decay_factor: f32) -> Self {
        Self { time_decay_factor }
    }
}

impl RetrievalStrategy for TemporalRetrieval {
    fn retrieve(
        &self,
        _query: String,
        _limit: usize,
        _filter: Option<MemoryFilter>,
    ) -> PendingRetrieval {
        let (tx, rx) = oneshot::channel();
        let time_decay = self.time_decay_factor;

        tokio::spawn(async move {
            // Apply time decay factor to scoring
            let now = chrono::Utc::now().timestamp() as f32;
            let _decay_score = |timestamp: f32| -> f32 {
                let age_hours = (now - timestamp) / 3600.0;
                (age_hours * time_decay * -1.0).exp()
            };

            // This would typically query a time-indexed database
            // For now, return empty results as this is a placeholder
            let _ = tx.send(Ok(Vec::new()));
        });

        PendingRetrieval::new(rx)
    }

    fn name(&self) -> &str {
        "temporal"
    }
}

/// Memory retrieval manager
pub struct RetrievalManager<V: VectorStore> {
    strategies: HashMap<String, std::sync::Arc<dyn RetrievalStrategy>>,
    default_strategy: String,
    vector_store: V,
}

impl<V: VectorStore + Clone + Send + Sync + 'static> RetrievalManager<V> {
    /// Create a new retrieval manager
    pub fn new(vector_store: V) -> Self {
        let mut strategies: HashMap<String, std::sync::Arc<dyn RetrievalStrategy>> = HashMap::new();

        // Add default strategies
        strategies.insert(
            "semantic".to_string(),
            std::sync::Arc::new(SemanticRetrieval::new(vector_store.clone())),
        );

        strategies.insert(
            "temporal".to_string(),
            std::sync::Arc::new(TemporalRetrieval::new(0.95)),
        );

        Self {
            strategies,
            default_strategy: "semantic".to_string(),
            vector_store,
        }
    }

    /// Set the default retrieval strategy
    pub fn set_default_strategy(&mut self, strategy_name: String) {
        self.default_strategy = strategy_name;
    }

    /// Add a custom retrieval strategy
    pub fn add_strategy(&mut self, name: String, strategy: std::sync::Arc<dyn RetrievalStrategy>) {
        self.strategies.insert(name, strategy);
    }

    /// Direct vector search using the managed vector store
    pub async fn direct_vector_search(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<crate::vector::VectorSearchResult>> {
        let filter = crate::memory::filter::MemoryFilter::new();
        self.vector_store
            .search(query_vector, limit, Some(filter))
            .await
    }

    /// Retrieve memories using the specified strategy
    pub async fn retrieve(
        &self,
        query: &str,
        strategy_name: Option<&str>,
        limit: usize,
        filter: Option<&MemoryFilter>,
    ) -> Result<Vec<RetrievalResult>> {
        let strategy_name = strategy_name.unwrap_or(&self.default_strategy);

        if let Some(strategy) = self.strategies.get(strategy_name) {
            strategy
                .retrieve(query.to_string(), limit, filter.cloned())
                .await
        } else {
            Err(crate::utils::error::Error::InvalidInput(format!(
                "Unknown retrieval strategy: {}",
                strategy_name
            )))
        }
    }

    /// Retrieve using multiple strategies and combine results
    pub async fn multi_strategy_retrieve(
        &self,
        query: &str,
        strategy_names: Vec<&str>,
        limit: usize,
        filter: Option<&MemoryFilter>,
    ) -> Result<Vec<RetrievalResult>> {
        let mut all_results = Vec::new();

        for strategy_name in strategy_names {
            if let Some(strategy) = self.strategies.get(strategy_name) {
                let results = strategy
                    .retrieve(query.to_string(), limit, filter.cloned())
                    .await?;
                all_results.extend(results);
            }
        }

        // Deduplicate and sort by score
        let mut unique_results: HashMap<String, RetrievalResult> = HashMap::new();
        for result in all_results {
            unique_results
                .entry(result.id.clone())
                .and_modify(|r| {
                    if result.score > r.score {
                        r.score = result.score;
                    }
                })
                .or_insert(result);
        }

        let mut sorted_results: Vec<_> = unique_results.into_values().collect();
        sorted_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        sorted_results.truncate(limit);

        Ok(sorted_results)
    }
}
