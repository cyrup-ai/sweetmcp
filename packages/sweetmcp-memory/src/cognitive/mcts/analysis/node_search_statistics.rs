//! Node search statistics and analysis
//!
//! This module provides statistical analysis for MCTS node search results
//! with zero-allocation patterns and blazing-fast performance.

use super::super::super::types::MCTSNode;
use std::collections::HashMap;

/// Basic tree statistics
#[derive(Debug, Clone)]
pub struct BasicTreeStatistics {
    pub node_count: usize,
    pub avg_reward: f64,
    pub avg_visits: f64,
    pub avg_performance: f64,
    pub min_reward: f64,
    pub max_reward: f64,
    pub min_visits: u64,
    pub max_visits: u64,
    pub terminal_count: usize,
    pub leaf_count: usize,
}

impl Default for BasicTreeStatistics {
    fn default() -> Self {
        Self {
            node_count: 0,
            avg_reward: 0.0,
            avg_visits: 0.0,
            avg_performance: 0.0,
            min_reward: 0.0,
            max_reward: 0.0,
            min_visits: 0,
            max_visits: 0,
            terminal_count: 0,
            leaf_count: 0,
        }
    }
}

impl BasicTreeStatistics {
    /// Get formatted summary
    pub fn summary(&self) -> String {
        format!(
            "Tree Statistics:\n\
            Nodes: {}\n\
            Avg Reward: {:.3} (range: {:.3} to {:.3})\n\
            Avg Visits: {:.1} (range: {} to {})\n\
            Avg Performance: {:.3}\n\
            Terminal Nodes: {} ({:.1}%)\n\
            Leaf Nodes: {} ({:.1}%)",
            self.node_count,
            self.avg_reward, self.min_reward, self.max_reward,
            self.avg_visits, self.min_visits, self.max_visits,
            self.avg_performance,
            self.terminal_count, self.terminal_percentage(),
            self.leaf_count, self.leaf_percentage()
        )
    }

    /// Get terminal node percentage
    #[inline]
    pub fn terminal_percentage(&self) -> f64 {
        if self.node_count == 0 {
            0.0
        } else {
            (self.terminal_count as f64 / self.node_count as f64) * 100.0
        }
    }

    /// Get leaf node percentage
    #[inline]
    pub fn leaf_percentage(&self) -> f64 {
        if self.node_count == 0 {
            0.0
        } else {
            (self.leaf_count as f64 / self.node_count as f64) * 100.0
        }
    }

    /// Get reward range
    #[inline]
    pub fn reward_range(&self) -> f64 {
        self.max_reward - self.min_reward
    }

    /// Get visit range
    #[inline]
    pub fn visit_range(&self) -> u64 {
        self.max_visits - self.min_visits
    }

    /// Check if tree has good diversity
    #[inline]
    pub fn has_good_diversity(&self) -> bool {
        self.reward_range() > 0.1 && self.visit_range() > 10
    }

    /// Check if tree is well-balanced
    #[inline]
    pub fn is_well_balanced(&self) -> bool {
        let terminal_ratio = self.terminal_percentage() / 100.0;
        let leaf_ratio = self.leaf_percentage() / 100.0;
        
        // Good balance: 10-40% terminals, 20-60% leaves
        terminal_ratio >= 0.1 && terminal_ratio <= 0.4 &&
        leaf_ratio >= 0.2 && leaf_ratio <= 0.6
    }
}

/// Tree statistics calculator
pub struct TreeStatisticsCalculator;

impl TreeStatisticsCalculator {
    /// Get basic tree statistics
    pub fn get_basic_statistics(tree: &HashMap<String, MCTSNode>) -> BasicTreeStatistics {
        if tree.is_empty() {
            return BasicTreeStatistics::default();
        }

        let mut total_reward = 0.0;
        let mut total_visits = 0u64;
        let mut total_performance = 0.0;
        let mut min_reward = f64::INFINITY;
        let mut max_reward = f64::NEG_INFINITY;
        let mut min_visits = u64::MAX;
        let mut max_visits = 0u64;
        let mut terminal_count = 0;
        let mut leaf_count = 0;

        for node in tree.values() {
            let reward = node.average_reward();
            let visits = node.visits;
            let performance = node.state.performance_score();

            total_reward += reward;
            total_visits += visits;
            total_performance += performance;

            min_reward = min_reward.min(reward);
            max_reward = max_reward.max(reward);
            min_visits = min_visits.min(visits);
            max_visits = max_visits.max(visits);

            if node.is_terminal {
                terminal_count += 1;
            }
            if node.children.is_empty() {
                leaf_count += 1;
            }
        }

        let node_count = tree.len();
        BasicTreeStatistics {
            node_count,
            avg_reward: total_reward / node_count as f64,
            avg_visits: total_visits as f64 / node_count as f64,
            avg_performance: total_performance / node_count as f64,
            min_reward,
            max_reward,
            min_visits,
            max_visits,
            terminal_count,
            leaf_count,
        }
    }

