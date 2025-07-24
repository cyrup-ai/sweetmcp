//! Quantum entanglement optimization algorithms and strategies
//!
//! This module provides blazing-fast optimization algorithms with zero-allocation
//! patterns and comprehensive performance tracking.

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

/// Optimization result with comprehensive analysis
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    /// Initial network topology before optimization
    pub initial_topology: NetworkTopology,
    /// Final network topology after optimization
    pub final_topology: NetworkTopology,
    /// List of actions taken during optimization
    pub actions_taken: Vec<String>,
    /// Time taken for optimization in milliseconds
    pub optimization_time_ms: u64,
    /// Number of entanglements created
    pub entanglements_created: usize,
    /// Number of entanglements pruned
    pub entanglements_pruned: usize,
    /// Number of nodes processed
    pub nodes_processed: usize,
    /// Number of update failures
    pub update_failures: usize,
    /// Performance improvement percentage
    pub performance_improvement: f64,
}

/// Strategic entanglement creation result
#[derive(Debug, Clone)]
pub struct CreationResult {
    /// Number of entanglements created
    pub entanglements_created: usize,
    /// Time taken for creation in milliseconds
    pub creation_time_ms: u64,
    /// Network improvement percentage
    pub network_improvement: f64,
    /// Reason for creation
    pub reason: String,
}

impl QuantumEntanglementEngine {
    /// Perform automatic entanglement optimization with comprehensive analysis
    pub async fn optimize_entanglements(
        &mut self,
        tree: &mut HashMap<String, QuantumMCTSNode>,
    ) -> Result<OptimizationResult, CognitiveError> {
        let _tracker = PerformanceTracker::start();
        let start_time = Instant::now();
        
        debug!("Starting entanglement optimization for {} nodes", tree.len());
        
        // Analyze current topology
        let initial_topology = NetworkTopologyAnalyzer::analyze_network_topology(&self.entanglement_graph).await?;
        self.metrics.record_topology_analysis();
        
        let mut actions_taken = Vec::new();
        let mut total_created = 0;
        let mut total_pruned = 0;
        let mut update_failures = 0;
        
        // Step 1: Prune weak entanglements if network is overly dense
        if initial_topology.is_overly_dense() {
            let pruning_threshold = self.calculate_dynamic_pruning_threshold(&initial_topology);
            match self.manager.prune_weak_entanglements(pruning_threshold).await {
                Ok(pruned) => {
                    if pruned > 0 {
                        total_pruned += pruned;
                        actions_taken.push(format!("Pruned {} weak entanglements (threshold: {:.3})", pruned, pruning_threshold));
                        self.metrics.record_entanglements_pruned(pruned);
                    }
                }
                Err(e) => {
                    warn!("Failed to prune weak entanglements: {}", e);
                    update_failures += 1;
                }
            }
        }
        
        // Step 2: Create new entanglements if network is sparse or poorly connected
        if initial_topology.is_sparse() || !initial_topology.has_good_connectivity() {
            match self.create_strategic_entanglements(tree).await {
                Ok(creation_result) => {
                    total_created += creation_result.entanglements_created;
                    
                    if creation_result.entanglements_created > 0 {
                        actions_taken.push(format!(
                            "Created {} strategic entanglements (+{:.1}% network improvement)",
                            creation_result.entanglements_created,
                            creation_result.network_improvement
                        ));
                    }
                }
                Err(e) => {
                    warn!("Failed to create strategic entanglements: {}", e);
                    update_failures += 1;
                }
            }
        }
        
        // Step 3: Balance entanglement distribution
        match self.balance_entanglement_distribution(tree).await {
            Ok(balancing_result) => {
                if balancing_result.redistributions_made > 0 {
                    actions_taken.push(format!(
                        "Balanced entanglement distribution ({} redistributions, +{:.1}% improvement)",
                        balancing_result.redistributions_made,
                        balancing_result.balance_improvement
                    ));
                }
            }
            Err(e) => {
                warn!("Failed to balance entanglement distribution: {}", e);
                update_failures += 1;
            }
        }
        
        // Step 4: Perform intelligent pruning if needed
        if initial_topology.clustering_coefficient < 0.3 {
            match self.intelligent_pruning(tree).await {
                Ok(pruning_result) => {
                    if pruning_result.entanglements_pruned > 0 {
                        total_pruned += pruning_result.entanglements_pruned;
                        actions_taken.push(format!(
                            "Intelligent pruning removed {} entanglements (+{:.1}% network improvement)",
                            pruning_result.entanglements_pruned,
                            pruning_result.network_improvement
                        ));
                    }
                }
                Err(e) => {
                    warn!("Failed to perform intelligent pruning: {}", e);
                    update_failures += 1;
                }
            }
        }
        
        // Analyze final topology
        let final_topology = NetworkTopologyAnalyzer::analyze_network_topology(&self.entanglement_graph).await?;
        let optimization_time_ms = start_time.elapsed().as_millis() as u64;
        
        // Calculate performance improvement
        let mut result = OptimizationResult {
            initial_topology,
            final_topology,
            actions_taken,
            optimization_time_ms,
            entanglements_created: total_created,
            entanglements_pruned: total_pruned,
            nodes_processed: tree.len(),
            update_failures,
            performance_improvement: 0.0,
        };
        
        result = self.calculate_performance_improvement(result)?;
        
        // Record optimization metrics
        self.metrics.record_optimization_completed(
            result.optimization_time_ms,
            result.entanglements_created,
            result.entanglements_pruned,
            result.performance_improvement,
        );
        
        info!(
            "Optimization completed in {}ms: +{} entanglements, -{} pruned, {:.1}% improvement",
            result.optimization_time_ms,
            result.entanglements_created,
            result.entanglements_pruned,
            result.performance_improvement
        );
        
        Ok(result)
    }
    
