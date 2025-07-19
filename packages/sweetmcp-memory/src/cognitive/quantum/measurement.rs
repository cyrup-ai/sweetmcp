//! Quantum measurement implementation

use crate::cognitive::quantum::{Complex64, types::CognitiveResult};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Measurement basis for quantum measurements
#[derive(Debug, Clone)]
pub struct MeasurementBasis {
    pub basis_vectors: Vec<Vec<Complex64>>,
    pub basis_type: BasisType,
    pub measurement_operators: Vec<MeasurementOperator>,
}

/// Basis types for quantum measurements
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BasisType {
    Computational,
    Hadamard,
    Bell,
    Custom(String),
}

/// Measurement operator for quantum measurements
#[derive(Debug, Clone)]
pub struct MeasurementOperator {
    pub matrix: Vec<Vec<Complex64>>,
    pub eigenvalues: Vec<f64>,
    pub eigenvectors: Vec<Vec<Complex64>>,
}

/// Measurement metadata
#[derive(Debug, Clone)]
pub struct MeasurementMetadata {
    pub measurement_time: Instant,
    pub environmental_conditions: EnvironmentalConditions,
    pub measurement_shots: usize,
    pub measurement_uncertainty: f64,
    pub calibration_data: CalibrationData,
}

/// Environmental conditions during measurement
#[derive(Debug, Clone)]
pub struct EnvironmentalConditions {
    pub temperature: f64,
    pub magnetic_field: f64,
    pub electromagnetic_noise: f64,
    pub system_load: f64,
}

/// Calibration data for measurements
#[derive(Debug, Clone)]
pub struct CalibrationData {
    pub readout_fidelity: f64,
    pub gate_fidelity: f64,
    pub coherence_time_t1: Duration,
    pub coherence_time_t2: Duration,
    pub calibration_timestamp: Instant,
    pub temperature_drift: f64,
    pub frequency_drift: f64,
}

