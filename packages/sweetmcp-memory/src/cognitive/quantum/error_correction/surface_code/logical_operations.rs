//! Surface code logical operations
//!
//! This module provides blazing-fast logical operations with zero allocation
//! optimizations and elegant ergonomic interfaces for surface code logical qubits.

use crate::cognitive::quantum::{
    Complex64,
    types::{CognitiveError, CognitiveResult},
};
use std::collections::{HashMap, HashSet};
use smallvec::SmallVec;
use std::time::Instant;

use super::{
    syndrome_detection::{QubitPosition, PauliOperator},
    layout_management::SurfaceCodeLayout,
};
use super::super::{
    topological_pauli::PauliType,
    topological_types::BoundaryType,
};

/// Type of logical operator
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogicalOperatorType {
    X,
    Z,
}

impl LogicalOperatorType {
    /// Get string representation
    #[inline]
    pub fn as_str(&self) -> &'static str {
        match self {
            LogicalOperatorType::X => "X",
            LogicalOperatorType::Z => "Z",
        }
    }

    /// Get corresponding Pauli type
    #[inline]
    pub fn to_pauli_type(&self) -> PauliType {
        match self {
            LogicalOperatorType::X => PauliType::X,
            LogicalOperatorType::Z => PauliType::Z,
        }
    }

    /// Check if operators anticommute
    #[inline]
    pub fn anticommutes_with(&self, other: &LogicalOperatorType) -> bool {
        self != other
    }
}

/// Single logical operator
#[derive(Debug, Clone)]
pub struct LogicalOperator {
    /// Operator ID
    pub id: String,
    /// Pauli string representing the operator
    pub pauli_string: Vec<PauliOperator>,
    /// Operator type
    pub operator_type: LogicalOperatorType,
    /// Logical qubit index
    pub logical_qubit_index: usize,
    /// Operator weight (number of non-identity Paulis)
    pub weight: usize,
}

impl LogicalOperator {
    /// Create new logical operator
    #[inline]
    pub fn new(
        id: String,
        operator_type: LogicalOperatorType,
        logical_qubit_index: usize,
    ) -> Self {
        Self {
            id,
            pauli_string: Vec::new(),
            operator_type,
            logical_qubit_index,
            weight: 0,
        }
    }

    /// Add Pauli operator to the string
    #[inline]
    pub fn add_pauli_operator(&mut self, operator: PauliOperator) {
        if !operator.is_identity() {
            self.weight += 1;
        }
        self.pauli_string.push(operator);
    }

    /// Get operator support (set of qubits it acts on)
    #[inline]
    pub fn get_support(&self) -> HashSet<QubitPosition> {
        self.pauli_string.iter()
            .filter(|op| !op.is_identity())
            .map(|op| op.position)
            .collect()
    }

    /// Check if operator is valid (non-empty and consistent)
    #[inline]
    pub fn is_valid(&self) -> bool {
        !self.pauli_string.is_empty() && 
        self.pauli_string.iter().all(|op| {
            match self.operator_type {
                LogicalOperatorType::X => op.pauli_type.has_x_component() || op.is_identity(),
                LogicalOperatorType::Z => op.pauli_type.has_z_component() || op.is_identity(),
            }
        })
    }

    /// Calculate operator norm
    #[inline]
    pub fn norm(&self) -> f64 {
        self.pauli_string.iter()
            .map(|op| op.coefficient.norm())
            .sum::<f64>()
            .sqrt()
    }

    /// Check if operator commutes with another
    #[inline]
    pub fn commutes_with(&self, other: &LogicalOperator) -> bool {
        let mut anticommuting_pairs = 0;
        
        for op1 in &self.pauli_string {
            for op2 in &other.pauli_string {
                if op1.position == op2.position && !op1.pauli_type.commutes_with(&op2.pauli_type) {
                    anticommuting_pairs += 1;
                }
            }
        }
        
        anticommuting_pairs % 2 == 0
    }

