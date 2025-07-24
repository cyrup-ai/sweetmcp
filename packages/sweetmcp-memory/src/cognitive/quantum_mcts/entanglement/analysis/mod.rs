//! Quantum entanglement analysis module coordination
//!
//! This module provides comprehensive quantum entanglement network analysis with blazing-fast
//! performance and zero allocation optimizations, integrating all analysis submodules.

pub mod network_topology;
pub mod quality_assessment;
pub mod neighborhood_analysis;
pub mod bottleneck_detection;

// Re-export key types and functions for ergonomic access
pub use network_topology::{
    NetworkTopology, NetworkTopologyAnalyzer, NetworkHealthStatus,
};

pub use quality_assessment::{
    EntanglementQuality, QualityAssessmentAnalyzer, PoorQualityNode,
    QualityDistribution, QualityRecommendation, RecommendationPriority, RecommendationCategory,
};

pub use neighborhood_analysis::{
    NeighborhoodAnalysis, NeighborhoodAnalyzer, PoorNeighborhoodNode,
    NeighborhoodHealthStatus, NeighborhoodStatistics, NetworkRecommendation,
};

pub use bottleneck_detection::{
    BottleneckDetector, NetworkBottleneck, BottleneckType, BottleneckImpact,
    ImpactType, RecoveryDifficulty, BottleneckResolutionPlan, ResolutionAction, ActionType,
};

// Common imports for all submodules
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

use crate::cognitive::{
    quantum::EntanglementGraph,
    types::CognitiveError,
};
use super::super::node_state::QuantumMCTSNode;

/// Comprehensive entanglement network analyzer facade
pub struct EntanglementNetworkAnalyzer;

impl EntanglementNetworkAnalyzer {
    /// Perform complete network analysis with all metrics
    #[inline]
    pub async fn analyze_complete(
        graph: &EntanglementGraph,
        nodes: &HashMap<String, Arc<RwLock<QuantumMCTSNode>>>,
    ) -> Result<CompleteNetworkAnalysis, CognitiveError> {
        let topology = NetworkTopologyAnalyzer::analyze_topology(graph, nodes).await?;
        let quality = QualityAssessmentAnalyzer::assess_quality(graph, nodes, 0.7, 0.4).await?;
        let neighborhood_stats = NeighborhoodAnalyzer::calculate_neighborhood_statistics(graph, nodes).await?;
        let bottlenecks = BottleneckDetector::identify_bottlenecks(graph, nodes).await?;
        let resolution_plan = BottleneckDetector::generate_resolution_plan(graph, nodes, &bottlenecks).await?;

        Ok(CompleteNetworkAnalysis {
            topology,
            quality,
            neighborhood_stats,
            bottlenecks,
            resolution_plan,
        })
    }

    /// Quick analysis for performance monitoring
    #[inline]
    pub async fn analyze_quick(
        graph: &EntanglementGraph,
        nodes: &HashMap<String, Arc<RwLock<QuantumMCTSNode>>>,
    ) -> Result<QuickNetworkAnalysis, CognitiveError> {
        let topology = NetworkTopologyAnalyzer::analyze_topology(graph, nodes).await?;
        let quality = QualityAssessmentAnalyzer::assess_quality(graph, nodes, 0.7, 0.4).await?;

        Ok(QuickNetworkAnalysis {
            total_nodes: topology.total_nodes,
            total_entanglements: topology.total_entanglements,
            network_density: topology.network_density,
            is_connected: topology.is_connected,
            overall_quality: quality.overall_quality,
            health_status: topology.health_status(),
        })
    }

