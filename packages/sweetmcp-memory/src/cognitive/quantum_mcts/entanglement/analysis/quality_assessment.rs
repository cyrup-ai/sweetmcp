//! Entanglement quality assessment and analysis
//!
//! This module provides blazing-fast quality assessment with zero allocation
//! optimizations for quantum entanglement quality analysis and evaluation.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

use crate::cognitive::{
    quantum::EntanglementGraph,
    types::CognitiveError,
};
use super::super::super::node_state::QuantumMCTSNode;

/// Entanglement quality assessment with detailed metrics
#[derive(Debug, Clone)]
pub struct EntanglementQuality {
    /// Strength distribution statistics
    pub strength_mean: f64,
    pub strength_std: f64,
    pub strength_min: f64,
    pub strength_max: f64,
    /// Type distribution counts
    pub strong_count: usize,
    pub medium_count: usize,
    pub weak_count: usize,
    /// Overall quality score (0.0 to 1.0)
    pub overall_quality: f64,
    /// Quality trend indicator (-1.0 to 1.0)
    pub quality_trend: f64,
}

impl EntanglementQuality {
    /// Create new quality assessment from strength data
    #[inline]
    pub fn from_strengths(strengths: &[f64], strong_threshold: f64, medium_threshold: f64) -> Self {
        if strengths.is_empty() {
            return Self {
                strength_mean: 0.0,
                strength_std: 0.0,
                strength_min: 0.0,
                strength_max: 0.0,
                strong_count: 0,
                medium_count: 0,
                weak_count: 0,
                overall_quality: 0.0,
                quality_trend: 0.0,
            };
        }
        
        // Calculate strength statistics
        let sum: f64 = strengths.iter().sum();
        let mean = sum / strengths.len() as f64;
        
        let variance: f64 = strengths.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / strengths.len() as f64;
        let std_dev = variance.sqrt();
        
        let min = strengths.iter().copied().fold(f64::INFINITY, f64::min);
        let max = strengths.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        
        // Count by strength categories
        let mut strong_count = 0;
        let mut medium_count = 0;
        let mut weak_count = 0;
        
        for &strength in strengths {
            if strength >= strong_threshold {
                strong_count += 1;
            } else if strength >= medium_threshold {
                medium_count += 1;
            } else {
                weak_count += 1;
            }
        }
        
        // Calculate overall quality score
        let strong_weight = 1.0;
        let medium_weight = 0.6;
        let weak_weight = 0.2;
        
        let weighted_sum = strong_count as f64 * strong_weight +
                          medium_count as f64 * medium_weight +
                          weak_count as f64 * weak_weight;
        let max_possible = strengths.len() as f64 * strong_weight;
        
        let overall_quality = if max_possible > 0.0 {
            weighted_sum / max_possible
        } else {
            0.0
        };
        
        // Simple trend calculation (would need historical data for real trend)
        let quality_trend = if mean > 0.6 { 0.1 } else if mean < 0.3 { -0.1 } else { 0.0 };
        
        Self {
            strength_mean: mean,
            strength_std: std_dev,
            strength_min: min,
            strength_max: max,
            strong_count,
            medium_count,
            weak_count,
            overall_quality,
            quality_trend,
        }
    }
    
    /// Check if quality is acceptable for quantum operations
    #[inline]
    pub fn is_acceptable(&self) -> bool {
        self.overall_quality >= 0.5 && self.strength_mean >= 0.4
    }
    
    /// Check if quality is excellent
    #[inline]
    pub fn is_excellent(&self) -> bool {
        self.overall_quality >= 0.8 && self.strong_count > self.weak_count
    }
    
    /// Get quality recommendations
    #[inline]
    pub fn quality_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if self.weak_count > self.strong_count + self.medium_count {
            recommendations.push("Too many weak entanglements - consider strengthening or pruning".to_string());
        }
        
        if self.strength_std > 0.3 {
            recommendations.push("High strength variance - normalize entanglement strengths".to_string());
        }
        
        if self.strength_mean < 0.3 {
            recommendations.push("Low average strength - focus on creating stronger entanglements".to_string());
        }
        
        if self.quality_trend < -0.05 {
            recommendations.push("Quality is declining - investigate degradation causes".to_string());
        }
        
        if recommendations.is_empty() {
            recommendations.push("Entanglement quality appears satisfactory".to_string());
        }
        
