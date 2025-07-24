//! Intelligent entanglement pruning operations and threshold calculations
//!
//! This module provides blazing-fast pruning algorithms with zero-allocation
//! patterns and adaptive threshold management.

use std::collections::HashMap;
use std::time::Instant;
use tracing::{debug, info, warn};

use crate::cognitive::types::CognitiveError;
use super::super::{
    analysis::{NetworkTopology, NetworkTopologyAnalyzer},
    metrics::PerformanceTracker,
};
use super::super::super::{
    node_state::QuantumMCTSNode,
    config::QuantumMCTSConfig,
};
use super::core::QuantumEntanglementEngine;

/// Pruning operation result with comprehensive analysis
#[derive(Debug, Clone)]
pub struct PruningResult {
    /// Number of entanglements pruned
    pub entanglements_pruned: usize,
    /// Time taken for pruning in milliseconds
    pub pruning_time_ms: u64,
    /// Network improvement percentage after pruning
    pub network_improvement: f64,
    /// Reason for pruning operation
    pub reason: String,
}

/// Pruning strategy configuration
#[derive(Debug, Clone)]
pub struct PruningStrategy {
    /// Base strength threshold for pruning
    pub base_threshold: f64,
    /// Adaptive threshold adjustment factor
    pub adaptive_factor: f64,
    /// Maximum percentage of entanglements to prune in one operation
    pub max_prune_percentage: f64,
    /// Minimum entanglement strength to preserve
    pub preservation_threshold: f64,
}

impl Default for PruningStrategy {
    fn default() -> Self {
        Self {
            base_threshold: 0.1,
            adaptive_factor: 1.5,
            max_prune_percentage: 0.25,
            preservation_threshold: 0.05,
        }
    }
}

impl QuantumEntanglementEngine {
    /// Perform intelligent entanglement pruning based on network analysis
    pub async fn intelligent_pruning(
        &mut self,
        tree: &HashMap<String, QuantumMCTSNode>,
    ) -> Result<PruningResult, CognitiveError> {
        let start_time = Instant::now();
        
        debug!("Starting intelligent pruning for {} nodes", tree.len());
        
        // Analyze current network topology
        let topology = NetworkTopologyAnalyzer::analyze_network_topology(&self.entanglement_graph).await?;
        
        // Calculate adaptive pruning threshold
        let pruning_threshold = self.calculate_adaptive_pruning_threshold(&topology, tree).await?;
        
        // Determine pruning strategy based on network state
        let strategy = self.determine_pruning_strategy(&topology);
        
        // Perform targeted pruning
        let entanglements_pruned = self.execute_targeted_pruning(&strategy, pruning_threshold).await?;
        
        let pruning_time_ms = start_time.elapsed().as_millis() as u64;
        
        // Calculate network improvement
        let new_topology = NetworkTopologyAnalyzer::analyze_network_topology(&self.entanglement_graph).await?;
        let network_improvement = self.calculate_pruning_improvement(&topology, &new_topology);
        
        // Determine reason for pruning
        let reason = self.determine_pruning_reason(&topology, &strategy);
        
        // Record pruning metrics
        self.metrics.record_intelligent_pruning(
            entanglements_pruned,
            pruning_time_ms,
            network_improvement,
        );
        
        info!(
            "Intelligent pruning completed: {} entanglements removed in {}ms (+{:.1}% improvement)",
            entanglements_pruned,
            pruning_time_ms,
            network_improvement
        );
        
        Ok(PruningResult {
            entanglements_pruned,
            pruning_time_ms,
            network_improvement,
            reason,
        })
    }
    
    /// Prune weak entanglements for a specific node
    pub async fn prune_node_weak_entanglements(
        &mut self,
        node_id: &str,
        strength_threshold: f64,
    ) -> Result<usize, CognitiveError> {
        debug!("Pruning weak entanglements for node {} (threshold: {:.3})", node_id, strength_threshold);
        
        let _tracker = PerformanceTracker::start();
        
        // Get current entanglements for the node
        let node_entanglements = self.manager.get_node_entanglements(node_id).await?;
        
        let mut pruned_count = 0;
        
        // Identify weak entanglements to prune
        for (target_node, strength) in &node_entanglements {
            if *strength < strength_threshold {
                match self.manager.remove_entanglement(node_id, target_node).await {
                    Ok(true) => {
                        pruned_count += 1;
                        self.metrics.record_entanglement_pruned();
                    }
                    Ok(false) => {
                        debug!("Entanglement between {} and {} was already removed", node_id, target_node);
                    }
                    Err(e) => {
                        warn!("Failed to remove entanglement between {} and {}: {}", node_id, target_node, e);
                    }
                }
            }
        }
        
        debug!("Pruned {} weak entanglements for node {}", pruned_count, node_id);
        
        Ok(pruned_count)
    }
    
