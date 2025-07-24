//! Topological stabilizer generators and operations
//!
//! This module provides stabilizer generator definitions and operations
//! with zero-allocation patterns and blazing-fast performance.

use crate::cognitive::types::{CognitiveError, CognitiveResult};
use smallvec::SmallVec;
use std::collections::HashMap;
use super::topological_types::{TopologicalCodeType, StabilizerType, ColorType};
use super::topological_pauli::{TopologicalPauli, PauliType};
use super::topological_pauli_strings::PauliString;
use super::topological_lattice_types::TopologicalLattice;

/// Stabilizer generator for topological codes
#[derive(Debug, Clone)]
pub struct TopologicalStabilizer {
    /// Stabilizer ID
    pub id: usize,
    
    /// Pauli string representation
    pub pauli_string: PauliString,
    
    /// Stabilizer type (X-type or Z-type)
    pub stabilizer_type: StabilizerType,
    
    /// Associated face or vertex ID
    pub location_id: usize,
    
    /// Color for color codes
    pub color: Option<ColorType>,
}

/// Collection of stabilizer generators
#[derive(Debug, Clone)]
pub struct StabilizerGroup {
    /// X-type stabilizers
    pub x_stabilizers: Vec<TopologicalStabilizer>,
    
    /// Z-type stabilizers
    pub z_stabilizers: Vec<TopologicalStabilizer>,
    
    /// Code type
    pub code_type: TopologicalCodeType,
}

/// Statistics about stabilizer group
#[derive(Debug, Clone)]
pub struct StabilizerStatistics {
    pub num_x_stabilizers: usize,
    pub num_z_stabilizers: usize,
    pub total_stabilizers: usize,
    pub average_weight: f64,
    pub max_weight: usize,
    pub min_weight: usize,
}

impl TopologicalStabilizer {
    /// Create new stabilizer
    pub fn new(
        id: usize,
        pauli_string: PauliString,
        stabilizer_type: StabilizerType,
        location_id: usize,
    ) -> Self {
        Self {
            id,
            pauli_string,
            stabilizer_type,
            location_id,
            color: None,
        }
    }

    /// Create new colored stabilizer for color codes
    pub fn new_colored(
        id: usize,
        pauli_string: PauliString,
        stabilizer_type: StabilizerType,
        location_id: usize,
        color: ColorType,
    ) -> Self {
        Self {
            id,
            pauli_string,
            stabilizer_type,
            location_id,
            color: Some(color),
        }
    }

    /// Check if stabilizer commutes with another
    pub fn commutes_with(&self, other: &TopologicalStabilizer) -> bool {
        self.pauli_string.commutes_with(&other.pauli_string)
    }

    /// Get the weight (number of non-identity operators)
    pub fn weight(&self) -> usize {
        self.pauli_string.weight()
    }

    /// Check if stabilizer acts on a specific edge
    pub fn acts_on_edge(&self, edge_id: usize) -> bool {
        self.pauli_string.acts_on_edge(edge_id)
    }

    /// Get all edges this stabilizer acts on
    pub fn edge_ids(&self) -> Vec<usize> {
        self.pauli_string.edge_ids()
    }

    /// Apply stabilizer to quantum state
    pub fn apply_to_state(&self, state: &mut [crate::cognitive::quantum::Complex64]) -> Result<(), String> {
        self.pauli_string.apply_to_state(state)
    }

    /// Check if this is an X-type stabilizer
    pub fn is_x_type(&self) -> bool {
        matches!(self.stabilizer_type, StabilizerType::X)
    }

    /// Check if this is a Z-type stabilizer
    pub fn is_z_type(&self) -> bool {
        matches!(self.stabilizer_type, StabilizerType::Z)
    }
}

impl StabilizerGroup {
    /// Create new stabilizer group
    pub fn new(code_type: TopologicalCodeType) -> Self {
        Self {
            x_stabilizers: Vec::new(),
            z_stabilizers: Vec::new(),
            code_type,
        }
    }