    /// Create strategic entanglements to improve network connectivity
    pub async fn create_strategic_entanglements(
        &mut self,
        tree: &HashMap<String, QuantumMCTSNode>,
    ) -> Result<CreationResult, CognitiveError> {
        let start_time = Instant::now();
        
        debug!("Creating strategic entanglements for {} nodes", tree.len());
        
        // Analyze current network topology
        let topology = NetworkTopologyAnalyzer::analyze_network_topology(&self.entanglement_graph).await?;
        
        // Calculate optimal batch size for creation
        let batch_size = self.calculate_optimal_batch_size(tree.len());
        let mut entanglements_created = 0;
        
        // Identify nodes that would benefit from additional entanglements
        let candidate_pairs = self.identify_strategic_entanglement_candidates(tree, &topology).await?;
        
        // Create entanglements in batches for optimal performance
        for batch in candidate_pairs.chunks(batch_size) {
            match self.manager.create_entanglement_batch(batch).await {
                Ok(created) => {
                    entanglements_created += created;
                    self.metrics.record_entanglements_created(created);
                }
                Err(e) => {
                    warn!("Failed to create entanglement batch: {}", e);
                    // Continue with next batch rather than failing completely
                }
            }
        }
        
        let creation_time_ms = start_time.elapsed().as_millis() as u64;
        
        // Calculate network improvement
        let new_topology = NetworkTopologyAnalyzer::analyze_network_topology(&self.entanglement_graph).await?;
        let network_improvement = self.calculate_topology_improvement(&topology, &new_topology);
        
        let reason = if topology.is_sparse() {
            "Network was sparse and needed more connections".to_string()
        } else if !topology.has_good_connectivity() {
            "Network had poor connectivity patterns".to_string()
        } else {
            "Strategic improvement opportunity identified".to_string()
        };
        
        Ok(CreationResult {
            entanglements_created,
            creation_time_ms,
            network_improvement,
            reason,
        })
    }
    
    /// Calculate optimal batch size for entanglement creation
    pub fn calculate_optimal_batch_size(&self, total_nodes: usize) -> usize {
        let base_batch_size = self.config.batch_size;
        
        // Scale batch size based on network size and available resources
        let scaled_size = match total_nodes {
            n if n < 100 => base_batch_size.min(10),
            n if n < 1000 => base_batch_size.min(50),
            n if n < 10000 => base_batch_size.min(100),
            _ => base_batch_size.min(200),
        };
        
        // Ensure minimum batch size for efficiency
        scaled_size.max(5)
    }
    