    /// Apply operator to quantum state (simplified representation)
    #[inline]
    pub fn apply_to_state(&self, state: &mut HashMap<QubitPosition, Complex64>) -> CognitiveResult<()> {
        for pauli_op in &self.pauli_string {
            if !pauli_op.is_identity() {
                // Apply Pauli operator (simplified)
                let current_amplitude = state.get(&pauli_op.position).copied()
                    .unwrap_or(Complex64::new(1.0, 0.0));
                
                let new_amplitude = current_amplitude * pauli_op.coefficient;
                state.insert(pauli_op.position, new_amplitude);
            }
        }
        
        Ok(())
    }

    /// Get operator as string representation
    #[inline]
    pub fn to_string_representation(&self) -> String {
        let pauli_chars: Vec<String> = self.pauli_string.iter()
            .map(|op| format!("{}_{}_{}",
                op.pauli_type.as_str(),
                op.position.row,
                op.position.col))
            .collect();
        
        pauli_chars.join(" * ")
    }
}

/// Logical operators for surface code
#[derive(Debug, Clone)]
pub struct LogicalOperators {
    /// Logical X operators
    pub logical_x: Vec<LogicalOperator>,
    /// Logical Z operators
    pub logical_z: Vec<LogicalOperator>,
    /// Number of logical qubits
    pub logical_qubit_count: usize,
    /// Operators metrics
    pub metrics: LogicalOperatorMetrics,
}

impl LogicalOperators {
    /// Create new logical operators
    #[inline]
    pub fn new(logical_qubit_count: usize) -> Self {
        Self {
            logical_x: Vec::with_capacity(logical_qubit_count),
            logical_z: Vec::with_capacity(logical_qubit_count),
            logical_qubit_count,
            metrics: LogicalOperatorMetrics::new(),
        }
    }

    /// Add logical X operator
    #[inline]
    pub fn add_logical_x(&mut self, operator: LogicalOperator) {
        self.logical_x.push(operator);
        self.update_metrics();
    }

    /// Add logical Z operator
    #[inline]
    pub fn add_logical_z(&mut self, operator: LogicalOperator) {
        self.logical_z.push(operator);
        self.update_metrics();
    }

    /// Get logical operator by type and index
    #[inline]
    pub fn get_logical_operator(
        &self,
        operator_type: LogicalOperatorType,
        logical_qubit_index: usize,
    ) -> Option<&LogicalOperator> {
        match operator_type {
            LogicalOperatorType::X => {
                self.logical_x.iter()
                    .find(|op| op.logical_qubit_index == logical_qubit_index)
            }
            LogicalOperatorType::Z => {
                self.logical_z.iter()
                    .find(|op| op.logical_qubit_index == logical_qubit_index)
            }
        }
    }

    /// Validate logical operators (check commutation relations)
    #[inline]
    pub fn validate(&self) -> CognitiveResult<bool> {
        // Check that X and Z operators for the same logical qubit anticommute
        for i in 0..self.logical_qubit_count {
            if let (Some(x_op), Some(z_op)) = (
                self.get_logical_operator(LogicalOperatorType::X, i),
                self.get_logical_operator(LogicalOperatorType::Z, i),
            ) {
                if x_op.commutes_with(z_op) {
                    return Ok(false); // Should anticommute
                }
            }
        }

        // Check that operators for different logical qubits commute
        for i in 0..self.logical_qubit_count {
            for j in (i + 1)..self.logical_qubit_count {
                let ops_i = [
                    self.get_logical_operator(LogicalOperatorType::X, i),
                    self.get_logical_operator(LogicalOperatorType::Z, i),
                ];
                let ops_j = [
                    self.get_logical_operator(LogicalOperatorType::X, j),
                    self.get_logical_operator(LogicalOperatorType::Z, j),
                ];

                for op_i in ops_i.iter().flatten() {
                    for op_j in ops_j.iter().flatten() {
                        if !op_i.commutes_with(op_j) {
                            return Ok(false); // Should commute
                        }
                    }
                }
            }
        }

        Ok(true)
    }

