//! Types and enums for quantum selection algorithms
//!
//! This module provides the core types, enums, and result structures
//! used throughout the quantum selection system for strategy configuration.

/// Selection strategy enumeration for different use cases
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionStrategy {
    /// Standard quantum UCT selection
    QuantumUCT,
    /// Selection with entanglement network effects
    EntanglementAware,
    /// Multi-objective weighted selection
    MultiObjective,
    /// Performance-optimized selection (minimal computation)
    FastSelection,
}

impl Default for SelectionStrategy {
    fn default() -> Self {
        Self::QuantumUCT
    }
}

impl SelectionStrategy {
    /// Get strategy description
    pub fn description(&self) -> &'static str {
        match self {
            Self::QuantumUCT => "Standard quantum UCT selection with superposition and coherence",
            Self::EntanglementAware => "Selection enhanced with entanglement network effects",
            Self::MultiObjective => "Multi-objective weighted selection balancing multiple factors",
            Self::FastSelection => "Performance-optimized selection with minimal computation",
        }
    }
    
    /// Check if strategy uses entanglement effects
    pub fn uses_entanglement(&self) -> bool {
        matches!(self, Self::EntanglementAware)
    }
    
    /// Check if strategy is computationally intensive
    pub fn is_intensive(&self) -> bool {
        matches!(self, Self::EntanglementAware | Self::MultiObjective)
    }
    
    /// Get computational complexity score (0-3)
    pub fn complexity_score(&self) -> u8 {
        match self {
            Self::FastSelection => 0,
            Self::QuantumUCT => 1,
            Self::MultiObjective => 2,
            Self::EntanglementAware => 3,
        }
    }
}

/// Selection result with additional metadata and performance information
#[derive(Debug, Clone)]
pub struct SelectionResult {
    /// Selected node ID
    pub node_id: String,
    /// Selection confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Number of candidates considered
    pub candidates_count: usize,
    /// Selection strategy used
    pub strategy: SelectionStrategy,
    /// Computation time in microseconds
    pub computation_time_us: u64,
    /// Selection entropy (measure of choice diversity)
    pub entropy: f64,
    /// Whether unvisited node was selected
    pub selected_unvisited: bool,
}

impl SelectionResult {
    /// Create new selection result
    pub fn new(
        node_id: String,
        confidence: f64,
        candidates_count: usize,
        strategy: SelectionStrategy,
        computation_time_us: u64,
    ) -> Self {
        Self {
            node_id,
            confidence,
            candidates_count,
            strategy,
            computation_time_us,
            entropy: 0.0,
            selected_unvisited: false,
        }
    }
    
    /// Check if selection was fast (sub-millisecond)
    pub fn is_fast(&self) -> bool {
        self.computation_time_us < 1000
    }
    
    /// Check if selection was high confidence
    pub fn is_high_confidence(&self) -> bool {
        self.confidence > 0.8
    }
    
    /// Check if selection was diverse (high entropy)
    pub fn is_diverse(&self) -> bool {
        self.entropy > 1.0
    }
    
    /// Get performance grade based on speed and confidence
    pub fn performance_grade(&self) -> char {
        let speed_score = if self.is_fast() { 1.0 } else { 0.5 };
        let confidence_score = self.confidence;
        let combined_score = (speed_score + confidence_score) / 2.0;
        
        if combined_score >= 0.9 { 'A' }
        else if combined_score >= 0.8 { 'B' }
        else if combined_score >= 0.7 { 'C' }
        else if combined_score >= 0.6 { 'D' }
        else { 'F' }
    }
    
    /// Format result summary for logging
    pub fn summary(&self) -> String {
        format!(
            "Selected {} using {:?} (confidence: {:.2}, candidates: {}, time: {}μs)",
            self.node_id,
            self.strategy,
            self.confidence,
            self.candidates_count,
            self.computation_time_us
        )
    }
}

/// Selection parameters for fine-tuning algorithms
#[derive(Debug, Clone)]
pub struct SelectionParameters {
    /// Exploration weight for multi-objective selection
    pub exploration_weight: f64,
    /// Exploitation weight for multi-objective selection
    pub exploitation_weight: f64,
    /// Quantum weight for quantum effects
    pub quantum_weight: f64,
    /// Entanglement influence factor
    pub entanglement_influence: f64,
    /// Temperature for selection randomness
    pub temperature: f64,
}

impl Default for SelectionParameters {
    fn default() -> Self {
        Self {
            exploration_weight: 1.0,
            exploitation_weight: 1.0,
            quantum_weight: 0.5,
            entanglement_influence: 0.5,
            temperature: 1.0,
        }
    }
}

