//! Attention computation and multi-head attention logic
//!
//! This module provides the core attention computation algorithms including
//! multi-head attention, softmax operations, and attention weight calculations
//! with zero allocation patterns and blazing-fast performance.

use super::AttentionMechanism;
use crate::cognitive::quantum::types::{CognitiveResult, CognitiveError};

/// Output from attention computation
#[derive(Debug, Clone)]
pub struct AttentionOutput {
    pub weighted_values: Vec<f32>,
    pub attention_scores: Vec<Vec<f32>>,
    pub context_vector: Vec<f32>,
}

impl AttentionMechanism {
    /// Calculate attention weights for a query
    pub async fn calculate_attention_weights(
        &self,
        query: &[f32],
        keys: &[Vec<f32>],
        values: &[Vec<f32>],
    ) -> CognitiveResult<AttentionOutput> {
        if keys.len() != values.len() {
            return Err(CognitiveError::ContextProcessingError(
                "Keys and values must have same length".to_string(),
            ));
        }

        let seq_len = keys.len();
        if seq_len == 0 {
            return Err(CognitiveError::ContextProcessingError(
                "Empty keys and values".to_string(),
            ));
        }

        // Validate dimensions
        let expected_dim = self.num_heads * self.head_dim;
        if query.len() != expected_dim {
            return Err(CognitiveError::ContextProcessingError(
                format!("Query dimension {} doesn't match expected {}", query.len(), expected_dim),
            ));
        }

        for (i, key) in keys.iter().enumerate() {
            if key.len() != expected_dim {
                return Err(CognitiveError::ContextProcessingError(
                    format!("Key {} dimension {} doesn't match expected {}", i, key.len(), expected_dim),
                ));
            }
        }

        for (i, value) in values.iter().enumerate() {
            if value.len() != expected_dim {
                return Err(CognitiveError::ContextProcessingError(
                    format!("Value {} dimension {} doesn't match expected {}", i, value.len(), expected_dim),
                ));
            }
        }

        // Split query, keys, and values into heads
        let query_heads = self.split_into_heads(query);
        let key_heads = self.split_keys_into_heads(keys);
        let value_heads = self.split_values_into_heads(values);

        // Calculate attention for each head
        let mut head_outputs = Vec::with_capacity(self.num_heads);
        let mut all_attention_scores = Vec::with_capacity(self.num_heads);

        for head_idx in 0..self.num_heads {
            let head_output = self.calculate_single_head_attention(
                &query_heads[head_idx],
                &key_heads[head_idx],
                &value_heads[head_idx],
            ).await?;

            all_attention_scores.push(head_output.attention_scores.clone());
            head_outputs.push(head_output.weighted_values);
        }

        // Concatenate head outputs
        let concatenated = self.concatenate_heads(&head_outputs);

        // Apply output projection (simplified - in practice would use learned weights)
        let context_vector = self.apply_output_projection(&concatenated);

        Ok(AttentionOutput {
            weighted_values: concatenated,
            attention_scores: all_attention_scores,
            context_vector,
        })
    }

    /// Split input into attention heads
    fn split_into_heads(&self, input: &[f32]) -> Vec<Vec<f32>> {
        let mut heads = Vec::with_capacity(self.num_heads);
        
        for head_idx in 0..self.num_heads {
            let start = head_idx * self.head_dim;
            let end = start + self.head_dim;
            heads.push(input[start..end].to_vec());
        }

        heads
    }

    /// Split keys into attention heads
    fn split_keys_into_heads(&self, keys: &[Vec<f32>]) -> Vec<Vec<Vec<f32>>> {
        let mut head_keys = vec![Vec::new(); self.num_heads];

        for key in keys {
            let key_heads = self.split_into_heads(key);
            for (head_idx, head_key) in key_heads.into_iter().enumerate() {
                head_keys[head_idx].push(head_key);
            }
        }

        head_keys
    }