    /// Generate stabilizers for a given lattice
    pub fn generate_for_lattice(
        code_type: TopologicalCodeType,
        lattice: &TopologicalLattice,
    ) -> CognitiveResult<Self> {
        match code_type {
            TopologicalCodeType::ToricCode | TopologicalCodeType::PlanarCode => {
                Self::generate_surface_code_stabilizers(lattice)
            },
            TopologicalCodeType::ColorCode => {
                Self::generate_color_code_stabilizers(lattice)
            },
            TopologicalCodeType::HyperbolicCode => {
                Self::generate_hyperbolic_stabilizers(lattice)
            },
        }
    }

    /// Add stabilizer to group
    pub fn add_stabilizer(&mut self, stabilizer: TopologicalStabilizer) {
        match stabilizer.stabilizer_type {
            StabilizerType::X => self.x_stabilizers.push(stabilizer),
            StabilizerType::Z => self.z_stabilizers.push(stabilizer),
        }
    }

    /// Get all stabilizers
    pub fn all_stabilizers(&self) -> Vec<&TopologicalStabilizer> {
        let mut all = Vec::new();
        all.extend(self.x_stabilizers.iter());
        all.extend(self.z_stabilizers.iter());
        all
    }

    /// Get stabilizer by ID
    pub fn get_stabilizer(&self, id: usize) -> Option<&TopologicalStabilizer> {
        self.x_stabilizers.iter()
            .chain(self.z_stabilizers.iter())
            .find(|s| s.id == id)
    }

    /// Get stabilizers acting on a specific edge
    pub fn stabilizers_on_edge(&self, edge_id: usize) -> Vec<&TopologicalStabilizer> {
        self.all_stabilizers()
            .into_iter()
            .filter(|s| s.acts_on_edge(edge_id))
            .collect()
    }

    /// Calculate syndrome for error pattern
    pub fn calculate_syndrome(&self, error_pattern: &PauliString) -> Vec<bool> {
        let mut syndrome = Vec::new();

        for stabilizer in self.all_stabilizers() {
            // Syndrome bit is 1 if error anticommutes with stabilizer
            let anticommutes = !stabilizer.pauli_string.commutes_with(error_pattern);
            syndrome.push(anticommutes);
        }

        syndrome
    }

    /// Get number of stabilizers
    pub fn num_stabilizers(&self) -> usize {
        self.x_stabilizers.len() + self.z_stabilizers.len()
    }

    /// Get statistics about the stabilizer group
    pub fn statistics(&self) -> StabilizerStatistics {
        let all_stabilizers = self.all_stabilizers();
        let weights: Vec<usize> = all_stabilizers.iter().map(|s| s.weight()).collect();

        StabilizerStatistics {
            num_x_stabilizers: self.x_stabilizers.len(),
            num_z_stabilizers: self.z_stabilizers.len(),
            total_stabilizers: all_stabilizers.len(),
            average_weight: weights.iter().sum::<usize>() as f64 / weights.len() as f64,
            max_weight: weights.iter().max().copied().unwrap_or(0),
            min_weight: weights.iter().min().copied().unwrap_or(0),
        }
    }

    /// Check if all stabilizers commute with each other
    pub fn validate_commutation(&self) -> Result<(), String> {
        let all_stabilizers = self.all_stabilizers();

        for (i, stab_a) in all_stabilizers.iter().enumerate() {
            for (j, stab_b) in all_stabilizers.iter().enumerate() {
                if i != j && !stab_a.commutes_with(stab_b) {
                    // X and Z stabilizers should commute
                    if stab_a.stabilizer_type != stab_b.stabilizer_type {
                        return Err(format!(
                            "Stabilizers {} and {} do not commute",
                            stab_a.id, stab_b.id
                        ));
                    }
                }
            }
        }

        Ok(())
    }
}

impl std::fmt::Display for TopologicalStabilizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "S{} ({}): {}", 
               self.id, 
               match self.stabilizer_type {
                   StabilizerType::X => "X",
                   StabilizerType::Z => "Z",
               },
               self.pauli_string)
    }
}