    /// Analyze network health and generate recommendations
    #[inline]
    pub async fn analyze_health(
        graph: &EntanglementGraph,
        nodes: &HashMap<String, Arc<RwLock<QuantumMCTSNode>>>,
    ) -> Result<NetworkHealthAnalysis, CognitiveError> {
        let topology = NetworkTopologyAnalyzer::analyze_topology(graph, nodes).await?;
        let quality = QualityAssessmentAnalyzer::assess_quality(graph, nodes, 0.7, 0.4).await?;
        let bottlenecks = BottleneckDetector::identify_bottlenecks(graph, nodes).await?;
        
        let mut recommendations = Vec::new();
        recommendations.extend(topology.optimization_recommendations());
        recommendations.extend(quality.quality_recommendations());
        
        let quality_recommendations = QualityAssessmentAnalyzer::generate_improvement_recommendations(graph, nodes).await?;
        for rec in quality_recommendations {
            recommendations.push(format!("{}: {}", rec.category as u8, rec.description));
        }

        let overall_health_score = (topology.efficiency_score() + quality.overall_quality) / 2.0;

        Ok(NetworkHealthAnalysis {
            overall_health_score,
            topology_health: topology.health_status(),
            quality_health: if quality.is_excellent() { 
                NetworkHealthStatus::Excellent 
            } else if quality.is_acceptable() { 
                NetworkHealthStatus::Good 
            } else { 
                NetworkHealthStatus::Poor 
            },
            critical_issues: bottlenecks.iter().filter(|b| b.severity > 5.0).count(),
            recommendations,
        })
    }

    /// Find problematic nodes across all analysis dimensions
    #[inline]
    pub async fn find_problematic_nodes(
        graph: &EntanglementGraph,
        nodes: &HashMap<String, Arc<RwLock<QuantumMCTSNode>>>,
    ) -> Result<ProblematicNodesAnalysis, CognitiveError> {
        let poor_quality_nodes = QualityAssessmentAnalyzer::find_poor_quality_nodes(graph, nodes, 0.4).await?;
        let poor_neighborhood_nodes = NeighborhoodAnalyzer::find_poor_neighborhoods(graph, nodes, 0.4).await?;
        let bottlenecks = BottleneckDetector::identify_bottlenecks(graph, nodes).await?;

        // Combine and deduplicate problematic nodes
        let mut problematic_nodes = std::collections::HashMap::new();
        
        for node in poor_quality_nodes {
            problematic_nodes.entry(node.node_id.clone())
                .or_insert_with(|| ProblematicNode {
                    node_id: node.node_id.clone(),
                    issues: Vec::new(),
                    severity_score: 0.0,
                    recommendations: Vec::new(),
                })
                .issues.push(format!("Poor quality: {:.2}", node.quality_score));
        }

        for node in poor_neighborhood_nodes {
            problematic_nodes.entry(node.node_id.clone())
                .or_insert_with(|| ProblematicNode {
                    node_id: node.node_id.clone(),
                    issues: Vec::new(),
                    severity_score: 0.0,
                    recommendations: Vec::new(),
                })
                .issues.push(format!("Poor neighborhood: {:.2}", node.neighborhood_score));
        }

        for bottleneck in bottlenecks {
            if bottleneck.node_id != "network" {
                problematic_nodes.entry(bottleneck.node_id.clone())
                    .or_insert_with(|| ProblematicNode {
                        node_id: bottleneck.node_id.clone(),
                        issues: Vec::new(),
                        severity_score: 0.0,
                        recommendations: Vec::new(),
                    })
                    .issues.push(format!("Bottleneck: {}", bottleneck.description));
            }
        }

        // Calculate severity scores and sort
        let mut nodes_list: Vec<_> = problematic_nodes.into_values().collect();
        for node in &mut nodes_list {
            node.severity_score = node.issues.len() as f64;
        }
        nodes_list.sort_by(|a, b| b.severity_score.partial_cmp(&a.severity_score).unwrap_or(std::cmp::Ordering::Equal));

        Ok(ProblematicNodesAnalysis {
            total_problematic_nodes: nodes_list.len(),
            critical_nodes: nodes_list.iter().filter(|n| n.severity_score >= 3.0).count(),
            nodes: nodes_list,
        })
    }
}

/// Complete network analysis result containing all metrics
#[derive(Debug, Clone)]
pub struct CompleteNetworkAnalysis {
    pub topology: NetworkTopology,
    pub quality: EntanglementQuality,
    pub neighborhood_stats: NeighborhoodStatistics,
    pub bottlenecks: Vec<NetworkBottleneck>,
    pub resolution_plan: BottleneckResolutionPlan,
}