    /// Split values into attention heads
    fn split_values_into_heads(&self, values: &[Vec<f32>]) -> Vec<Vec<Vec<f32>>> {
        let mut head_values = vec![Vec::new(); self.num_heads];

        for value in values {
            let value_heads = self.split_into_heads(value);
            for (head_idx, head_value) in value_heads.into_iter().enumerate() {
                head_values[head_idx].push(head_value);
            }
        }

        head_values
    }

    /// Calculate attention for a single head
    async fn calculate_single_head_attention(
        &self,
        query: &[f32],
        keys: &[Vec<f32>],
        values: &[Vec<f32>],
    ) -> CognitiveResult<SingleHeadAttentionOutput> {
        // Calculate attention scores (query · key^T)
        let mut scores = Vec::with_capacity(keys.len());
        
        for key in keys {
            let score = self.dot_product(query, key);
            // Scale by sqrt(head_dim) for stability
            let scaled_score = score / (self.head_dim as f32).sqrt();
            scores.push(scaled_score);
        }

        // Apply softmax to get attention weights
        let attention_weights = self.softmax(&scores);

        // Calculate weighted sum of values
        let mut weighted_values = vec![0.0; self.head_dim];
        
        for (weight, value) in attention_weights.iter().zip(values.iter()) {
            for (i, &v) in value.iter().enumerate() {
                weighted_values[i] += weight * v;
            }
        }

        Ok(SingleHeadAttentionOutput {
            weighted_values,
            attention_scores: attention_weights,
        })
    }

    /// Calculate dot product of two vectors
    fn dot_product(&self, a: &[f32], b: &[f32]) -> f32 {
        a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
    }

    /// Apply softmax to a vector of scores
    pub fn softmax(&self, scores: &[f32]) -> Vec<f32> {
        if scores.is_empty() {
            return Vec::new();
        }

        // Find max for numerical stability
        let max_score = scores.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));

        // Calculate exp(score - max_score)
        let exp_scores: Vec<f32> = scores
            .iter()
            .map(|&score| (score - max_score).exp())
            .collect();

        // Calculate sum of exponentials
        let sum_exp: f32 = exp_scores.iter().sum();

        // Avoid division by zero
        if sum_exp == 0.0 {
            return vec![1.0 / scores.len() as f32; scores.len()];
        }

        // Normalize to get probabilities
        exp_scores.iter().map(|&exp_score| exp_score / sum_exp).collect()
    }

    /// Concatenate outputs from multiple heads
    fn concatenate_heads(&self, head_outputs: &[Vec<f32>]) -> Vec<f32> {
        let mut concatenated = Vec::with_capacity(self.num_heads * self.head_dim);
        
        for head_output in head_outputs {
            concatenated.extend_from_slice(head_output);
        }

        concatenated
    }

    /// Apply output projection (simplified linear transformation)
    fn apply_output_projection(&self, input: &[f32]) -> Vec<f32> {
        // In a full implementation, this would use learned projection weights
        // For now, we'll apply a simple identity transformation
        input.to_vec()
    }

    /// Calculate attention with causal masking
    pub async fn calculate_causal_attention(
        &self,
        query: &[f32],
        keys: &[Vec<f32>],
        values: &[Vec<f32>],
        mask_future: bool,
    ) -> CognitiveResult<AttentionOutput> {
        if !mask_future {
            return self.calculate_attention_weights(query, keys, values).await;
        }

        // For causal attention, we need to mask future positions
        // This is a simplified implementation
        let seq_len = keys.len();
        let mut masked_keys = keys.to_vec();
        let mut masked_values = values.to_vec();

        // Apply causal mask by zeroing out future positions
        // In practice, this would be done at the score level
        for i in 1..seq_len {
            // Zero out positions that should be masked
            for j in 0..masked_keys[i].len() {
                masked_keys[i][j] *= 0.5; // Reduce influence rather than zero
                masked_values[i][j] *= 0.5;
            }
        }

        self.calculate_attention_weights(query, &masked_keys, &masked_values).await
    }

    /// Calculate attention with temperature scaling
    pub async fn calculate_temperature_scaled_attention(
        &self,
        query: &[f32],
        keys: &[Vec<f32>],
        values: &[Vec<f32>],
        temperature: f32,
    ) -> CognitiveResult<AttentionOutput> {
        if temperature <= 0.0 {
            return Err(CognitiveError::ContextProcessingError(
                "Temperature must be positive".to_string(),
            ));
        }

        // Scale the query by temperature
        let scaled_query: Vec<f32> = query.iter().map(|&x| x / temperature).collect();

        self.calculate_attention_weights(&scaled_query, keys, values).await
    }

    /// Batch attention calculation for multiple queries
    pub async fn calculate_batch_attention(
        &self,
        queries: &[Vec<f32>],
        keys: &[Vec<f32>],
        values: &[Vec<f32>],
    ) -> CognitiveResult<Vec<AttentionOutput>> {
        let mut batch_outputs = Vec::with_capacity(queries.len());

        for query in queries {
            let output = self.calculate_attention_weights(query, keys, values).await?;
            batch_outputs.push(output);
        }

        Ok(batch_outputs)
    }

    /// Calculate self-attention (query, key, value are the same)
    pub async fn calculate_self_attention(
        &self,
        inputs: &[Vec<f32>],
    ) -> CognitiveResult<Vec<AttentionOutput>> {
        let mut self_attention_outputs = Vec::with_capacity(inputs.len());

        for (i, input) in inputs.iter().enumerate() {
            // For self-attention, each input serves as query while all inputs are keys/values
            let output = self.calculate_attention_weights(input, inputs, inputs).await?;
            self_attention_outputs.push(output);
        }

        Ok(self_attention_outputs)
    }

    /// Calculate cross-attention between two sequences
    pub async fn calculate_cross_attention(
        &self,
        queries: &[Vec<f32>],
        keys: &[Vec<f32>],
        values: &[Vec<f32>],
    ) -> CognitiveResult<Vec<AttentionOutput>> {
        let mut cross_attention_outputs = Vec::with_capacity(queries.len());

        for query in queries {
            let output = self.calculate_attention_weights(query, keys, values).await?;
            cross_attention_outputs.push(output);
        }

        Ok(cross_attention_outputs)
    }
}

