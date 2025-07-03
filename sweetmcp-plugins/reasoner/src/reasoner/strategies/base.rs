use crate::state::StateManager;
use crate::types::{ReasoningRequest, ReasoningResponse, StrategyMetrics, ThoughtNode};
use futures::Stream;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use serde_json::json;
use std::collections::HashSet;
use std::env;
use std::fmt;
use std::future::Future;
#[allow(unused_imports)]
use std::hash::Hash;
#[allow(unused_imports)]
use std::hash::Hasher;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::{mpsc, oneshot};
use tokio_stream::wrappers::ReceiverStream;

/// Error type for reasoning operations
#[derive(Debug)]
pub enum ReasoningError {
    StrategyUnavailable,
    Other(String),
}

impl fmt::Display for ReasoningError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StrategyUnavailable => write!(f, "Reasoning strategy unavailable"),
            Self::Other(msg) => write!(f, "Reasoning error: {}", msg),
        }
    }
}

impl std::error::Error for ReasoningError {}

/// A convenience type alias for reasoning results
pub type ReasoningResult<T> = Result<T, ReasoningError>;

//==============================================================================
// AsyncTask - Generic awaitable type for all single-value operations
//==============================================================================

/// Generic awaitable future for any operation that returns a single value
pub struct AsyncTask<T> {
    rx: oneshot::Receiver<ReasoningResult<T>>,
}

impl<T> AsyncTask<T> {
    /// Creates a new AsyncTask from a receiver
    pub fn new(rx: oneshot::Receiver<ReasoningResult<T>>) -> Self {
        Self { rx }
    }

    /// Creates an AsyncTask from a direct value
    pub fn from_value(value: T) -> Self {
        let (tx, rx) = oneshot::channel();
        let _ = tx.send(Ok(value));
        Self { rx }
    }

    /// Creates an AsyncTask that will produce an error
    pub fn from_error(error: ReasoningError) -> Self {
        let (tx, rx) = oneshot::channel();
        let _ = tx.send(Err(error));
        Self { rx }
    }
}

impl<T> Future for AsyncTask<T> {
    type Output = ReasoningResult<T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.rx).poll(cx) {
            Poll::Ready(Ok(result)) => Poll::Ready(result),
            Poll::Ready(Err(_)) => Poll::Ready(Err(ReasoningError::Other("Task failed".into()))),
            Poll::Pending => Poll::Pending,
        }
    }
}

//==============================================================================
// TaskStream - Generic stream type for all multi-value operations
//==============================================================================

/// Generic stream for any operation that returns multiple values
pub struct TaskStream<T> {
    inner: ReceiverStream<ReasoningResult<T>>,
}

impl<T> TaskStream<T> {
    /// Creates a new stream from a receiver
    pub fn new(rx: mpsc::Receiver<ReasoningResult<T>>) -> Self {
        Self {
            inner: ReceiverStream::new(rx),
        }
    }

    /// Creates a stream containing a single value
    pub fn from_value(value: T) -> Self {
        let (tx, rx) = mpsc::channel(1);
        let _ = tx.try_send(Ok(value));
        Self::new(rx)
    }

    /// Creates a stream that produces an error
    pub fn from_error(error: ReasoningError) -> Self {
        let (tx, rx) = mpsc::channel(1);
        let _ = tx.try_send(Err(error));
        Self::new(rx)
    }
}

impl<T> Stream for TaskStream<T> {
    type Item = ReasoningResult<T>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}

// Type aliases for backward compatibility with existing code
pub type AsyncPath = AsyncTask<Vec<ThoughtNode>>;
pub type ClearedSignal = AsyncTask<()>;
pub type MetricStream = TaskStream<StrategyMetrics>;
pub type Reasoning = TaskStream<ReasoningResponse>;
pub type Metric = StrategyMetrics;

// --- Structs for VoyageAI API ---
#[derive(Serialize)]
struct VoyageAIRequest {
    input: Vec<String>,
    model: String,
}

#[derive(Deserialize, Debug)]
struct VoyageAIEmbedding {
    embedding: Vec<f32>,
    // object: String, // Optional fields if needed
    // index: usize, // Optional fields if needed
}

#[derive(Deserialize, Debug)]
struct VoyageAIUsage {
    #[allow(dead_code)]
    total_tokens: u32,
    // prompt_tokens: u32, // Optional fields if needed
}

#[derive(Deserialize, Debug)]
struct VoyageAIResponse {
    data: Vec<VoyageAIEmbedding>,
    #[allow(dead_code)]
    model: String,
    #[allow(dead_code)]
    usage: VoyageAIUsage,
    // object: String, // Optional fields if needed
}
// --- End Structs for VoyageAI API ---


/// Strategy trait without async_trait
pub trait Strategy: Send + Sync {
    /// Process a thought with the selected strategy
    fn process_thought(&self, request: ReasoningRequest) -> Reasoning;

    /// Get the best reasoning path found by this strategy
    fn get_best_path(&self) -> AsyncPath;

