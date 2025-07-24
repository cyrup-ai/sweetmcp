//! Neighborhood analysis for quantum entanglement networks
//!
//! This module provides blazing-fast neighborhood analysis with zero allocation
//! optimizations for quantum entanglement local network structure examination.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

use crate::cognitive::{
    quantum::EntanglementGraph,
    types::CognitiveError,
};
use super::super::super::node_state::QuantumMCTSNode;

/// Neighborhood analysis results for a specific node
#[derive(Debug, Clone)]
pub struct NeighborhoodAnalysis {
    /// Number of direct neighbors
    pub neighbor_count: usize,
    /// Average entanglement strength with neighbors
    pub average_strength: f64,
    /// Average quality of neighbor nodes
    pub average_quality: f64,
    /// Ratio of coherent vs incoherent neighbors
    pub coherence_ratio: f64,
    /// Ratio of terminal vs non-terminal neighbors
    pub terminal_ratio: f64,
    /// Distribution of entanglement strengths
    pub strength_distribution: Vec<f64>,
    /// Distribution of neighbor quality scores
    pub quality_distribution: Vec<f64>,
}

impl NeighborhoodAnalysis {
    /// Create empty neighborhood analysis
    #[inline]
    pub fn empty() -> Self {
        Self {
            neighbor_count: 0,
            average_strength: 0.0,
            average_quality: 0.0,
            coherence_ratio: 0.0,
            terminal_ratio: 0.0,
            strength_distribution: Vec::new(),
            quality_distribution: Vec::new(),
        }
    }

    /// Check if node is well connected to its neighborhood
    #[inline]
    pub fn is_well_connected(&self) -> bool {
        self.neighbor_count >= 3 && self.average_strength >= 0.5
    }

    /// Check if neighborhood has high quality
    #[inline]
    pub fn has_high_quality(&self) -> bool {
        self.average_quality >= 0.7 && self.coherence_ratio >= 0.6
    }

    /// Calculate neighborhood score (0.0 to 1.0)
    #[inline]
    pub fn neighborhood_score(&self) -> f64 {
        if self.neighbor_count == 0 {
            return 0.0;
        }

        let connectivity_score = (self.neighbor_count as f64 / 5.0).min(1.0); // Optimal around 5 neighbors
        let strength_score = self.average_strength;
        let quality_score = self.average_quality;
        let coherence_score = self.coherence_ratio;

        // Weighted combination
        (connectivity_score * 0.25 + 
         strength_score * 0.35 + 
         quality_score * 0.25 + 
         coherence_score * 0.15).max(0.0).min(1.0)
    }

    /// Get neighborhood health status
    #[inline]
    pub fn health_status(&self) -> NeighborhoodHealthStatus {
        let score = self.neighborhood_score();
        match score {
            s if s >= 0.8 => NeighborhoodHealthStatus::Excellent,
            s if s >= 0.6 => NeighborhoodHealthStatus::Good,
            s if s >= 0.4 => NeighborhoodHealthStatus::Fair,
            s if s >= 0.2 => NeighborhoodHealthStatus::Poor,
            _ => NeighborhoodHealthStatus::Critical,
        }
    }

    /// Get recommendations for neighborhood improvement
    #[inline]
    pub fn improvement_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();

        if self.neighbor_count < 2 {
            recommendations.push("Too few neighbors - create more entanglements".to_string());
        } else if self.neighbor_count > 10 {
            recommendations.push("Too many neighbors - consider pruning weak entanglements".to_string());
        }

        if self.average_strength < 0.4 {
            recommendations.push("Low average strength - strengthen existing entanglements".to_string());
        }

        if self.average_quality < 0.5 {
            recommendations.push("Low neighbor quality - connect to higher quality nodes".to_string());
        }

        if self.coherence_ratio < 0.4 {
            recommendations.push("Low coherence ratio - improve quantum coherence with neighbors".to_string());
        }

