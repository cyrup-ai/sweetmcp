//! Surface code layout management
//!
//! This module provides blazing-fast layout management with zero allocation
//! optimizations and elegant ergonomic interfaces for surface code qubit layouts.

use crate::cognitive::quantum::{
    Complex64,
    types::{CognitiveError, CognitiveResult},
};
use std::collections::{HashMap, HashSet};
use smallvec::SmallVec;
use std::time::Instant;

use super::syndrome_detection::{QubitPosition, PauliOperator, StabilizerGenerator, StabilizerType};
use super::super::topological_pauli::PauliType;
use super::super::topological_types::BoundaryType;


/// 2D layout for surface code qubits
#[derive(Debug, Clone)]
pub struct SurfaceCodeLayout {
    /// Grid dimensions (rows x columns)
    pub dimensions: (usize, usize),
    /// Data qubit positions
    pub data_qubits: Vec<QubitPosition>,
    /// X-syndrome qubit positions
    pub x_syndrome_qubits: Vec<QubitPosition>,
    /// Z-syndrome qubit positions
    pub z_syndrome_qubits: Vec<QubitPosition>,
    /// Boundary conditions
    pub boundary_type: BoundaryType,
    /// Layout metrics
    pub metrics: LayoutMetrics,
}

impl SurfaceCodeLayout {
    /// Create new surface code layout
    #[inline]
    pub fn new(dimensions: (usize, usize), boundary_type: BoundaryType) -> CognitiveResult<Self> {
        let mut layout = Self {
            dimensions,
            data_qubits: Vec::new(),
            x_syndrome_qubits: Vec::new(),
            z_syndrome_qubits: Vec::new(),
            boundary_type,
            metrics: LayoutMetrics::new(),
        };

        layout.generate_qubit_positions()?;
        layout.calculate_metrics();

        Ok(layout)
    }

    /// Generate qubit positions for the layout
    #[inline]
    fn generate_qubit_positions(&mut self) -> CognitiveResult<()> {
        let (rows, cols) = self.dimensions;
        
        // Reserve capacity for efficiency
        let estimated_data_qubits = (rows * cols) / 2;
        let estimated_syndrome_qubits = estimated_data_qubits / 2;
        
        self.data_qubits.reserve(estimated_data_qubits);
        self.x_syndrome_qubits.reserve(estimated_syndrome_qubits);
        self.z_syndrome_qubits.reserve(estimated_syndrome_qubits);

        // Generate positions based on checkerboard pattern
        for row in 0..rows {
            for col in 0..cols {
                let position = QubitPosition::new(row, col);
                
                if self.is_data_qubit_position(row, col) {
                    self.data_qubits.push(position);
                } else if self.is_x_syndrome_position(row, col) {
                    self.x_syndrome_qubits.push(position);
                } else if self.is_z_syndrome_position(row, col) {
                    self.z_syndrome_qubits.push(position);
                }
            }
        }

        Ok(())
    }

    /// Check if position is a data qubit
    #[inline]
    fn is_data_qubit_position(&self, row: usize, col: usize) -> bool {
        // Data qubits are on even positions in checkerboard pattern
        (row + col) % 2 == 0
    }

    /// Check if position is an X-syndrome qubit
    #[inline]
    fn is_x_syndrome_position(&self, row: usize, col: usize) -> bool {
        // X-syndrome qubits are on odd positions with specific pattern
        (row + col) % 2 == 1 && row % 2 == 0
    }

    /// Check if position is a Z-syndrome qubit
    #[inline]
    fn is_z_syndrome_position(&self, row: usize, col: usize) -> bool {
        // Z-syndrome qubits are on odd positions with specific pattern
        (row + col) % 2 == 1 && row % 2 == 1
    }

    /// Get all qubit positions
    #[inline]
    pub fn get_all_positions(&self) -> Vec<QubitPosition> {
        let mut positions = Vec::with_capacity(
            self.data_qubits.len() + self.x_syndrome_qubits.len() + self.z_syndrome_qubits.len()
        );
        
        positions.extend(&self.data_qubits);
        positions.extend(&self.x_syndrome_qubits);
        positions.extend(&self.z_syndrome_qubits);
        
        positions
    }