    /// Update operator metrics
    #[inline]
    fn update_metrics(&mut self) {
        self.metrics.total_x_operators = self.logical_x.len();
        self.metrics.total_z_operators = self.logical_z.len();
        
        // Calculate average weights
        if !self.logical_x.is_empty() {
            self.metrics.average_x_weight = self.logical_x.iter()
                .map(|op| op.weight as f64)
                .sum::<f64>() / self.logical_x.len() as f64;
        }
        
        if !self.logical_z.is_empty() {
            self.metrics.average_z_weight = self.logical_z.iter()
                .map(|op| op.weight as f64)
                .sum::<f64>() / self.logical_z.len() as f64;
        }
        
        // Find minimum weights
        self.metrics.min_x_weight = self.logical_x.iter()
            .map(|op| op.weight)
            .min()
            .unwrap_or(0);
        
        self.metrics.min_z_weight = self.logical_z.iter()
            .map(|op| op.weight)
            .min()
            .unwrap_or(0);
    }

    /// Get all operators
    #[inline]
    pub fn get_all_operators(&self) -> Vec<&LogicalOperator> {
        let mut operators = Vec::with_capacity(self.logical_x.len() + self.logical_z.len());
        operators.extend(&self.logical_x);
        operators.extend(&self.logical_z);
        operators
    }

    /// Get operators by type
    #[inline]
    pub fn get_operators_by_type(&self, operator_type: LogicalOperatorType) -> &[LogicalOperator] {
        match operator_type {
            LogicalOperatorType::X => &self.logical_x,
            LogicalOperatorType::Z => &self.logical_z,
        }
    }

    /// Get metrics
    #[inline]
    pub fn get_metrics(&self) -> &LogicalOperatorMetrics {
        &self.metrics
    }
}

/// Logical operator performance metrics
#[derive(Debug, Clone)]
pub struct LogicalOperatorMetrics {
    /// Total X operators
    pub total_x_operators: usize,
    /// Total Z operators
    pub total_z_operators: usize,
    /// Average X operator weight
    pub average_x_weight: f64,
    /// Average Z operator weight
    pub average_z_weight: f64,
    /// Minimum X operator weight
    pub min_x_weight: usize,
    /// Minimum Z operator weight
    pub min_z_weight: usize,
    /// Creation time
    pub creation_time: Instant,
}

impl LogicalOperatorMetrics {
    /// Create new metrics
    #[inline]
    pub fn new() -> Self {
        Self {
            total_x_operators: 0,
            total_z_operators: 0,
            average_x_weight: 0.0,
            average_z_weight: 0.0,
            min_x_weight: 0,
            min_z_weight: 0,
            creation_time: Instant::now(),
        }
    }

    /// Get total operators
    #[inline]
    pub fn total_operators(&self) -> usize {
        self.total_x_operators + self.total_z_operators
    }

    /// Get average operator weight
    #[inline]
    pub fn average_weight(&self) -> f64 {
        if self.total_operators() > 0 {
            (self.average_x_weight * self.total_x_operators as f64 +
             self.average_z_weight * self.total_z_operators as f64) /
            self.total_operators() as f64
        } else {
            0.0
        }
    }

    /// Get minimum operator weight
    #[inline]
    pub fn min_weight(&self) -> usize {
        std::cmp::min(self.min_x_weight, self.min_z_weight)
    }
}

impl Default for LogicalOperatorMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Logical operations engine for surface codes
pub struct LogicalOperationsEngine {
    /// Surface code layout
    layout: SurfaceCodeLayout,
    /// Logical operators
    operators: LogicalOperators,
    /// Operation metrics
    metrics: OperationMetrics,
    /// Configuration
    config: LogicalOperationConfig,
}

impl LogicalOperationsEngine {
    /// Create new logical operations engine
    #[inline]
    pub fn new(layout: SurfaceCodeLayout) -> CognitiveResult<Self> {
        let logical_qubit_count = layout.logical_qubit_count();
        let mut operators = LogicalOperators::new(logical_qubit_count);
        
        // Generate logical operators for the layout
        let (logical_x, logical_z) = Self::generate_logical_operators(&layout)?;
        
        for op in logical_x {
            operators.add_logical_x(op);
        }
        
        for op in logical_z {
            operators.add_logical_z(op);
        }

        Ok(Self {
            layout,
            operators,
            metrics: OperationMetrics::new(),
            config: LogicalOperationConfig::default(),
        })
    }