        if self.terminal_ratio > 0.8 {
            recommendations.push("Too many terminal neighbors - connect to more active nodes".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("Neighborhood appears well-balanced".to_string());
        }

        recommendations
    }

    /// Calculate neighborhood diversity score
    #[inline]
    pub fn diversity_score(&self) -> f64 {
        if self.strength_distribution.len() < 2 {
            return 0.0;
        }

        // Calculate coefficient of variation for strength distribution
        let mean = self.average_strength;
        if mean == 0.0 {
            return 0.0;
        }

        let variance = self.strength_distribution.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / self.strength_distribution.len() as f64;
        let std_dev = variance.sqrt();
        let cv = std_dev / mean;

        // Normalize to 0-1 range (higher CV = more diversity)
        (cv / 2.0).min(1.0)
    }
}

/// Neighborhood analyzer for quantum entanglement networks
pub struct NeighborhoodAnalyzer;

impl NeighborhoodAnalyzer {
    /// Analyze neighborhood for a specific node
    #[inline]
    pub async fn analyze_neighborhood(
        graph: &EntanglementGraph,
        nodes: &HashMap<String, Arc<RwLock<QuantumMCTSNode>>>,
        node_id: &str,
    ) -> Result<NeighborhoodAnalysis, CognitiveError> {
        let entanglements = graph.get_entanglements(node_id).await?;
        
        if entanglements.is_empty() {
            return Ok(NeighborhoodAnalysis::empty());
        }

        let neighbor_count = entanglements.len();
        let mut strength_sum = 0.0;
        let mut quality_sum = 0.0;
        let mut coherent_count = 0;
        let mut terminal_count = 0;
        let mut strength_distribution = Vec::new();
        let mut quality_distribution = Vec::new();

        for entanglement in &entanglements {
            let neighbor_id = &entanglement.target_node;
            strength_sum += entanglement.strength;
            strength_distribution.push(entanglement.strength);

            if let Some(neighbor_node) = nodes.get(neighbor_id) {
                let neighbor = neighbor_node.read().await;
                let quality = neighbor.state.performance_score();
                quality_sum += quality;
                quality_distribution.push(quality);

                if neighbor.state.is_coherent() {
                    coherent_count += 1;
                }

                if neighbor.is_terminal {
                    terminal_count += 1;
                }
            }
        }

        let average_strength = strength_sum / neighbor_count as f64;
        let average_quality = quality_sum / neighbor_count as f64;
        let coherence_ratio = coherent_count as f64 / neighbor_count as f64;
        let terminal_ratio = terminal_count as f64 / neighbor_count as f64;

        Ok(NeighborhoodAnalysis {
            neighbor_count,
            average_strength,
            average_quality,
            coherence_ratio,
            terminal_ratio,
            strength_distribution,
            quality_distribution,
        })
    }

    /// Analyze neighborhoods for all nodes
    #[inline]
    pub async fn analyze_all_neighborhoods(
        graph: &EntanglementGraph,
        nodes: &HashMap<String, Arc<RwLock<QuantumMCTSNode>>>,
    ) -> Result<HashMap<String, NeighborhoodAnalysis>, CognitiveError> {
        let mut analyses = HashMap::new();

        for node_id in nodes.keys() {
            let analysis = Self::analyze_neighborhood(graph, nodes, node_id).await?;
            analyses.insert(node_id.clone(), analysis);
        }

        Ok(analyses)
    }