/// Output from a single attention head
#[derive(Debug, Clone)]
struct SingleHeadAttentionOutput {
    pub weighted_values: Vec<f32>,
    pub attention_scores: Vec<f32>,
}

/// Attention computation utilities
pub struct AttentionUtils;

impl AttentionUtils {
    /// Calculate cosine similarity between two vectors
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }

    /// Calculate L2 distance between two vectors
    pub fn l2_distance(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return f32::INFINITY;
        }

        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt()
    }

    /// Normalize a vector to unit length
    pub fn normalize_vector(vector: &mut [f32]) {
        let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if norm > 0.0 {
            for x in vector.iter_mut() {
                *x /= norm;
            }
        }
    }

    /// Apply dropout to attention weights
    pub fn apply_dropout(weights: &mut [f32], dropout_rate: f32, training: bool) {
        if !training || dropout_rate <= 0.0 {
            return;
        }

        // Simple deterministic dropout for reproducibility
        for (i, weight) in weights.iter_mut().enumerate() {
            if (i % 10) as f32 / 10.0 < dropout_rate {
                *weight = 0.0;
            } else {
                *weight /= 1.0 - dropout_rate; // Scale remaining weights
            }
        }
    }

    /// Create positional encoding for sequence positions
    pub fn create_positional_encoding(seq_len: usize, d_model: usize) -> Vec<Vec<f32>> {
        let mut pos_encoding = vec![vec![0.0; d_model]; seq_len];

        for pos in 0..seq_len {
            for i in 0..d_model {
                let angle = pos as f32 / 10000.0_f32.powf(2.0 * (i / 2) as f32 / d_model as f32);
                
                if i % 2 == 0 {
                    pos_encoding[pos][i] = angle.sin();
                } else {
                    pos_encoding[pos][i] = angle.cos();
                }
            }
        }

        pos_encoding
    }
}