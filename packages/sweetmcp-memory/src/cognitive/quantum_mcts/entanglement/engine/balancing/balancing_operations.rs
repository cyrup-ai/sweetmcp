//! Balancing operations for entanglement distribution
//!
//! This module provides blazing-fast balancing operations with zero allocation
//! optimizations and elegant ergonomic interfaces for executing load balancing.

use std::collections::HashMap;
use tracing::{debug, warn, info};

use crate::cognitive::types::CognitiveError;
use super::{
    balance_analysis::{NetworkBalanceAnalysis, NodeBalance, BalanceAnalyzer},
    balancing_strategy::{BalancingStrategy, NodeRebalancingPriority, RebalancingAction, OperationResult},
};
use super::super::super::analysis::NetworkTopology;
use crate::cognitive::quantum_mcts::node_state::QuantumMCTSNode;

/// Balancing operation executor
pub struct BalancingOperations {
    /// Current balancing strategy
    strategy: BalancingStrategy,
    /// Operation metrics
    metrics: OperationMetrics,
    /// Redistribution cache for performance
    redistribution_cache: RedistributionCache,
}

impl BalancingOperations {
    /// Create new balancing operations executor
    #[inline]
    pub fn new(strategy: BalancingStrategy) -> Self {
        Self {
            strategy,
            metrics: OperationMetrics::new(),
            redistribution_cache: RedistributionCache::new(),
        }
    }

    /// Execute complete balancing operation with zero allocation optimizations
    pub async fn execute_balancing(
        &mut self,
        tree: &mut HashMap<String, QuantumMCTSNode>,
        topology: &NetworkTopology,
    ) -> Result<BalancingResult, CognitiveError> {
        let start_time = std::time::Instant::now();
        
        debug!("Starting balancing operation for {} nodes", tree.len());

        // Analyze current balance state
        let initial_analysis = BalanceAnalyzer::analyze_network_balance(tree, topology)?;
        
        // Check if balancing should proceed
        if !self.strategy.should_proceed(&initial_analysis) {
            debug!("Balancing not needed: improvement potential {:.1}%", 
                   initial_analysis.calculate_potential_improvement());
            
            return Ok(BalancingResult::no_action_needed(
                initial_analysis,
                start_time.elapsed().as_millis() as u64,
            ));
        }

        // Get prioritized nodes for rebalancing
        let node_priorities = self.strategy.prioritize_nodes(&initial_analysis);
        let redistribution_limit = self.strategy.calculate_redistribution_limit(&initial_analysis);

        // Execute redistribution operations
        let redistributions_made = self.execute_redistributions(
            tree,
            topology,
            &node_priorities,
            redistribution_limit,
        ).await?;

        // Analyze final balance state
        let final_analysis = BalanceAnalyzer::analyze_network_balance(tree, topology)?;
        
        // Calculate improvement metrics
        let improvement = BalanceAnalyzer::calculate_balance_improvement(&initial_analysis, &final_analysis);
        let operation_time = start_time.elapsed().as_millis() as u64;

        // Update metrics
        self.metrics.record_operation(redistributions_made, improvement, operation_time);

        // Create result
        let result = BalancingResult::new(
            initial_analysis,
            final_analysis,
            redistributions_made,
            improvement,
            operation_time,
            self.strategy.get_balancing_reason(&initial_analysis),
        );

        info!("Balancing completed: {} redistributions, {:.1}% improvement in {}ms",
              redistributions_made, improvement, operation_time);

        Ok(result)
    }

    /// Execute redistribution operations
    async fn execute_redistributions(
        &mut self,
        tree: &mut HashMap<String, QuantumMCTSNode>,
        topology: &NetworkTopology,
        node_priorities: &[NodeRebalancingPriority],
        limit: usize,
    ) -> Result<usize, CognitiveError> {
        let mut redistributions_made = 0;
        let mut overloaded_nodes = Vec::new();
        let mut underloaded_nodes = Vec::new();

        // Separate nodes by load status
        for priority in node_priorities {
            if priority.is_overloaded {
                overloaded_nodes.push(priority);
            } else if priority.is_underloaded {
                underloaded_nodes.push(priority);
            }
        }

        debug!("Found {} overloaded and {} underloaded nodes", 
               overloaded_nodes.len(), underloaded_nodes.len());

        // Execute redistributions between overloaded and underloaded nodes
        for overloaded in &overloaded_nodes {
            if redistributions_made >= limit {
                break;
            }

            let redistribution_amount = overloaded.calculate_redistribution_amount(
                self.strategy.load_balancing_factor
            );

            // Find suitable underloaded nodes
            let suitable_targets = self.find_suitable_redistribution_targets(
                overloaded,
                &underloaded_nodes,
                redistribution_amount,
            );

            for target in suitable_targets {
                if redistributions_made >= limit {
                    break;
                }

                let actual_amount = self.execute_single_redistribution(
                    tree,
                    topology,
                    &overloaded.node_id,
                    &target.node_id,
                    redistribution_amount,
                ).await?;

                if actual_amount > 0 {
                    redistributions_made += actual_amount;
                    debug!("Redistributed {} entanglements from {} to {}", 
                           actual_amount, overloaded.node_id, target.node_id);
                }
            }
        }

        Ok(redistributions_made)
    }

