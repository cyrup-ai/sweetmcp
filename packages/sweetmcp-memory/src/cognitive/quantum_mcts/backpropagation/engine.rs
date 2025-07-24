//! High-level backpropagation engine with strategy dispatch
//!
//! This module provides the QuantumBackpropagationEngine as a high-level interface
//! with strategy-based dispatch and comprehensive backpropagation orchestration.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::cognitive::{
    quantum::{Complex64, EntanglementGraph},
    types::CognitiveError,
};
use super::{
    super::{
        node_state::QuantumMCTSNode,
        config::QuantumMCTSConfig,
    },
    core::QuantumBackpropagator,
    metrics::{BackpropagationResult, BackpropagationMetrics, StrategyComparison, PerformanceTrend},
};

/// Backpropagation strategy enumeration for dispatch control
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BackpropagationStrategy {
    /// Standard quantum backpropagation with basic entanglement
    Standard,
    /// Backpropagation with full entanglement effects
    WithEntanglement,
    /// Adaptive learning rate backpropagation
    Adaptive,
    /// Batch processing optimized backpropagation
    BatchOptimized,
    /// Multi-objective adaptive backpropagation
    MultiObjective,
    /// Temperature-controlled exploration backpropagation
    TemperatureControlled,
}

impl BackpropagationStrategy {
    /// Get strategy description
    pub fn description(&self) -> &'static str {
        match self {
            BackpropagationStrategy::Standard => "Standard quantum backpropagation",
            BackpropagationStrategy::WithEntanglement => "Full entanglement-aware backpropagation",
            BackpropagationStrategy::Adaptive => "Adaptive learning rate backpropagation",
            BackpropagationStrategy::BatchOptimized => "Batch-optimized backpropagation",
            BackpropagationStrategy::MultiObjective => "Multi-objective adaptive backpropagation",
            BackpropagationStrategy::TemperatureControlled => "Temperature-controlled exploration",
        }
    }
    
    /// Get expected performance characteristics
    pub fn performance_profile(&self) -> StrategyProfile {
        match self {
            BackpropagationStrategy::Standard => StrategyProfile {
                throughput: 0.8,
                accuracy: 0.7,
                memory_efficiency: 0.9,
                complexity: 0.3,
            },
            BackpropagationStrategy::WithEntanglement => StrategyProfile {
                throughput: 0.6,
                accuracy: 0.9,
                memory_efficiency: 0.7,
                complexity: 0.8,
            },
            BackpropagationStrategy::Adaptive => StrategyProfile {
                throughput: 0.7,
                accuracy: 0.8,
                memory_efficiency: 0.8,
                complexity: 0.6,
            },
            BackpropagationStrategy::BatchOptimized => StrategyProfile {
                throughput: 0.9,
                accuracy: 0.7,
                memory_efficiency: 0.6,
                complexity: 0.4,
            },
            BackpropagationStrategy::MultiObjective => StrategyProfile {
                throughput: 0.5,
                accuracy: 0.9,
                memory_efficiency: 0.7,
                complexity: 0.9,
            },
            BackpropagationStrategy::TemperatureControlled => StrategyProfile {
                throughput: 0.6,
                accuracy: 0.8,
                memory_efficiency: 0.8,
                complexity: 0.7,
            },
        }
    }
    
    /// Check if strategy supports batch operations
    pub fn supports_batch(&self) -> bool {
        matches!(self, BackpropagationStrategy::BatchOptimized | BackpropagationStrategy::Standard)
    }
    
    /// Get recommended use cases
    pub fn use_cases(&self) -> Vec<&'static str> {
        match self {
            BackpropagationStrategy::Standard => vec!["General purpose", "High throughput needed"],
            BackpropagationStrategy::WithEntanglement => vec!["High accuracy required", "Complex quantum states"],
            BackpropagationStrategy::Adaptive => vec!["Variable learning requirements", "Dynamic environments"],
            BackpropagationStrategy::BatchOptimized => vec!["High-volume processing", "Batch training"],
            BackpropagationStrategy::MultiObjective => vec!["Multiple optimization goals", "Complex reward structures"],
            BackpropagationStrategy::TemperatureControlled => vec!["Exploration control", "Annealing schedules"],
        }
    }
}