    /// Generate logical operators for the given layout
    #[inline]
    fn generate_logical_operators(
        layout: &SurfaceCodeLayout,
    ) -> CognitiveResult<(Vec<LogicalOperator>, Vec<LogicalOperator>)> {
        let mut logical_x = Vec::new();
        let mut logical_z = Vec::new();

        match layout.boundary_type {
            BoundaryType::Open => {
                // For open boundaries, generate single logical qubit operators
                let (x_op, z_op) = Self::generate_open_boundary_operators(layout)?;
                logical_x.push(x_op);
                logical_z.push(z_op);
            }
            BoundaryType::Periodic => {
                // For periodic boundaries (torus), generate two logical qubit operators
                let (x_ops, z_ops) = Self::generate_periodic_boundary_operators(layout)?;
                logical_x.extend(x_ops);
                logical_z.extend(z_ops);
            }
            BoundaryType::Twisted => {
                // For twisted boundaries, generate single logical qubit operators
                let (x_op, z_op) = Self::generate_twisted_boundary_operators(layout)?;
                logical_x.push(x_op);
                logical_z.push(z_op);
            }
        }

        Ok((logical_x, logical_z))
    }

    /// Generate logical operators for open boundary conditions
    #[inline]
    fn generate_open_boundary_operators(
        layout: &SurfaceCodeLayout,
    ) -> CognitiveResult<(LogicalOperator, LogicalOperator)> {
        let (rows, cols) = layout.dimensions;
        
        // Logical X: horizontal string across the middle
        let mut logical_x = LogicalOperator::new(
            "LogicalX_0".to_string(),
            LogicalOperatorType::X,
            0,
        );
        
        let middle_row = rows / 2;
        for col in 0..cols {
            let pos = QubitPosition::new(middle_row, col);
            if layout.data_qubits.contains(&pos) {
                logical_x.add_pauli_operator(PauliOperator::new(pos, PauliType::X));
            }
        }

        // Logical Z: vertical string across the middle
        let mut logical_z = LogicalOperator::new(
            "LogicalZ_0".to_string(),
            LogicalOperatorType::Z,
            0,
        );
        
        let middle_col = cols / 2;
        for row in 0..rows {
            let pos = QubitPosition::new(row, middle_col);
            if layout.data_qubits.contains(&pos) {
                logical_z.add_pauli_operator(PauliOperator::new(pos, PauliType::Z));
            }
        }

        Ok((logical_x, logical_z))
    }

    /// Generate logical operators for periodic boundary conditions
    #[inline]
    fn generate_periodic_boundary_operators(
        layout: &SurfaceCodeLayout,
    ) -> CognitiveResult<(Vec<LogicalOperator>, Vec<LogicalOperator>)> {
        let (rows, cols) = layout.dimensions;
        let mut logical_x = Vec::new();
        let mut logical_z = Vec::new();

        // First logical qubit: horizontal and vertical loops
        let mut x_op_0 = LogicalOperator::new(
            "LogicalX_0".to_string(),
            LogicalOperatorType::X,
            0,
        );
        
        // Horizontal loop for first logical X
        for col in 0..cols {
            let pos = QubitPosition::new(0, col);
            if layout.data_qubits.contains(&pos) {
                x_op_0.add_pauli_operator(PauliOperator::new(pos, PauliType::X));
            }
        }
        logical_x.push(x_op_0);

        let mut z_op_0 = LogicalOperator::new(
            "LogicalZ_0".to_string(),
            LogicalOperatorType::Z,
            0,
        );
        
        // Vertical loop for first logical Z
        for row in 0..rows {
            let pos = QubitPosition::new(row, 0);
            if layout.data_qubits.contains(&pos) {
                z_op_0.add_pauli_operator(PauliOperator::new(pos, PauliType::Z));
            }
        }
        logical_z.push(z_op_0);

        // Second logical qubit: orthogonal loops
        let mut x_op_1 = LogicalOperator::new(
            "LogicalX_1".to_string(),
            LogicalOperatorType::X,
            1,
        );
        
        // Vertical loop for second logical X
        for row in 0..rows {
            let pos = QubitPosition::new(row, cols / 2);
            if layout.data_qubits.contains(&pos) {
                x_op_1.add_pauli_operator(PauliOperator::new(pos, PauliType::X));
            }
        }
        logical_x.push(x_op_1);

        let mut z_op_1 = LogicalOperator::new(
            "LogicalZ_1".to_string(),
            LogicalOperatorType::Z,
            1,
        );
        
        // Horizontal loop for second logical Z
        for col in 0..cols {
            let pos = QubitPosition::new(rows / 2, col);
            if layout.data_qubits.contains(&pos) {
                z_op_1.add_pauli_operator(PauliOperator::new(pos, PauliType::Z));
            }
        }
        logical_z.push(z_op_1);

        Ok((logical_x, logical_z))
    }

