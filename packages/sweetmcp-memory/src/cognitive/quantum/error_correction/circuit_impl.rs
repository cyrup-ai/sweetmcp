//! Quantum circuit implementation for error correction
//!
//! This module provides quantum circuit structures with optimized
//! gate storage and manipulation operations.

use super::QuantumGate;
use std::collections::HashSet;

/// Quantum circuit representation with optimized gate storage
#[derive(Debug, Clone)]
pub struct QuantumCircuit {
    pub gates: Vec<QuantumGate>,
    pub qubit_count: usize,
}

impl QuantumCircuit {
    /// Create a new quantum circuit
    pub fn new(qubit_count: usize) -> Self {
        Self {
            gates: Vec::with_capacity(16), // Pre-allocate for common case
            qubit_count,
        }
    }

    /// Create with gate capacity for zero-allocation building
    pub fn with_capacity(qubit_count: usize, gate_capacity: usize) -> Self {
        Self {
            gates: Vec::with_capacity(gate_capacity),
            qubit_count,
        }
    }

    /// Add a gate to the circuit
    pub fn add_gate(&mut self, gate: QuantumGate) {
        self.gates.push(gate);
    }

    /// Add multiple gates efficiently
    pub fn add_gates(&mut self, gates: Vec<QuantumGate>) {
        self.gates.extend(gates);
    }

    /// Insert gate at specific position
    pub fn insert_gate(&mut self, index: usize, gate: QuantumGate) {
        self.gates.insert(index, gate);
    }

    /// Remove gate at specific position
    pub fn remove_gate(&mut self, index: usize) -> Option<QuantumGate> {
        if index < self.gates.len() {
            Some(self.gates.remove(index))
        } else {
            None
        }
    }

    /// Get circuit depth (simplified calculation)
    pub fn depth(&self) -> usize {
        // Simplified depth calculation - in practice would track dependencies
        self.gates.len()
    }

    /// Get gate count
    pub fn gate_count(&self) -> usize {
        self.gates.len()
    }

    /// Check if circuit is empty
    pub fn is_empty(&self) -> bool {
        self.gates.is_empty()
    }

    /// Clear all gates
    pub fn clear(&mut self) {
        self.gates.clear();
    }

    /// Get gates affecting specific qubit
    pub fn gates_on_qubit(&self, qubit: usize) -> Vec<&QuantumGate> {
        self.gates.iter().filter(|gate| gate.affects_qubit(qubit)).collect()
    }

    /// Count gates of specific type
    pub fn count_gate_type(&self, gate_type: &str) -> usize {
        self.gates.iter().filter(|gate| gate.gate_type() == gate_type).count()
    }

    /// Get single-qubit gate count
    pub fn single_qubit_gate_count(&self) -> usize {
        self.gates.iter().filter(|gate| gate.is_single_qubit()).count()
    }

    /// Get two-qubit gate count
    pub fn two_qubit_gate_count(&self) -> usize {
        self.gates.iter().filter(|gate| gate.is_two_qubit()).count()
    }

    /// Get measurement count
    pub fn measurement_count(&self) -> usize {
        self.gates.iter().filter(|gate| gate.is_measurement()).count()
    }

    /// Reverse the circuit (for uncomputation)
    pub fn reverse(&self) -> Self {
        let mut reversed = Self::with_capacity(self.qubit_count, self.gates.len());
        for gate in self.gates.iter().rev() {
            reversed.add_gate(gate.inverse());
        }
        reversed
    }

    /// Compose with another circuit
    pub fn compose(&mut self, other: &QuantumCircuit) {
        if self.qubit_count == other.qubit_count {
            self.gates.extend(other.gates.clone());
        }
    }

    /// Get all qubits used in the circuit
    pub fn used_qubits(&self) -> HashSet<usize> {
        let mut qubits = HashSet::new();
        for gate in &self.gates {
            qubits.extend(gate.affected_qubits());
        }
        qubits
    }
}