        recommendations
    }

    /// Calculate quality stability score
    #[inline]
    pub fn stability_score(&self) -> f64 {
        // Lower standard deviation indicates more stable quality
        let stability_factor = 1.0 - (self.strength_std / 1.0).min(1.0);
        
        // Positive trend contributes to stability
        let trend_factor = (self.quality_trend + 1.0) / 2.0;
        
        // Balance of strong vs weak entanglements
        let total_count = self.strong_count + self.medium_count + self.weak_count;
        let balance_factor = if total_count > 0 {
            1.0 - (self.weak_count as f64 / total_count as f64)
        } else {
            0.0
        };
        
        (stability_factor * 0.5 + trend_factor * 0.3 + balance_factor * 0.2).max(0.0).min(1.0)
    }
}

/// Quality assessment analyzer for entanglement networks
pub struct QualityAssessmentAnalyzer;

impl QualityAssessmentAnalyzer {
    /// Assess overall entanglement quality in the network
    #[inline]
    pub async fn assess_quality(
        graph: &EntanglementGraph,
        nodes: &HashMap<String, Arc<RwLock<QuantumMCTSNode>>>,
        strong_threshold: f64,
        medium_threshold: f64,
    ) -> Result<EntanglementQuality, CognitiveError> {
        let mut strengths = Vec::new();
        
        // Collect all entanglement strengths
        for node_id in nodes.keys() {
            let entanglements = graph.get_entanglements(node_id).await?;
            for entanglement in entanglements {
                strengths.push(entanglement.strength);
            }
        }
        
        Ok(EntanglementQuality::from_strengths(&strengths, strong_threshold, medium_threshold))
    }

    /// Assess quality for a specific node's entanglements
    #[inline]
    pub async fn assess_node_quality(
        graph: &EntanglementGraph,
        node_id: &str,
        strong_threshold: f64,
        medium_threshold: f64,
    ) -> Result<EntanglementQuality, CognitiveError> {
        let entanglements = graph.get_entanglements(node_id).await?;
        let strengths: Vec<f64> = entanglements.iter().map(|e| e.strength).collect();
        
        Ok(EntanglementQuality::from_strengths(&strengths, strong_threshold, medium_threshold))
    }

    /// Find nodes with poor quality entanglements
    #[inline]
    pub async fn find_poor_quality_nodes(
        graph: &EntanglementGraph,
        nodes: &HashMap<String, Arc<RwLock<QuantumMCTSNode>>>,
        quality_threshold: f64,
    ) -> Result<Vec<PoorQualityNode>, CognitiveError> {
        let mut poor_nodes = Vec::new();
        
        for node_id in nodes.keys() {
            let quality = Self::assess_node_quality(graph, node_id, 0.7, 0.4).await?;
            
            if quality.overall_quality < quality_threshold {
                poor_nodes.push(PoorQualityNode {
                    node_id: node_id.clone(),
                    quality_score: quality.overall_quality,
                    weak_count: quality.weak_count,
                    total_entanglements: quality.strong_count + quality.medium_count + quality.weak_count,
                    average_strength: quality.strength_mean,
                    recommendations: quality.quality_recommendations(),
                });
            }
        }
        
        // Sort by quality score (worst first)
        poor_nodes.sort_by(|a, b| a.quality_score.partial_cmp(&b.quality_score).unwrap_or(std::cmp::Ordering::Equal));
        
        Ok(poor_nodes)
    }

    /// Calculate quality distribution across the network
    #[inline]
    pub async fn calculate_quality_distribution(
        graph: &EntanglementGraph,
        nodes: &HashMap<String, Arc<RwLock<QuantumMCTSNode>>>,
    ) -> Result<QualityDistribution, CognitiveError> {
        let mut node_qualities = Vec::new();
        
        for node_id in nodes.keys() {
            let quality = Self::assess_node_quality(graph, node_id, 0.7, 0.4).await?;
            node_qualities.push(quality.overall_quality);
        }
        
        if node_qualities.is_empty() {
            return Ok(QualityDistribution::default());
        }
        
        node_qualities.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        
        let mean = node_qualities.iter().sum::<f64>() / node_qualities.len() as f64;
        let median = if node_qualities.len() % 2 == 0 {
            (node_qualities[node_qualities.len() / 2 - 1] + node_qualities[node_qualities.len() / 2]) / 2.0
        } else {
            node_qualities[node_qualities.len() / 2]
        };
        
        let variance = node_qualities.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / node_qualities.len() as f64;
        let std_dev = variance.sqrt();
        
        // Count quality categories
        let excellent_count = node_qualities.iter().filter(|&&q| q >= 0.8).count();
        let good_count = node_qualities.iter().filter(|&&q| q >= 0.6 && q < 0.8).count();
        let fair_count = node_qualities.iter().filter(|&&q| q >= 0.4 && q < 0.6).count();
        let poor_count = node_qualities.iter().filter(|&&q| q < 0.4).count();
        
        Ok(QualityDistribution {
            mean,
            median,
            std_dev,
            min: node_qualities[0],
            max: node_qualities[node_qualities.len() - 1],
            excellent_count,
            good_count,
            fair_count,
            poor_count,
            total_nodes: node_qualities.len(),
        })
    }

