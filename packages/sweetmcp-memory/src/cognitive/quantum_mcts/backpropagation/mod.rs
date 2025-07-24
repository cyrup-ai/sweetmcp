//! Backpropagation module coordination and re-exports
//!
//! This module provides zero-cost re-exports and coordination for all backpropagation
//! submodules following zero-allocation, lock-free, blazing-fast patterns.

// Module declarations
pub mod core;
pub mod adaptive;
pub mod metrics;
pub mod engine;

// Re-export core types for backward compatibility
pub use core::{QuantumBackpropagator, CacheStats};
pub use metrics::{
    BackpropagationMetrics, BackpropagationResult, NormalizationResult,
    BatchAnalysis, PerformanceTrend, StrategyComparison,
};
pub use engine::{
    QuantumBackpropagationEngine, BackpropagationStrategy, StrategyProfile,
};

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::cognitive::{
    quantum::{Complex64, EntanglementGraph},
    types::CognitiveError,
};
use super::{
    node_state::QuantumMCTSNode,
    config::QuantumMCTSConfig,
};

/// Backpropagation coordinator for managing all backpropagation functionality
#[derive(Debug)]
pub struct BackpropagationCoordinator {
    /// High-level backpropagation engine
    pub engine: QuantumBackpropagationEngine,
    /// Configuration for backpropagation operations
    config: QuantumMCTSConfig,
    /// Start time for coordinator lifecycle tracking
    start_time: std::time::Instant,
}

impl BackpropagationCoordinator {
    /// Create new backpropagation coordinator with comprehensive initialization
    pub fn new(
        config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
        default_strategy: BackpropagationStrategy,
    ) -> Self {
        info!("Initializing BackpropagationCoordinator with zero-allocation patterns");
        
        let engine = QuantumBackpropagationEngine::new(
            config.clone(),
            entanglement_graph,
            default_strategy,
        );
        
        Self {
            engine,
            config,
            start_time: std::time::Instant::now(),
        }
    }
    
    /// Perform single backpropagation with automatic strategy selection
    pub async fn backpropagate(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        node_id: String,
        reward: Complex64,
    ) -> Result<BackpropagationResult, CognitiveError> {
        debug!("Coordinating backpropagation for node: {}", node_id);
        
        self.engine.backpropagate(tree, node_id, reward, None).await
    }
    
    /// Perform backpropagation with specific strategy
    pub async fn backpropagate_with_strategy(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        node_id: String,
        reward: Complex64,
        strategy: BackpropagationStrategy,
    ) -> Result<BackpropagationResult, CognitiveError> {
        debug!("Coordinating backpropagation with strategy {:?} for node: {}", strategy, node_id);
        
        self.engine.backpropagate(tree, node_id, reward, Some(strategy)).await
    }
    
    /// Perform batch backpropagation with optimal strategy
    pub async fn batch_backpropagate(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        node_rewards: &[(String, Complex64)],
    ) -> Result<Vec<BackpropagationResult>, CognitiveError> {
        debug!("Coordinating batch backpropagation: {} items", node_rewards.len());
        
        self.engine.batch_backpropagate(tree, node_rewards, None).await
    }
    
    /// Perform multi-objective backpropagation
    pub async fn multi_objective_backpropagate(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        node_id: String,
        rewards: &[Complex64],
        weights: &[f64],
    ) -> Result<BackpropagationResult, CognitiveError> {
        debug!("Coordinating multi-objective backpropagation: {} objectives", rewards.len());
        
        let learning_rate = 0.1; // Default learning rate
        self.engine.multi_objective_backpropagate(
            tree, node_id, rewards, weights, learning_rate
        ).await
    }
    
    /// Perform temperature-controlled backpropagation for exploration
    pub async fn temperature_controlled_backpropagate(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        node_id: String,
        reward: Complex64,
        temperature: f64,
    ) -> Result<BackpropagationResult, CognitiveError> {
        debug!("Coordinating temperature-controlled backpropagation: temp={:.3}", temperature);
        
        let learning_rate = 0.1; // Default learning rate
        self.engine.temperature_controlled_backpropagate(
            tree, node_id, reward, temperature, learning_rate
        ).await
    }
    
