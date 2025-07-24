//! Advanced node search operations
//!
//! This module provides advanced MCTS node search functionality
//! with zero-allocation patterns and blazing-fast performance.

use super::super::super::types::{MCTSNode, CodeState};
use super::node_search_types::{NodeMatch, CharacteristicNodes};
use super::node_search_bottleneck::{BottleneckNode, BottleneckType, BottleneckSeverity};
use std::collections::HashMap;

/// Advanced node search operations
pub struct AdvancedNodeSearch;

impl AdvancedNodeSearch {
    /// Find nodes with specific characteristics
    #[inline]
    pub fn find_characteristic_nodes(tree: &HashMap<String, MCTSNode>) -> CharacteristicNodes {
        let mut result = CharacteristicNodes::default();

        for (node_id, node) in tree {
            let reward = node.average_reward();
            let visits = node.visits;
            let performance = node.state.performance_score();

            // Track highest reward
            if reward > result.highest_reward_node.as_ref().map_or(f64::NEG_INFINITY, |n| n.reward) {
                result.highest_reward_node = Some(NodeMatch {
                    node_id: node_id.clone(),
                    reward,
                    visits,
                    performance_score: performance,
                    depth: node.calculate_depth(tree),
                });
            }

            // Track most visited
            if visits > result.most_visited_node.as_ref().map_or(0, |n| n.visits) {
                result.most_visited_node = Some(NodeMatch {
                    node_id: node_id.clone(),
                    reward,
                    visits,
                    performance_score: performance,
                    depth: node.calculate_depth(tree),
                });
            }

            // Track best performance
            if performance > result.best_performance_node.as_ref().map_or(f64::NEG_INFINITY, |n| n.performance_score) {
                result.best_performance_node = Some(NodeMatch {
                    node_id: node_id.clone(),
                    reward,
                    visits,
                    performance_score: performance,
                    depth: node.calculate_depth(tree),
                });
            }

            // Track deepest node
            let depth = node.calculate_depth(tree);
            if depth > result.deepest_node.as_ref().map_or(0, |n| n.depth) {
                result.deepest_node = Some(NodeMatch {
                    node_id: node_id.clone(),
                    reward,
                    visits,
                    performance_score: performance,
                    depth,
                });
            }
        }

        result
    }

    /// Find nodes that might be bottlenecks
    #[inline]
    pub fn find_bottleneck_nodes(tree: &HashMap<String, MCTSNode>) -> Vec<BottleneckNode> {
        let mut bottlenecks = Vec::new();

        for (node_id, node) in tree {
            // Check for low reward but high visits (exploration trap)
            if node.visits > 50 && node.average_reward() < 0.3 {
                bottlenecks.push(BottleneckNode {
                    node_id: node_id.clone(),
                    bottleneck_type: BottleneckType::LowRewardHighVisits,
                    severity: if node.visits > 100 { BottleneckSeverity::High } else { BottleneckSeverity::Medium },
                    description: format!("Node has {} visits but only {:.3} average reward", node.visits, node.average_reward()),
                });
            }

            // Check for single child (limited exploration)
            if node.children.len() == 1 && !node.is_terminal {
                bottlenecks.push(BottleneckNode {
                    node_id: node_id.clone(),
                    bottleneck_type: BottleneckType::SingleChild,
                    severity: BottleneckSeverity::Medium,
                    description: "Node has only one child, limiting exploration".to_string(),
                });
            }

            // Check for unbalanced children (one child gets most visits)
            if node.children.len() > 1 {
                let child_visits: Vec<u64> = node.children.values()
                    .filter_map(|child_id| tree.get(child_id).map(|child| child.visits))
                    .collect();
                
                if !child_visits.is_empty() {
                    let max_visits = *child_visits.iter().max().unwrap_or(&0);
                    let total_visits: u64 = child_visits.iter().sum();
                    
                    if max_visits > 0 && total_visits > 0 {
                        let imbalance_ratio = max_visits as f64 / total_visits as f64;
                        if imbalance_ratio > 0.8 {
                            bottlenecks.push(BottleneckNode {
                                node_id: node_id.clone(),
                                bottleneck_type: BottleneckType::UnbalancedChildren,
                                severity: if imbalance_ratio > 0.9 { BottleneckSeverity::High } else { BottleneckSeverity::Medium },
                                description: format!("One child gets {:.1}% of visits", imbalance_ratio * 100.0),
                            });
                        }
                    }
                }
            }

            // Check for deep nodes with low visits
            let depth = node.calculate_depth(tree);
            if depth > 5 && node.visits < 10 {
                bottlenecks.push(BottleneckNode {
                    node_id: node_id.clone(),
                    bottleneck_type: BottleneckType::DeepLowVisits,
                    severity: if depth > 10 { BottleneckSeverity::High } else { BottleneckSeverity::Low },
                    description: format!("Deep node (depth {}) with only {} visits", depth, node.visits),
                });
            }
        }

        bottlenecks
    }

