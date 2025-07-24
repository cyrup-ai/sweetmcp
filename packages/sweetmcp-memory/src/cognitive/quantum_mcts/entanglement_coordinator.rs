//! Comprehensive entanglement coordinator with all functionality
//!
//! This module provides the main EntanglementCoordinator struct that orchestrates
//! all entanglement operations with zero-allocation patterns and blazing-fast performance.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cognitive::{
    quantum::EntanglementGraph,
    types::CognitiveError,
};
use super::{
    node_state::QuantumMCTSNode,
    config::QuantumMCTSConfig,
};

// Import submodule types
use super::entanglement::analysis::{NetworkTopology, NetworkInfluenceCalculator, NetworkTopologyAnalyzer};
use super::entanglement::metrics::{EntanglementMetrics};
use super::entanglement::engine::{QuantumEntanglementEngine, OptimizationResult, PruningResult, NetworkHealthReport};

/// Comprehensive entanglement coordinator with all functionality
pub struct EntanglementCoordinator {
    /// High-level entanglement engine
    engine: QuantumEntanglementEngine,
    /// Metrics collector for periodic reporting
    metrics_collector: MetricsCollector,
}

impl EntanglementCoordinator {
    /// Create new entanglement coordinator
    pub fn new(
        config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
    ) -> Self {
        let engine = QuantumEntanglementEngine::new(config, entanglement_graph);
        let metrics_collector = MetricsCollector::new(std::time::Duration::from_secs(60));
        
        Self { engine, metrics_collector }
    }
    
    /// Create entanglement coordinator with custom reporting interval
    pub fn with_reporting_interval(
        config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
        reporting_interval: std::time::Duration,
    ) -> Self {
        let engine = QuantumEntanglementEngine::new(config, entanglement_graph);
        let metrics_collector = MetricsCollector::new(reporting_interval);
        
        Self { engine, metrics_collector }
    }
    
    /// Create entanglement between nodes
    pub async fn create_entanglement(
        &mut self,
        node_id: &str,
        tree: &HashMap<String, QuantumMCTSNode>,
    ) -> Result<Vec<String>, CognitiveError> {
        self.engine.manager().create_entanglement(node_id, tree).await
    }
    
    /// Remove entanglement between nodes
    pub async fn remove_entanglement(
        &mut self,
        node1_id: &str,
        node2_id: &str,
    ) -> Result<bool, CognitiveError> {
        self.engine.manager().remove_entanglement(node1_id, node2_id).await
    }
    
    /// Get entangled nodes
    pub async fn get_entangled_nodes(
        &self,
        node_id: &str,
    ) -> Result<Vec<(String, f64)>, CognitiveError> {
        self.engine.manager().get_entangled_nodes(node_id).await
    }
    
    /// Calculate network influence
    pub async fn calculate_network_influence(
        &self,
        node_id: &str,
        tree: &HashMap<String, QuantumMCTSNode>,
    ) -> Result<f64, CognitiveError> {
        NetworkInfluenceCalculator::calculate_network_influence(
            &self.engine.manager().entanglement_graph,
            node_id,
            tree,
        ).await
    }
    
    /// Optimize entanglements
    pub async fn optimize_entanglements(
        &mut self,
        tree: &mut HashMap<String, QuantumMCTSNode>,
    ) -> Result<OptimizationResult, CognitiveError> {
        let result = self.engine.optimize_entanglements(tree).await?;
        self.metrics_collector.maybe_report();
        Ok(result)
    }
    
    /// Prune weak entanglements intelligently
    pub async fn intelligent_pruning(
        &mut self,
        tree: &HashMap<String, QuantumMCTSNode>,
    ) -> Result<PruningResult, CognitiveError> {
        let result = self.engine.intelligent_pruning(tree).await?;
        self.metrics_collector.maybe_report();
        Ok(result)
    }
    
    /// Analyze network topology
    pub async fn analyze_network_topology(&self) -> Result<NetworkTopology, CognitiveError> {
        NetworkTopologyAnalyzer::analyze_network_topology(
            &self.engine.manager().entanglement_graph
        ).await
    }
    
    /// Perform network health check
    pub async fn health_check(&self) -> Result<NetworkHealthReport, CognitiveError> {
        self.engine.health_check().await
    }
    
    /// Batch create entanglements
    pub async fn batch_create_entanglements(
        &mut self,
        node_ids: &[String],
        tree: &HashMap<String, QuantumMCTSNode>,
    ) -> Result<HashMap<String, Vec<String>>, CognitiveError> {
        self.engine.manager().batch_create_entanglements(node_ids, tree).await
    }
    