    /// Normalize tree rewards for numerical stability
    pub async fn normalize_tree_rewards(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        max_magnitude: f64,
    ) -> Result<NormalizationResult, CognitiveError> {
        debug!("Coordinating reward normalization: max_magnitude={:.3}", max_magnitude);
        
        self.engine.backpropagator().normalize_tree_rewards(tree, max_magnitude).await
    }
    
    /// Perform adaptive reward normalization based on statistical analysis
    pub async fn adaptive_normalize_tree_rewards(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        target_percentile: f64,
    ) -> Result<NormalizationResult, CognitiveError> {
        debug!("Coordinating adaptive normalization: percentile={:.1}%", target_percentile);
        
        self.engine.backpropagator()
            .adaptive_normalize_tree_rewards(tree, target_percentile).await
    }
    
    /// Enable adaptive strategy selection
    pub fn enable_adaptive_strategy_selection(&mut self) {
        self.engine.set_adaptive_strategy_selection(true);
        info!("Adaptive strategy selection enabled for backpropagation coordinator");
    }
    
    /// Disable adaptive strategy selection
    pub fn disable_adaptive_strategy_selection(&mut self) {
        self.engine.set_adaptive_strategy_selection(false);
        info!("Adaptive strategy selection disabled for backpropagation coordinator");
    }
    
    /// Get comprehensive performance analysis
    pub fn get_performance_analysis(&self) -> PerformanceAnalysis {
        let combined_metrics = self.engine.get_combined_metrics();
        let strategy_comparison = self.engine.get_strategy_comparison();
        let cache_stats = self.engine.backpropagator().cache_stats();
        
        PerformanceAnalysis {
            combined_metrics,
            strategy_comparison,
            cache_stats,
            coordinator_uptime: self.start_time.elapsed(),
            adaptive_selection_enabled: self.engine.is_adaptive_selection_enabled(),
            performance_history_length: self.engine.performance_history_length(),
        }
    }
    
    /// Get strategy-specific metrics
    pub fn get_strategy_metrics(&self, strategy: BackpropagationStrategy) -> Option<&BackpropagationMetrics> {
        self.engine.get_strategy_metrics(strategy)
    }
    
    /// Reset all metrics and performance tracking
    pub fn reset_metrics(&mut self) {
        self.engine.reset_metrics();
        info!("Backpropagation coordinator metrics reset");
    }
    
    /// Clear caches for memory optimization
    pub fn clear_caches(&mut self) {
        self.engine.backpropagator().clear_caches();
        debug!("Backpropagation coordinator caches cleared");
    }
    
    /// Perform cache maintenance with existing nodes
    pub fn maintain_caches(&mut self, existing_nodes: &HashMap<String, QuantumMCTSNode>) {
        let cache_stats_before = self.engine.backpropagator().cache_stats();
        self.engine.backpropagator().prune_caches(existing_nodes);
        let cache_stats_after = self.engine.backpropagator().cache_stats();
        
        debug!(
            "Cache maintenance: paths {}->{}, rewards {}->{}",
            cache_stats_before.path_cache_size,
            cache_stats_after.path_cache_size,
            cache_stats_before.reward_cache_size,
            cache_stats_after.reward_cache_size
        );
    }
    
    /// Update configuration
    pub fn update_config(&mut self, new_config: QuantumMCTSConfig) {
        self.config = new_config.clone();
        self.engine.backpropagator().update_config(new_config);
        info!("Backpropagation coordinator configuration updated");
    }
    
    /// Get current configuration
    pub fn get_config(&self) -> &QuantumMCTSConfig {
        &self.config
    }
    
