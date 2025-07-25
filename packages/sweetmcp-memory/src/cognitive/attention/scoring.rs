//! Memory scoring and relevance calculation
//!
//! This module provides memory scoring algorithms for relevance calculation,
//! top memory retrieval, and attention score management with zero allocation
//! patterns and blazing-fast performance.

use super::{AttentionMechanism, AttentionUtils};

impl AttentionMechanism {
    /// Score memories based on similarity to query
    pub async fn score_memories(
        &mut self,
        query: &[f32],
        memories: &[(String, Vec<f32>)],
    ) -> Vec<(String, f32)> {
        let mut scored_memories = Vec::with_capacity(memories.len());

        for (memory_id, memory_vector) in memories {
            let similarity = AttentionUtils::cosine_similarity(query, memory_vector);
            scored_memories.push((memory_id.clone(), similarity));
            
            // Update internal attention scores
            self.attention_scores.insert(memory_id.clone(), similarity);
        }

        // Sort by similarity score (descending)
        scored_memories.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        scored_memories
    }

    /// Update attention scores for specific memories
    pub fn update_attention_scores(&mut self, memory_scores: &[(String, f32)]) {
        for (memory_id, score) in memory_scores {
            *self.attention_scores.entry(memory_id.clone()).or_insert(0.0) += score;
        }
    }

    /// Get top N memories by attention score
    pub fn get_top_memories(&self, n: usize) -> Vec<(String, f32)> {
        let mut sorted_scores: Vec<_> = self.attention_scores.iter().collect();
        sorted_scores.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        sorted_scores
            .into_iter()
            .take(n)
            .map(|(id, &score)| (id.clone(), score))
            .collect()
    }

    /// Calculate relevance decay over time
    pub fn apply_temporal_decay(&mut self, decay_factor: f32) {
        if decay_factor <= 0.0 || decay_factor >= 1.0 {
            return; // Invalid decay factor
        }

        for score in self.attention_scores.values_mut() {
            *score *= decay_factor;
        }

        // Remove scores that have decayed below threshold
        let threshold = 0.001;
        self.attention_scores.retain(|_, &mut score| score > threshold);
    }

    /// Calculate memory importance based on multiple factors
    pub fn calculate_memory_importance(
        &self,
        memory_id: &str,
        access_count: u32,
        recency_score: f32,
        content_relevance: f32,
    ) -> f32 {
        let attention_score = self.attention_scores.get(memory_id).copied().unwrap_or(0.0);
        
        // Weighted combination of factors
        let access_weight = 0.3;
        let recency_weight = 0.2;
        let content_weight = 0.3;
        let attention_weight = 0.2;

        let normalized_access = (access_count as f32).ln_1p() / 10.0; // Log normalization
        
        access_weight * normalized_access +
        recency_weight * recency_score +
        content_weight * content_relevance +
        attention_weight * attention_score
    }

    /// Find memories similar to a given memory
    pub async fn find_similar_memories(
        &self,
        target_memory: &[f32],
        candidate_memories: &[(String, Vec<f32>)],
        similarity_threshold: f32,
    ) -> Vec<(String, f32)> {
        let mut similar_memories = Vec::new();

        for (memory_id, memory_vector) in candidate_memories {
            let similarity = AttentionUtils::cosine_similarity(target_memory, memory_vector);
            
            if similarity >= similarity_threshold {
                similar_memories.push((memory_id.clone(), similarity));
            }
        }

        // Sort by similarity (descending)
        similar_memories.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        similar_memories
    }

    /// Calculate attention distribution entropy
    pub fn calculate_attention_entropy(&self) -> f32 {
        if self.attention_scores.is_empty() {
            return 0.0;
        }

        let total_score: f32 = self.attention_scores.values().sum();
        if total_score == 0.0 {
            return 0.0;
        }

        let mut entropy = 0.0;
        for &score in self.attention_scores.values() {
            if score > 0.0 {
                let probability = score / total_score;
                entropy -= probability * probability.log2();
            }
        }

        entropy
    }

    /// Normalize attention scores to sum to 1.0
    pub fn normalize_attention_scores(&mut self) {
        let total_score: f32 = self.attention_scores.values().sum();
        
        if total_score > 0.0 {
            for score in self.attention_scores.values_mut() {
                *score /= total_score;
            }
        }
    }