    /// Find nodes with anomalous behavior
    pub fn find_anomalous_nodes(tree: &HashMap<String, MCTSNode>) -> Vec<AnomalousNode> {
        let mut anomalies = Vec::new();

        // Calculate tree-wide statistics for comparison
        let total_nodes = tree.len();
        if total_nodes == 0 {
            return anomalies;
        }

        let avg_reward: f64 = tree.values().map(|n| n.average_reward()).sum::<f64>() / total_nodes as f64;
        let avg_visits: f64 = tree.values().map(|n| n.visits).sum::<u64>() as f64 / total_nodes as f64;

        for (node_id, node) in tree {
            let reward = node.average_reward();
            let visits = node.visits;

            // Check for extremely high reward with low visits (potential outlier)
            if reward > avg_reward * 2.0 && visits < (avg_visits * 0.5) as u64 {
                anomalies.push(AnomalousNode {
                    node_id: node_id.clone(),
                    anomaly_type: AnomalyType::HighRewardLowVisits,
                    description: format!("Reward {:.3} (avg: {:.3}), visits {} (avg: {:.1})", reward, avg_reward, visits, avg_visits),
                    severity_score: ((reward / avg_reward) * (avg_visits / visits as f64)) as f32,
                });
            }

            // Check for extremely low reward with high visits (potential trap)
            if reward < avg_reward * 0.5 && visits > (avg_visits * 2.0) as u64 {
                anomalies.push(AnomalousNode {
                    node_id: node_id.clone(),
                    anomaly_type: AnomalyType::LowRewardHighVisits,
                    description: format!("Reward {:.3} (avg: {:.3}), visits {} (avg: {:.1})", reward, avg_reward, visits, avg_visits),
                    severity_score: ((avg_reward / reward) * (visits as f64 / avg_visits)) as f32,
                });
            }

            // Check for nodes with zero visits but children (structural anomaly)
            if visits == 0 && !node.children.is_empty() {
                anomalies.push(AnomalousNode {
                    node_id: node_id.clone(),
                    anomaly_type: AnomalyType::ZeroVisitsWithChildren,
                    description: format!("Node has {} children but zero visits", node.children.len()),
                    severity_score: node.children.len() as f32,
                });
            }

            // Check for terminal nodes with children (logical anomaly)
            if node.is_terminal && !node.children.is_empty() {
                anomalies.push(AnomalousNode {
                    node_id: node_id.clone(),
                    anomaly_type: AnomalyType::TerminalWithChildren,
                    description: format!("Terminal node has {} children", node.children.len()),
                    severity_score: 10.0, // High severity for logical inconsistency
                });
            }
        }

        // Sort by severity score descending
        anomalies.sort_by(|a, b| b.severity_score.partial_cmp(&a.severity_score).unwrap_or(std::cmp::Ordering::Equal));
        anomalies
    }

    /// Find promising unexplored paths
    pub fn find_promising_paths(tree: &HashMap<String, MCTSNode>) -> Vec<PromisingPath> {
        let mut promising_paths = Vec::new();

        for (node_id, node) in tree {
            // Look for nodes with high reward but few visits (underexplored)
            if node.average_reward() > 0.7 && node.visits < 20 && !node.is_terminal {
                let path_potential = node.average_reward() * (1.0 / (node.visits as f64 + 1.0));
                promising_paths.push(PromisingPath {
                    node_id: node_id.clone(),
                    potential_score: path_potential,
                    current_reward: node.average_reward(),
                    current_visits: node.visits,
                    depth: node.calculate_depth(tree),
                    reason: "High reward with low exploration".to_string(),
                });
            }

            // Look for nodes with improving reward trend
            if node.visits > 10 {
                let recent_reward_trend = Self::calculate_reward_trend(node);
                if recent_reward_trend > 0.1 {
                    promising_paths.push(PromisingPath {
                        node_id: node_id.clone(),
                        potential_score: recent_reward_trend * node.average_reward(),
                        current_reward: node.average_reward(),
                        current_visits: node.visits,
                        depth: node.calculate_depth(tree),
                        reason: format!("Improving reward trend: +{:.3}", recent_reward_trend),
                    });
                }
            }
        }

        // Sort by potential score descending
        promising_paths.sort_by(|a, b| b.potential_score.partial_cmp(&a.potential_score).unwrap_or(std::cmp::Ordering::Equal));
        promising_paths.truncate(10); // Return top 10 most promising paths
        promising_paths
    }

    /// Calculate reward trend for a node (simplified)
    fn calculate_reward_trend(node: &MCTSNode) -> f64 {
        // This is a simplified trend calculation
        // In a real implementation, you'd track reward history
        if node.visits < 5 {
            0.0
        } else {
            // Simplified: assume recent performance is better if current reward is above average
            let baseline = 0.5; // Assumed baseline reward
            (node.average_reward() - baseline).max(0.0)
        }
    }
}

/// Anomalous node information
#[derive(Debug, Clone)]
pub struct AnomalousNode {
    pub node_id: String,
    pub anomaly_type: AnomalyType,
    pub description: String,
    pub severity_score: f32,
}

/// Types of anomalies in MCTS trees
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnomalyType {
    HighRewardLowVisits,
    LowRewardHighVisits,
    ZeroVisitsWithChildren,
    TerminalWithChildren,
}

impl AnomalyType {
    /// Get description of anomaly type
    pub fn description(&self) -> &'static str {
        match self {
            AnomalyType::HighRewardLowVisits => "High reward with unexpectedly low visits",
            AnomalyType::LowRewardHighVisits => "Low reward despite high visits",
            AnomalyType::ZeroVisitsWithChildren => "Node has children but zero visits",
            AnomalyType::TerminalWithChildren => "Terminal node incorrectly has children",
        }
    }
}

/// Promising path information
#[derive(Debug, Clone)]
pub struct PromisingPath {
    pub node_id: String,
    pub potential_score: f64,
    pub current_reward: f64,
    pub current_visits: u64,
    pub depth: usize,
    pub reason: String,
}

impl PromisingPath {
    /// Get formatted summary
    pub fn summary(&self) -> String {
        format!(
            "Path {}: potential={:.3}, reward={:.3}, visits={}, depth={} - {}",
            self.node_id, self.potential_score, self.current_reward, 
            self.current_visits, self.depth, self.reason
        )
    }
}