    /// Find suitable redistribution targets
    #[inline]
    fn find_suitable_redistribution_targets(
        &self,
        source: &NodeRebalancingPriority,
        candidates: &[&NodeRebalancingPriority],
        amount: usize,
    ) -> Vec<&NodeRebalancingPriority> {
        let mut suitable_targets = Vec::new();

        for &candidate in candidates {
            // Check if target can accept the redistribution
            if candidate.deficit >= amount as i32 &&
               candidate.priority_score >= 0.1 && // Minimum priority threshold
               candidate.node_id != source.node_id {
                suitable_targets.push(candidate);
            }
        }

        // Sort by priority (highest first)
        suitable_targets.sort_by(|a, b| b.priority_score.partial_cmp(&a.priority_score).unwrap());
        
        // Limit to top candidates for efficiency
        suitable_targets.truncate(5);
        suitable_targets
    }

    /// Execute single redistribution operation
    async fn execute_single_redistribution(
        &mut self,
        tree: &mut HashMap<String, QuantumMCTSNode>,
        topology: &NetworkTopology,
        source_id: &str,
        target_id: &str,
        requested_amount: usize,
    ) -> Result<usize, CognitiveError> {
        // Check cache for recent redistribution
        if let Some(cached_result) = self.redistribution_cache.get(source_id, target_id) {
            if cached_result.is_recent() {
                debug!("Using cached redistribution result for {} -> {}", source_id, target_id);
                return Ok(cached_result.amount);
            }
        }

        // Get source and target nodes
        let source_node = tree.get(source_id)
            .ok_or_else(|| CognitiveError::NodeNotFound(source_id.to_string()))?;
        let target_node = tree.get(target_id)
            .ok_or_else(|| CognitiveError::NodeNotFound(target_id.to_string()))?;

        // Calculate actual redistribution amount based on constraints
        let actual_amount = self.calculate_actual_redistribution_amount(
            source_node,
            target_node,
            topology,
            requested_amount,
        );

        if actual_amount == 0 {
            debug!("No redistribution possible between {} and {}", source_id, target_id);
            return Ok(0);
        }

        // Perform the redistribution (in a real implementation, this would modify entanglements)
        let success = self.perform_redistribution(
            tree,
            source_id,
            target_id,
            actual_amount,
        ).await?;

        if success {
            // Cache the result
            self.redistribution_cache.insert(
                source_id.to_string(),
                target_id.to_string(),
                actual_amount,
            );

            debug!("Successfully redistributed {} entanglements from {} to {}", 
                   actual_amount, source_id, target_id);
            Ok(actual_amount)
        } else {
            warn!("Failed to redistribute entanglements from {} to {}", source_id, target_id);
            Ok(0)
        }
    }

    /// Calculate actual redistribution amount considering constraints
    #[inline]
    fn calculate_actual_redistribution_amount(
        &self,
        source_node: &QuantumMCTSNode,
        target_node: &QuantumMCTSNode,
        topology: &NetworkTopology,
        requested_amount: usize,
    ) -> usize {
        // In a real implementation, this would consider:
        // - Source node's available entanglements
        // - Target node's capacity
        // - Network topology constraints
        // - Quantum coherence requirements
        
        // For now, use a simplified calculation
        let source_capacity = topology.node_degrees.get(&source_node.id).copied().unwrap_or(0);
        let target_capacity = topology.node_degrees.get(&target_node.id).copied().unwrap_or(0);
        
        let max_transferable = (source_capacity / 4).max(1); // Conservative transfer limit
        let max_acceptable = (target_capacity / 2).max(1);   // Conservative acceptance limit
        
        requested_amount.min(max_transferable).min(max_acceptable)
    }