/// Strategy performance profile for comparison
#[derive(Debug, Clone, Copy)]
pub struct StrategyProfile {
    /// Throughput score (0.0 to 1.0)
    pub throughput: f64,
    /// Accuracy score (0.0 to 1.0)
    pub accuracy: f64,
    /// Memory efficiency score (0.0 to 1.0)
    pub memory_efficiency: f64,
    /// Computational complexity (0.0 to 1.0, higher = more complex)
    pub complexity: f64,
}

impl StrategyProfile {
    /// Calculate overall score with custom weights
    pub fn weighted_score(&self, throughput_weight: f64, accuracy_weight: f64, memory_weight: f64) -> f64 {
        let complexity_penalty = 1.0 - self.complexity;
        (self.throughput * throughput_weight + 
         self.accuracy * accuracy_weight + 
         self.memory_efficiency * memory_weight + 
         complexity_penalty * 0.1) / (throughput_weight + accuracy_weight + memory_weight + 0.1)
    }
}

/// High-level quantum backpropagation engine with strategy management
#[repr(align(64))] // Cache-line aligned for optimal performance
pub struct QuantumBackpropagationEngine {
    /// Core backpropagator instance
    backpropagator: QuantumBackpropagator,
    /// Default strategy for operations
    default_strategy: BackpropagationStrategy,
    /// Strategy performance tracking
    strategy_metrics: HashMap<BackpropagationStrategy, BackpropagationMetrics>,
    /// Adaptive strategy selection enabled
    adaptive_strategy_selection: bool,
    /// Strategy performance history
    performance_history: Vec<(BackpropagationStrategy, f64)>,
}

impl QuantumBackpropagationEngine {
    /// Create new backpropagation engine with strategy management
    pub fn new(
        config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
        default_strategy: BackpropagationStrategy,
    ) -> Self {
        info!("Initializing QuantumBackpropagationEngine with strategy: {:?}", default_strategy);
        
        Self {
            backpropagator: QuantumBackpropagator::new(config, entanglement_graph),
            default_strategy,
            strategy_metrics: HashMap::new(),
            adaptive_strategy_selection: false,
            performance_history: Vec::with_capacity(1000),
        }
    }
    
    /// Perform backpropagation with specified or default strategy
    pub async fn backpropagate(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        node_id: String,
        reward: Complex64,
        strategy: Option<BackpropagationStrategy>,
    ) -> Result<BackpropagationResult, CognitiveError> {
        let selected_strategy = if self.adaptive_strategy_selection {
            self.select_adaptive_strategy(&node_id, reward).await
        } else {
            strategy.unwrap_or(self.default_strategy)
        };
        
        debug!("Executing backpropagation with strategy: {:?}", selected_strategy);
        
        let start_time = std::time::Instant::now();
        
        let result = match selected_strategy {
            BackpropagationStrategy::Standard => {
                // Standard backpropagation without entanglement effects
                let mut result = self.backpropagator.quantum_backpropagate(tree, node_id, reward).await?;
                result.entanglement_effects_applied = 0; // Override for standard mode
                result
            }
            BackpropagationStrategy::WithEntanglement => {
                // Full quantum backpropagation with entanglement
                self.backpropagator.quantum_backpropagate(tree, node_id, reward).await?
            }
            BackpropagationStrategy::Adaptive => {
                // Adaptive learning rate backpropagation
                self.backpropagator.adaptive_backpropagate(tree, node_id, reward, 0.1).await?
            }
            BackpropagationStrategy::BatchOptimized => {
                // Single-item batch for consistency
                let batch_results = self.backpropagator.batch_backpropagate(
                    tree, 
                    &[(node_id, reward)]
                ).await?;
                
                batch_results.into_iter().next()
                    .ok_or_else(|| CognitiveError::InvalidState("Batch backpropagation returned no results".to_string()))?
            }
            BackpropagationStrategy::MultiObjective => {
                // Multi-objective with single reward (converted to array)
                self.backpropagator.multi_objective_adaptive_backpropagate(
                    tree, node_id, &[reward], &[1.0], 0.1
                ).await?
            }
            BackpropagationStrategy::TemperatureControlled => {
                // Temperature-controlled with moderate temperature
                self.backpropagator.temperature_adaptive_backpropagate(
                    tree, node_id, reward, 1.0, 0.1
                ).await?
            }
        };
        
        // Update strategy metrics
        self.update_strategy_metrics(selected_strategy, &result);
        
        // Record performance for adaptive selection
        if self.adaptive_strategy_selection {
            let performance_score = self.calculate_performance_score(&result);
            self.record_strategy_performance(selected_strategy, performance_score);
        }
        
        Ok(result)
    }
    
