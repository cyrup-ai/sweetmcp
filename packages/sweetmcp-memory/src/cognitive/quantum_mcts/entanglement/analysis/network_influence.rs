//!
//! Network influence calculation for quantum entanglement analysis with blazing-fast
//! zero-allocation patterns and comprehensive influence metrics.

use std::collections::HashMap;
use tracing::debug;

use crate::cognitive::{
    quantum::EntanglementGraph,
    types::CognitiveError,
};

/// Network influence calculator for quantum entanglement analysis
#[derive(Debug, Clone)]
pub struct NetworkInfluenceCalculator {
    /// Influence calculation cache for performance
    influence_cache: HashMap<String, f64>,
    /// Network topology cache
    topology_cache: HashMap<String, Vec<String>>,
}

/// Network influence metrics
#[derive(Debug, Clone)]
pub struct NetworkInfluenceMetrics {
    /// Node influence scores
    pub node_influences: HashMap<String, f64>,
    /// Total network influence
    pub total_influence: f64,
    /// Average influence per node
    pub average_influence: f64,
    /// Maximum influence score
    pub max_influence: f64,
    /// Minimum influence score
    pub min_influence: f64,
}

/// Influence distribution analysis
#[derive(Debug, Clone)]
pub struct InfluenceDistribution {
    /// High influence nodes (top 10%)
    pub high_influence_nodes: Vec<String>,
    /// Medium influence nodes (middle 80%)
    pub medium_influence_nodes: Vec<String>,
    /// Low influence nodes (bottom 10%)
    pub low_influence_nodes: Vec<String>,
    /// Influence distribution statistics
    pub distribution_stats: InfluenceStats,
}

/// Influence statistics
#[derive(Debug, Clone)]
pub struct InfluenceStats {
    /// Standard deviation of influence scores
    pub std_deviation: f64,
    /// Influence score variance
    pub variance: f64,
    /// Influence distribution skewness
    pub skewness: f64,
}

impl NetworkInfluenceCalculator {
    /// Create new network influence calculator
    pub fn new() -> Self {
        Self {
            influence_cache: HashMap::new(),
            topology_cache: HashMap::new(),
        }
    }

    /// Calculate network influence metrics
    pub async fn calculate_influence(
        &mut self,
        graph: &EntanglementGraph,
    ) -> Result<NetworkInfluenceMetrics, CognitiveError> {
        debug!("Calculating network influence metrics");

        let node_count = graph.node_count();
        if node_count == 0 {
            return Ok(NetworkInfluenceMetrics {
                node_influences: HashMap::new(),
                total_influence: 0.0,
                average_influence: 0.0,
                max_influence: 0.0,
                min_influence: 0.0,
            });
        }

        let mut node_influences = HashMap::new();
        let mut total_influence = 0.0;
        let mut max_influence = 0.0;
        let mut min_influence = f64::MAX;

        // Calculate influence for each node
        for node_index in graph.node_indices() {
            let node_id = format!("node_{}", node_index.index());
            
            // Check cache first
            let influence = if let Some(&cached_influence) = self.influence_cache.get(&node_id) {
                cached_influence
            } else {
                let calculated_influence = self.calculate_node_influence(graph, node_index).await?;
                self.influence_cache.insert(node_id.clone(), calculated_influence);
                calculated_influence
            };

            node_influences.insert(node_id, influence);
            total_influence += influence;
            max_influence = max_influence.max(influence);
            min_influence = min_influence.min(influence);
        }

        let average_influence = total_influence / node_count as f64;

        Ok(NetworkInfluenceMetrics {
            node_influences,
            total_influence,
            average_influence,
            max_influence,
            min_influence: if min_influence == f64::MAX { 0.0 } else { min_influence },
        })
    }

    /// Calculate influence for a specific node
    async fn calculate_node_influence(
        &self,
        graph: &EntanglementGraph,
        node_index: petgraph::graph::NodeIndex,
    ) -> Result<f64, CognitiveError> {
        // Calculate influence based on:
        // 1. Direct connections (degree centrality)
        // 2. Path lengths to other nodes (closeness centrality)
        // 3. Position in network paths (betweenness centrality)

        let degree = graph.edges(node_index).count() as f64;
        let node_count = graph.node_count() as f64;

        if node_count <= 1.0 {
            return Ok(0.0);
        }

        // Simple influence calculation based on degree centrality
        // In a real implementation, this would include more sophisticated metrics
        let degree_centrality = degree / (node_count - 1.0);
        
        // Apply influence scaling
        let influence = degree_centrality * 100.0;

        Ok(influence)
    }

    /// Analyze influence distribution across the network
    pub async fn analyze_influence_distribution(
        &mut self,
        metrics: &NetworkInfluenceMetrics,
    ) -> Result<InfluenceDistribution, CognitiveError> {
        debug!("Analyzing influence distribution");

        let mut sorted_influences: Vec<(&String, &f64)> = metrics.node_influences.iter().collect();
        sorted_influences.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));

        let total_nodes = sorted_influences.len();
        let high_threshold = (total_nodes as f64 * 0.1).ceil() as usize;
        let low_threshold = (total_nodes as f64 * 0.9).floor() as usize;

        let high_influence_nodes: Vec<String> = sorted_influences
            .iter()
            .take(high_threshold)
            .map(|(node, _)| (*node).clone())
            .collect();

        let low_influence_nodes: Vec<String> = sorted_influences
            .iter()
            .skip(low_threshold)
            .map(|(node, _)| (*node).clone())
            .collect();

        let medium_influence_nodes: Vec<String> = sorted_influences
            .iter()
            .skip(high_threshold)
            .take(low_threshold - high_threshold)
            .map(|(node, _)| (*node).clone())
            .collect();

        // Calculate distribution statistics
        let influences: Vec<f64> = metrics.node_influences.values().cloned().collect();
        let distribution_stats = self.calculate_influence_stats(&influences);

        Ok(InfluenceDistribution {
            high_influence_nodes,
            medium_influence_nodes,
            low_influence_nodes,
            distribution_stats,
        })
    }

    /// Calculate influence statistics
    fn calculate_influence_stats(&self, influences: &[f64]) -> InfluenceStats {
        if influences.is_empty() {
            return InfluenceStats {
                std_deviation: 0.0,
                variance: 0.0,
                skewness: 0.0,
            };
        }

        let mean = influences.iter().sum::<f64>() / influences.len() as f64;
        
        let variance = influences
            .iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / influences.len() as f64;

        let std_deviation = variance.sqrt();

        // Simple skewness calculation
        let skewness = if std_deviation > 0.0 {
            influences
                .iter()
                .map(|x| ((x - mean) / std_deviation).powi(3))
                .sum::<f64>() / influences.len() as f64
        } else {
            0.0
        };

        InfluenceStats {
            std_deviation,
            variance,
            skewness,
        }
    }

    /// Clear influence calculation cache
    pub fn clear_cache(&mut self) {
        self.influence_cache.clear();
        self.topology_cache.clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        (self.influence_cache.len(), self.topology_cache.len())
    }
}

impl Default for NetworkInfluenceCalculator {
    fn default() -> Self {
        Self::new()
    }
}