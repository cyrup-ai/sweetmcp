//! Topological Pauli string operations
//!
//! This module provides Pauli string implementations for topological quantum error correction
//! with zero-allocation patterns and blazing-fast performance.

use crate::cognitive::quantum::Complex64;
use smallvec::SmallVec;
use super::topological_pauli::{TopologicalPauli, PauliType};

/// Pauli string (collection of Pauli operators)
#[derive(Debug, Clone)]
pub struct PauliString {
    /// Pauli operators
    pub paulis: SmallVec<[TopologicalPauli; 8]>,
    
    /// Global phase
    pub phase: Complex64,
}

impl PauliString {
    /// Create new empty Pauli string
    pub fn new() -> Self {
        Self {
            paulis: SmallVec::new(),
            phase: Complex64::new(1.0, 0.0),
        }
    }

    /// Create identity string
    pub fn identity() -> Self {
        Self::new()
    }

    /// Create Pauli string from edge IDs and Pauli types
    pub fn from_paulis(edge_paulis: &[(usize, PauliType)]) -> Self {
        let mut string = Self::new();
        for &(edge_id, pauli_type) in edge_paulis {
            string.add_pauli(TopologicalPauli::new(
                edge_id,
                pauli_type,
                Complex64::new(1.0, 0.0),
            ));
        }
        string
    }

    /// Create X string on given edges
    pub fn x_string(edge_ids: &[usize]) -> Self {
        let edge_paulis: Vec<(usize, PauliType)> = edge_ids.iter()
            .map(|&id| (id, PauliType::X))
            .collect();
        Self::from_paulis(&edge_paulis)
    }

    /// Create Z string on given edges
    pub fn z_string(edge_ids: &[usize]) -> Self {
        let edge_paulis: Vec<(usize, PauliType)> = edge_ids.iter()
            .map(|&id| (id, PauliType::Z))
            .collect();
        Self::from_paulis(&edge_paulis)
    }

    /// Add a Pauli operator to the string
    pub fn add_pauli(&mut self, pauli: TopologicalPauli) {
        // Check if we already have an operator on this edge
        if let Some(existing_pos) = self.paulis.iter().position(|p| p.edge_id == pauli.edge_id) {
            // Multiply with existing operator
            if let Some(result) = self.paulis[existing_pos].multiply(&pauli) {
                if result.is_identity() {
                    // Remove identity operator
                    self.paulis.remove(existing_pos);
                } else {
                    // Replace with result
                    self.paulis[existing_pos] = result;
                }
            }
        } else {
            // Add new operator if not identity
            if !pauli.is_identity() {
                self.paulis.push(pauli);
            }
        }
    }

    /// Get the weight (number of non-identity operators)
    pub fn weight(&self) -> usize {
        self.paulis.iter().map(|p| p.weight()).sum()
    }

    /// Check if string commutes with another
    pub fn commutes_with(&self, other: &PauliString) -> bool {
        let mut anticommuting_pairs = 0;
        
        for pauli_a in &self.paulis {
            for pauli_b in &other.paulis {
                if pauli_a.edge_id == pauli_b.edge_id && !pauli_a.commutes_with(pauli_b) {
                    anticommuting_pairs += 1;
                }
            }
        }
        
        anticommuting_pairs % 2 == 0
    }

    /// Multiply with another Pauli string
    pub fn multiply(&self, other: &PauliString) -> PauliString {
        let mut result = self.clone();
        result.phase *= other.phase;
        
        // Add all Pauli operators from other string
        for pauli in &other.paulis {
            result.add_pauli(pauli.clone());
        }
        
        result
    }

    /// Check if string is identity
    pub fn is_identity(&self) -> bool {
        self.paulis.is_empty() && (self.phase - Complex64::new(1.0, 0.0)).norm() < 1e-10
    }