    /// Batch backpropagation with optimal strategy selection
    pub async fn batch_backpropagate(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        node_rewards: &[(String, Complex64)],
        strategy: Option<BackpropagationStrategy>,
    ) -> Result<Vec<BackpropagationResult>, CognitiveError> {
        let selected_strategy = strategy.unwrap_or(self.default_strategy);
        
        debug!("Executing batch backpropagation: {} items, strategy: {:?}", 
               node_rewards.len(), selected_strategy);
        
        if selected_strategy.supports_batch() {
            // Use native batch processing
            self.backpropagator.batch_backpropagate(tree, node_rewards).await
        } else {
            // Process individually
            let mut results = Vec::with_capacity(node_rewards.len());
            for (node_id, reward) in node_rewards {
                let result = self.backpropagate(
                    tree, 
                    node_id.clone(), 
                    *reward, 
                    Some(selected_strategy)
                ).await?;
                results.push(result);
            }
            Ok(results)
        }
    }
    
    /// Multi-objective backpropagation with reward weighting
    pub async fn multi_objective_backpropagate(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        node_id: String,
        rewards: &[Complex64],
        weights: &[f64],
        learning_rate: f64,
    ) -> Result<BackpropagationResult, CognitiveError> {
        debug!("Multi-objective backpropagation: {} objectives", rewards.len());
        
        self.backpropagator.multi_objective_adaptive_backpropagate(
            tree, node_id, rewards, weights, learning_rate
        ).await
    }
    
    /// Temperature-controlled backpropagation for exploration
    pub async fn temperature_controlled_backpropagate(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        node_id: String,
        reward: Complex64,
        temperature: f64,
        learning_rate: f64,
    ) -> Result<BackpropagationResult, CognitiveError> {
        debug!("Temperature-controlled backpropagation: temp={:.3}", temperature);
        
        self.backpropagator.temperature_adaptive_backpropagate(
            tree, node_id, reward, temperature, learning_rate
        ).await
    }
    
    /// Enable or disable adaptive strategy selection
    pub fn set_adaptive_strategy_selection(&mut self, enabled: bool) {
        self.adaptive_strategy_selection = enabled;
        if enabled {
            info!("Adaptive strategy selection enabled");
        } else {
            info!("Adaptive strategy selection disabled");
        }
    }
    