    /// Get strategy metrics
    fn get_metrics(&self) -> MetricStream;

    /// Clear strategy state
    fn clear(&self) -> ClearedSignal;
}

/// Base strategy implementation that provides common functionality
pub struct BaseStrategy {
    pub state_manager: Arc<StateManager>,
}

impl BaseStrategy {
    pub fn new(state_manager: Arc<StateManager>) -> Self {
        Self { state_manager }
    }

    pub fn get_node(&self, id: &str) -> AsyncTask<Option<ThoughtNode>> {
        let state_manager = Arc::clone(&self.state_manager);
        let id = id.to_string();

        let (tx, rx) = oneshot::channel();

        tokio::spawn(async move {
            let result = state_manager.get_node(&id).await;
            let _ = tx.send(Ok(result));
        });

        AsyncTask::new(rx)
    }

    pub fn save_node(&self, node: ThoughtNode) -> AsyncTask<()> {
        let state_manager = Arc::clone(&self.state_manager);
        let node = node.clone();

        let (tx, rx) = oneshot::channel();

        tokio::spawn(async move {
            state_manager.save_node(node).await;
            let _ = tx.send(Ok(()));
        });

        AsyncTask::new(rx)
    }

    /// Calculates cosine similarity between two vectors.
    fn cosine_similarity(v1: &[f32], v2: &[f32]) -> f64 {
        if v1.len() != v2.len() || v1.is_empty() {
            return 0.0; // Return 0 if vectors are different lengths or empty
        }

        let dot_product: f32 = v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum();
        let magnitude1: f32 = v1.iter().map(|x| x * x).sum::<f32>().sqrt();
        let magnitude2: f32 = v2.iter().map(|x| x * x).sum::<f32>().sqrt();

        if magnitude1 == 0.0 || magnitude2 == 0.0 {
            return 0.0; // Avoid division by zero
        }

        (dot_product / (magnitude1 * magnitude2)) as f64
    }


    pub fn evaluate_thought(&self, node: &ThoughtNode, parent: Option<&ThoughtNode>) -> f64 {
        // Base evaluation logic - Semantic coherence is now handled async by strategies
        let logical_score = self.calculate_logical_score(node, parent);
        let depth_penalty = self.calculate_depth_penalty(node);
        let completion_bonus = if node.is_complete { 0.2 } else { 0.0 };

        (logical_score + depth_penalty + completion_bonus) / 3.0
    }

    fn calculate_logical_score(&self, node: &ThoughtNode, _parent: Option<&ThoughtNode>) -> f64 {
        let mut score = 0.0;

        // Length and complexity
        score += (node.thought.len() as f64 / 200.0).min(0.3);

        // Logical connectors
        // This regex is a constant pattern so it's safe to panic if it's invalid
        let re = Regex::new(r"\b(therefore|because|if|then|thus|hence|so)\b")
            .expect("Failed to compile logical connectors regex");
        if re.is_match(&node.thought) {
            score += 0.2;
        }

        // Mathematical/logical expressions
        // This regex is a constant pattern so it's safe to panic if it's invalid
        let re = Regex::new(r"[+\-*/=<>]")
            .expect("Failed to compile mathematical expressions regex");
        if re.is_match(&node.thought) {
            score += 0.2;
        }

        // Parent-child semantic coherence is now handled asynchronously by strategies
        // We use a placeholder/neutral value here in the synchronous evaluation.
        // A strategy wanting semantic coherence must call `calculate_semantic_coherence`
        // and incorporate its awaited result.
        let coherence_placeholder = 0.5; // Neutral placeholder
        score += coherence_placeholder * 0.4;


        // Ensure score is within a reasonable range (e.g., 0 to 1) before returning
        score.max(0.0).min(1.0)
    }

    fn calculate_depth_penalty(&self, node: &ThoughtNode) -> f64 {
        // Penalize deeper thoughts slightly less aggressively
        (1.0 - (node.depth as f64 / crate::types::CONFIG.max_depth as f64) * 0.2).max(0.0)
    }