impl SelectionParameters {
    /// Create exploration-focused parameters
    pub fn exploration_focused() -> Self {
        Self {
            exploration_weight: 2.0,
            exploitation_weight: 0.5,
            quantum_weight: 0.8,
            entanglement_influence: 0.3,
            temperature: 1.5,
        }
    }
    
    /// Create exploitation-focused parameters
    pub fn exploitation_focused() -> Self {
        Self {
            exploration_weight: 0.3,
            exploitation_weight: 2.0,
            quantum_weight: 0.2,
            entanglement_influence: 0.1,
            temperature: 0.5,
        }
    }
    
    /// Create balanced parameters
    pub fn balanced() -> Self {
        Self::default()
    }
    
    /// Validate parameter ranges
    pub fn validate(&self) -> Result<(), String> {
        if self.exploration_weight < 0.0 || self.exploration_weight > 10.0 {
            return Err("Exploration weight must be between 0.0 and 10.0".to_string());
        }
        if self.exploitation_weight < 0.0 || self.exploitation_weight > 10.0 {
            return Err("Exploitation weight must be between 0.0 and 10.0".to_string());
        }
        if self.quantum_weight < 0.0 || self.quantum_weight > 5.0 {
            return Err("Quantum weight must be between 0.0 and 5.0".to_string());
        }
        if self.entanglement_influence < 0.0 || self.entanglement_influence > 2.0 {
            return Err("Entanglement influence must be between 0.0 and 2.0".to_string());
        }
        if self.temperature <= 0.0 || self.temperature > 10.0 {
            return Err("Temperature must be between 0.0 and 10.0".to_string());
        }
        Ok(())
    }
    
    /// Normalize weights to sum to specified total
    pub fn normalize_weights(&mut self, target_sum: f64) {
        let current_sum = self.exploration_weight + self.exploitation_weight + self.quantum_weight;
        if current_sum > 0.0 {
            let scale = target_sum / current_sum;
            self.exploration_weight *= scale;
            self.exploitation_weight *= scale;
            self.quantum_weight *= scale;
        }
    }
}

/// Selection statistics for performance analysis
#[derive(Debug, Clone, Default)]
pub struct SelectionStatistics {
    /// Total number of selections performed
    pub total_selections: u64,
    /// Total computation time in microseconds
    pub total_time_us: u64,
    /// Number of high-confidence selections
    pub high_confidence_count: u64,
    /// Number of fast selections (sub-millisecond)
    pub fast_selection_count: u64,
    /// Number of unvisited nodes selected
    pub unvisited_selected_count: u64,
    /// Strategy usage counts
    pub strategy_counts: [u64; 4], // Indexed by strategy complexity score
    /// Average entropy of selections
    pub average_entropy: f64,
    /// Maximum computation time seen
    pub max_time_us: u64,
    /// Minimum computation time seen
    pub min_time_us: u64,
}

impl SelectionStatistics {
    /// Create new empty statistics
    pub fn new() -> Self {
        Self {
            min_time_us: u64::MAX,
            ..Default::default()
        }
    }
    
    /// Record a selection result
    pub fn record_selection(&mut self, result: &SelectionResult) {
        self.total_selections += 1;
        self.total_time_us += result.computation_time_us;
        
        if result.is_high_confidence() {
            self.high_confidence_count += 1;
        }
        
        if result.is_fast() {
            self.fast_selection_count += 1;
        }
        
        if result.selected_unvisited {
            self.unvisited_selected_count += 1;
        }
        
        // Update strategy counts
        let strategy_index = result.strategy.complexity_score() as usize;
        if strategy_index < self.strategy_counts.len() {
            self.strategy_counts[strategy_index] += 1;
        }
        
        // Update entropy (running average)
        if self.total_selections == 1 {
            self.average_entropy = result.entropy;
        } else {
            self.average_entropy = (self.average_entropy * (self.total_selections - 1) as f64 + result.entropy) / self.total_selections as f64;
        }
        
        // Update time bounds
        self.max_time_us = self.max_time_us.max(result.computation_time_us);
        self.min_time_us = self.min_time_us.min(result.computation_time_us);
    }
    
    /// Calculate average computation time
    pub fn average_time_us(&self) -> f64 {
        if self.total_selections == 0 {
            0.0
        } else {
            self.total_time_us as f64 / self.total_selections as f64
        }
    }
    
    /// Calculate high confidence rate
    pub fn high_confidence_rate(&self) -> f64 {
        if self.total_selections == 0 {
            0.0
        } else {
            self.high_confidence_count as f64 / self.total_selections as f64
        }
    }
    