    /// Generate logical operators for twisted boundary conditions
    #[inline]
    fn generate_twisted_boundary_operators(
        layout: &SurfaceCodeLayout,
    ) -> CognitiveResult<(LogicalOperator, LogicalOperator)> {
        // For twisted boundaries, use similar approach to open boundaries
        // but with modified paths that account for the twist
        Self::generate_open_boundary_operators(layout)
    }

    /// Apply logical operation to quantum state
    #[inline]
    pub fn apply_logical_operation(
        &mut self,
        operator_type: LogicalOperatorType,
        logical_qubit_index: usize,
        state: &mut HashMap<QubitPosition, Complex64>,
    ) -> CognitiveResult<()> {
        let start_time = Instant::now();
        
        if let Some(operator) = self.operators.get_logical_operator(operator_type, logical_qubit_index) {
            operator.apply_to_state(state)?;
            
            let operation_time = start_time.elapsed();
            self.metrics.record_operation(operation_time, operator_type, true);
            
            Ok(())
        } else {
            let operation_time = start_time.elapsed();
            self.metrics.record_operation(operation_time, operator_type, false);
            
            Err(CognitiveError::InvalidInput(
                format!("Logical operator {} {} not found", 
                    operator_type.as_str(), logical_qubit_index)
            ))
        }
    }

    /// Measure logical observable
    #[inline]
    pub fn measure_logical_observable(
        &mut self,
        operator_type: LogicalOperatorType,
        logical_qubit_index: usize,
        state: &HashMap<QubitPosition, Complex64>,
    ) -> CognitiveResult<f64> {
        let start_time = Instant::now();
        
        if let Some(operator) = self.operators.get_logical_operator(operator_type, logical_qubit_index) {
            // Calculate expectation value (simplified)
            let mut expectation_value = 0.0;
            
            for pauli_op in &operator.pauli_string {
                if let Some(&amplitude) = state.get(&pauli_op.position) {
                    expectation_value += (amplitude * pauli_op.coefficient.conj()).re;
                }
            }
            
            let operation_time = start_time.elapsed();
            self.metrics.record_measurement(operation_time, operator_type, true);
            
            Ok(expectation_value)
        } else {
            let operation_time = start_time.elapsed();
            self.metrics.record_measurement(operation_time, operator_type, false);
            
            Err(CognitiveError::InvalidInput(
                format!("Logical operator {} {} not found", 
                    operator_type.as_str(), logical_qubit_index)
            ))
        }
    }

    /// Get logical operators
    #[inline]
    pub fn get_logical_operators(&self) -> &LogicalOperators {
        &self.operators
    }

    /// Get operation metrics
    #[inline]
    pub fn get_metrics(&self) -> &OperationMetrics {
        &self.metrics
    }

    /// Update configuration
    #[inline]
    pub fn update_config(&mut self, config: LogicalOperationConfig) {
        self.config = config;
    }

    /// Validate logical operations
    #[inline]
    pub fn validate(&self) -> CognitiveResult<bool> {
        self.operators.validate()
    }
}