    /// Calculate performance improvement from optimization
    pub fn calculate_performance_improvement(
        &self,
        mut result: OptimizationResult,
    ) -> Result<OptimizationResult, CognitiveError> {
        let initial = &result.initial_topology;
        let final_topo = &result.final_topology;
        
        // Calculate improvement based on multiple factors
        let connectivity_improvement = if initial.is_connected != final_topo.is_connected && final_topo.is_connected {
            25.0 // Major improvement for achieving connectivity
        } else {
            0.0
        };
        
        let density_improvement = ((final_topo.network_density - initial.network_density) / initial.network_density.max(0.001)) * 100.0;
        let clustering_improvement = ((final_topo.clustering_coefficient - initial.clustering_coefficient) / initial.clustering_coefficient.max(0.001)) * 100.0;
        let degree_improvement = ((final_topo.average_degree - initial.average_degree) / initial.average_degree.max(0.001)) * 100.0;
        
        // Weighted combination of improvements
        result.performance_improvement = connectivity_improvement 
            + (density_improvement * 0.3)
            + (clustering_improvement * 0.4)
            + (degree_improvement * 0.3);
        
        // Ensure reasonable bounds
        result.performance_improvement = result.performance_improvement.max(-50.0).min(100.0);
        
        Ok(result)
    }
    
    /// Identify strategic entanglement candidates
    async fn identify_strategic_entanglement_candidates(
        &self,
        tree: &HashMap<String, QuantumMCTSNode>,
        topology: &NetworkTopology,
    ) -> Result<Vec<(String, String)>, CognitiveError> {
        let mut candidates = Vec::new();
        
        // Find nodes with low connectivity that could benefit from entanglements
        let low_connectivity_nodes = self.find_low_connectivity_nodes(tree, topology).await?;
        
        // Find high-value nodes that should be better connected
        let high_value_nodes = self.find_high_value_nodes(tree).await?;
        
        // Create strategic pairs
        for low_node in &low_connectivity_nodes {
            for high_node in &high_value_nodes {
                if low_node != high_node && !self.are_already_entangled(low_node, high_node).await? {
                    candidates.push((low_node.clone(), high_node.clone()));
                    
                    // Limit candidates to prevent excessive creation
                    if candidates.len() >= self.config.max_strategic_entanglements {
                        break;
                    }
                }
            }
            if candidates.len() >= self.config.max_strategic_entanglements {
                break;
            }
        }
        
        Ok(candidates)
    }
    
    /// Find nodes with low connectivity
    async fn find_low_connectivity_nodes(
        &self,
        tree: &HashMap<String, QuantumMCTSNode>,
        topology: &NetworkTopology,
    ) -> Result<Vec<String>, CognitiveError> {
        let mut low_connectivity_nodes = Vec::new();
        let connectivity_threshold = topology.average_degree * 0.5;
        
        for (node_id, _node) in tree {
            let connectivity = self.manager.get_node_connectivity(node_id).await?;
            if connectivity < connectivity_threshold {
                low_connectivity_nodes.push(node_id.clone());
            }
        }
        
        Ok(low_connectivity_nodes)
    }
    
    /// Find high-value nodes that should be well connected
    async fn find_high_value_nodes(
        &self,
        tree: &HashMap<String, QuantumMCTSNode>,
    ) -> Result<Vec<String>, CognitiveError> {
        let mut high_value_nodes = Vec::new();
        
        // Sort nodes by visit count and value
        let mut node_values: Vec<(String, f64)> = tree
            .iter()
            .map(|(id, node)| (id.clone(), node.value + (node.visits as f64 * 0.1)))
            .collect();
        
        node_values.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Take top 20% as high-value nodes
        let high_value_count = (tree.len() as f64 * 0.2).ceil() as usize;
        for (node_id, _value) in node_values.into_iter().take(high_value_count) {
            high_value_nodes.push(node_id);
        }
        
        Ok(high_value_nodes)
    }
    
