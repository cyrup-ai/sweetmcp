//! Topological logical operators
//!
//! This module provides logical operator definitions and operations for topological codes
//! with zero-allocation patterns and blazing-fast performance.

use crate::cognitive::types::{CognitiveError, CognitiveResult};
use smallvec::SmallVec;
use super::topological_types::{TopologicalCodeType, LogicalOperatorType};
use super::topological_pauli::{PauliType};
use super::topological_pauli_strings::PauliString;
use super::topological_lattice_types::TopologicalLattice;

/// Logical operator for topological codes
#[derive(Debug, Clone)]
pub struct TopologicalLogicalOperator {
    /// Operator ID
    pub id: usize,
    
    /// Pauli string representation
    pub pauli_string: PauliString,
    
    /// Operator type (X or Z)
    pub operator_type: LogicalOperatorType,
    
    /// Logical qubit index
    pub logical_qubit: usize,
    
    /// Path description (for debugging)
    pub path_description: String,
}

/// Collection of logical operators
#[derive(Debug, Clone)]
pub struct LogicalOperatorSet {
    /// X-type logical operators
    pub x_operators: Vec<TopologicalLogicalOperator>,
    
    /// Z-type logical operators
    pub z_operators: Vec<TopologicalLogicalOperator>,
    
    /// Code type
    pub code_type: TopologicalCodeType,
    
    /// Number of logical qubits
    pub num_logical_qubits: usize,
}

/// Statistics about logical operators
#[derive(Debug, Clone)]
pub struct LogicalOperatorStatistics {
    pub num_x_operators: usize,
    pub num_z_operators: usize,
    pub total_operators: usize,
    pub num_logical_qubits: usize,
    pub average_weight: f64,
    pub max_weight: usize,
    pub min_weight: usize,
}

impl TopologicalLogicalOperator {
    /// Create new logical operator
    pub fn new(
        id: usize,
        pauli_string: PauliString,
        operator_type: LogicalOperatorType,
        logical_qubit: usize,
        path_description: String,
    ) -> Self {
        Self {
            id,
            pauli_string,
            operator_type,
            logical_qubit,
            path_description,
        }
    }

    /// Get the weight (number of non-identity operators)
    pub fn weight(&self) -> usize {
        self.pauli_string.weight()
    }

    /// Check if operator commutes with another
    pub fn commutes_with(&self, other: &TopologicalLogicalOperator) -> bool {
        self.pauli_string.commutes_with(&other.pauli_string)
    }

    /// Check if operator acts on a specific edge
    pub fn acts_on_edge(&self, edge_id: usize) -> bool {
        self.pauli_string.acts_on_edge(edge_id)
    }

    /// Get all edges this operator acts on
    pub fn edge_ids(&self) -> Vec<usize> {
        self.pauli_string.edge_ids()
    }

    /// Apply logical operator to quantum state
    pub fn apply_to_state(&self, state: &mut [crate::cognitive::quantum::Complex64]) -> Result<(), String> {
        self.pauli_string.apply_to_state(state)
    }

    /// Check if this is an X-type operator
    pub fn is_x_type(&self) -> bool {
        matches!(self.operator_type, LogicalOperatorType::X)
    }

    /// Check if this is a Z-type operator
    pub fn is_z_type(&self) -> bool {
        matches!(self.operator_type, LogicalOperatorType::Z)
    }
}

impl LogicalOperatorSet {
    /// Create new logical operator set
    pub fn new(code_type: TopologicalCodeType, num_logical_qubits: usize) -> Self {
        Self {
            x_operators: Vec::new(),
            z_operators: Vec::new(),
            code_type,
            num_logical_qubits,
        }
    }