    /// Get coordinator uptime
    pub fn uptime(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
    
    /// Get current default strategy
    pub fn default_strategy(&self) -> BackpropagationStrategy {
        self.engine.default_strategy()
    }
    
    /// Set new default strategy
    pub fn set_default_strategy(&mut self, strategy: BackpropagationStrategy) {
        self.engine.set_default_strategy(strategy);
        info!("Coordinator default strategy changed to: {:?}", strategy);
    }
}

/// Comprehensive performance analysis for the coordinator
#[derive(Debug, Clone)]
pub struct PerformanceAnalysis {
    /// Combined metrics across all strategies
    pub combined_metrics: BackpropagationMetrics,
    /// Strategy-specific comparison
    pub strategy_comparison: StrategyComparison,
    /// Cache performance statistics
    pub cache_stats: CacheStats,
    /// Coordinator uptime
    pub coordinator_uptime: std::time::Duration,
    /// Whether adaptive strategy selection is enabled
    pub adaptive_selection_enabled: bool,
    /// Length of performance history
    pub performance_history_length: usize,
}

impl PerformanceAnalysis {
    /// Get overall health assessment
    pub fn health_assessment(&self) -> String {
        let grade = self.combined_metrics.performance_grade();
        let cache_utilization = self.cache_stats.path_cache_utilization();
        let throughput = self.combined_metrics.throughput();
        
        format!(
            "Grade: {}, Throughput: {:.1} ops/sec, Cache: {:.1}%, Uptime: {:.1}s",
            grade,
            throughput,
            cache_utilization,
            self.coordinator_uptime.as_secs_f64()
        )
    }
    
    /// Check if performance is acceptable
    pub fn is_healthy(&self) -> bool {
        self.combined_metrics.is_performing_well() &&
        !self.cache_stats.needs_pruning() &&
        self.strategy_comparison.overall_score() > 0.6
    }
    
    /// Get performance recommendations
    pub fn recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if !self.combined_metrics.is_performing_well() {
            recommendations.push("Consider optimizing backpropagation performance".to_string());
        }
        
        if self.cache_stats.needs_pruning() {
            recommendations.push("Cache maintenance recommended".to_string());
        }
        
        if !self.adaptive_selection_enabled && self.performance_history_length > 50 {
            recommendations.push("Consider enabling adaptive strategy selection".to_string());
        }
        
        if self.combined_metrics.cache_hit_rate() < 70.0 {
            recommendations.push("Improve cache hit rate through better locality".to_string());
        }
        
        recommendations.extend(self.strategy_comparison.recommendations());
        
        if recommendations.is_empty() {
            recommendations.push("Performance is acceptable, maintain current configuration".to_string());
        }
        
        recommendations
    }
}

/// Factory functions for creating backpropagation components
pub struct BackpropagationFactory;

impl BackpropagationFactory {
    /// Create a new backpropagation coordinator with default settings
    pub fn create_coordinator(
        config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
    ) -> BackpropagationCoordinator {
        BackpropagationCoordinator::new(
            config,
            entanglement_graph,
            BackpropagationStrategy::Standard,
        )
    }
    
    /// Create a coordinator optimized for high throughput
    pub fn create_high_throughput_coordinator(
        config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
    ) -> BackpropagationCoordinator {
        let mut coordinator = BackpropagationCoordinator::new(
            config,
            entanglement_graph,
            BackpropagationStrategy::BatchOptimized,
        );
        coordinator.enable_adaptive_strategy_selection();
        coordinator
    }
    
    /// Create a coordinator optimized for accuracy
    pub fn create_high_accuracy_coordinator(
        config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
    ) -> BackpropagationCoordinator {
        BackpropagationCoordinator::new(
            config,
            entanglement_graph,
            BackpropagationStrategy::WithEntanglement,
        )
    }
    
    /// Create a standalone backpropagation engine
    pub fn create_engine(
        config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
        strategy: BackpropagationStrategy,
    ) -> QuantumBackpropagationEngine {
        QuantumBackpropagationEngine::new(config, entanglement_graph, strategy)
    }
    
    /// Create a standalone backpropagator
    pub fn create_backpropagator(
        config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
    ) -> QuantumBackpropagator {
        QuantumBackpropagator::new(config, entanglement_graph)
    }
}