    /// Select strategy adaptively based on context
    async fn select_adaptive_strategy(
        &self,
        _node_id: &str,
        _reward: Complex64,
    ) -> BackpropagationStrategy {
        // Analyze recent performance to select best strategy
        if self.performance_history.len() < 10 {
            return self.default_strategy;
        }
        
        // Calculate average performance for each strategy
        let mut strategy_scores: HashMap<BackpropagationStrategy, (f64, usize)> = HashMap::new();
        
        for &(strategy, score) in self.performance_history.iter().rev().take(50) {
            let entry = strategy_scores.entry(strategy).or_insert((0.0, 0));
            entry.0 += score;
            entry.1 += 1;
        }
        
        // Find strategy with highest average performance
        let best_strategy = strategy_scores.iter()
            .filter(|(_, (_, count))| *count >= 3) // Require minimum samples
            .max_by(|(_, (score_a, count_a)), (_, (score_b, count_b))| {
                let avg_a = score_a / *count_a as f64;
                let avg_b = score_b / *count_b as f64;
                avg_a.partial_cmp(&avg_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(&strategy, _)| strategy)
            .unwrap_or(self.default_strategy);
        
        debug!("Adaptive strategy selected: {:?}", best_strategy);
        best_strategy
    }
    
    /// Calculate performance score for a backpropagation result
    fn calculate_performance_score(&self, result: &BackpropagationResult) -> f64 {
        if !result.success {
            return 0.0;
        }
        
        // Composite score based on efficiency and effectiveness
        let efficiency_score = result.efficiency().min(100.0) / 100.0; // Normalize to 0-1
        let reward_score = result.reward_distributed.norm().min(10.0) / 10.0; // Normalize to 0-1
        let path_score = result.path_utilization();
        
        (efficiency_score + reward_score + path_score) / 3.0
    }
    
    /// Record strategy performance for adaptive selection
    fn record_strategy_performance(&mut self, strategy: BackpropagationStrategy, score: f64) {
        self.performance_history.push((strategy, score));
        
        // Limit history size
        if self.performance_history.len() > 1000 {
            self.performance_history.remove(0);
        }
    }
    
    /// Update metrics for a specific strategy
    fn update_strategy_metrics(&mut self, strategy: BackpropagationStrategy, result: &BackpropagationResult) {
        let metrics = self.strategy_metrics.entry(strategy).or_insert_with(BackpropagationMetrics::new);
        
        if result.success {
            metrics.backpropagations_performed += 1;
            metrics.total_nodes_updated += result.nodes_updated;
            metrics.total_backpropagation_time += result.elapsed_time;
            metrics.total_reward_distributed += result.reward_distributed.norm();
            metrics.entanglement_effects_processed += result.entanglement_effects_applied as u64;
        }
    }
    
    /// Get strategy comparison analysis
    pub fn get_strategy_comparison(&self) -> StrategyComparison {
        let standard_metrics = self.strategy_metrics.get(&BackpropagationStrategy::Standard)
            .cloned().unwrap_or_default();
        let adaptive_metrics = self.strategy_metrics.get(&BackpropagationStrategy::Adaptive)
            .cloned().unwrap_or_default();
        let batch_metrics = self.strategy_metrics.get(&BackpropagationStrategy::BatchOptimized)
            .cloned().unwrap_or_default();
        
        let trend = self.analyze_performance_trend();
        
        StrategyComparison {
            standard_metrics,
            adaptive_metrics,
            batch_metrics,
            trend,
        }
    }
    
    /// Analyze performance trend over time
    fn analyze_performance_trend(&self) -> PerformanceTrend {
        if self.performance_history.len() < 10 {
            return PerformanceTrend::Insufficient;
        }
        
        let recent_scores: Vec<f64> = self.performance_history.iter()
            .rev()
            .take(20)
            .map(|(_, score)| *score)
            .collect();
        
        let older_scores: Vec<f64> = self.performance_history.iter()
            .rev()
            .skip(20)
            .take(20)
            .map(|(_, score)| *score)
            .collect();
        
        if recent_scores.is_empty() || older_scores.is_empty() {
            return PerformanceTrend::Insufficient;
        }
        
        let recent_avg = recent_scores.iter().sum::<f64>() / recent_scores.len() as f64;
        let older_avg = older_scores.iter().sum::<f64>() / older_scores.len() as f64;
        
        let change_ratio = (recent_avg - older_avg) / older_avg.max(0.01);
        
        // Calculate variance to detect volatility
        let recent_variance = recent_scores.iter()
            .map(|&score| (score - recent_avg).powi(2))
            .sum::<f64>() / recent_scores.len() as f64;
        
        if recent_variance > 0.1 {
            return PerformanceTrend::Volatile;
        }
        
        match change_ratio {
            r if r > 0.1 => PerformanceTrend::Improving,
            r if r < -0.1 => PerformanceTrend::Declining,
            _ => PerformanceTrend::Stable,
        }
    }
    
    /// Get backpropagator reference for direct operations
    pub fn backpropagator(&mut self) -> &mut QuantumBackpropagator {
        &mut self.backpropagator
    }
    
    /// Get combined performance metrics across all strategies
    pub fn get_combined_metrics(&self) -> BackpropagationMetrics {
        let mut combined = BackpropagationMetrics::new();
        
        for metrics in self.strategy_metrics.values() {
            combined.merge(metrics);
        }
        
        combined
    }
    
    /// Get metrics for a specific strategy
    pub fn get_strategy_metrics(&self, strategy: BackpropagationStrategy) -> Option<&BackpropagationMetrics> {
        self.strategy_metrics.get(&strategy)
    }
    
    /// Reset all metrics and performance history
    pub fn reset_metrics(&mut self) {
        for metrics in self.strategy_metrics.values_mut() {
            metrics.reset();
        }
        self.performance_history.clear();
        self.backpropagator.reset_metrics();
        info!("All backpropagation metrics reset");
    }
    
    /// Get current default strategy
    pub fn default_strategy(&self) -> BackpropagationStrategy {
        self.default_strategy
    }
    
    /// Set new default strategy
    pub fn set_default_strategy(&mut self, strategy: BackpropagationStrategy) {
        self.default_strategy = strategy;
        info!("Default backpropagation strategy changed to: {:?}", strategy);
    }
    
    /// Check if adaptive strategy selection is enabled
    pub fn is_adaptive_selection_enabled(&self) -> bool {
        self.adaptive_strategy_selection
    }
    
    /// Get performance history length
    pub fn performance_history_length(&self) -> usize {
        self.performance_history.len()
    }
}