    /// Find nodes with poor neighborhoods
    #[inline]
    pub async fn find_poor_neighborhoods(
        graph: &EntanglementGraph,
        nodes: &HashMap<String, Arc<RwLock<QuantumMCTSNode>>>,
        score_threshold: f64,
    ) -> Result<Vec<PoorNeighborhoodNode>, CognitiveError> {
        let mut poor_neighborhoods = Vec::new();

        for node_id in nodes.keys() {
            let analysis = Self::analyze_neighborhood(graph, nodes, node_id).await?;
            let score = analysis.neighborhood_score();

            if score < score_threshold {
                poor_neighborhoods.push(PoorNeighborhoodNode {
                    node_id: node_id.clone(),
                    neighborhood_score: score,
                    neighbor_count: analysis.neighbor_count,
                    average_strength: analysis.average_strength,
                    average_quality: analysis.average_quality,
                    health_status: analysis.health_status(),
                    recommendations: analysis.improvement_recommendations(),
                });
            }
        }

        // Sort by score (worst first)
        poor_neighborhoods.sort_by(|a, b| {
            a.neighborhood_score.partial_cmp(&b.neighborhood_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(poor_neighborhoods)
    }

    /// Calculate neighborhood statistics across the network
    #[inline]
    pub async fn calculate_neighborhood_statistics(
        graph: &EntanglementGraph,
        nodes: &HashMap<String, Arc<RwLock<QuantumMCTSNode>>>,
    ) -> Result<NeighborhoodStatistics, CognitiveError> {
        let analyses = Self::analyze_all_neighborhoods(graph, nodes).await?;
        
        if analyses.is_empty() {
            return Ok(NeighborhoodStatistics::default());
        }

        let mut scores = Vec::new();
        let mut neighbor_counts = Vec::new();
        let mut strength_averages = Vec::new();
        let mut quality_averages = Vec::new();

        for analysis in analyses.values() {
            scores.push(analysis.neighborhood_score());
            neighbor_counts.push(analysis.neighbor_count);
            strength_averages.push(analysis.average_strength);
            quality_averages.push(analysis.average_quality);
        }

        // Calculate statistics
        let score_mean = scores.iter().sum::<f64>() / scores.len() as f64;
        let neighbor_mean = neighbor_counts.iter().sum::<usize>() as f64 / neighbor_counts.len() as f64;
        let strength_mean = strength_averages.iter().sum::<f64>() / strength_averages.len() as f64;
        let quality_mean = quality_averages.iter().sum::<f64>() / quality_averages.len() as f64;

        scores.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let score_median = if scores.len() % 2 == 0 {
            (scores[scores.len() / 2 - 1] + scores[scores.len() / 2]) / 2.0
        } else {
            scores[scores.len() / 2]
        };

        // Count health categories
        let excellent_count = analyses.values().filter(|a| matches!(a.health_status(), NeighborhoodHealthStatus::Excellent)).count();
        let good_count = analyses.values().filter(|a| matches!(a.health_status(), NeighborhoodHealthStatus::Good)).count();
        let fair_count = analyses.values().filter(|a| matches!(a.health_status(), NeighborhoodHealthStatus::Fair)).count();
        let poor_count = analyses.values().filter(|a| matches!(a.health_status(), NeighborhoodHealthStatus::Poor)).count();
        let critical_count = analyses.values().filter(|a| matches!(a.health_status(), NeighborhoodHealthStatus::Critical)).count();

        Ok(NeighborhoodStatistics {
            total_nodes: analyses.len(),
            average_neighborhood_score: score_mean,
            median_neighborhood_score: score_median,
            average_neighbor_count: neighbor_mean,
            average_strength: strength_mean,
            average_quality: quality_mean,
            excellent_count,
            good_count,
            fair_count,
            poor_count,
            critical_count,
        })
    }

    /// Generate network-wide neighborhood recommendations
    #[inline]
    pub async fn generate_network_recommendations(
        graph: &EntanglementGraph,
        nodes: &HashMap<String, Arc<RwLock<QuantumMCTSNode>>>,
    ) -> Result<Vec<NetworkRecommendation>, CognitiveError> {
        let statistics = Self::calculate_neighborhood_statistics(graph, nodes).await?;
        let mut recommendations = Vec::new();

        // Global recommendations based on statistics
        if statistics.poor_count + statistics.critical_count > statistics.total_nodes / 3 {
            recommendations.push(NetworkRecommendation {
                priority: RecommendationPriority::High,
                category: RecommendationCategory::GlobalImprovement,
                description: "Many nodes have poor neighborhood quality".to_string(),
                action: "Systematic neighborhood improvement across the network".to_string(),
                affected_nodes: statistics.poor_count + statistics.critical_count,
            });
        }

        if statistics.average_neighbor_count < 2.0 {
            recommendations.push(NetworkRecommendation {
                priority: RecommendationPriority::High,
                category: RecommendationCategory::Connectivity,
                description: "Network has low average connectivity".to_string(),
                action: "Create more entanglements to improve connectivity".to_string(),
                affected_nodes: statistics.total_nodes,
            });
        }

        if statistics.average_strength < 0.4 {
            recommendations.push(NetworkRecommendation {
                priority: RecommendationPriority::Medium,
                category: RecommendationCategory::Strength,
                description: "Low average entanglement strength across neighborhoods".to_string(),
                action: "Focus on strengthening existing entanglements".to_string(),
                affected_nodes: statistics.total_nodes,
            });
        }

        if statistics.excellent_count == 0 {
            recommendations.push(NetworkRecommendation {
                priority: RecommendationPriority::Medium,
                category: RecommendationCategory::Excellence,
                description: "No nodes have excellent neighborhoods".to_string(),
                action: "Create high-quality neighborhood hubs".to_string(),
                affected_nodes: 0,
            });
        }

        Ok(recommendations)
    }
}

/// Node with poor neighborhood quality
#[derive(Debug, Clone)]
pub struct PoorNeighborhoodNode {
    pub node_id: String,
    pub neighborhood_score: f64,
    pub neighbor_count: usize,
    pub average_strength: f64,
    pub average_quality: f64,
    pub health_status: NeighborhoodHealthStatus,
    pub recommendations: Vec<String>,
}

/// Neighborhood health status categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeighborhoodHealthStatus {
    Excellent,
    Good,
    Fair,
    Poor,
    Critical,
}

impl std::fmt::Display for NeighborhoodHealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NeighborhoodHealthStatus::Excellent => write!(f, "Excellent"),
            NeighborhoodHealthStatus::Good => write!(f, "Good"),
            NeighborhoodHealthStatus::Fair => write!(f, "Fair"),
            NeighborhoodHealthStatus::Poor => write!(f, "Poor"),
            NeighborhoodHealthStatus::Critical => write!(f, "Critical"),
        }
    }
}