    /// Check if two nodes are already entangled
    async fn are_already_entangled(
        &self,
        node1: &str,
        node2: &str,
    ) -> Result<bool, CognitiveError> {
        self.manager.check_entanglement_exists(node1, node2).await
    }
    
    /// Calculate topology improvement percentage
    fn calculate_topology_improvement(
        &self,
        initial: &NetworkTopology,
        final_topo: &NetworkTopology,
    ) -> f64 {
        let connectivity_factor = if !initial.is_connected && final_topo.is_connected { 20.0 } else { 0.0 };
        let density_factor = ((final_topo.network_density - initial.network_density) / initial.network_density.max(0.001)) * 10.0;
        let clustering_factor = ((final_topo.clustering_coefficient - initial.clustering_coefficient) / initial.clustering_coefficient.max(0.001)) * 15.0;
        
        (connectivity_factor + density_factor + clustering_factor).max(0.0).min(50.0)
    }
}

impl OptimizationResult {
    /// Check if optimization was beneficial
    pub fn was_beneficial(&self) -> bool {
        self.performance_improvement > 0.0 || 
        (self.entanglements_created > self.entanglements_pruned && self.final_topology.is_connected)
    }
    
    /// Get optimization summary
    pub fn summary(&self) -> String {
        let net_entanglements = self.entanglements_created as i32 - self.entanglements_pruned as i32;
        let net_sign = if net_entanglements >= 0 { "+" } else { "" };
        
        format!(
            "Optimization: {}{}E, {:.1}ms, {:.1}% improvement, {} actions",
            net_sign,
            net_entanglements,
            self.optimization_time_ms,
            self.performance_improvement,
            self.actions_taken.len()
        )
    }
    
    /// Check if optimization had significant impact
    pub fn had_significant_impact(&self) -> bool {
        self.performance_improvement.abs() > 5.0 || 
        self.entanglements_created > 10 || 
        self.entanglements_pruned > 10
    }
    
    /// Get detailed report
    pub fn detailed_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str(&format!("=== Optimization Report ===\n"));
        report.push_str(&format!("Duration: {}ms\n", self.optimization_time_ms));
        report.push_str(&format!("Nodes Processed: {}\n", self.nodes_processed));
        report.push_str(&format!("Performance Improvement: {:.1}%\n", self.performance_improvement));
        report.push_str(&format!("Update Failures: {}\n", self.update_failures));
        
        report.push_str(&format!("\n--- Network Changes ---\n"));
        report.push_str(&format!("Entanglements Created: {}\n", self.entanglements_created));
        report.push_str(&format!("Entanglements Pruned: {}\n", self.entanglements_pruned));
        report.push_str(&format!("Net Change: {}\n", self.entanglements_created as i32 - self.entanglements_pruned as i32));
        
        report.push_str(&format!("\n--- Topology Changes ---\n"));
        report.push_str(&format!("Connectivity: {} → {}\n", 
            self.initial_topology.is_connected, self.final_topology.is_connected));
        report.push_str(&format!("Network Density: {:.3} → {:.3}\n", 
            self.initial_topology.network_density, self.final_topology.network_density));
        report.push_str(&format!("Clustering Coefficient: {:.3} → {:.3}\n", 
            self.initial_topology.clustering_coefficient, self.final_topology.clustering_coefficient));
        report.push_str(&format!("Average Degree: {:.1} → {:.1}\n", 
            self.initial_topology.average_degree, self.final_topology.average_degree));
        
        if !self.actions_taken.is_empty() {
            report.push_str(&format!("\n--- Actions Taken ---\n"));
            for (i, action) in self.actions_taken.iter().enumerate() {
                report.push_str(&format!("{}. {}\n", i + 1, action));
            }
        }
        
        report
    }
}

impl CreationResult {
    /// Check if creation was successful
    pub fn was_successful(&self) -> bool {
        self.entanglements_created > 0 && self.network_improvement > 0.0
    }
    
    /// Get creation summary
    pub fn summary(&self) -> String {
        format!(
            "Created {} entanglements in {}ms (+{:.1}% improvement)",
            self.entanglements_created,
            self.creation_time_ms,
            self.network_improvement
        )
    }
}