    /// Get neighbors of a qubit position
    #[inline]
    pub fn get_neighbors(&self, position: QubitPosition) -> SmallVec<[QubitPosition; 4]> {
        let (max_row, max_col) = self.dimensions;
        
        match self.boundary_type {
            BoundaryType::Open => {
                position.get_adjacent_positions(max_row, max_col)
            }
            BoundaryType::Periodic => {
                self.get_periodic_neighbors(position)
            }
            BoundaryType::Twisted => {
                self.get_twisted_neighbors(position)
            }
        }
    }

    /// Get neighbors with periodic boundary conditions
    #[inline]
    fn get_periodic_neighbors(&self, position: QubitPosition) -> SmallVec<[QubitPosition; 4]> {
        let (max_row, max_col) = self.dimensions;
        let mut neighbors = SmallVec::new();
        
        // Up neighbor (with wraparound)
        let up_row = if position.row == 0 { max_row - 1 } else { position.row - 1 };
        neighbors.push(QubitPosition::new(up_row, position.col));
        
        // Down neighbor (with wraparound)
        let down_row = if position.row == max_row - 1 { 0 } else { position.row + 1 };
        neighbors.push(QubitPosition::new(down_row, position.col));
        
        // Left neighbor (with wraparound)
        let left_col = if position.col == 0 { max_col - 1 } else { position.col - 1 };
        neighbors.push(QubitPosition::new(position.row, left_col));
        
        // Right neighbor (with wraparound)
        let right_col = if position.col == max_col - 1 { 0 } else { position.col + 1 };
        neighbors.push(QubitPosition::new(position.row, right_col));
        
        neighbors
    }

    /// Get neighbors with twisted boundary conditions
    #[inline]
    fn get_twisted_neighbors(&self, position: QubitPosition) -> SmallVec<[QubitPosition; 4]> {
        let (max_row, max_col) = self.dimensions;
        let mut neighbors = SmallVec::new();
        
        // Twisted boundaries introduce phase factors - simplified implementation
        // Up neighbor
        let up_row = if position.row == 0 { max_row - 1 } else { position.row - 1 };
        neighbors.push(QubitPosition::new(up_row, position.col));
        
        // Down neighbor
        let down_row = if position.row == max_row - 1 { 0 } else { position.row + 1 };
        neighbors.push(QubitPosition::new(down_row, position.col));
        
        // Left neighbor with twist
        let left_col = if position.col == 0 { max_col - 1 } else { position.col - 1 };
        let left_row = if position.col == 0 && position.row < max_row / 2 { 
            max_row - 1 - position.row 
        } else { 
            position.row 
        };
        neighbors.push(QubitPosition::new(left_row, left_col));
        
        // Right neighbor with twist
        let right_col = if position.col == max_col - 1 { 0 } else { position.col + 1 };
        let right_row = if position.col == max_col - 1 && position.row < max_row / 2 { 
            max_row - 1 - position.row 
        } else { 
            position.row 
        };
        neighbors.push(QubitPosition::new(right_row, right_col));
        
        neighbors
    }

    /// Check if two positions are neighbors
    #[inline]
    pub fn are_neighbors(&self, pos1: QubitPosition, pos2: QubitPosition) -> bool {
        self.get_neighbors(pos1).contains(&pos2)
    }

    /// Get distance between two positions
    #[inline]
    pub fn get_distance(&self, pos1: QubitPosition, pos2: QubitPosition) -> usize {
        match self.boundary_type {
            BoundaryType::Open => pos1.manhattan_distance(&pos2),
            BoundaryType::Periodic => self.get_periodic_distance(pos1, pos2),
            BoundaryType::Twisted => self.get_twisted_distance(pos1, pos2),
        }
    }

    /// Get distance with periodic boundaries
    #[inline]
    fn get_periodic_distance(&self, pos1: QubitPosition, pos2: QubitPosition) -> usize {
        let (max_row, max_col) = self.dimensions;
        
        let row_dist = std::cmp::min(
            (pos1.row as i32 - pos2.row as i32).abs() as usize,
            max_row - (pos1.row as i32 - pos2.row as i32).abs() as usize,
        );
        
        let col_dist = std::cmp::min(
            (pos1.col as i32 - pos2.col as i32).abs() as usize,
            max_col - (pos1.col as i32 - pos2.col as i32).abs() as usize,
        );
        
        row_dist + col_dist
    }

    /// Get distance with twisted boundaries
    #[inline]
    fn get_twisted_distance(&self, pos1: QubitPosition, pos2: QubitPosition) -> usize {
        // Simplified twisted distance calculation
        self.get_periodic_distance(pos1, pos2)
    }

