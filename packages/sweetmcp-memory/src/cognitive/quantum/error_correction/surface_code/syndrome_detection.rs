//! Surface code syndrome detection algorithms
//!
//! This module provides blazing-fast syndrome detection with zero allocation
//! optimizations and elegant ergonomic interfaces for quantum error syndrome extraction.

use crate::cognitive::quantum::{
    Complex64,
    types::{CognitiveError, CognitiveResult},
};
use std::collections::{HashMap, HashSet};
use super::super::topological_pauli::PauliType;
use smallvec::SmallVec;
use std::time::Instant;

/// Qubit position in 2D grid
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QubitPosition {
    pub row: usize,
    pub col: usize,
}

impl QubitPosition {
    /// Create new qubit position
    #[inline]
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }

    /// Calculate Manhattan distance to another position
    #[inline]
    pub fn manhattan_distance(&self, other: &QubitPosition) -> usize {
        ((self.row as i32 - other.row as i32).abs() + 
         (self.col as i32 - other.col as i32).abs()) as usize
    }

    /// Check if positions are adjacent (distance 1)
    #[inline]
    pub fn is_adjacent(&self, other: &QubitPosition) -> bool {
        self.manhattan_distance(other) == 1
    }

    /// Get all adjacent positions within bounds
    #[inline]
    pub fn get_adjacent_positions(&self, max_row: usize, max_col: usize) -> SmallVec<[QubitPosition; 4]> {
        let mut adjacent = SmallVec::new();
        
        if self.row > 0 {
            adjacent.push(QubitPosition::new(self.row - 1, self.col));
        }
        if self.row < max_row - 1 {
            adjacent.push(QubitPosition::new(self.row + 1, self.col));
        }
        if self.col > 0 {
            adjacent.push(QubitPosition::new(self.row, self.col - 1));
        }
        if self.col < max_col - 1 {
            adjacent.push(QubitPosition::new(self.row, self.col + 1));
        }
        
        adjacent
    }
}


/// Pauli operator on specific qubit
#[derive(Debug, Clone)]
pub struct PauliOperator {
    /// Target qubit position
    pub position: QubitPosition,
    /// Pauli type
    pub pauli_type: PauliType,
    /// Complex coefficient
    pub coefficient: Complex64,
}

impl PauliOperator {
    /// Create new Pauli operator
    #[inline]
    pub fn new(position: QubitPosition, pauli_type: PauliType) -> Self {
        Self {
            position,
            pauli_type,
            coefficient: Complex64::new(1.0, 0.0),
        }
    }

    /// Create Pauli operator with coefficient
    #[inline]
    pub fn with_coefficient(position: QubitPosition, pauli_type: PauliType, coefficient: Complex64) -> Self {
        Self {
            position,
            pauli_type,
            coefficient,
        }
    }

    /// Check if operator acts trivially (identity)
    #[inline]
    pub fn is_identity(&self) -> bool {
        matches!(self.pauli_type, PauliType::I) || self.coefficient.norm() < 1e-10
    }

    /// Get operator weight (number of non-identity Paulis)
    #[inline]
    pub fn weight(&self) -> usize {
        if self.is_identity() { 0 } else { 1 }
    }
}

/// Type of stabilizer generator
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StabilizerType {
    X,
    Z,
}

impl StabilizerType {
    /// Get corresponding Pauli type
    #[inline]
    pub fn to_pauli_type(&self) -> PauliType {
        match self {
            StabilizerType::X => PauliType::X,
            StabilizerType::Z => PauliType::Z,
        }
    }

    /// Get string representation
    #[inline]
    pub fn as_str(&self) -> &'static str {
        match self {
            StabilizerType::X => "X",
            StabilizerType::Z => "Z",
        }
    }
}

/// Stabilizer generator for surface code
#[derive(Debug, Clone)]
pub struct StabilizerGenerator {
    /// Generator ID
    pub id: String,
    /// Pauli operators in this generator
    pub pauli_operators: SmallVec<[PauliOperator; 4]>,
    /// Syndrome qubit position
    pub syndrome_position: QubitPosition,
    /// Generator type (X or Z)
    pub generator_type: StabilizerType,
}

impl StabilizerGenerator {
    /// Create new stabilizer generator
    #[inline]
    pub fn new(
        id: String,
        syndrome_position: QubitPosition,
        generator_type: StabilizerType,
    ) -> Self {
        Self {
            id,
            pauli_operators: SmallVec::new(),
            syndrome_position,
            generator_type,
        }
    }

