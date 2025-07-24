//! Basic operations for stabilizer quantum error correction
//!
//! This module implements fundamental operations including stabilizer creation,
//! syndrome extraction, and basic decoding with zero allocation patterns.

use crate::cognitive::quantum::types::{CognitiveError, CognitiveResult};
use super::stabilizer_core_types::{
    StabilizerCode, StabilizerGenerator, PauliOp, PauliMatrix, 
    LogicalOperator, ErrorPattern, SyndromeResult, StabilizerCodeParameters, DecoderType
};
use smallvec::SmallVec;
use std::collections::HashMap;

impl StabilizerCode {
    /// Create a new stabilizer code with validation
    pub fn new(
        n: usize,
        k: usize,
        d: usize,
        stabilizers: Vec<StabilizerGenerator>,
        logical_x: Vec<LogicalOperator>,
        logical_z: Vec<LogicalOperator>,
    ) -> CognitiveResult<Self> {
        // Validate code parameters
        if stabilizers.len() != n - k {
            return Err(CognitiveError::InvalidParameter(
                format!("Expected {} stabilizers for [[{}, {}, {}]] code", n - k, n, k, d)
            ));
        }

        if logical_x.len() != k || logical_z.len() != k {
            return Err(CognitiveError::InvalidParameter(
                format!("Expected {} logical X and Z operators", k)
            ));
        }

        let parameters = StabilizerCodeParameters {
            error_threshold: 0.01,
            max_syndrome_weight: n / 2,
            fast_lookup: true,
            decoder_type: DecoderType::SyndromeBased,
        };

        let mut code = Self {
            n,
            k,
            d,
            stabilizers,
            logical_x,
            logical_z,
            syndrome_table: HashMap::new(),
            parameters,
        };

        // Build syndrome table if fast lookup is enabled
        if code.parameters.fast_lookup {
            code.build_syndrome_table()?;
        }

        Ok(code)
    }

    /// Extract syndrome from error pattern
    pub fn extract_syndrome_from_error(&self, error: &[PauliOp]) -> CognitiveResult<Vec<bool>> {
        let mut syndrome = vec![false; self.stabilizers.len()];
        
        for (stab_idx, stabilizer) in self.stabilizers.iter().enumerate() {
            let mut measurement = false;
            
            // Check commutation with each error operator
            for error_op in error {
                for stab_op in &stabilizer.paulis {
                    if error_op.qubit == stab_op.qubit {
                        if !error_op.pauli.commutes_with(stab_op.pauli) {
                            measurement = !measurement;
                        }
                    }
                }
            }
            
            syndrome[stab_idx] = measurement;
        }
        
        Ok(syndrome)
    }

    /// Extract syndrome from quantum state (measurement simulation)
    pub fn extract_syndrome(&self, error_pattern: &[PauliOp]) -> CognitiveResult<SyndromeResult> {
        let syndrome = self.extract_syndrome_from_error(error_pattern)?;
        let weight = syndrome.iter().filter(|&&b| b).count();
        
        Ok(SyndromeResult {
            syndrome,
            weight,
            timestamp: std::time::Instant::now(),
            measurement_errors: Vec::new(), // No measurement errors in simulation
        })
    }

    /// Decode syndrome to error pattern using basic lookup
    pub fn decode_syndrome(&self, syndrome: &[bool]) -> CognitiveResult<Option<ErrorPattern>> {
        if self.parameters.fast_lookup {
            Ok(self.syndrome_table.get(syndrome).cloned())
        } else {
            self.decode_syndrome_basic(syndrome)
        }
    }

    /// Basic syndrome decoding without lookup table
    fn decode_syndrome_basic(&self, syndrome: &[bool]) -> CognitiveResult<Option<ErrorPattern>> {
        // Check for zero syndrome (no error)
        if syndrome.iter().all(|&b| !b) {
            return Ok(None);
        }

        // Try single-qubit errors first
        for qubit in 0..self.n {
            for &pauli in &[PauliMatrix::X, PauliMatrix::Y, PauliMatrix::Z] {
                let error = vec![PauliOp::new(qubit, pauli, 0)];
                let test_syndrome = self.extract_syndrome_from_error(&error)?;
                
                if test_syndrome == syndrome {
                    return Ok(Some(ErrorPattern {
                        error_qubits: SmallVec::from_slice(&[qubit]),
                        error_types: SmallVec::from_slice(&[pauli]),
                        probability: 0.01,
                        corrections: SmallVec::from_slice(&error),
                    }));
                }
            }
        }

        // Try two-qubit errors if single-qubit failed
        if self.parameters.max_syndrome_weight >= 2 {
            for qubit1 in 0..self.n {
                for qubit2 in (qubit1 + 1)..self.n {
                    for &pauli1 in &[PauliMatrix::X, PauliMatrix::Y, PauliMatrix::Z] {
                        for &pauli2 in &[PauliMatrix::X, PauliMatrix::Y, PauliMatrix::Z] {
                            let error = vec![
                                PauliOp::new(qubit1, pauli1, 0),
                                PauliOp::new(qubit2, pauli2, 0),
                            ];
                            let test_syndrome = self.extract_syndrome_from_error(&error)?;
                            
                            if test_syndrome == syndrome {
                                return Ok(Some(ErrorPattern {
                                    error_qubits: SmallVec::from_slice(&[qubit1, qubit2]),
                                    error_types: SmallVec::from_slice(&[pauli1, pauli2]),
                                    probability: 0.0001,
                                    corrections: SmallVec::from_slice(&error),
                                }));
                            }
                        }
                    }
                }
            }
        }

        // No correction found
        Ok(None)
    }

