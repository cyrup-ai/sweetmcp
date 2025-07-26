//! Quantum measurement results for error correction
//!
//! This module provides measurement result structures for
//! error correction with fidelity and probability tracking.

use crate::cognitive::quantum::{Complex64, entanglement::MeasurementBasis};
use std::collections::BTreeMap;

/// Metadata for quantum measurements - stub implementation
/// TODO: Move to appropriate module and implement properly
#[derive(Debug, Clone, Default)]
pub struct MeasurementMetadata {
    pub timestamp: Option<std::time::SystemTime>,
    pub measurement_id: Option<String>,
    pub error_rate: Option<f64>,
}

/// Quantum measurement result for error correction
#[derive(Debug, Clone)]
pub struct QuantumMeasurementResult {
    pub outcome_context: String,
    pub probability: f64,
    pub fidelity: f64,
    pub post_measurement_state: BTreeMap<String, Complex64>,
    pub measurement_basis: MeasurementBasis,
    pub measurement_metadata: MeasurementMetadata,
}

impl QuantumMeasurementResult {
    /// Create a new measurement result
    pub fn new(outcome_context: String, probability: f64, fidelity: f64) -> Self {
        Self {
            outcome_context,
            probability,
            fidelity,
            post_measurement_state: BTreeMap::new(),
            measurement_basis: MeasurementBasis::computational(),
            measurement_metadata: MeasurementMetadata::default(),
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