impl MeasurementBasis {
    /// Create computational basis (|0⟩, |1⟩)
    pub fn computational() -> Self {
        let basis_vectors = vec![
            vec![Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0)], // |0⟩
            vec![Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0)], // |1⟩
        ];

        let measurement_operators = Self::generate_computational_operators();

        Self {
            basis_vectors,
            basis_type: BasisType::Computational,
            measurement_operators,
        }
    }

    /// Create Hadamard basis (|+⟩, |-⟩)
    pub fn hadamard() -> Self {
        let sqrt_half = (0.5_f64).sqrt();

        let basis_vectors = vec![
            vec![
                Complex64::new(sqrt_half, 0.0),
                Complex64::new(sqrt_half, 0.0),
            ], // |+⟩
            vec![
                Complex64::new(sqrt_half, 0.0),
                Complex64::new(-sqrt_half, 0.0),
            ], // |-⟩
        ];

        let measurement_operators = Self::generate_hadamard_operators();

        Self {
            basis_vectors,
            basis_type: BasisType::Hadamard,
            measurement_operators,
        }
    }

    /// Create Bell basis for entangled states
    pub fn bell() -> Self {
        let sqrt_half = (0.5_f64).sqrt();

        let basis_vectors = vec![
            // |Φ+⟩ = (|00⟩ + |11⟩)/√2
            vec![
                Complex64::new(sqrt_half, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(sqrt_half, 0.0),
            ],
            // |Φ-⟩ = (|00⟩ - |11⟩)/√2
            vec![
                Complex64::new(sqrt_half, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(-sqrt_half, 0.0),
            ],
            // |Ψ+⟩ = (|01⟩ + |10⟩)/√2
            vec![
                Complex64::new(0.0, 0.0),
                Complex64::new(sqrt_half, 0.0),
                Complex64::new(sqrt_half, 0.0),
                Complex64::new(0.0, 0.0),
            ],
            // |Ψ-⟩ = (|01⟩ - |10⟩)/√2
            vec![
                Complex64::new(0.0, 0.0),
                Complex64::new(sqrt_half, 0.0),
                Complex64::new(-sqrt_half, 0.0),
                Complex64::new(0.0, 0.0),
            ],
        ];

        let measurement_operators = Self::generate_bell_operators();

        Self {
            basis_vectors,
            basis_type: BasisType::Bell,
            measurement_operators,
        }
    }

    /// Generate computational basis measurement operators
    fn generate_computational_operators() -> Vec<MeasurementOperator> {
        vec![
            // |0⟩⟨0| projector
            MeasurementOperator {
                matrix: vec![
                    vec![Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0)],
                    vec![Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0)],
                ],
                eigenvalues: vec![1.0, 0.0],
                eigenvectors: vec![
                    vec![Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0)],
                    vec![Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0)],
                ],
            },
            // |1⟩⟨1| projector
            MeasurementOperator {
                matrix: vec![
                    vec![Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0)],
                    vec![Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0)],
                ],
                eigenvalues: vec![0.0, 1.0],
                eigenvectors: vec![
                    vec![Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0)],
                    vec![Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0)],
                ],
            },
        ]
    }

    /// Generate Hadamard basis measurement operators
    fn generate_hadamard_operators() -> Vec<MeasurementOperator> {
        vec![
            // |+⟩⟨+| projector
            MeasurementOperator {
                matrix: vec![
                    vec![Complex64::new(0.5, 0.0), Complex64::new(0.5, 0.0)],
                    vec![Complex64::new(0.5, 0.0), Complex64::new(0.5, 0.0)],
                ],
                eigenvalues: vec![1.0, 0.0],
                eigenvectors: vec![
                    vec![Complex64::new(0.707, 0.0), Complex64::new(0.707, 0.0)],
                    vec![Complex64::new(0.707, 0.0), Complex64::new(-0.707, 0.0)],
                ],
            },
            // |-⟩⟨-| projector
            MeasurementOperator {
                matrix: vec![
                    vec![Complex64::new(0.5, 0.0), Complex64::new(-0.5, 0.0)],
                    vec![Complex64::new(-0.5, 0.0), Complex64::new(0.5, 0.0)],
                ],
                eigenvalues: vec![0.0, 1.0],
                eigenvectors: vec![
                    vec![Complex64::new(0.707, 0.0), Complex64::new(0.707, 0.0)],
                    vec![Complex64::new(0.707, 0.0), Complex64::new(-0.707, 0.0)],
                ],
            },
        ]
    }

    /// Generate Bell basis measurement operators
    fn generate_bell_operators() -> Vec<MeasurementOperator> {
        // Simplified Bell measurement operators
        // In practice, these would be 4x4 matrices for two-qubit systems
        vec![MeasurementOperator {
            matrix: vec![vec![Complex64::new(1.0, 0.0)]],
            eigenvalues: vec![1.0],
            eigenvectors: vec![vec![Complex64::new(1.0, 0.0)]],
        }]
    }

    /// Perform measurement in this basis
    pub fn measure(&self, state: &[Complex64]) -> CognitiveResult<MeasurementResult> {
        let probabilities = self.calculate_probabilities(state)?;
        let outcome = self.select_outcome(&probabilities);

        Ok(MeasurementResult {
            outcome_index: outcome,
            probability: probabilities[outcome],
            post_measurement_state: self.collapse_state(state, outcome)?,
            measurement_basis: self.basis_type.clone(),
        })
    }

    /// Calculate measurement probabilities
    fn calculate_probabilities(&self, state: &[Complex64]) -> CognitiveResult<Vec<f64>> {
        let mut probabilities = Vec::new();

        for basis_vector in &self.basis_vectors {
            let inner_product = self.complex_inner_product(state, basis_vector)?;
            probabilities.push(inner_product.magnitude().powi(2));
        }

        Ok(probabilities)
    }

    /// Calculate complex inner product
    fn complex_inner_product(
        &self,
        a: &[Complex64],
        b: &[Complex64],
    ) -> CognitiveResult<Complex64> {
        if a.len() != b.len() {
            return Err(
                crate::cognitive::quantum::types::CognitiveError::InvalidQuantumState(
                    "State vectors must have same dimension".to_string(),
                ),
            );
        }

        let mut result = Complex64::new(0.0, 0.0);
        for (ai, bi) in a.iter().zip(b.iter()) {
            result = result + ai.conjugate().multiply(bi);
        }

        Ok(result)
    }

    /// Select measurement outcome probabilistically
    fn select_outcome(&self, probabilities: &[f64]) -> usize {
        let random_value: f64 = rand::random();
        let mut cumulative = 0.0;

        for (i, &prob) in probabilities.iter().enumerate() {
            cumulative += prob;
            if random_value <= cumulative {
                return i;
            }
        }

        probabilities.len() - 1
    }

    /// Collapse state after measurement
    fn collapse_state(
        &self,
        _state: &[Complex64],
        outcome: usize,
    ) -> CognitiveResult<Vec<Complex64>> {
        // Return the basis state corresponding to the measurement outcome
        Ok(self.basis_vectors[outcome].clone())
    }
}