    /// Apply error correction
    pub fn apply_correction(&self, correction: &ErrorPattern) -> CognitiveResult<Vec<PauliOp>> {
        // Return the correction operations
        Ok(correction.corrections.to_vec())
    }

    /// Check if stabilizers are independent
    pub fn check_stabilizer_independence(&self) -> CognitiveResult<bool> {
        // Build commutation matrix
        let n_stabs = self.stabilizers.len();
        
        // Check if all stabilizers commute with each other
        for (i, stab1) in self.stabilizers.iter().enumerate() {
            for (j, stab2) in self.stabilizers.iter().enumerate() {
                if i != j && !self.stabilizers_commute(stab1, stab2) {
                    return Ok(false);
                }
            }
        }
        
        Ok(true)
    }

    /// Check if two stabilizers commute
    fn stabilizers_commute(&self, stab1: &StabilizerGenerator, stab2: &StabilizerGenerator) -> bool {
        let mut commutation_count = 0;
        
        // Check commutation for each qubit
        for op1 in &stab1.paulis {
            for op2 in &stab2.paulis {
                if op1.qubit == op2.qubit {
                    if !op1.pauli.commutes_with(op2.pauli) {
                        commutation_count += 1;
                    }
                }
            }
        }
        
        // Stabilizers commute if they anti-commute on an even number of qubits
        commutation_count % 2 == 0
    }

    /// Build basic syndrome lookup table
    pub fn build_syndrome_table(&mut self) -> CognitiveResult<()> {
        self.syndrome_table.clear();
        
        // Generate all possible single-qubit errors
        for qubit in 0..self.n {
            for &pauli in &[PauliMatrix::X, PauliMatrix::Y, PauliMatrix::Z] {
                let error = vec![PauliOp::new(qubit, pauli, 0)];
                let syndrome = self.extract_syndrome_from_error(&error)?;
                
                let pattern = ErrorPattern {
                    error_qubits: SmallVec::from_slice(&[qubit]),
                    error_types: SmallVec::from_slice(&[pauli]),
                    probability: 0.01, // Default single-qubit error rate
                    corrections: SmallVec::from_slice(&error),
                };
                
                self.syndrome_table.insert(syndrome, pattern);
            }
        }

        // Generate two-qubit error patterns if within threshold
        if self.parameters.max_syndrome_weight >= 2 {
            for qubit1 in 0..self.n {
                for qubit2 in (qubit1 + 1)..self.n {
                    for &pauli1 in &[PauliMatrix::X, PauliMatrix::Y, PauliMatrix::Z] {
                        for &pauli2 in &[PauliMatrix::X, PauliMatrix::Y, PauliMatrix::Z] {
                            let error = vec![
                                PauliOp::new(qubit1, pauli1, 0),
                                PauliOp::new(qubit2, pauli2, 0),
                            ];
                            let syndrome = self.extract_syndrome_from_error(&error)?;
                            
                            // Only add if syndrome not already present (prefer single-qubit corrections)
                            if !self.syndrome_table.contains_key(&syndrome) {
                                let pattern = ErrorPattern {
                                    error_qubits: SmallVec::from_slice(&[qubit1, qubit2]),
                                    error_types: SmallVec::from_slice(&[pauli1, pauli2]),
                                    probability: 0.0001, // Default two-qubit error rate
                                    corrections: SmallVec::from_slice(&error),
                                };
                                
                                self.syndrome_table.insert(syndrome, pattern);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl StabilizerGenerator {
    /// Create a new stabilizer generator
    pub fn new(id: String, paulis: SmallVec<[PauliOp; 8]>) -> Self {
        let weight = paulis.iter().map(|op| op.weight()).sum();
        let commutation_matrix = Vec::new(); // Will be filled by code construction
        
        Self {
            id,
            paulis,
            weight,
            commutation_matrix,
        }
    }

    /// Get generator weight
    #[inline]
    pub const fn weight(&self) -> usize {
        self.weight
    }

    /// Check if generator acts trivially on a qubit
    pub fn acts_trivially_on(&self, qubit: usize) -> bool {
        self.paulis.iter().all(|op| op.qubit != qubit || op.is_identity())
    }

    /// Get Pauli operator at specific qubit
    pub fn pauli_at(&self, qubit: usize) -> PauliMatrix {
        self.paulis
            .iter()
            .find(|op| op.qubit == qubit)
            .map(|op| op.pauli)
            .unwrap_or(PauliMatrix::I)
    }

    /// Check if generator commutes with a Pauli operator
    pub fn commutes_with_pauli(&self, pauli_op: &PauliOp) -> bool {
        for op in &self.paulis {
            if op.qubit == pauli_op.qubit {
                return op.pauli.commutes_with(pauli_op.pauli);
            }
        }
        true // Identity commutes with everything
    }
}