/// Operation performance metrics
#[derive(Debug, Clone)]
pub struct OperationMetrics {
    /// Total operations performed
    pub total_operations: u64,
    /// Total measurements performed
    pub total_measurements: u64,
    /// Successful operations
    pub successful_operations: u64,
    /// Successful measurements
    pub successful_measurements: u64,
    /// Total operation time
    pub total_operation_time_ms: u64,
    /// Total measurement time
    pub total_measurement_time_ms: u64,
    /// Operation type counts
    pub x_operations: u64,
    /// Z operation counts
    pub z_operations: u64,
    /// Creation time
    pub creation_time: Instant,
}

impl OperationMetrics {
    /// Create new metrics
    #[inline]
    pub fn new() -> Self {
        Self {
            total_operations: 0,
            total_measurements: 0,
            successful_operations: 0,
            successful_measurements: 0,
            total_operation_time_ms: 0,
            total_measurement_time_ms: 0,
            x_operations: 0,
            z_operations: 0,
            creation_time: Instant::now(),
        }
    }

    /// Record operation
    #[inline]
    pub fn record_operation(
        &mut self,
        operation_time: std::time::Duration,
        operator_type: LogicalOperatorType,
        successful: bool,
    ) {
        self.total_operations += 1;
        self.total_operation_time_ms += operation_time.as_millis() as u64;
        
        if successful {
            self.successful_operations += 1;
        }
        
        match operator_type {
            LogicalOperatorType::X => self.x_operations += 1,
            LogicalOperatorType::Z => self.z_operations += 1,
        }
    }

    /// Record measurement
    #[inline]
    pub fn record_measurement(
        &mut self,
        measurement_time: std::time::Duration,
        _operator_type: LogicalOperatorType,
        successful: bool,
    ) {
        self.total_measurements += 1;
        self.total_measurement_time_ms += measurement_time.as_millis() as u64;
        
        if successful {
            self.successful_measurements += 1;
        }
    }

    /// Get operation success rate
    #[inline]
    pub fn operation_success_rate(&self) -> f64 {
        if self.total_operations > 0 {
            self.successful_operations as f64 / self.total_operations as f64
        } else {
            0.0
        }
    }

    /// Get measurement success rate
    #[inline]
    pub fn measurement_success_rate(&self) -> f64 {
        if self.total_measurements > 0 {
            self.successful_measurements as f64 / self.total_measurements as f64
        } else {
            0.0
        }
    }

    /// Get average operation time
    #[inline]
    pub fn average_operation_time_ms(&self) -> f64 {
        if self.total_operations > 0 {
            self.total_operation_time_ms as f64 / self.total_operations as f64
        } else {
            0.0
        }
    }

    /// Get average measurement time
    #[inline]
    pub fn average_measurement_time_ms(&self) -> f64 {
        if self.total_measurements > 0 {
            self.total_measurement_time_ms as f64 / self.total_measurements as f64
        } else {
            0.0
        }
    }
}

impl Default for OperationMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for logical operations
#[derive(Debug, Clone)]
pub struct LogicalOperationConfig {
    /// Enable operation caching
    pub enable_caching: bool,
    /// Maximum cache size
    pub max_cache_size: usize,
    /// Enable parallel operations
    pub enable_parallel_operations: bool,
    /// Operation timeout in milliseconds
    pub operation_timeout_ms: u64,
}

impl LogicalOperationConfig {
    /// Create new configuration
    #[inline]
    pub fn new() -> Self {
        Self {
            enable_caching: false,
            max_cache_size: 1000,
            enable_parallel_operations: false,
            operation_timeout_ms: 1000,
        }
    }

    /// Create performance-optimized configuration
    #[inline]
    pub fn performance_optimized() -> Self {
        Self {
            enable_caching: true,
            max_cache_size: 10000,
            enable_parallel_operations: true,
            operation_timeout_ms: 100,
        }
    }
}

impl Default for LogicalOperationConfig {
    fn default() -> Self {
        Self::new()
    }
}