/// Result of a quantum measurement
#[derive(Debug, Clone)]
pub struct MeasurementResult {
    pub outcome_index: usize,
    pub probability: f64,
    pub post_measurement_state: Vec<Complex64>,
    pub measurement_basis: BasisType,
}

impl Default for MeasurementMetadata {
    fn default() -> Self {
        Self {
            measurement_time: Instant::now(),
            environmental_conditions: EnvironmentalConditions::default(),
            measurement_shots: 1000,
            measurement_uncertainty: 0.01,
            calibration_data: CalibrationData::default(),
        }
    }
}

impl Default for EnvironmentalConditions {
    fn default() -> Self {
        Self {
            temperature: 300.0,      // Room temperature in Kelvin
            magnetic_field: 0.00005, // Earth's magnetic field in Tesla
            electromagnetic_noise: 0.001,
            system_load: 0.5,
        }
    }
}

impl Default for CalibrationData {
    fn default() -> Self {
        Self {
            readout_fidelity: 0.98,
            gate_fidelity: 0.995,
            coherence_time_t1: Duration::from_micros(100),
            coherence_time_t2: Duration::from_micros(50),
            calibration_timestamp: Instant::now(),
            temperature_drift: 0.001,
            frequency_drift: 0.0001,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_computational_basis() {
        let basis = MeasurementBasis::computational();
        assert_eq!(basis.basis_vectors.len(), 2);
        assert_eq!(basis.measurement_operators.len(), 2);
        assert!(matches!(basis.basis_type, BasisType::Computational));
    }

    #[test]
    fn test_hadamard_basis() {
        let basis = MeasurementBasis::hadamard();
        assert_eq!(basis.basis_vectors.len(), 2);

        // Check that basis vectors are normalized
        for vector in &basis.basis_vectors {
            let norm: f64 = vector.iter().map(|c| c.magnitude().powi(2)).sum();
            assert!((norm - 1.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_measurement() {
        let basis = MeasurementBasis::computational();

        // Test measuring |0⟩ state
        let state = vec![Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0)];
        let result = basis.measure(&state).unwrap();

        assert_eq!(result.outcome_index, 0);
        assert!((result.probability - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_superposition_measurement() {
        let basis = MeasurementBasis::computational();

        // Test measuring |+⟩ state = (|0⟩ + |1⟩)/√2
        let sqrt_half = (0.5_f64).sqrt();
        let state = vec![
            Complex64::new(sqrt_half, 0.0),
            Complex64::new(sqrt_half, 0.0),
        ];

        let result = basis.measure(&state).unwrap();

        // Should get either 0 or 1 with 50% probability
        assert!(result.outcome_index == 0 || result.outcome_index == 1);
        assert!((result.probability - 0.5).abs() < 0.1); // Allow some variation due to randomness
    }
}