    /// Get attention statistics
    pub fn get_attention_statistics(&self) -> AttentionStatistics {
        if self.attention_scores.is_empty() {
            return AttentionStatistics::default();
        }

        let scores: Vec<f32> = self.attention_scores.values().copied().collect();
        let count = scores.len();
        let sum: f32 = scores.iter().sum();
        let mean = sum / count as f32;
        
        let variance: f32 = scores.iter()
            .map(|&score| (score - mean).powi(2))
            .sum::<f32>() / count as f32;
        let std_dev = variance.sqrt();

        let min_score = scores.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max_score = scores.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));

        AttentionStatistics {
            count,
            mean,
            std_dev,
            min_score,
            max_score,
            entropy: self.calculate_attention_entropy(),
        }
    }

    /// Apply attention boosting to high-performing memories
    pub fn boost_high_performers(&mut self, boost_threshold: f32, boost_factor: f32) {
        for score in self.attention_scores.values_mut() {
            if *score >= boost_threshold {
                *score *= boost_factor;
            }
        }
    }

    /// Apply attention dampening to low-performing memories
    pub fn dampen_low_performers(&mut self, dampen_threshold: f32, dampen_factor: f32) {
        for score in self.attention_scores.values_mut() {
            if *score <= dampen_threshold {
                *score *= dampen_factor;
            }
        }
    }

    /// Calculate memory clustering based on attention patterns
    pub fn calculate_attention_clusters(&self, num_clusters: usize) -> Vec<AttentionCluster> {
        if self.attention_scores.is_empty() || num_clusters == 0 {
            return Vec::new();
        }

        let mut clusters = Vec::with_capacity(num_clusters);
        let scores: Vec<_> = self.attention_scores.iter().collect();
        
        // Simple k-means-like clustering based on attention scores
        let min_score = scores.iter().map(|(_, score)| **score).fold(f32::INFINITY, f32::min);
        let max_score = scores.iter().map(|(_, score)| **score).fold(f32::NEG_INFINITY, f32::max);
        
        if max_score <= min_score {
            return clusters;
        }

        let score_range = max_score - min_score;
        let cluster_width = score_range / num_clusters as f32;

        for i in 0..num_clusters {
            let cluster_min = min_score + i as f32 * cluster_width;
            let cluster_max = if i == num_clusters - 1 {
                max_score + f32::EPSILON
            } else {
                min_score + (i + 1) as f32 * cluster_width
            };

            let cluster_memories: Vec<_> = scores.iter()
                .filter(|(_, score)| **score >= cluster_min && **score < cluster_max)
                .map(|(id, score)| ((*id).clone(), **score))
                .collect();

            if !cluster_memories.is_empty() {
                let cluster_center = cluster_memories.iter()
                    .map(|(_, score)| score)
                    .sum::<f32>() / cluster_memories.len() as f32;

                clusters.push(AttentionCluster {
                    id: i,
                    center_score: cluster_center,
                    memories: cluster_memories,
                    size: cluster_memories.len(),
                });
            }
        }

        clusters
    }

    /// Calculate attention focus score (how concentrated attention is)
    pub fn calculate_focus_score(&self) -> f32 {
        if self.attention_scores.len() <= 1 {
            return 1.0; // Perfect focus with 0 or 1 items
        }

        let statistics = self.get_attention_statistics();
        
        // Focus is inversely related to entropy and standard deviation
        let entropy_factor = 1.0 / (1.0 + statistics.entropy);
        let variance_factor = 1.0 / (1.0 + statistics.std_dev);
        
        (entropy_factor + variance_factor) / 2.0
    }

    /// Get memories above attention threshold
    pub fn get_memories_above_threshold(&self, threshold: f32) -> Vec<(String, f32)> {
        self.attention_scores.iter()
            .filter(|(_, score)| **score > threshold)
            .map(|(id, score)| (id.clone(), **score))
            .collect()
    }

    /// Calculate relative attention ranks
    pub fn calculate_attention_ranks(&self) -> Vec<(String, usize)> {
        let mut sorted_scores: Vec<_> = self.attention_scores.iter().collect();
        sorted_scores.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        sorted_scores.into_iter()
            .enumerate()
            .map(|(rank, (id, _))| (id.clone(), rank + 1))
            .collect()
    }

    /// Apply exponential smoothing to attention scores
    pub fn apply_exponential_smoothing(&mut self, alpha: f32, new_scores: &[(String, f32)]) {
        if alpha <= 0.0 || alpha > 1.0 {
            return; // Invalid smoothing factor
        }

        for (memory_id, new_score) in new_scores {
            let current_score = self.attention_scores.get(memory_id).copied().unwrap_or(0.0);
            let smoothed_score = alpha * new_score + (1.0 - alpha) * current_score;
            self.attention_scores.insert(memory_id.clone(), smoothed_score);
        }
    }
}

/// Attention statistics
#[derive(Debug, Clone)]
pub struct AttentionStatistics {
    pub count: usize,
    pub mean: f32,
    pub std_dev: f32,
    pub min_score: f32,
    pub max_score: f32,
    pub entropy: f32,
}

impl Default for AttentionStatistics {
    fn default() -> Self {
        Self {
            count: 0,
            mean: 0.0,
            std_dev: 0.0,
            min_score: 0.0,
            max_score: 0.0,
            entropy: 0.0,
        }
    }
}

impl AttentionStatistics {
    /// Check if attention distribution is healthy
    pub fn is_healthy(&self) -> bool {
        self.count > 0 
            && self.mean > 0.0 
            && self.std_dev >= 0.0 
            && self.entropy >= 0.0
    }

    /// Get coefficient of variation (std_dev / mean)
    pub fn coefficient_of_variation(&self) -> f32 {
        if self.mean > 0.0 {
            self.std_dev / self.mean
        } else {
            0.0
        }
    }

    /// Get score range
    pub fn score_range(&self) -> f32 {
        self.max_score - self.min_score
    }
}

/// Attention cluster for grouping similar attention patterns
#[derive(Debug, Clone)]
pub struct AttentionCluster {
    pub id: usize,
    pub center_score: f32,
    pub memories: Vec<(String, f32)>,
    pub size: usize,
}

impl AttentionCluster {
    /// Get cluster density (size relative to total)
    pub fn density(&self, total_memories: usize) -> f32 {
        if total_memories > 0 {
            self.size as f32 / total_memories as f32
        } else {
            0.0
        }
    }

    /// Get cluster coherence (how tightly grouped the scores are)
    pub fn coherence(&self) -> f32 {
        if self.memories.len() <= 1 {
            return 1.0;
        }

        let mean_score = self.center_score;
        let variance: f32 = self.memories.iter()
            .map(|(_, score)| (score - mean_score).powi(2))
            .sum::<f32>() / self.memories.len() as f32;

        1.0 / (1.0 + variance.sqrt())
    }

    /// Check if cluster is significant
    pub fn is_significant(&self, min_size: usize, min_coherence: f32) -> bool {
        self.size >= min_size && self.coherence() >= min_coherence
    }
}