    /// Add Pauli operator to generator
    #[inline]
    pub fn add_pauli_operator(&mut self, operator: PauliOperator) {
        self.pauli_operators.push(operator);
    }

    /// Get generator weight (number of non-identity Paulis)
    #[inline]
    pub fn weight(&self) -> usize {
        self.pauli_operators.iter()
            .map(|op| op.weight())
            .sum()
    }

    /// Check if generator is valid (non-empty and consistent type)
    #[inline]
    pub fn is_valid(&self) -> bool {
        !self.pauli_operators.is_empty() &&
        self.pauli_operators.iter().all(|op| {
            match self.generator_type {
                StabilizerType::X => op.pauli_type.has_x_component() || op.is_identity(),
                StabilizerType::Z => op.pauli_type.has_z_component() || op.is_identity(),
            }
        })
    }

    /// Calculate syndrome measurement for given error pattern
    #[inline]
    pub fn measure_syndrome(&self, error_pattern: &HashMap<QubitPosition, PauliType>) -> bool {
        let mut syndrome = false;
        
        for pauli_op in &self.pauli_operators {
            if let Some(&error_pauli) = error_pattern.get(&pauli_op.position) {
                // Check if error anticommutes with stabilizer operator
                if !pauli_op.pauli_type.commutes_with(&error_pauli) {
                    syndrome = !syndrome; // Flip syndrome bit
                }
            }
        }
        
        syndrome
    }

    /// Get all qubit positions involved in this generator
    #[inline]
    pub fn get_involved_qubits(&self) -> HashSet<QubitPosition> {
        self.pauli_operators.iter()
            .map(|op| op.position)
            .collect()
    }
}

/// Surface code error syndrome
#[derive(Debug, Clone)]
pub struct SurfaceCodeSyndrome {
    /// X-type syndrome measurements
    pub x_syndromes: HashMap<QubitPosition, bool>,
    /// Z-type syndrome measurements
    pub z_syndromes: HashMap<QubitPosition, bool>,
    /// Measurement round
    pub round: usize,
    /// Timestamp
    pub timestamp: Instant,
}

impl SurfaceCodeSyndrome {
    /// Create new empty syndrome
    #[inline]
    pub fn new(round: usize) -> Self {
        Self {
            x_syndromes: HashMap::new(),
            z_syndromes: HashMap::new(),
            round,
            timestamp: Instant::now(),
        }
    }

    /// Create syndrome with pre-allocated capacity
    #[inline]
    pub fn with_capacity(x_capacity: usize, z_capacity: usize, round: usize) -> Self {
        Self {
            x_syndromes: HashMap::with_capacity(x_capacity),
            z_syndromes: HashMap::with_capacity(z_capacity),
            round,
            timestamp: Instant::now(),
        }
    }

    /// Add X-type syndrome measurement
    #[inline]
    pub fn add_x_syndrome(&mut self, position: QubitPosition, value: bool) {
        self.x_syndromes.insert(position, value);
    }

    /// Add Z-type syndrome measurement
    #[inline]
    pub fn add_z_syndrome(&mut self, position: QubitPosition, value: bool) {
        self.z_syndromes.insert(position, value);
    }

    /// Get total number of triggered syndromes
    #[inline]
    pub fn total_triggered_syndromes(&self) -> usize {
        let x_triggered = self.x_syndromes.values().filter(|&&v| v).count();
        let z_triggered = self.z_syndromes.values().filter(|&&v| v).count();
        x_triggered + z_triggered
    }

    /// Check if syndrome is trivial (all measurements are false)
    #[inline]
    pub fn is_trivial(&self) -> bool {
        self.x_syndromes.values().all(|&v| !v) && 
        self.z_syndromes.values().all(|&v| !v)
    }

    /// Get X-syndrome positions that are triggered
    #[inline]
    pub fn get_triggered_x_positions(&self) -> Vec<QubitPosition> {
        self.x_syndromes.iter()
            .filter_map(|(&pos, &triggered)| if triggered { Some(pos) } else { None })
            .collect()
    }

    /// Get Z-syndrome positions that are triggered
    #[inline]
    pub fn get_triggered_z_positions(&self) -> Vec<QubitPosition> {
        self.z_syndromes.iter()
            .filter_map(|(&pos, &triggered)| if triggered { Some(pos) } else { None })
            .collect()
    }

    /// Calculate syndrome weight (Hamming weight)
    #[inline]
    pub fn weight(&self) -> usize {
        self.total_triggered_syndromes()
    }