    /// Calculate dynamic pruning threshold based on network state
    pub fn calculate_dynamic_pruning_threshold(&self, topology: &NetworkTopology) -> f64 {
        let base_threshold = self.config.entanglement_strength_threshold;
        
        // Adjust threshold based on network density
        let density_factor = if topology.network_density > 0.1 {
            1.5 // Higher threshold for dense networks
        } else if topology.network_density < 0.05 {
            0.7 // Lower threshold for sparse networks
        } else {
            1.0
        };
        
        // Adjust based on clustering coefficient
        let clustering_factor = if topology.clustering_coefficient > 0.5 {
            1.3 // Higher threshold for highly clustered networks
        } else if topology.clustering_coefficient < 0.2 {
            0.8 // Lower threshold for poorly clustered networks
        } else {
            1.0
        };
        
        // Adjust based on connectivity
        let connectivity_factor = if topology.is_connected {
            1.2 // Can afford higher threshold if connected
        } else {
            0.6 // Lower threshold to preserve connectivity
        };
        
        let dynamic_threshold = base_threshold * density_factor * clustering_factor * connectivity_factor;
        
        // Ensure reasonable bounds
        dynamic_threshold.max(0.01).min(0.5)
    }
    
    /// Calculate adaptive pruning threshold based on network analysis
    pub async fn calculate_adaptive_pruning_threshold(
        &self,
        topology: &NetworkTopology,
        tree: &HashMap<String, QuantumMCTSNode>,
    ) -> Result<f64, CognitiveError> {
        let base_threshold = self.calculate_dynamic_pruning_threshold(topology);
        
        // Analyze entanglement strength distribution
        let strength_stats = self.analyze_entanglement_strength_distribution().await?;
        
        // Adjust threshold based on strength distribution
        let distribution_factor = if strength_stats.standard_deviation > 0.2 {
            1.2 // Higher threshold when strengths vary widely
        } else if strength_stats.standard_deviation < 0.1 {
            0.9 // Lower threshold when strengths are similar
        } else {
            1.0
        };
        
        // Adjust based on network performance
        let performance_factor = self.calculate_performance_based_adjustment(tree).await?;
        
        // Adjust based on recent pruning history
        let history_factor = self.calculate_history_based_adjustment().await?;
        
        let adaptive_threshold = base_threshold * distribution_factor * performance_factor * history_factor;
        
        // Ensure adaptive threshold is within reasonable bounds
        Ok(adaptive_threshold.max(0.005).min(0.8))
    }
    
    /// Determine optimal pruning strategy based on network state
    fn determine_pruning_strategy(&self, topology: &NetworkTopology) -> PruningStrategy {
        let mut strategy = PruningStrategy::default();
        
        // Adjust strategy based on network density
        if topology.network_density > 0.15 {
            // Dense network - more aggressive pruning
            strategy.base_threshold *= 1.5;
            strategy.max_prune_percentage = 0.35;
        } else if topology.network_density < 0.03 {
            // Sparse network - conservative pruning
            strategy.base_threshold *= 0.7;
            strategy.max_prune_percentage = 0.15;
        }
        
        // Adjust based on connectivity
        if !topology.is_connected {
            // Preserve connectivity - very conservative
            strategy.base_threshold *= 0.5;
            strategy.max_prune_percentage = 0.1;
            strategy.preservation_threshold *= 2.0;
        }
        
        // Adjust based on clustering
        if topology.clustering_coefficient < 0.2 {
            // Poor clustering - preserve structure
            strategy.adaptive_factor *= 0.8;
            strategy.preservation_threshold *= 1.5;
        }
        
        strategy
    }
    