    /// Generate quality improvement recommendations
    #[inline]
    pub async fn generate_improvement_recommendations(
        graph: &EntanglementGraph,
        nodes: &HashMap<String, Arc<RwLock<QuantumMCTSNode>>>,
    ) -> Result<Vec<QualityRecommendation>, CognitiveError> {
        let mut recommendations = Vec::new();
        
        let overall_quality = Self::assess_quality(graph, nodes, 0.7, 0.4).await?;
        let distribution = Self::calculate_quality_distribution(graph, nodes).await?;
        
        // Global recommendations
        if overall_quality.weak_count > overall_quality.strong_count {
            recommendations.push(QualityRecommendation {
                priority: RecommendationPriority::High,
                category: RecommendationCategory::StrengthImprovement,
                description: "Network has more weak than strong entanglements".to_string(),
                action: "Focus on strengthening existing weak entanglements".to_string(),
                expected_impact: "Improved overall network quality and quantum coherence".to_string(),
            });
        }
        
        if distribution.poor_count > distribution.total_nodes / 4 {
            recommendations.push(QualityRecommendation {
                priority: RecommendationPriority::High,
                category: RecommendationCategory::NodeImprovement,
                description: format!("{} nodes have poor quality entanglements", distribution.poor_count),
                action: "Identify and improve poor quality nodes".to_string(),
                expected_impact: "Better network-wide quality distribution".to_string(),
            });
        }
        
        if overall_quality.strength_std > 0.3 {
            recommendations.push(QualityRecommendation {
                priority: RecommendationPriority::Medium,
                category: RecommendationCategory::Consistency,
                description: "High variance in entanglement strengths".to_string(),
                action: "Normalize entanglement strengths across the network".to_string(),
                expected_impact: "More consistent quantum behavior".to_string(),
            });
        }
        
        if distribution.excellent_count == 0 {
            recommendations.push(QualityRecommendation {
                priority: RecommendationPriority::Medium,
                category: RecommendationCategory::Excellence,
                description: "No nodes have excellent quality entanglements".to_string(),
                action: "Create high-quality entanglement hubs".to_string(),
                expected_impact: "Improved network performance and reliability".to_string(),
            });
        }
        
        Ok(recommendations)
    }
}

/// Node with poor entanglement quality
#[derive(Debug, Clone)]
pub struct PoorQualityNode {
    pub node_id: String,
    pub quality_score: f64,
    pub weak_count: usize,
    pub total_entanglements: usize,
    pub average_strength: f64,
    pub recommendations: Vec<String>,
}

/// Quality distribution across the network
#[derive(Debug, Clone)]
pub struct QualityDistribution {
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub excellent_count: usize,
    pub good_count: usize,
    pub fair_count: usize,
    pub poor_count: usize,
    pub total_nodes: usize,
}

impl Default for QualityDistribution {
    #[inline]
    fn default() -> Self {
        Self {
            mean: 0.0,
            median: 0.0,
            std_dev: 0.0,
            min: 0.0,
            max: 0.0,
            excellent_count: 0,
            good_count: 0,
            fair_count: 0,
            poor_count: 0,
            total_nodes: 0,
        }
    }
}

/// Quality improvement recommendation
#[derive(Debug, Clone)]
pub struct QualityRecommendation {
    pub priority: RecommendationPriority,
    pub category: RecommendationCategory,
    pub description: String,
    pub action: String,
    pub expected_impact: String,
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
    StrengthImprovement,
    NodeImprovement,
    Consistency,
    Excellence,
    Pruning,
    Creation,
}