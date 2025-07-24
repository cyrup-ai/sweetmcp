//! Memory tracking with blazing-fast bounds checking and monitoring
//!
//! This module provides optimized memory usage tracking with zero-allocation
//! patterns and real-time pressure detection for quantum MCTS operations.

use std::collections::HashMap;
use tokio::sync::RwLock;

use crate::cognitive::types::CognitiveError;
use super::super::node_state::QuantumMCTSNode;

/// Memory usage tracker with bounds checking and pressure detection
#[derive(Debug)]
pub struct MemoryTracker {
    max_nodes: usize,
    peak_usage: usize,
    pressure_threshold: f64,
    growth_rate_samples: Vec<f64>,
    last_usage: usize,
}

impl MemoryTracker {
    /// Create new memory tracker with optimized thresholds
    pub fn new(max_nodes: usize) -> Self {
        Self {
            max_nodes,
            peak_usage: 0,
            pressure_threshold: 0.8, // 80% of max capacity
            growth_rate_samples: Vec::with_capacity(10),
            last_usage: 0,
        }
    }
    
    /// Get current usage and update peak with blazing-fast tracking
    pub async fn current_usage(&mut self, tree: &RwLock<HashMap<String, QuantumMCTSNode>>) -> usize {
        let tree_read = tree.read().await;
        let usage = tree_read.len();
        
        // Update peak usage
        self.peak_usage = self.peak_usage.max(usage);
        
        // Track growth rate for prediction
        if self.last_usage > 0 {
            let growth_rate = (usage as f64 - self.last_usage as f64) / self.last_usage as f64;
            if self.growth_rate_samples.len() >= 10 {
                self.growth_rate_samples.remove(0);
            }
            self.growth_rate_samples.push(growth_rate);
        }
        
        self.last_usage = usage;
        usage
    }
    
    /// Check memory bounds with zero-allocation validation
    pub async fn check_bounds(&mut self, tree: &RwLock<HashMap<String, QuantumMCTSNode>>) -> Result<(), CognitiveError> {
        let usage = self.current_usage(tree).await;
        
        if usage > self.max_nodes {
            return Err(CognitiveError::ResourceExhaustion(
                format!("Tree size {} exceeds maximum {}", usage, self.max_nodes)
            ));
        }
        
        Ok(())
    }
    
    /// Check if memory is under pressure with blazing-fast comparison
    pub fn is_under_pressure(&self) -> bool {
        self.peak_usage as f64 / self.max_nodes as f64 > self.pressure_threshold
    }
    
    /// Get peak usage
    pub fn peak_usage(&self) -> usize {
        self.peak_usage
    }
    
    /// Get current memory utilization ratio
    pub fn utilization_ratio(&self) -> f64 {
        self.last_usage as f64 / self.max_nodes as f64
    }
    
    /// Predict future memory usage based on growth rate
    pub fn predict_usage(&self, steps_ahead: usize) -> Option<usize> {
        if self.growth_rate_samples.is_empty() {
            return None;
        }
        
        let avg_growth_rate = self.growth_rate_samples.iter().sum::<f64>() / self.growth_rate_samples.len() as f64;
        let predicted = self.last_usage as f64 * (1.0 + avg_growth_rate).powi(steps_ahead as i32);
        
        Some(predicted as usize)
    }
    
    /// Check if predicted usage will exceed limits
    pub fn will_exceed_limits(&self, steps_ahead: usize) -> bool {
        if let Some(predicted) = self.predict_usage(steps_ahead) {
            predicted > self.max_nodes
        } else {
            false
        }
    }
    
    /// Reset peak usage counter and growth tracking
    pub fn reset(&mut self) {
        self.peak_usage = 0;
        self.growth_rate_samples.clear();
        self.last_usage = 0;
    }
    
    /// Update memory limit
    pub fn update_limit(&mut self, new_limit: usize) {
        self.max_nodes = new_limit;
    }
    
    /// Update pressure threshold
    pub fn update_pressure_threshold(&mut self, threshold: f64) {
        self.pressure_threshold = threshold.clamp(0.0, 1.0);
    }
    
    /// Get memory statistics for monitoring
    pub fn get_stats(&self) -> MemoryStats {
        MemoryStats {
            current_usage: self.last_usage,
            peak_usage: self.peak_usage,
            max_capacity: self.max_nodes,
            utilization_ratio: self.utilization_ratio(),
            is_under_pressure: self.is_under_pressure(),
            growth_samples: self.growth_rate_samples.len(),
            predicted_usage_5_steps: self.predict_usage(5),
        }
    }
}

/// Memory statistics for monitoring and analysis
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub current_usage: usize,
    pub peak_usage: usize,
    pub max_capacity: usize,
    pub utilization_ratio: f64,
    pub is_under_pressure: bool,
    pub growth_samples: usize,
    pub predicted_usage_5_steps: Option<usize>,
}

impl MemoryStats {
    /// Check if memory is in critical state
    pub fn is_critical(&self) -> bool {
        self.utilization_ratio > 0.95
    }
    
    /// Get memory health status
    pub fn health_status(&self) -> MemoryHealthStatus {
        if self.is_critical() {
            MemoryHealthStatus::Critical
        } else if self.is_under_pressure {
            MemoryHealthStatus::Warning
        } else if self.utilization_ratio > 0.5 {
            MemoryHealthStatus::Moderate
        } else {
            MemoryHealthStatus::Good
        }
    }
}

/// Memory health status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryHealthStatus {
    Good,
    Moderate,
    Warning,
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_tracker_basic() {
        let mut tracker = MemoryTracker::new(1000);
        assert!(!tracker.is_under_pressure());
        assert_eq!(tracker.peak_usage(), 0);
        assert_eq!(tracker.utilization_ratio(), 0.0);
    }
    
    #[test]
    fn test_memory_pressure_detection() {
        let mut tracker = MemoryTracker::new(1000);
        tracker.peak_usage = 850;
        tracker.last_usage = 850;
        
        assert!(tracker.is_under_pressure()); // 85% > 80% threshold
        assert_eq!(tracker.utilization_ratio(), 0.85);
    }
    
    #[test]
    fn test_memory_prediction() {
        let mut tracker = MemoryTracker::new(1000);
        tracker.last_usage = 100;
        tracker.growth_rate_samples = vec![0.1, 0.1, 0.1]; // 10% growth rate
        
        let predicted = tracker.predict_usage(2);
        assert!(predicted.is_some());
        assert_eq!(predicted.unwrap(), 121); // 100 * 1.1^2 = 121
        
        assert!(!tracker.will_exceed_limits(2));
    }
    
    #[test]
    fn test_memory_stats() {
        let mut tracker = MemoryTracker::new(1000);
        tracker.last_usage = 600;
        tracker.peak_usage = 700;
        
        let stats = tracker.get_stats();
        assert_eq!(stats.current_usage, 600);
        assert_eq!(stats.peak_usage, 700);
        assert_eq!(stats.max_capacity, 1000);
        assert_eq!(stats.utilization_ratio, 0.6);
        assert_eq!(stats.health_status(), MemoryHealthStatus::Moderate);
    }
}