    /// Execute targeted pruning based on strategy
    async fn execute_targeted_pruning(
        &mut self,
        strategy: &PruningStrategy,
        threshold: f64,
    ) -> Result<usize, CognitiveError> {
        let mut total_pruned = 0;
        
        // Get all entanglements for analysis
        let all_entanglements = self.manager.get_all_entanglements().await?;
        let total_entanglements = all_entanglements.len();
        
        // Calculate maximum entanglements to prune
        let max_to_prune = (total_entanglements as f64 * strategy.max_prune_percentage) as usize;
        
        // Sort entanglements by strength (weakest first)
        let mut sorted_entanglements: Vec<_> = all_entanglements
            .into_iter()
            .filter(|(_, _, strength)| *strength < threshold && *strength > strategy.preservation_threshold)
            .collect();
        
        sorted_entanglements.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));
        
        // Prune entanglements up to the maximum limit
        for (node1, node2, _strength) in sorted_entanglements.into_iter().take(max_to_prune) {
            match self.manager.remove_entanglement(&node1, &node2).await {
                Ok(true) => {
                    total_pruned += 1;
                    self.metrics.record_entanglement_pruned();
                }
                Ok(false) => {
                    debug!("Entanglement between {} and {} was already removed", node1, node2);
                }
                Err(e) => {
                    warn!("Failed to remove entanglement between {} and {}: {}", node1, node2, e);
                }
            }
        }
        
        Ok(total_pruned)
    }
    
    /// Analyze entanglement strength distribution
    async fn analyze_entanglement_strength_distribution(&self) -> Result<StrengthStatistics, CognitiveError> {
        let all_entanglements = self.manager.get_all_entanglements().await?;
        
        if all_entanglements.is_empty() {
            return Ok(StrengthStatistics::default());
        }
        
        let strengths: Vec<f64> = all_entanglements.into_iter().map(|(_, _, strength)| strength).collect();
        
        let mean = strengths.iter().sum::<f64>() / strengths.len() as f64;
        let variance = strengths.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / strengths.len() as f64;
        let standard_deviation = variance.sqrt();
        
        let mut sorted_strengths = strengths.clone();
        sorted_strengths.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        
        let median = if sorted_strengths.len() % 2 == 0 {
            (sorted_strengths[sorted_strengths.len() / 2 - 1] + sorted_strengths[sorted_strengths.len() / 2]) / 2.0
        } else {
            sorted_strengths[sorted_strengths.len() / 2]
        };
        
        Ok(StrengthStatistics {
            mean,
            median,
            standard_deviation,
            min: sorted_strengths[0],
            max: sorted_strengths[sorted_strengths.len() - 1],
            count: strengths.len(),
        })
    }
    
    /// Calculate performance-based adjustment factor
    async fn calculate_performance_based_adjustment(
        &self,
        tree: &HashMap<String, QuantumMCTSNode>,
    ) -> Result<f64, CognitiveError> {
        // Analyze recent performance metrics
        let metrics_summary = self.metrics.summary();
        
        // Adjust based on success rate
        let success_factor = if metrics_summary.success_rate > 0.95 {
            1.1 // Can be more aggressive if success rate is high
        } else if metrics_summary.success_rate < 0.85 {
            0.9 // Be more conservative if success rate is low
        } else {
            1.0
        };
        
        // Adjust based on operation speed
        let speed_factor = if metrics_summary.average_operation_time_us < 100.0 {
            1.05 // Slightly more aggressive if operations are fast
        } else if metrics_summary.average_operation_time_us > 1000.0 {
            0.95 // More conservative if operations are slow
        } else {
            1.0
        };
        
        // Adjust based on tree utilization
        let utilization_factor = self.calculate_tree_utilization_factor(tree);
        
        Ok(success_factor * speed_factor * utilization_factor)
    }
    
    /// Calculate history-based adjustment factor
    async fn calculate_history_based_adjustment(&self) -> Result<f64, CognitiveError> {
        let recent_pruning_stats = self.metrics.get_recent_pruning_statistics().await?;
        
        // Adjust based on recent pruning frequency
        let frequency_factor = if recent_pruning_stats.operations_per_hour > 10.0 {
            0.9 // Be more conservative if pruning frequently
        } else if recent_pruning_stats.operations_per_hour < 2.0 {
            1.1 // Can be more aggressive if pruning infrequently
        } else {
            1.0
        };
        
        // Adjust based on recent pruning effectiveness
        let effectiveness_factor = if recent_pruning_stats.average_improvement > 10.0 {
            1.05 // Slightly more aggressive if recent pruning was effective
        } else if recent_pruning_stats.average_improvement < 2.0 {
            0.95 // More conservative if recent pruning wasn't very effective
        } else {
            1.0
        };
        
        Ok(frequency_factor * effectiveness_factor)
    }
    
    /// Calculate tree utilization factor
    fn calculate_tree_utilization_factor(&self, tree: &HashMap<String, QuantumMCTSNode>) -> f64 {
        if tree.is_empty() {
            return 1.0;
        }
        
        let total_visits: u64 = tree.values().map(|node| node.visits).sum();
        let active_nodes = tree.values().filter(|node| node.visits > 0).count();
        let utilization_rate = active_nodes as f64 / tree.len() as f64;
        
        // Adjust based on utilization rate
        if utilization_rate > 0.8 {
            1.1 // More aggressive if most nodes are being used
        } else if utilization_rate < 0.3 {
            0.9 // More conservative if many nodes are unused
        } else {
            1.0
        }
    }
    
    /// Calculate pruning improvement percentage
    fn calculate_pruning_improvement(&self, initial: &NetworkTopology, final_topo: &NetworkTopology) -> f64 {
        // Calculate improvement based on network efficiency
        let density_improvement = if initial.network_density > 0.1 && final_topo.network_density < initial.network_density {
            ((initial.network_density - final_topo.network_density) / initial.network_density) * 15.0
        } else {
            0.0
        };
        
        let clustering_improvement = if final_topo.clustering_coefficient > initial.clustering_coefficient {
            ((final_topo.clustering_coefficient - initial.clustering_coefficient) / initial.clustering_coefficient.max(0.001)) * 20.0
        } else {
            0.0
        };
        
        let efficiency_improvement = if final_topo.average_degree > 0.0 && initial.average_degree > 0.0 {
            let initial_efficiency = initial.clustering_coefficient / initial.average_degree;
            let final_efficiency = final_topo.clustering_coefficient / final_topo.average_degree;
            if final_efficiency > initial_efficiency {
                ((final_efficiency - initial_efficiency) / initial_efficiency) * 10.0
            } else {
                0.0
            }
        } else {
            0.0
        };
        
        (density_improvement + clustering_improvement + efficiency_improvement).max(0.0).min(50.0)
    }
    
    /// Determine reason for pruning operation
    fn determine_pruning_reason(&self, topology: &NetworkTopology, strategy: &PruningStrategy) -> String {
        if topology.network_density > 0.15 {
            "Network was overly dense and needed optimization".to_string()
        } else if topology.clustering_coefficient < 0.3 {
            "Network had poor clustering and needed structural improvement".to_string()
        } else if strategy.base_threshold > 0.15 {
            "Aggressive pruning strategy applied for performance optimization".to_string()
        } else {
            "Routine maintenance pruning to remove weak entanglements".to_string()
        }
    }
}