impl CompleteNetworkAnalysis {
    /// Generate comprehensive summary report
    #[inline]
    pub fn summary_report(&self) -> String {
        format!(
            "=== COMPLETE ENTANGLEMENT NETWORK ANALYSIS ===\n\n\
             Network Topology:\n\
             - Nodes: {}, Entanglements: {}\n\
             - Density: {:.3}, Connected: {}\n\
             - Efficiency: {:.3}, Health: {}\n\n\
             Quality Assessment:\n\
             - Overall Quality: {:.3}\n\
             - Strong: {}, Medium: {}, Weak: {}\n\
             - Average Strength: {:.3}\n\n\
             Neighborhood Statistics:\n\
             - Average Score: {:.3}\n\
             - Excellent: {}, Good: {}, Poor: {}\n\n\
             Bottlenecks: {} total ({} critical)\n\
             Resolution Plan: {:.1}% estimated improvement",
            self.topology.total_nodes, self.topology.total_entanglements,
            self.topology.network_density, self.topology.is_connected,
            self.topology.efficiency_score(), self.topology.health_status(),
            self.quality.overall_quality,
            self.quality.strong_count, self.quality.medium_count, self.quality.weak_count,
            self.quality.strength_mean,
            self.neighborhood_stats.average_neighborhood_score,
            self.neighborhood_stats.excellent_count, self.neighborhood_stats.good_count, self.neighborhood_stats.poor_count,
            self.bottlenecks.len(), self.resolution_plan.critical_bottlenecks,
            self.resolution_plan.estimated_improvement * 100.0
        )
    }
}

/// Quick network analysis result for performance monitoring
#[derive(Debug, Clone)]
pub struct QuickNetworkAnalysis {
    pub total_nodes: usize,
    pub total_entanglements: usize,
    pub network_density: f64,
    pub is_connected: bool,
    pub overall_quality: f64,
    pub health_status: NetworkHealthStatus,
}

impl QuickNetworkAnalysis {
    /// Generate quick summary
    #[inline]
    pub fn summary(&self) -> String {
        format!(
            "Nodes: {}, Entanglements: {}, Density: {:.3}, Connected: {}, Quality: {:.3}, Health: {}",
            self.total_nodes, self.total_entanglements, self.network_density,
            self.is_connected, self.overall_quality, self.health_status
        )
    }
}

/// Network health analysis result
#[derive(Debug, Clone)]
pub struct NetworkHealthAnalysis {
    pub overall_health_score: f64,
    pub topology_health: NetworkHealthStatus,
    pub quality_health: NetworkHealthStatus,
    pub critical_issues: usize,
    pub recommendations: Vec<String>,
}

/// Problematic nodes analysis result
#[derive(Debug, Clone)]
pub struct ProblematicNodesAnalysis {
    pub total_problematic_nodes: usize,
    pub critical_nodes: usize,
    pub nodes: Vec<ProblematicNode>,
}

/// Problematic node information
#[derive(Debug, Clone)]
pub struct ProblematicNode {
    pub node_id: String,
    pub issues: Vec<String>,
    pub severity_score: f64,
    pub recommendations: Vec<String>,
}

/// Convenience macros for common analysis patterns
#[macro_export]
macro_rules! analyze_network {
    ($graph:expr, $nodes:expr) => {
        EntanglementNetworkAnalyzer::analyze_complete($graph, $nodes).await
    };
    ($graph:expr, $nodes:expr, quick) => {
        EntanglementNetworkAnalyzer::analyze_quick($graph, $nodes).await
    };
    ($graph:expr, $nodes:expr, health) => {
        EntanglementNetworkAnalyzer::analyze_health($graph, $nodes).await
    };
}

#[macro_export]
macro_rules! find_issues {
    ($graph:expr, $nodes:expr, quality) => {
        QualityAssessmentAnalyzer::find_poor_quality_nodes($graph, $nodes, 0.4).await
    };
    ($graph:expr, $nodes:expr, neighborhoods) => {
        NeighborhoodAnalyzer::find_poor_neighborhoods($graph, $nodes, 0.4).await
    };
    ($graph:expr, $nodes:expr, bottlenecks) => {
        BottleneckDetector::identify_bottlenecks($graph, $nodes).await
    };
}