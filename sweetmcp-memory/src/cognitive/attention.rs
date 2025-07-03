//! Attention mechanism for cognitive memory management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Attention mechanism for relevance scoring and focus management
#[derive(Debug, Clone)]
pub struct AttentionMechanism {
    pub num_heads: usize,
    pub head_dim: usize,
    pub dropout_rate: f32,
    pub attention_scores: HashMap<String, f32>,
}

/// Multi-head attention configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionConfig {
    pub num_heads: usize,
    pub hidden_dim: usize,
    pub dropout_rate: f32,
    pub use_causal_mask: bool,
}

/// Attention weights for memory nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionWeights {
    pub query_weights: Vec<f32>,
    pub key_weights: Vec<f32>,
    pub value_weights: Vec<f32>,
    pub output_weights: Vec<f32>,
}

/// Attention output
#[derive(Debug, Clone)]
pub struct AttentionOutput {
    pub weighted_values: Vec<f32>,
    pub attention_scores: Vec<Vec<f32>>,
    pub context_vector: Vec<f32>,
}

impl AttentionMechanism {
    /// Create a new attention mechanism
    pub fn new(config: AttentionConfig) -> Self {
        let head_dim = config.hidden_dim / config.num_heads;

        Self {
            num_heads: config.num_heads,
            head_dim,
            dropout_rate: config.dropout_rate,
            attention_scores: HashMap::new(),
        }
    }

    /// Calculate attention weights for a query
    pub async fn calculate_attention_weights(
        &self,
        query: &[f32],
        keys: &[Vec<f32>],
        values: &[Vec<f32>],
    ) -> crate::cognitive::quantum::types::CognitiveResult<AttentionOutput> {
        if keys.len() != values.len() {
            return Err(
                crate::cognitive::quantum::types::CognitiveError::ContextProcessingError(
                    "Keys and values must have same length".to_string(),
                ),
            );
        }

        let seq_len = keys.len();
        let mut all_attention_scores = Vec::with_capacity(self.num_heads);
        let mut all_weighted_values = Vec::with_capacity(self.num_heads * self.head_dim);

        // Process each attention head
        for head in 0..self.num_heads {
            let head_scores = self.compute_head_attention(query, keys, values, head)?;

            all_attention_scores.push(head_scores.attention_scores.clone());
            all_weighted_values.extend(&head_scores.weighted_values);
        }

        // Concatenate heads and create output
        let context_vector = self.merge_heads(&all_weighted_values);

        Ok(AttentionOutput {
            weighted_values: all_weighted_values,
            attention_scores: all_attention_scores,
            context_vector,
        })
    }

    /// Compute attention for a single head
    fn compute_head_attention(
        &self,
        query: &[f32],
        keys: &[Vec<f32>],
        values: &[Vec<f32>],
        head_idx: usize,
    ) -> crate::cognitive::quantum::types::CognitiveResult<HeadAttentionOutput> {
        let seq_len = keys.len();
        let mut scores = vec![0.0; seq_len];

        // Project query for this head
        let head_query = self.project_to_head(query, head_idx);

        // Compute attention scores
        for (i, key) in keys.iter().enumerate() {
            let head_key = self.project_to_head(key, head_idx);
            scores[i] = self.scaled_dot_product(&head_query, &head_key);
        }

        // Apply softmax
        let attention_weights = self.softmax(&scores);

        // Apply attention to values
        let mut weighted_values = vec![0.0; self.head_dim];
        for (i, value) in values.iter().enumerate() {
            let head_value = self.project_to_head(value, head_idx);
            for j in 0..self.head_dim {
                weighted_values[j] += attention_weights[i] * head_value[j];
            }
        }

        Ok(HeadAttentionOutput {
            weighted_values,
            attention_scores: attention_weights,
        })
    }

    /// Project vector to attention head subspace
    fn project_to_head(&self, vector: &[f32], head_idx: usize) -> Vec<f32> {
        let start = head_idx * self.head_dim;
        let end = start + self.head_dim;

        if end <= vector.len() {
            vector[start..end].to_vec()
        } else {
            // Pad with zeros if vector is too short
            let mut result = vec![0.0; self.head_dim];
            let available = vector.len() - start;
            if available > 0 {
                result[..available].copy_from_slice(&vector[start..]);
            }
            result
        }
    }

    /// Scaled dot product attention score
    fn scaled_dot_product(&self, query: &[f32], key: &[f32]) -> f32 {
        let dot_product: f32 = query.iter().zip(key.iter()).map(|(q, k)| q * k).sum();

        dot_product / (self.head_dim as f32).sqrt()
    }