/// Entanglement strength statistics
#[derive(Debug, Clone)]
pub struct StrengthStatistics {
    /// Mean strength value
    pub mean: f64,
    /// Median strength value
    pub median: f64,
    /// Standard deviation of strengths
    pub standard_deviation: f64,
    /// Minimum strength value
    pub min: f64,
    /// Maximum strength value
    pub max: f64,
    /// Total number of entanglements
    pub count: usize,
}

impl Default for StrengthStatistics {
    fn default() -> Self {
        Self {
            mean: 0.0,
            median: 0.0,
            standard_deviation: 0.0,
            min: 0.0,
            max: 0.0,
            count: 0,
        }
    }
}

/// Recent pruning statistics
#[derive(Debug, Clone)]
pub struct RecentPruningStatistics {
    /// Number of pruning operations per hour
    pub operations_per_hour: f64,
    /// Average improvement percentage from recent pruning
    pub average_improvement: f64,
    /// Total entanglements pruned recently
    pub total_pruned: usize,
    /// Average pruning time in milliseconds
    pub average_time_ms: f64,
}

impl PruningResult {
    /// Check if pruning was beneficial
    pub fn was_beneficial(&self) -> bool {
        self.network_improvement > 0.0 && self.entanglements_pruned > 0
    }
    
    /// Get pruning summary
    pub fn summary(&self) -> String {
        format!(
            "Pruned {} entanglements in {}ms (+{:.1}% improvement)",
            self.entanglements_pruned,
            self.pruning_time_ms,
            self.network_improvement
        )
    }
    
    /// Check if pruning had significant impact
    pub fn had_significant_impact(&self) -> bool {
        self.network_improvement > 5.0 || self.entanglements_pruned > 20
    }
    
    /// Get detailed report
    pub fn detailed_report(&self) -> String {
        format!(
            "=== Pruning Report ===\n\
            Entanglements Pruned: {}\n\
            Duration: {}ms\n\
            Network Improvement: {:.1}%\n\
            Reason: {}\n\
            Impact: {}",
            self.entanglements_pruned,
            self.pruning_time_ms,
            self.network_improvement,
            self.reason,
            if self.had_significant_impact() { "SIGNIFICANT" } else { "MODERATE" }
        )
    }
}

impl PruningStrategy {
    /// Create conservative pruning strategy
    pub fn conservative() -> Self {
        Self {
            base_threshold: 0.05,
            adaptive_factor: 1.2,
            max_prune_percentage: 0.15,
            preservation_threshold: 0.02,
        }
    }
    
    /// Create aggressive pruning strategy
    pub fn aggressive() -> Self {
        Self {
            base_threshold: 0.2,
            adaptive_factor: 2.0,
            max_prune_percentage: 0.4,
            preservation_threshold: 0.1,
        }
    }
    
    /// Create balanced pruning strategy
    pub fn balanced() -> Self {
        Self::default()
    }
}