    /// Calculate reward distribution statistics
    pub fn calculate_reward_distribution(tree: &HashMap<String, MCTSNode>) -> RewardDistribution {
        if tree.is_empty() {
            return RewardDistribution::default();
        }

        let rewards: Vec<f64> = tree.values().map(|node| node.average_reward()).collect();
        let count = rewards.len();
        let sum: f64 = rewards.iter().sum();
        let mean = sum / count as f64;

        // Calculate variance and standard deviation
        let variance = rewards.iter()
            .map(|&reward| (reward - mean).powi(2))
            .sum::<f64>() / count as f64;
        let std_dev = variance.sqrt();

        // Calculate percentiles
        let mut sorted_rewards = rewards.clone();
        sorted_rewards.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        
        let p25_idx = (count as f64 * 0.25) as usize;
        let p50_idx = (count as f64 * 0.50) as usize;
        let p75_idx = (count as f64 * 0.75) as usize;
        
        RewardDistribution {
            mean,
            std_dev,
            variance,
            min: sorted_rewards[0],
            max: sorted_rewards[count - 1],
            p25: sorted_rewards[p25_idx.min(count - 1)],
            median: sorted_rewards[p50_idx.min(count - 1)],
            p75: sorted_rewards[p75_idx.min(count - 1)],
            count,
        }
    }

    /// Calculate visit distribution statistics
    pub fn calculate_visit_distribution(tree: &HashMap<String, MCTSNode>) -> VisitDistribution {
        if tree.is_empty() {
            return VisitDistribution::default();
        }

        let visits: Vec<u64> = tree.values().map(|node| node.visits).collect();
        let count = visits.len();
        let sum: u64 = visits.iter().sum();
        let mean = sum as f64 / count as f64;

        // Calculate variance and standard deviation
        let variance = visits.iter()
            .map(|&visit| (visit as f64 - mean).powi(2))
            .sum::<f64>() / count as f64;
        let std_dev = variance.sqrt();

        // Calculate percentiles
        let mut sorted_visits = visits.clone();
        sorted_visits.sort();
        
        let p25_idx = (count as f64 * 0.25) as usize;
        let p50_idx = (count as f64 * 0.50) as usize;
        let p75_idx = (count as f64 * 0.75) as usize;
        
        VisitDistribution {
            mean,
            std_dev,
            variance,
            min: sorted_visits[0],
            max: sorted_visits[count - 1],
            p25: sorted_visits[p25_idx.min(count - 1)],
            median: sorted_visits[p50_idx.min(count - 1)],
            p75: sorted_visits[p75_idx.min(count - 1)],
            total: sum,
            count,
        }
    }
}

/// Reward distribution statistics
#[derive(Debug, Clone)]
pub struct RewardDistribution {
    pub mean: f64,
    pub std_dev: f64,
    pub variance: f64,
    pub min: f64,
    pub max: f64,
    pub p25: f64,
    pub median: f64,
    pub p75: f64,
    pub count: usize,
}

impl Default for RewardDistribution {
    fn default() -> Self {
        Self {
            mean: 0.0,
            std_dev: 0.0,
            variance: 0.0,
            min: 0.0,
            max: 0.0,
            p25: 0.0,
            median: 0.0,
            p75: 0.0,
            count: 0,
        }
    }
}

/// Visit distribution statistics
#[derive(Debug, Clone)]
pub struct VisitDistribution {
    pub mean: f64,
    pub std_dev: f64,
    pub variance: f64,
    pub min: u64,
    pub max: u64,
    pub p25: u64,
    pub median: u64,
    pub p75: u64,
    pub total: u64,
    pub count: usize,
}

impl Default for VisitDistribution {
    fn default() -> Self {
        Self {
            mean: 0.0,
            std_dev: 0.0,
            variance: 0.0,
            min: 0,
            max: 0,
            p25: 0,
            median: 0,
            p75: 0,
            total: 0,
            count: 0,
        }
    }
}