    /// Generate logical operators for a given lattice
    pub fn generate_for_lattice(
        code_type: TopologicalCodeType,
        lattice: &TopologicalLattice,
    ) -> CognitiveResult<Self> {
        match code_type {
            TopologicalCodeType::ToricCode => {
                Self::generate_toric_code_operators(lattice)
            },
            TopologicalCodeType::PlanarCode => {
                Self::generate_planar_code_operators(lattice)
            },
            TopologicalCodeType::ColorCode => {
                Self::generate_color_code_operators(lattice)
            },
            TopologicalCodeType::HyperbolicCode => {
                Self::generate_hyperbolic_operators(lattice)
            },
        }
    }

    /// Add logical operator to set
    pub fn add_operator(&mut self, operator: TopologicalLogicalOperator) {
        match operator.operator_type {
            LogicalOperatorType::X => self.x_operators.push(operator),
            LogicalOperatorType::Z => self.z_operators.push(operator),
        }
    }

    /// Get all logical operators
    pub fn all_operators(&self) -> Vec<&TopologicalLogicalOperator> {
        let mut all = Vec::new();
        all.extend(self.x_operators.iter());
        all.extend(self.z_operators.iter());
        all
    }

    /// Get logical operator by ID
    pub fn get_operator(&self, id: usize) -> Option<&TopologicalLogicalOperator> {
        self.x_operators.iter()
            .chain(self.z_operators.iter())
            .find(|op| op.id == id)
    }

    /// Get X-type operators for a specific logical qubit
    pub fn x_operators_for_qubit(&self, logical_qubit: usize) -> Vec<&TopologicalLogicalOperator> {
        self.x_operators.iter()
            .filter(|op| op.logical_qubit == logical_qubit)
            .collect()
    }

    /// Get Z-type operators for a specific logical qubit
    pub fn z_operators_for_qubit(&self, logical_qubit: usize) -> Vec<&TopologicalLogicalOperator> {
        self.z_operators.iter()
            .filter(|op| op.logical_qubit == logical_qubit)
            .collect()
    }

    /// Get statistics about logical operators
    pub fn statistics(&self) -> LogicalOperatorStatistics {
        let all_operators = self.all_operators();
        let weights: Vec<usize> = all_operators.iter().map(|op| op.weight()).collect();

        LogicalOperatorStatistics {
            num_x_operators: self.x_operators.len(),
            num_z_operators: self.z_operators.len(),
            total_operators: all_operators.len(),
            num_logical_qubits: self.num_logical_qubits,
            average_weight: weights.iter().sum::<usize>() as f64 / weights.len() as f64,
            max_weight: weights.iter().max().copied().unwrap_or(0),
            min_weight: weights.iter().min().copied().unwrap_or(0),
        }
    }

    /// Validate logical operators (check commutation relations)
    pub fn validate(&self) -> Result<(), String> {
        // Check that X and Z operators for the same logical qubit anticommute
        for logical_qubit in 0..self.num_logical_qubits {
            let x_ops = self.x_operators_for_qubit(logical_qubit);
            let z_ops = self.z_operators_for_qubit(logical_qubit);

            for x_op in &x_ops {
                for z_op in &z_ops {
                    if x_op.logical_qubit == z_op.logical_qubit {
                        // Same logical qubit - should anticommute
                        if x_op.commutes_with(z_op) {
                            return Err(format!(
                                "X operator {} and Z operator {} for logical qubit {} should anticommute",
                                x_op.id, z_op.id, logical_qubit
                            ));
                        }
                    } else {
                        // Different logical qubits - should commute
                        if !x_op.commutes_with(z_op) {
                            return Err(format!(
                                "X operator {} and Z operator {} for different logical qubits should commute",
                                x_op.id, z_op.id
                            ));
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl std::fmt::Display for TopologicalLogicalOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "L{} ({}{}): {} [{}]", 
               self.id,
               match self.operator_type {
                   LogicalOperatorType::X => "X",
                   LogicalOperatorType::Z => "Z",
               },
               self.logical_qubit,
               self.pauli_string,
               self.path_description)
    }
}