/// Network-wide neighborhood statistics
#[derive(Debug, Clone)]
pub struct NeighborhoodStatistics {
    pub total_nodes: usize,
    pub average_neighborhood_score: f64,
    pub median_neighborhood_score: f64,
    pub average_neighbor_count: f64,
    pub average_strength: f64,
    pub average_quality: f64,
    pub excellent_count: usize,
    pub good_count: usize,
    pub fair_count: usize,
    pub poor_count: usize,
    pub critical_count: usize,
}

impl Default for NeighborhoodStatistics {
    #[inline]
    fn default() -> Self {
        Self {
            total_nodes: 0,
            average_neighborhood_score: 0.0,
            median_neighborhood_score: 0.0,
            average_neighbor_count: 0.0,
            average_strength: 0.0,
            average_quality: 0.0,
            excellent_count: 0,
            good_count: 0,
            fair_count: 0,
            poor_count: 0,
            critical_count: 0,
        }
    }
}

/// Network-wide neighborhood recommendation
#[derive(Debug, Clone)]
pub struct NetworkRecommendation {
    pub priority: RecommendationPriority,
    pub category: RecommendationCategory,
    pub description: String,
    pub action: String,
    pub affected_nodes: usize,
}

/// Recommendation priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Recommendation categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecommendationCategory {
    GlobalImprovement,
    Connectivity,
    Strength,
    Excellence,
    Pruning,
}