    /// Generate stabilizer generators for the layout
    #[inline]
    pub fn generate_stabilizers(&self) -> CognitiveResult<(Vec<StabilizerGenerator>, Vec<StabilizerGenerator>)> {
        let mut x_stabilizers = Vec::with_capacity(self.x_syndrome_qubits.len());
        let mut z_stabilizers = Vec::with_capacity(self.z_syndrome_qubits.len());

        // Generate X-type stabilizers
        for &syndrome_pos in &self.x_syndrome_qubits {
            let mut pauli_ops = SmallVec::new();
            
            // Add X operators on neighboring data qubits
            for &data_pos in &self.data_qubits {
                if self.are_neighbors(syndrome_pos, data_pos) {
                    pauli_ops.push(PauliOperator::new(data_pos, PauliType::X));
                }
            }

            if !pauli_ops.is_empty() {
                let mut stabilizer = StabilizerGenerator::new(
                    format!("X_{}_{}", syndrome_pos.row, syndrome_pos.col),
                    syndrome_pos,
                    StabilizerType::X,
                );
                
                for op in pauli_ops {
                    stabilizer.add_pauli_operator(op);
                }
                
                x_stabilizers.push(stabilizer);
            }
        }

        // Generate Z-type stabilizers
        for &syndrome_pos in &self.z_syndrome_qubits {
            let mut pauli_ops = SmallVec::new();
            
            // Add Z operators on neighboring data qubits
            for &data_pos in &self.data_qubits {
                if self.are_neighbors(syndrome_pos, data_pos) {
                    pauli_ops.push(PauliOperator::new(data_pos, PauliType::Z));
                }
            }

            if !pauli_ops.is_empty() {
                let mut stabilizer = StabilizerGenerator::new(
                    format!("Z_{}_{}", syndrome_pos.row, syndrome_pos.col),
                    syndrome_pos,
                    StabilizerType::Z,
                );
                
                for op in pauli_ops {
                    stabilizer.add_pauli_operator(op);
                }
                
                z_stabilizers.push(stabilizer);
            }
        }

        Ok((x_stabilizers, z_stabilizers))
    }

    /// Calculate layout metrics
    #[inline]
    fn calculate_metrics(&mut self) {
        self.metrics.total_qubits = self.data_qubits.len() + 
            self.x_syndrome_qubits.len() + 
            self.z_syndrome_qubits.len();
        
        self.metrics.data_qubit_count = self.data_qubits.len();
        self.metrics.syndrome_qubit_count = self.x_syndrome_qubits.len() + self.z_syndrome_qubits.len();
        
        // Calculate average connectivity
        let mut total_connections = 0;
        for &pos in &self.data_qubits {
            total_connections += self.get_neighbors(pos).len();
        }
        
        self.metrics.average_connectivity = if !self.data_qubits.is_empty() {
            total_connections as f64 / self.data_qubits.len() as f64
        } else {
            0.0
        };
        
        // Calculate code distance (simplified)
        self.metrics.code_distance = std::cmp::min(self.dimensions.0, self.dimensions.1);
        
        // Calculate layout efficiency
        let theoretical_max_qubits = self.dimensions.0 * self.dimensions.1;
        self.metrics.layout_efficiency = if theoretical_max_qubits > 0 {
            self.metrics.total_qubits as f64 / theoretical_max_qubits as f64
        } else {
            0.0
        };
    }

    /// Validate layout consistency
    #[inline]
    pub fn validate(&self) -> CognitiveResult<bool> {
        // Check for overlapping qubit positions
        let mut all_positions = HashSet::new();
        
        for &pos in &self.data_qubits {
            if !all_positions.insert(pos) {
                return Ok(false); // Duplicate position found
            }
        }
        
        for &pos in &self.x_syndrome_qubits {
            if !all_positions.insert(pos) {
                return Ok(false); // Duplicate position found
            }
        }
        
        for &pos in &self.z_syndrome_qubits {
            if !all_positions.insert(pos) {
                return Ok(false); // Duplicate position found
            }
        }
        
        // Check that all positions are within bounds
        let (max_row, max_col) = self.dimensions;
        for &pos in &all_positions {
            if pos.row >= max_row || pos.col >= max_col {
                return Ok(false); // Position out of bounds
            }
        }
        
        // Check stabilizer connectivity
        let (x_stabilizers, z_stabilizers) = self.generate_stabilizers()?;
        
        for stabilizer in &x_stabilizers {
            if !stabilizer.is_valid() {
                return Ok(false); // Invalid stabilizer
            }
        }
        
        for stabilizer in &z_stabilizers {
            if !stabilizer.is_valid() {
                return Ok(false); // Invalid stabilizer
            }
        }
        
        Ok(true)
    }