    /// Calculates semantic coherence using VoyageAI embeddings. Returns an AsyncTask.
    /// Note: This is now async and returns a future. Strategies must await this.
    pub fn calculate_semantic_coherence(
        &self,
        parent_thought: &str,
        child_thought: &str,
    ) -> AsyncTask<f64> {
        let parent_thought = parent_thought.to_string();
        let child_thought = child_thought.to_string();
        let (tx, rx) = oneshot::channel();

        tokio::spawn(async move {
            // --- VoyageAI API Call ---
            // Use environment variable for API key
            let api_key = match env::var("VOYAGE_API_KEY") {
                Ok(key) => key,
                Err(_) => {
                    let _ = tx.send(Err(ReasoningError::Other(
                        "VOYAGE_API_KEY environment variable not set".into(),
                    )));
                    return;
                }
            };
            // Note: Model and API URL are currently hardcoded. Consider making configurable.
            let model = "voyage-2";
            let api_url = "https://api.voyageai.com/v1/embeddings"; // Hardcoded URL

            let client = Client::new();
            let request_body = VoyageAIRequest {
                input: vec![parent_thought, child_thought],
                model: model.to_string(),
            };

            let response = match client
                .post(api_url)
                .bearer_auth(api_key)
                .json(&request_body)
                .send()
                .await
            {
                Ok(res) => res,
                Err(e) => {
                    let _ = tx.send(Err(ReasoningError::Other(format!(
                        "Failed to send request to VoyageAI: {}",
                        e
                    ))));
                    return;
                }
            };

            if !response.status().is_success() {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".into());
                 let _ = tx.send(Err(ReasoningError::Other(format!(
                    "VoyageAI API error ({}): {}",
                    status, error_text
                ))));
                return;
            }

            let voyage_response = match response.json::<VoyageAIResponse>().await {
                Ok(data) => data,
                Err(e) => {
                     let _ = tx.send(Err(ReasoningError::Other(format!(
                        "Failed to parse VoyageAI response: {}",
                        e
                    ))));
                    return;
                }
            };

            if voyage_response.data.len() < 2 {
                 let _ = tx.send(Err(ReasoningError::Other(
                    "VoyageAI response did not contain enough embeddings".into(),
                )));
                return;
            }

            let parent_embedding = &voyage_response.data[0].embedding;
            let child_embedding = &voyage_response.data[1].embedding;

            let similarity = Self::cosine_similarity(parent_embedding, child_embedding);
            // Scale similarity from [-1, 1] to [0, 1] for scoring consistency
            let scaled_similarity = (similarity + 1.0) / 2.0;

            let _ = tx.send(Ok(scaled_similarity));
            // --- End VoyageAI API Call ---
        });

        AsyncTask::new(rx)
    }

    // Original word overlap coherence function (kept for reference or fallback if needed)
    #[allow(dead_code)]
    fn calculate_word_overlap_coherence(&self, parent_thought: &str, child_thought: &str) -> f64 {
         let parent_terms: HashSet<String> = parent_thought
             .to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();

        let child_terms: Vec<String> = child_thought
            .to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();

        let shared_terms = child_terms
            .iter()
            .filter(|term| parent_terms.contains(*term))
            .count();

        if child_terms.is_empty() {
            return 0.0;
        }
        let overlap_score = (shared_terms as f64 / child_terms.len() as f64).min(1.0);

        overlap_score
    }

    /// Get base metrics
    pub fn get_base_metrics(&self) -> AsyncTask<StrategyMetrics> {
        let state_manager = Arc::clone(&self.state_manager);

        let (tx, rx) = oneshot::channel();

        tokio::spawn(async move {
            let nodes = state_manager.get_all_nodes().await;

            let avg_score = if nodes.is_empty() {
                0.0
            } else {
                nodes.iter().map(|n| n.score).sum::<f64>() / nodes.len() as f64
            };

            let max_depth = nodes.iter().map(|n| n.depth).max().unwrap_or(0);

            let metrics = StrategyMetrics {
                name: String::from("BaseStrategy"),
                nodes_explored: nodes.len(),
                average_score: avg_score,
                max_depth,
                active: None,
                extra: Default::default(),
            };

            let _ = tx.send(Ok(metrics));
        });

        AsyncTask::new(rx)
    }
}

/// Default implementation of Strategy for BaseStrategy
impl Strategy for BaseStrategy {
    fn process_thought(&self, _request: ReasoningRequest) -> Reasoning {
        TaskStream::from_error(ReasoningError::Other(
            "Base strategy does not implement process_thought".into(),
        ))
    }

    fn get_best_path(&self) -> AsyncPath {
        let state_manager = Arc::clone(&self.state_manager);

        let (tx, rx) = oneshot::channel();

        tokio::spawn(async move {
            let nodes = state_manager.get_all_nodes().await;
            if nodes.is_empty() {
                let _ = tx.send(Ok(vec![]));
                return;
            }

            // Find highest scoring complete path
            let mut completed_nodes: Vec<ThoughtNode> =
                nodes.into_iter().filter(|n| n.is_complete).collect();

            if completed_nodes.is_empty() {
                let _ = tx.send(Ok(vec![]));
                return;
            }

            completed_nodes.sort_by(|a, b| {
                b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal)
            });
            let path = state_manager.get_path(&completed_nodes[0].id).await;
            let _ = tx.send(Ok(path));
        });

        AsyncTask::new(rx)
    }

    fn get_metrics(&self) -> MetricStream {
        // Convert AsyncTask to TaskStream
        let async_metrics = self.get_base_metrics();
        let (tx, rx) = mpsc::channel(1);
        
        tokio::spawn(async move {
            match async_metrics.await {
                Ok(metrics) => {
                    let _ = tx.send(Ok(metrics)).await;
                },
                Err(err) => {
                    let _ = tx.send(Err(err)).await;
                }
            }
        });
        
        TaskStream::new(rx)
    }

    fn clear(&self) -> ClearedSignal {
        AsyncTask::from_value(())
    }
}