    /// Softmax normalization
    fn softmax(&self, scores: &[f32]) -> Vec<f32> {
        let max_score = scores.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));

        let exp_scores: Vec<f32> = scores.iter().map(|&s| (s - max_score).exp()).collect();

        let sum: f32 = exp_scores.iter().sum();

        exp_scores.iter().map(|&e| e / sum).collect()
    }

    /// Merge attention heads
    fn merge_heads(&self, all_head_outputs: &[f32]) -> Vec<f32> {
        // Simple concatenation for now
        all_head_outputs.to_vec()
    }

    /// Calculate attention scores for memory retrieval
    pub async fn score_memories(
        &mut self,
        query_embedding: &[f32],
        memory_embeddings: &[(String, Vec<f32>)],
    ) -> Vec<(String, f32)> {
        let mut scores = Vec::new();

        for (memory_id, embedding) in memory_embeddings {
            let score = self.calculate_similarity(query_embedding, embedding);
            scores.push((memory_id.clone(), score));
            self.attention_scores.insert(memory_id.clone(), score);
        }

        // Sort by score descending
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        scores
    }

    /// Calculate similarity between embeddings
    fn calculate_similarity(&self, query: &[f32], memory: &[f32]) -> f32 {
        // Cosine similarity
        let dot_product: f32 = query.iter().zip(memory.iter()).map(|(q, m)| q * m).sum();

        let query_norm: f32 = query.iter().map(|x| x * x).sum::<f32>().sqrt();
        let memory_norm: f32 = memory.iter().map(|x| x * x).sum::<f32>().sqrt();

        if query_norm > 0.0 && memory_norm > 0.0 {
            dot_product / (query_norm * memory_norm)
        } else {
            0.0
        }
    }

    /// Update attention scores based on feedback
    pub fn update_scores(&mut self, memory_id: &str, feedback: f32) {
        if let Some(score) = self.attention_scores.get_mut(memory_id) {
            // Exponential moving average update
            let alpha = 0.1;
            *score = (1.0 - alpha) * *score + alpha * feedback;
        }
    }

    /// Get top-k attended memories
    pub fn get_top_k(&self, k: usize) -> Vec<(String, f32)> {
        let mut scores: Vec<_> = self
            .attention_scores
            .iter()
            .map(|(id, score)| (id.clone(), *score))
            .collect();

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores.truncate(k);

        scores
    }
}

/// Output from a single attention head
struct HeadAttentionOutput {
    weighted_values: Vec<f32>,
    attention_scores: Vec<f32>,
}

impl Default for AttentionConfig {
    fn default() -> Self {
        Self {
            num_heads: 8,
            hidden_dim: 512,
            dropout_rate: 0.1,
            use_causal_mask: false,
        }
    }
}

impl AttentionWeights {
    /// Create random attention weights
    pub fn random(hidden_dim: usize) -> Self {
        Self {
            query_weights: Self::random_vector(hidden_dim),
            key_weights: Self::random_vector(hidden_dim),
            value_weights: Self::random_vector(hidden_dim),
            output_weights: Self::random_vector(hidden_dim),
        }
    }

    /// Generate random vector
    fn random_vector(size: usize) -> Vec<f32> {
        (0..size)
            .map(|_| rand::random::<f32>() * 0.1 - 0.05)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attention_mechanism_creation() {
        let config = AttentionConfig::default();
        let attention = AttentionMechanism::new(config);

        assert_eq!(attention.num_heads, 8);
        assert_eq!(attention.head_dim, 64); // 512 / 8
    }

    #[tokio::test]
    async fn test_attention_calculation() {
        let config = AttentionConfig {
            num_heads: 2,
            hidden_dim: 4,
            dropout_rate: 0.0,
            use_causal_mask: false,
        };

        let attention = AttentionMechanism::new(config);

        let query = vec![0.1, 0.2, 0.3, 0.4];
        let keys = vec![vec![0.1, 0.1, 0.1, 0.1], vec![0.2, 0.2, 0.2, 0.2]];
        let values = vec![vec![1.0, 0.0, 0.0, 0.0], vec![0.0, 1.0, 0.0, 0.0]];

        let output = attention
            .calculate_attention_weights(&query, &keys, &values)
            .await
            .unwrap();

        assert_eq!(output.attention_scores.len(), 2); // 2 heads
        assert!(!output.context_vector.is_empty());
    }

    #[test]
    fn test_softmax() {
        let attention = AttentionMechanism::new(AttentionConfig::default());

        let scores = vec![1.0, 2.0, 3.0];
        let softmax = attention.softmax(&scores);

        // Check that softmax sums to 1
        let sum: f32 = softmax.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6);

        // Check that higher scores get higher probabilities
        assert!(softmax[2] > softmax[1]);
        assert!(softmax[1] > softmax[0]);
    }

    #[tokio::test]
    async fn test_memory_scoring() {
        let mut attention = AttentionMechanism::new(AttentionConfig::default());

        let query = vec![0.1, 0.2, 0.3];
        let memories = vec![
            ("mem1".to_string(), vec![0.1, 0.2, 0.3]), // Same as query
            ("mem2".to_string(), vec![0.3, 0.2, 0.1]), // Different
            ("mem3".to_string(), vec![0.2, 0.4, 0.6]), // Scaled version
        ];

        let scores = attention.score_memories(&query, &memories).await;

        assert_eq!(scores.len(), 3);
        assert_eq!(scores[0].0, "mem1"); // Should be highest similarity
        assert!(scores[0].1 > 0.99); // Should be ~1.0 for identical vectors
    }
}
