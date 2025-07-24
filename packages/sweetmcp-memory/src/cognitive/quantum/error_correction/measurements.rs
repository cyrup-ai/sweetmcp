//! Quantum measurement results for error correction
//!
//! This module provides measurement result structures for
//! error correction with fidelity and probability tracking.

use crate::cognitive::quantum::Complex64;
use std::collections::BTreeMap;

/// Quantum measurement result for error correction
#[derive(Debug, Clone)]
pub struct QuantumMeasurementResult {
    pub outcome_context: String,
    pub probability: f64,
    pub fidelity: f64,
    pub post_measurement_state: BTreeMap<String, Complex64>,
    pub measurement_basis: crate::cognitive::quantum::measurement::MeasurementBasis,
    pub measurement_metadata: crate::cognitive::quantum::measurement::MeasurementMetadata,
}

impl QuantumMeasurementResult {
    /// Create a new measurement result
    pub fn new(outcome_context: String, probability: f64, fidelity: f64) -> Self {
        Self {
            outcome_context,
            probability,
            fidelity,
            post_measurement_state: BTreeMap::new(),
            measurement_basis: crate::cognitive::quantum::measurement::MeasurementBasis::computational(),
            measurement_metadata: crate::cognitive::quantum::measurement::MeasurementMetadata::default(),
        }
    }

    /// Check if measurement is high fidelity
    pub fn is_high_fidelity(&self, threshold: f64) -> bool {
        self.fidelity >= threshold
    }

    /// Check if measurement is high probability
    pub fn is_high_probability(&self, threshold: f64) -> bool {
        self.probability >= threshold
    }
}