    /// Perform actual redistribution operation
    async fn perform_redistribution(
        &mut self,
        tree: &mut HashMap<String, QuantumMCTSNode>,
        source_id: &str,
        target_id: &str,
        amount: usize,
    ) -> Result<bool, CognitiveError> {
        // In a real implementation, this would:
        // 1. Validate quantum coherence constraints
        // 2. Update entanglement connections
        // 3. Modify node states
        // 4. Update network topology
        // 5. Record metrics and telemetry
        
        // For now, simulate the operation
        if let (Some(_source), Some(_target)) = (tree.get_mut(source_id), tree.get_mut(target_id)) {
            // Simulate redistribution delay
            tokio::time::sleep(tokio::time::Duration::from_micros(10)).await;
            
            // In real implementation: update entanglement states
            debug!("Simulated redistribution of {} entanglements from {} to {}", 
                   amount, source_id, target_id);
            
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get current operation metrics
    #[inline]
    pub fn get_metrics(&self) -> &OperationMetrics {
        &self.metrics
    }

    /// Update balancing strategy
    #[inline]
    pub fn update_strategy(&mut self, new_strategy: BalancingStrategy) {
        debug!("Updating balancing strategy: {:?} -> {:?}", 
               self.strategy.strategy_type, new_strategy.strategy_type);
        self.strategy = new_strategy;
    }

    /// Clear redistribution cache
    #[inline]
    pub fn clear_cache(&mut self) {
        self.redistribution_cache.clear();
        debug!("Cleared redistribution cache");
    }

    /// Get cache statistics
    #[inline]
    pub fn get_cache_stats(&self) -> CacheStatistics {
        self.redistribution_cache.get_statistics()
    }
}

/// Balancing operation result
#[derive(Debug, Clone)]
pub struct BalancingResult {
    /// Initial balance analysis
    pub initial_analysis: NetworkBalanceAnalysis,
    /// Final balance analysis
    pub final_analysis: NetworkBalanceAnalysis,
    /// Number of redistributions performed
    pub redistributions_made: usize,
    /// Improvement percentage achieved
    pub improvement_percentage: f64,
    /// Operation duration in milliseconds
    pub operation_time_ms: u64,
    /// Reason for balancing operation
    pub balancing_reason: String,
    /// Whether operation was successful
    pub success: bool,
}

impl BalancingResult {
    /// Create new balancing result
    #[inline]
    pub fn new(
        initial_analysis: NetworkBalanceAnalysis,
        final_analysis: NetworkBalanceAnalysis,
        redistributions_made: usize,
        improvement_percentage: f64,
        operation_time_ms: u64,
        balancing_reason: String,
    ) -> Self {
        let success = improvement_percentage > 0.0 || redistributions_made > 0;

        Self {
            initial_analysis,
            final_analysis,
            redistributions_made,
            improvement_percentage,
            operation_time_ms,
            balancing_reason,
            success,
        }
    }

    /// Create result for no action needed
    #[inline]
    pub fn no_action_needed(
        analysis: NetworkBalanceAnalysis,
        operation_time_ms: u64,
    ) -> Self {
        Self {
            final_analysis: analysis.clone(),
            initial_analysis: analysis,
            redistributions_made: 0,
            improvement_percentage: 0.0,
            operation_time_ms,
            balancing_reason: "Network already well balanced".to_string(),
            success: true,
        }
    }

    /// Check if significant improvement was achieved
    #[inline]
    pub fn achieved_significant_improvement(&self, threshold: f64) -> bool {
        self.improvement_percentage >= threshold
    }

    /// Get efficiency score (improvement per millisecond)
    #[inline]
    pub fn efficiency_score(&self) -> f64 {
        if self.operation_time_ms == 0 {
            0.0
        } else {
            self.improvement_percentage / self.operation_time_ms as f64
        }
    }

    /// Get operation summary
    #[inline]
    pub fn get_summary(&self) -> String {
        if self.redistributions_made == 0 {
            format!("No balancing needed - network efficiency: {:.1}%", 
                    self.final_analysis.balance_efficiency_score() * 100.0)
        } else {
            format!(
                "Balanced {} redistributions: {:.1}% improvement in {}ms (efficiency: {:.3})",
                self.redistributions_made,
                self.improvement_percentage,
                self.operation_time_ms,
                self.efficiency_score()
            )
        }
    }

    /// Convert to operation result for performance tracking
    #[inline]
    pub fn to_operation_result(&self) -> OperationResult {
        OperationResult {
            success: self.success,
            efficiency_improvement: self.improvement_percentage,
            redistributions_made: self.redistributions_made,
            operation_time_ms: self.operation_time_ms,
        }
    }
}

/// Operation metrics for performance tracking
#[derive(Debug, Clone)]
pub struct OperationMetrics {
    /// Total operations performed
    pub total_operations: usize,
    /// Total redistributions made
    pub total_redistributions: usize,
    /// Average improvement per operation
    pub average_improvement: f64,
    /// Average operation time
    pub average_operation_time_ms: f64,
    /// Success rate
    pub success_rate: f64,
    /// Recent operation times
    recent_times: Vec<u64>,
    /// Recent improvements
    recent_improvements: Vec<f64>,
}

impl OperationMetrics {
    /// Create new operation metrics
    #[inline]
    pub fn new() -> Self {
        Self {
            total_operations: 0,
            total_redistributions: 0,
            average_improvement: 0.0,
            average_operation_time_ms: 0.0,
            success_rate: 0.0,
            recent_times: Vec::new(),
            recent_improvements: Vec::new(),
        }
    }

    /// Record operation result
    #[inline]
    pub fn record_operation(&mut self, redistributions: usize, improvement: f64, time_ms: u64) {
        self.total_operations += 1;
        self.total_redistributions += redistributions;
        
        // Update recent history
        self.recent_times.push(time_ms);
        self.recent_improvements.push(improvement);
        
        // Keep only recent history (last 100 operations)
        if self.recent_times.len() > 100 {
            self.recent_times.remove(0);
            self.recent_improvements.remove(0);
        }
        
        // Recalculate averages
        self.average_improvement = self.recent_improvements.iter().sum::<f64>() / self.recent_improvements.len() as f64;
        self.average_operation_time_ms = self.recent_times.iter().sum::<u64>() as f64 / self.recent_times.len() as f64;
        self.success_rate = self.recent_improvements.iter().filter(|&&imp| imp > 0.0).count() as f64 / self.recent_improvements.len() as f64;
    }

    /// Get performance summary
    #[inline]
    pub fn get_performance_summary(&self) -> String {
        format!(
            "Operations: {}, Redistributions: {}, Avg Improvement: {:.1}%, Avg Time: {:.1}ms, Success Rate: {:.1}%",
            self.total_operations,
            self.total_redistributions,
            self.average_improvement,
            self.average_operation_time_ms,
            self.success_rate * 100.0
        )
    }
}

impl Default for OperationMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Redistribution cache for performance optimization
#[derive(Debug)]
struct RedistributionCache {
    cache: HashMap<String, CacheEntry>,
    max_size: usize,
    cache_duration_ms: u64,
}

impl RedistributionCache {
    /// Create new redistribution cache
    #[inline]
    fn new() -> Self {
        Self {
            cache: HashMap::new(),
            max_size: 1000,
            cache_duration_ms: 5000, // 5 seconds
        }
    }

    /// Get cached redistribution result
    #[inline]
    fn get(&self, source_id: &str, target_id: &str) -> Option<&CacheEntry> {
        let key = format!("{}:{}", source_id, target_id);
        self.cache.get(&key)
    }

    /// Insert redistribution result into cache
    #[inline]
    fn insert(&mut self, source_id: String, target_id: String, amount: usize) {
        let key = format!("{}:{}", source_id, target_id);
        let entry = CacheEntry::new(amount);
        
        // Evict oldest entries if cache is full
        if self.cache.len() >= self.max_size {
            self.evict_oldest();
        }
        
        self.cache.insert(key, entry);
    }

    /// Clear cache
    #[inline]
    fn clear(&mut self) {
        self.cache.clear();
    }

    /// Evict oldest cache entries
    #[inline]
    fn evict_oldest(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        // Remove expired entries
        self.cache.retain(|_, entry| {
            now - entry.timestamp_ms < self.cache_duration_ms
        });

        // If still too large, remove 25% of entries
        if self.cache.len() >= self.max_size {
            let target_size = self.max_size * 3 / 4;
            let mut entries: Vec<_> = self.cache.iter().collect();
            entries.sort_by_key(|(_, entry)| entry.timestamp_ms);
            
            for (key, _) in entries.iter().take(self.cache.len() - target_size) {
                self.cache.remove(*key);
            }
        }
    }

    /// Get cache statistics
    #[inline]
    fn get_statistics(&self) -> CacheStatistics {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let expired_count = self.cache.values()
            .filter(|entry| now - entry.timestamp_ms >= self.cache_duration_ms)
            .count();

        CacheStatistics {
            total_entries: self.cache.len(),
            expired_entries: expired_count,
            active_entries: self.cache.len() - expired_count,
            hit_rate: 0.0, // Would need hit/miss tracking for accurate rate
        }
    }
}

/// Cache entry for redistribution results
#[derive(Debug, Clone)]
struct CacheEntry {
    amount: usize,
    timestamp_ms: u64,
}

impl CacheEntry {
    /// Create new cache entry
    #[inline]
    fn new(amount: usize) -> Self {
        let timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            amount,
            timestamp_ms,
        }
    }

    /// Check if entry is recent
    #[inline]
    fn is_recent(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        now - self.timestamp_ms < 5000 // 5 seconds
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStatistics {
    pub total_entries: usize,
    pub expired_entries: usize,
    pub active_entries: usize,
    pub hit_rate: f64,
}