    /// Get layout metrics
    #[inline]
    pub fn get_metrics(&self) -> &LayoutMetrics {
        &self.metrics
    }

    /// Get qubit type at position
    #[inline]
    pub fn get_qubit_type(&self, position: QubitPosition) -> Option<QubitType> {
        if self.data_qubits.contains(&position) {
            Some(QubitType::Data)
        } else if self.x_syndrome_qubits.contains(&position) {
            Some(QubitType::XSyndrome)
        } else if self.z_syndrome_qubits.contains(&position) {
            Some(QubitType::ZSyndrome)
        } else {
            None
        }
    }

    /// Get logical qubit count (number of encoded logical qubits)
    #[inline]
    pub fn logical_qubit_count(&self) -> usize {
        match self.boundary_type {
            BoundaryType::Open => 1, // Single logical qubit for open boundaries
            BoundaryType::Periodic => 2, // Two logical qubits for torus
            BoundaryType::Twisted => 1, // One logical qubit for twisted torus
        }
    }

    /// Check if layout supports error correction for given error rate
    #[inline]
    pub fn supports_error_correction(&self, error_rate: f64) -> bool {
        let threshold = match self.boundary_type {
            BoundaryType::Open => 0.11, // Approximate threshold for surface code
            BoundaryType::Periodic => 0.10,
            BoundaryType::Twisted => 0.09,
        };
        
        error_rate < threshold && self.metrics.code_distance >= 3
    }
}

/// Type of qubit in surface code layout
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QubitType {
    /// Data qubit storing logical information
    Data,
    /// X-syndrome measurement qubit
    XSyndrome,
    /// Z-syndrome measurement qubit
    ZSyndrome,
}

impl QubitType {
    /// Get string representation
    #[inline]
    pub fn as_str(&self) -> &'static str {
        match self {
            QubitType::Data => "Data",
            QubitType::XSyndrome => "X-Syndrome",
            QubitType::ZSyndrome => "Z-Syndrome",
        }
    }

    /// Check if qubit is a syndrome qubit
    #[inline]
    pub fn is_syndrome(&self) -> bool {
        matches!(self, QubitType::XSyndrome | QubitType::ZSyndrome)
    }

    /// Check if qubit is a data qubit
    #[inline]
    pub fn is_data(&self) -> bool {
        matches!(self, QubitType::Data)
    }
}

/// Layout performance metrics
#[derive(Debug, Clone)]
pub struct LayoutMetrics {
    /// Total number of qubits
    pub total_qubits: usize,
    /// Number of data qubits
    pub data_qubit_count: usize,
    /// Number of syndrome qubits
    pub syndrome_qubit_count: usize,
    /// Average qubit connectivity
    pub average_connectivity: f64,
    /// Code distance
    pub code_distance: usize,
    /// Layout efficiency (qubits used / total grid positions)
    pub layout_efficiency: f64,
    /// Creation time
    pub creation_time: Instant,
}

impl LayoutMetrics {
    /// Create new metrics
    #[inline]
    pub fn new() -> Self {
        Self {
            total_qubits: 0,
            data_qubit_count: 0,
            syndrome_qubit_count: 0,
            average_connectivity: 0.0,
            code_distance: 0,
            layout_efficiency: 0.0,
            creation_time: Instant::now(),
        }
    }

    /// Get data to syndrome ratio
    #[inline]
    pub fn data_to_syndrome_ratio(&self) -> f64 {
        if self.syndrome_qubit_count > 0 {
            self.data_qubit_count as f64 / self.syndrome_qubit_count as f64
        } else {
            0.0
        }
    }

    /// Get theoretical error correction capability
    #[inline]
    pub fn error_correction_capability(&self) -> usize {
        self.code_distance / 2
    }