    /// Update node entanglement states  
    pub async fn update_node_entanglements(
        &mut self,
        node_id: &str,
        tree: &mut HashMap<String, QuantumMCTSNode>,
    ) -> Result<(), CognitiveError> {
        self.engine.manager().update_node_entanglements(node_id, tree).await
    }
    
    /// Prune weak entanglements with threshold
    pub async fn prune_weak_entanglements(
        &mut self,
        minimum_strength: f64,
    ) -> Result<usize, CognitiveError> {
        self.engine.manager().prune_weak_entanglements(minimum_strength).await
    }
    
    /// Get metrics summary
    pub fn get_metrics_summary(&self) -> EntanglementMetricsSummary {
        self.engine.metrics().summary()
    }
    
    /// Get current metrics
    pub fn get_metrics(&self) -> &EntanglementMetrics {
        self.engine.metrics()
    }
    
    /// Reset metrics
    pub fn reset_metrics(&mut self) {
        self.engine.metrics().reset();
        self.metrics_collector.reset();
    }
    
    /// Get configuration
    pub fn get_config(&self) -> &QuantumMCTSConfig {
        self.engine.config()
    }
    
    /// Update configuration
    pub fn update_config(&mut self, new_config: QuantumMCTSConfig) {
        self.engine.update_config(new_config);
    }
    
    /// Get mutable reference to engine for advanced operations
    pub fn engine(&mut self) -> &mut QuantumEntanglementEngine {
        &mut self.engine
    }
    
    /// Comprehensive network analysis
    pub async fn comprehensive_analysis(
        &self,
        tree: &HashMap<String, QuantumMCTSNode>,
    ) -> Result<super::entanglement_analysis::ComprehensiveAnalysisReport, CognitiveError> {
        super::entanglement_analysis::ComprehensiveAnalysisReport::generate(
            &self.engine,
            tree,
        ).await
    }
    
    /// Get performance grade based on current metrics
    pub fn performance_grade(&self) -> char {
        let metrics = self.get_metrics();
        let success_rate = if metrics.operations_attempted() > 0 {
            metrics.operations_successful() as f64 / metrics.operations_attempted() as f64
        } else {
            1.0
        };
        
        let avg_latency = metrics.average_operation_latency();
        let latency_score = if avg_latency.as_millis() < 10 {
            1.0
        } else if avg_latency.as_millis() < 50 {
            0.8
        } else if avg_latency.as_millis() < 100 {
            0.6
        } else if avg_latency.as_millis() < 500 {
            0.4
        } else {
            0.2
        };
        
        let overall_score = (success_rate * 0.7) + (latency_score * 0.3);
        
        if overall_score >= 0.9 { 'A' }
        else if overall_score >= 0.8 { 'B' }
        else if overall_score >= 0.7 { 'C' }
        else if overall_score >= 0.6 { 'D' }
        else { 'F' }
    }
    
    /// Check if coordinator is performing well
    pub fn is_performing_well(&self) -> bool {
        let grade = self.performance_grade();
        matches!(grade, 'A' | 'B' | 'C')
    }
    
    /// Get coordinator status summary
    pub fn status_summary(&self) -> String {
        let metrics = self.get_metrics();
        let grade = self.performance_grade();
        
        format!(
            "EntanglementCoordinator Status:\n\
             - Performance Grade: {}\n\
             - Entanglements Created: {}\n\
             - Operations Attempted: {}\n\
             - Success Rate: {:.1}%\n\
             - Average Latency: {:.2}ms\n\
             - Active Entanglements: {}",
            grade,
            metrics.entanglements_created(),
            metrics.operations_attempted(),
            if metrics.operations_attempted() > 0 {
                (metrics.operations_successful() as f64 / metrics.operations_attempted() as f64) * 100.0
            } else {
                100.0
            },
            metrics.average_operation_latency().as_secs_f64() * 1000.0,
            metrics.active_entanglements()
        )
    }
    
    /// Perform maintenance operations
    pub async fn perform_maintenance(&mut self) -> Result<(), CognitiveError> {
        // Trigger metrics reporting
        self.metrics_collector.force_report();
        
        // Perform engine maintenance
        self.engine.perform_maintenance().await
    }
    
    /// Check if maintenance is needed
    pub fn needs_maintenance(&self) -> bool {
        self.metrics_collector.needs_report() || self.engine.needs_maintenance()
    }
}
