//! Quantum superposition state management

use crate::cognitive::quantum::{Complex64, EntanglementLink};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

/// Quantum superposition state with full quantum properties
#[derive(Debug, Clone)]
pub struct SuperpositionState {
    pub probability_amplitudes: BTreeMap<String, Complex64>,
    pub coherence_time: Duration,
    pub last_observation: Option<Instant>,
    pub entangled_memories: Vec<EntanglementLink>,
    pub phase_evolution: PhaseEvolution,
    pub decoherence_rate: f64,
    pub creation_time: Instant,
    pub observation_count: u64,
}

/// Phase evolution tracking for quantum states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseEvolution {
    pub initial_phase: f64,
    pub evolution_rate: f64,
    pub hamiltonian_coefficients: Vec<f64>,
    pub time_dependent_terms: Vec<TimeDependentTerm>,
}

/// Time-dependent term for Hamiltonian evolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeDependentTerm {
    pub amplitude: f64,
    pub frequency: f64,
    pub phase_offset: f64,
}

impl SuperpositionState {
    /// Create a new superposition state
    pub fn new(coherence_time: Duration) -> Self {
        Self {
            probability_amplitudes: BTreeMap::new(),
            coherence_time,
            last_observation: None,
            entangled_memories: Vec::new(),
            phase_evolution: PhaseEvolution::default(),
            decoherence_rate: 0.01,
            creation_time: Instant::now(),
            observation_count: 0,
        }
    }

    /// Add a quantum state with given amplitude
    pub fn add_state(&mut self, label: String, amplitude: Complex64) {
        self.probability_amplitudes.insert(label, amplitude);
    }

    /// Normalize the superposition to maintain quantum constraint
    pub fn normalize(&mut self) -> Result<(), String> {
        let total_probability: f64 = self
            .probability_amplitudes
            .values()
            .map(|amp| amp.magnitude().powi(2))
            .sum();

        if total_probability == 0.0 {
            return Err("Cannot normalize: zero total probability".to_string());
        }

        let normalization_factor = total_probability.sqrt();
        for amplitude in self.probability_amplitudes.values_mut() {
            *amplitude = *amplitude / normalization_factor;
        }

        Ok(())
    }

    /// Check if the state is still coherent
    pub fn is_coherent(&self) -> bool {
        let elapsed = self.creation_time.elapsed();
        elapsed < self.coherence_time
    }

    /// Calculate the von Neumann entropy of the state
    pub fn entropy(&self) -> f64 {
        let mut entropy = 0.0;

        for amplitude in self.probability_amplitudes.values() {
            let probability = amplitude.magnitude().powi(2);
            if probability > 0.0 {
                entropy -= probability * probability.ln();
            }
        }

        entropy
    }

    /// Apply decoherence based on elapsed time
    pub fn apply_decoherence(&mut self, elapsed: Duration) {
        let decay_factor = (-self.decoherence_rate * elapsed.as_secs_f64()).exp();

        for amplitude in self.probability_amplitudes.values_mut() {
            *amplitude = *amplitude * decay_factor;
        }
    }

    /// Mark state as observed
    pub fn observe(&mut self) {
        self.last_observation = Some(Instant::now());
        self.observation_count += 1;
    }
}

impl Default for PhaseEvolution {
    fn default() -> Self {
        Self {
            initial_phase: 0.0,
            evolution_rate: 1.0,
            hamiltonian_coefficients: Vec::new(),
            time_dependent_terms: Vec::new(),
        }
    }
}

impl PhaseEvolution {
    /// Calculate the phase at a given time
    pub fn phase_at_time(&self, time: f64) -> f64 {
        let mut phase = self.initial_phase + self.evolution_rate * time;

        // Add time-dependent contributions
        for term in &self.time_dependent_terms {
            phase += term.amplitude * (term.frequency * time + term.phase_offset).sin();
        }

        phase
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_superposition_normalization() {
        let mut state = SuperpositionState::new(Duration::from_secs(1));
        state.add_state("state1".to_string(), Complex64::new(0.6, 0.0));
        state.add_state("state2".to_string(), Complex64::new(0.8, 0.0));

        state.normalize().unwrap();

        let total_prob: f64 = state
            .probability_amplitudes
            .values()
            .map(|amp| amp.magnitude().powi(2))
            .sum();

        assert!((total_prob - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_entropy_calculation() {
        let mut state = SuperpositionState::new(Duration::from_secs(1));
        state.add_state("state1".to_string(), Complex64::new(1.0, 0.0));
        state.normalize().unwrap();

        // Single state should have zero entropy
        assert_eq!(state.entropy(), 0.0);

        // Equal superposition should have maximum entropy
        state.add_state("state2".to_string(), Complex64::new(1.0, 0.0));
        state.normalize().unwrap();

        let entropy = state.entropy();
        assert!(entropy > 0.0);
    }
}