    /// Get layout quality score
    #[inline]
    pub fn quality_score(&self) -> f64 {
        let connectivity_score = (self.average_connectivity / 4.0).clamp(0.0, 1.0);
        let efficiency_score = self.layout_efficiency;
        let distance_score = (self.code_distance as f64 / 10.0).clamp(0.0, 1.0);
        
        (connectivity_score * 0.4 + efficiency_score * 0.3 + distance_score * 0.3)
    }
}

impl Default for LayoutMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Layout builder for creating optimized surface code layouts
pub struct SurfaceCodeLayoutBuilder {
    dimensions: Option<(usize, usize)>,
    boundary_type: BoundaryType,
    optimization_target: OptimizationTarget,
    constraints: LayoutConstraints,
}

/// Optimization target for layout generation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OptimizationTarget {
    /// Minimize total qubit count
    MinimizeQubits,
    /// Maximize error correction capability
    MaximizeErrorCorrection,
    /// Optimize for specific error rate
    OptimizeForErrorRate(f64),
    /// Balance between qubits and performance
    Balanced,
}

/// Constraints for layout generation
#[derive(Debug, Clone)]
pub struct LayoutConstraints {
    /// Maximum allowed qubits
    pub max_qubits: Option<usize>,
    /// Minimum code distance
    pub min_code_distance: usize,
    /// Required connectivity
    pub min_connectivity: f64,
    /// Target efficiency
    pub target_efficiency: f64,
}

impl SurfaceCodeLayoutBuilder {
    /// Create new layout builder
    #[inline]
    pub fn new() -> Self {
        Self {
            dimensions: None,
            boundary_type: BoundaryType::Open,
            optimization_target: OptimizationTarget::Balanced,
            constraints: LayoutConstraints::default(),
        }
    }

    /// Set layout dimensions
    #[inline]
    pub fn with_dimensions(mut self, rows: usize, cols: usize) -> Self {
        self.dimensions = Some((rows, cols));
        self
    }

    /// Set boundary type
    #[inline]
    pub fn with_boundary_type(mut self, boundary_type: BoundaryType) -> Self {
        self.boundary_type = boundary_type;
        self
    }

    /// Set optimization target
    #[inline]
    pub fn with_optimization_target(mut self, target: OptimizationTarget) -> Self {
        self.optimization_target = target;
        self
    }

    /// Set layout constraints
    #[inline]
    pub fn with_constraints(mut self, constraints: LayoutConstraints) -> Self {
        self.constraints = constraints;
        self
    }

    /// Build optimized layout
    #[inline]
    pub fn build(self) -> CognitiveResult<SurfaceCodeLayout> {
        let dimensions = self.dimensions.ok_or_else(|| {
            CognitiveError::InvalidInput("Layout dimensions must be specified".to_string())
        })?;

        // Validate dimensions meet constraints
        let estimated_distance = std::cmp::min(dimensions.0, dimensions.1);
        if estimated_distance < self.constraints.min_code_distance {
            return Err(CognitiveError::InvalidInput(
                format!("Dimensions too small for required code distance {}", self.constraints.min_code_distance)
            ));
        }

        let layout = SurfaceCodeLayout::new(dimensions, self.boundary_type)?;

        // Validate layout meets constraints
        if !self.validate_constraints(&layout)? {
            return Err(CognitiveError::InvalidInput("Layout does not meet specified constraints".to_string()));
        }

        Ok(layout)
    }

    /// Validate layout against constraints
    #[inline]
    fn validate_constraints(&self, layout: &SurfaceCodeLayout) -> CognitiveResult<bool> {
        let metrics = layout.get_metrics();

        // Check maximum qubits constraint
        if let Some(max_qubits) = self.constraints.max_qubits {
            if metrics.total_qubits > max_qubits {
                return Ok(false);
            }
        }

        // Check minimum code distance
        if metrics.code_distance < self.constraints.min_code_distance {
            return Ok(false);
        }

        // Check minimum connectivity
        if metrics.average_connectivity < self.constraints.min_connectivity {
            return Ok(false);
        }

        // Check target efficiency
        if metrics.layout_efficiency < self.constraints.target_efficiency {
            return Ok(false);
        }

        Ok(true)
    }
}

impl Default for SurfaceCodeLayoutBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for LayoutConstraints {
    fn default() -> Self {
        Self {
            max_qubits: None,
            min_code_distance: 3,
            min_connectivity: 2.0,
            target_efficiency: 0.5,
        }
    }
}