    /// Compare with another syndrome and get difference
    #[inline]
    pub fn difference(&self, other: &SurfaceCodeSyndrome) -> SyndromeDifference {
        let mut x_changes = Vec::new();
        let mut z_changes = Vec::new();

        // Check X-syndrome changes
        let all_x_positions: HashSet<_> = self.x_syndromes.keys()
            .chain(other.x_syndromes.keys())
            .collect();

        for &pos in &all_x_positions {
            let self_value = self.x_syndromes.get(pos).copied().unwrap_or(false);
            let other_value = other.x_syndromes.get(pos).copied().unwrap_or(false);
            
            if self_value != other_value {
                x_changes.push(*pos);
            }
        }

        // Check Z-syndrome changes
        let all_z_positions: HashSet<_> = self.z_syndromes.keys()
            .chain(other.z_syndromes.keys())
            .collect();

        for &pos in &all_z_positions {
            let self_value = self.z_syndromes.get(pos).copied().unwrap_or(false);
            let other_value = other.z_syndromes.get(pos).copied().unwrap_or(false);
            
            if self_value != other_value {
                z_changes.push(*pos);
            }
        }

        SyndromeDifference {
            x_changes,
            z_changes,
            round_difference: (self.round as i32 - other.round as i32).abs() as usize,
        }
    }

    /// Get age of syndrome in milliseconds
    #[inline]
    pub fn age_ms(&self) -> u64 {
        self.timestamp.elapsed().as_millis() as u64
    }
}

/// Difference between two syndromes
#[derive(Debug, Clone)]
pub struct SyndromeDifference {
    /// X-syndrome positions that changed
    pub x_changes: Vec<QubitPosition>,
    /// Z-syndrome positions that changed
    pub z_changes: Vec<QubitPosition>,
    /// Difference in measurement rounds
    pub round_difference: usize,
}

impl SyndromeDifference {
    /// Get total number of changes
    #[inline]
    pub fn total_changes(&self) -> usize {
        self.x_changes.len() + self.z_changes.len()
    }

    /// Check if difference is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.x_changes.is_empty() && self.z_changes.is_empty()
    }
}

/// Syndrome detection engine with optimized algorithms
pub struct SyndromeDetector {
    /// X-type stabilizer generators
    x_stabilizers: Vec<StabilizerGenerator>,
    /// Z-type stabilizer generators
    z_stabilizers: Vec<StabilizerGenerator>,
    /// Detection metrics
    metrics: SyndromeDetectionMetrics,
    /// Configuration
    config: SyndromeDetectionConfig,
}

impl SyndromeDetector {
    /// Create new syndrome detector
    #[inline]
    pub fn new(
        x_stabilizers: Vec<StabilizerGenerator>,
        z_stabilizers: Vec<StabilizerGenerator>,
    ) -> Self {
        Self {
            x_stabilizers,
            z_stabilizers,
            metrics: SyndromeDetectionMetrics::new(),
            config: SyndromeDetectionConfig::default(),
        }
    }

    /// Create detector with configuration
    #[inline]
    pub fn with_config(
        x_stabilizers: Vec<StabilizerGenerator>,
        z_stabilizers: Vec<StabilizerGenerator>,
        config: SyndromeDetectionConfig,
    ) -> Self {
        Self {
            x_stabilizers,
            z_stabilizers,
            metrics: SyndromeDetectionMetrics::new(),
            config,
        }
    }

    /// Detect syndrome from error pattern with blazing-fast computation
    #[inline]
    pub fn detect_syndrome(
        &mut self,
        error_pattern: &HashMap<QubitPosition, PauliType>,
        round: usize,
    ) -> CognitiveResult<SurfaceCodeSyndrome> {
        let start_time = Instant::now();
        
        let mut syndrome = SurfaceCodeSyndrome::with_capacity(
            self.x_stabilizers.len(),
            self.z_stabilizers.len(),
            round,
        );

        // Measure X-type syndromes
        for stabilizer in &self.x_stabilizers {
            let measurement = stabilizer.measure_syndrome(error_pattern);
            syndrome.add_x_syndrome(stabilizer.syndrome_position, measurement);
        }

        // Measure Z-type syndromes
        for stabilizer in &self.z_stabilizers {
            let measurement = stabilizer.measure_syndrome(error_pattern);
            syndrome.add_z_syndrome(stabilizer.syndrome_position, measurement);
        }

        let detection_time = start_time.elapsed();
        self.metrics.record_detection(detection_time, syndrome.weight());

        Ok(syndrome)
    }