    /// Calculate fast selection rate
    pub fn fast_selection_rate(&self) -> f64 {
        if self.total_selections == 0 {
            0.0
        } else {
            self.fast_selection_count as f64 / self.total_selections as f64
        }
    }
    
    /// Calculate exploration rate (unvisited node selection)
    pub fn exploration_rate(&self) -> f64 {
        if self.total_selections == 0 {
            0.0
        } else {
            self.unvisited_selected_count as f64 / self.total_selections as f64
        }
    }
    
    /// Get most used strategy
    pub fn most_used_strategy(&self) -> SelectionStrategy {
        let max_index = self.strategy_counts
            .iter()
            .enumerate()
            .max_by_key(|&(_, count)| count)
            .map(|(i, _)| i)
            .unwrap_or(0);
        
        match max_index {
            0 => SelectionStrategy::FastSelection,
            1 => SelectionStrategy::QuantumUCT,
            2 => SelectionStrategy::MultiObjective,
            3 => SelectionStrategy::EntanglementAware,
            _ => SelectionStrategy::QuantumUCT,
        }
    }
    
    /// Reset all statistics
    pub fn reset(&mut self) {
        *self = Self::new();
    }
    
    /// Format statistics summary
    pub fn summary(&self) -> String {
        format!(
            "Selection Stats: {} total, avg time: {:.1}μs, confidence: {:.1}%, fast: {:.1}%, exploration: {:.1}%",
            self.total_selections,
            self.average_time_us(),
            self.high_confidence_rate() * 100.0,
            self.fast_selection_rate() * 100.0,
            self.exploration_rate() * 100.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_selection_strategy_methods() {
        assert!(SelectionStrategy::EntanglementAware.uses_entanglement());
        assert!(!SelectionStrategy::QuantumUCT.uses_entanglement());
        
        assert!(SelectionStrategy::EntanglementAware.is_intensive());
        assert!(!SelectionStrategy::FastSelection.is_intensive());
        
        assert_eq!(SelectionStrategy::FastSelection.complexity_score(), 0);
        assert_eq!(SelectionStrategy::EntanglementAware.complexity_score(), 3);
    }
    
    #[test]
    fn test_selection_result() {
        let result = SelectionResult::new(
            "test_node".to_string(),
            0.9,
            5,
            SelectionStrategy::QuantumUCT,
            500,
        );
        
        assert!(result.is_fast());
        assert!(result.is_high_confidence());
        assert_eq!(result.performance_grade(), 'A');
        
        let summary = result.summary();
        assert!(summary.contains("test_node"));
        assert!(summary.contains("QuantumUCT"));
    }
    
    #[test]
    fn test_selection_parameters() {
        let mut params = SelectionParameters::default();
        assert!(params.validate().is_ok());
        
        // Test validation
        params.exploration_weight = -1.0;
        assert!(params.validate().is_err());
        
        // Test normalization
        let mut params = SelectionParameters {
            exploration_weight: 2.0,
            exploitation_weight: 3.0,
            quantum_weight: 1.0,
            ..Default::default()
        };
        
        params.normalize_weights(3.0);
        let sum = params.exploration_weight + params.exploitation_weight + params.quantum_weight;
        assert!((sum - 3.0).abs() < 1e-10);
    }
    
    #[test]
    fn test_selection_statistics() {
        let mut stats = SelectionStatistics::new();
        
        let result1 = SelectionResult {
            node_id: "node1".to_string(),
            confidence: 0.9,
            candidates_count: 5,
            strategy: SelectionStrategy::QuantumUCT,
            computation_time_us: 500,
            entropy: 1.5,
            selected_unvisited: true,
        };
        
        let result2 = SelectionResult {
            node_id: "node2".to_string(),
            confidence: 0.5,
            candidates_count: 3,
            strategy: SelectionStrategy::FastSelection,
            computation_time_us: 2000,
            entropy: 0.8,
            selected_unvisited: false,
        };
        
        stats.record_selection(&result1);
        stats.record_selection(&result2);
        
        assert_eq!(stats.total_selections, 2);
        assert_eq!(stats.high_confidence_count, 1);
        assert_eq!(stats.fast_selection_count, 1);
        assert_eq!(stats.unvisited_selected_count, 1);
        
        assert_eq!(stats.average_time_us(), 1250.0);
        assert_eq!(stats.high_confidence_rate(), 0.5);
        assert_eq!(stats.fast_selection_rate(), 0.5);
        assert_eq!(stats.exploration_rate(), 0.5);
        
        let summary = stats.summary();
        assert!(summary.contains("2 total"));
    }
}