    /// Get the Hermitian conjugate
    pub fn dagger(&self) -> PauliString {
        let mut result = PauliString::new();
        result.phase = self.phase.conj();
        result.paulis = self.paulis.iter().map(|p| p.dagger()).collect();
        result
    }

    /// Get the maximum edge ID in the string
    pub fn max_edge_id(&self) -> Option<usize> {
        self.paulis.iter().map(|p| p.edge_id).max()
    }

    /// Get all edge IDs in the string
    pub fn edge_ids(&self) -> Vec<usize> {
        let mut ids: Vec<usize> = self.paulis.iter().map(|p| p.edge_id).collect();
        ids.sort_unstable();
        ids.dedup();
        ids
    }

    /// Check if string acts on a specific edge
    pub fn acts_on_edge(&self, edge_id: usize) -> bool {
        self.paulis.iter().any(|p| p.edge_id == edge_id)
    }

    /// Get the Pauli type acting on a specific edge
    pub fn pauli_on_edge(&self, edge_id: usize) -> Option<PauliType> {
        self.paulis.iter()
            .find(|p| p.edge_id == edge_id)
            .map(|p| p.pauli_type)
    }

    /// Simplify the Pauli string by combining like terms
    pub fn simplify(&mut self) {
        // Sort by edge ID for easier processing
        self.paulis.sort_by_key(|p| p.edge_id);
        
        // Combine operators on the same edge
        let mut simplified = SmallVec::new();
        let mut current_edge_id = None;
        let mut current_pauli = None;
        let mut current_coefficient = Complex64::new(1.0, 0.0);
        
        for pauli in &self.paulis {
            if Some(pauli.edge_id) == current_edge_id {
                // Combine with current operator
                if let Some(ref mut current) = current_pauli {
                    let (phase, result_pauli) = current.multiply(&pauli.pauli_type);
                    current_coefficient *= pauli.coefficient * phase;
                    *current = result_pauli;
                }
            } else {
                // Save previous operator if not identity
                if let Some(pauli_type) = current_pauli {
                    if !pauli_type.is_identity() && current_coefficient.norm() > 1e-10 {
                        simplified.push(TopologicalPauli::new(
                            current_edge_id.unwrap(),
                            pauli_type,
                            current_coefficient,
                        ));
                    }
                }
                
                // Start new operator
                current_edge_id = Some(pauli.edge_id);
                current_pauli = Some(pauli.pauli_type);
                current_coefficient = pauli.coefficient;
            }
        }
        
        // Don't forget the last operator
        if let Some(pauli_type) = current_pauli {
            if !pauli_type.is_identity() && current_coefficient.norm() > 1e-10 {
                simplified.push(TopologicalPauli::new(
                    current_edge_id.unwrap(),
                    pauli_type,
                    current_coefficient,
                ));
            }
        }
        
        self.paulis = simplified;
    }

    /// Convert to string representation
    pub fn to_string(&self) -> String {
        if self.is_identity() {
            return "I".to_string();
        }

        let mut result = String::new();
        
        // Add phase if not 1
        if (self.phase - Complex64::new(1.0, 0.0)).norm() > 1e-10 {
            result.push_str(&format!("({:.3})", self.phase));
        }

        // Add Pauli operators
        for (i, pauli) in self.paulis.iter().enumerate() {
            if i > 0 {
                result.push(' ');
            }
            
            let pauli_char = match pauli.pauli_type {
                PauliType::I => 'I',
                PauliType::X => 'X',
                PauliType::Y => 'Y',
                PauliType::Z => 'Z',
            };
            
            if (pauli.coefficient - Complex64::new(1.0, 0.0)).norm() > 1e-10 {
                result.push_str(&format!("({:.3}){}{}", pauli.coefficient, pauli_char, pauli.edge_id));
            } else {
                result.push_str(&format!("{}{}", pauli_char, pauli.edge_id));
            }
        }

        result
    }
}

impl Default for PauliString {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for PauliString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}