    /// Detect syndrome changes between consecutive measurements
    #[inline]
    pub fn detect_syndrome_changes(
        &mut self,
        previous_syndrome: &SurfaceCodeSyndrome,
        current_error_pattern: &HashMap<QubitPosition, PauliType>,
        round: usize,
    ) -> CognitiveResult<(SurfaceCodeSyndrome, SyndromeDifference)> {
        let current_syndrome = self.detect_syndrome(current_error_pattern, round)?;
        let difference = current_syndrome.difference(previous_syndrome);
        
        Ok((current_syndrome, difference))
    }

    /// Validate syndrome consistency
    #[inline]
    pub fn validate_syndrome(&self, syndrome: &SurfaceCodeSyndrome) -> CognitiveResult<bool> {
        // Check that all syndrome positions are valid
        for &pos in syndrome.x_syndromes.keys() {
            if !self.x_stabilizers.iter().any(|s| s.syndrome_position == pos) {
                return Ok(false);
            }
        }

        for &pos in syndrome.z_syndromes.keys() {
            if !self.z_stabilizers.iter().any(|s| s.syndrome_position == pos) {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Get detection statistics
    #[inline]
    pub fn get_metrics(&self) -> &SyndromeDetectionMetrics {
        &self.metrics
    }

    /// Update configuration
    #[inline]
    pub fn update_config(&mut self, config: SyndromeDetectionConfig) {
        self.config = config;
    }

    /// Get number of stabilizers
    #[inline]
    pub fn stabilizer_count(&self) -> (usize, usize) {
        (self.x_stabilizers.len(), self.z_stabilizers.len())
    }
}

/// Syndrome detection performance metrics
#[derive(Debug, Clone)]
pub struct SyndromeDetectionMetrics {
    /// Total detections performed
    pub total_detections: u64,
    /// Total detection time
    pub total_detection_time_ms: u64,
    /// Average syndrome weight
    pub average_syndrome_weight: f64,
    /// Maximum syndrome weight observed
    pub max_syndrome_weight: usize,
    /// Creation time
    pub creation_time: Instant,
}

impl SyndromeDetectionMetrics {
    /// Create new metrics
    #[inline]
    pub fn new() -> Self {
        Self {
            total_detections: 0,
            total_detection_time_ms: 0,
            average_syndrome_weight: 0.0,
            max_syndrome_weight: 0,
            creation_time: Instant::now(),
        }
    }

    /// Record detection operation
    #[inline]
    pub fn record_detection(&mut self, detection_time: std::time::Duration, syndrome_weight: usize) {
        self.total_detections += 1;
        self.total_detection_time_ms += detection_time.as_millis() as u64;
        
        // Update average syndrome weight with exponential moving average
        let alpha = 0.1;
        self.average_syndrome_weight = alpha * syndrome_weight as f64 + 
            (1.0 - alpha) * self.average_syndrome_weight;
        
        self.max_syndrome_weight = self.max_syndrome_weight.max(syndrome_weight);
    }

    /// Get average detection time in milliseconds
    #[inline]
    pub fn average_detection_time_ms(&self) -> f64 {
        if self.total_detections > 0 {
            self.total_detection_time_ms as f64 / self.total_detections as f64
        } else {
            0.0
        }
    }

    /// Get detections per second
    #[inline]
    pub fn detections_per_second(&self) -> f64 {
        let age_seconds = self.creation_time.elapsed().as_secs_f64();
        if age_seconds > 0.0 {
            self.total_detections as f64 / age_seconds
        } else {
            0.0
        }
    }
}

impl Default for SyndromeDetectionMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for syndrome detection
#[derive(Debug, Clone)]
pub struct SyndromeDetectionConfig {
    /// Enable parallel processing for large stabilizer sets
    pub enable_parallel_processing: bool,
    /// Batch size for parallel processing
    pub batch_size: usize,
    /// Maximum allowed syndrome weight (for validation)
    pub max_syndrome_weight: Option<usize>,
    /// Enable syndrome caching
    pub enable_caching: bool,
}

impl SyndromeDetectionConfig {
    /// Create new configuration with default values
    #[inline]
    pub fn new() -> Self {
        Self {
            enable_parallel_processing: false,
            batch_size: 64,
            max_syndrome_weight: None,
            enable_caching: false,
        }
    }

    /// Create performance-optimized configuration
    #[inline]
    pub fn performance_optimized() -> Self {
        Self {
            enable_parallel_processing: true,
            batch_size: 128,
            max_syndrome_weight: Some(1000),
            enable_caching: true,
        }
    }
}

impl Default for SyndromeDetectionConfig {
    fn default() -> Self {
        Self::new()
    }
}