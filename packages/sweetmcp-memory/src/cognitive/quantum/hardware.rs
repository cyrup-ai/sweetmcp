//! Quantum hardware backend configurations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Configuration for quantum router
#[derive(Debug, Clone)]
pub struct QuantumConfig {
    pub max_superposition_states: usize,
    pub default_coherence_time: Duration,
    pub decoherence_threshold: f64,
    pub max_entanglement_depth: usize,
    pub error_correction_enabled: bool,
    pub real_time_optimization: bool,
    pub hardware_backend: QuantumHardwareBackend,
    pub simulation_parameters: SimulationParameters,
}

/// Quantum hardware backend options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuantumHardwareBackend {
    Simulator {
        precision: SimulationPrecision,
        parallelization: bool,
        gpu_acceleration: bool,
    },
    IBM {
        device_name: String,
        api_token: String,
        queue_priority: Priority,
    },
    Google {
        processor_id: String,
        project_id: String,
        credentials_path: String,
    },
    IonQ {
        api_key: String,
        backend_type: String,
    },
    Rigetti {
        quantum_processor_id: String,
        api_key: String,
    },
    Azure {
        subscription_id: String,
        resource_group: String,
        workspace_name: String,
    },
    Hybrid {
        primary: Box<QuantumHardwareBackend>,
        fallback: Box<QuantumHardwareBackend>,
        failover_criteria: FailoverCriteria,
    },
}

/// Simulation precision options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimulationPrecision {
    Float32,
    Float64,
    Arbitrary { precision_bits: usize },
}

/// Queue priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Normal,
    High,
    Premium,
}

/// Failover criteria for hybrid backends
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailoverCriteria {
    pub max_queue_time: Duration,
    pub min_fidelity_threshold: f64,
    pub max_error_rate: f64,
    pub availability_threshold: f64,
}

/// Simulation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationParameters {
    pub shot_count: usize,
    pub noise_model: NoiseModel,
    pub optimization_level: usize,
    pub basis_gates: Vec<String>,
    pub coupling_map: Option<CouplingMap>,
}

/// Noise model for quantum simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoiseModel {
    pub gate_errors: HashMap<String, f64>,
    pub readout_errors: Vec<Vec<f64>>,
    pub thermal_relaxation: bool,
    pub dephasing: bool,
    pub depolarizing: bool,
    pub amplitude_damping: bool,
}

/// Coupling map for quantum hardware topology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouplingMap {
    pub edges: Vec<(usize, usize)>,
    pub topology: TopologyType,
}

/// Quantum hardware topology types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TopologyType {
    Linear,
    Grid { rows: usize, cols: usize },
    HeavyHex,
    Falcon,
    Custom,
}

impl Default for QuantumConfig {
    fn default() -> Self {
        Self {
            max_superposition_states: 1000,
            default_coherence_time: Duration::from_millis(100),
            decoherence_threshold: 0.1,
            max_entanglement_depth: 5,
            error_correction_enabled: true,
            real_time_optimization: true,
            hardware_backend: QuantumHardwareBackend::default_simulator(),
            simulation_parameters: SimulationParameters::default(),
        }
    }
}

impl QuantumHardwareBackend {
    /// Create default simulator backend
    pub fn default_simulator() -> Self {
        Self::Simulator {
            precision: SimulationPrecision::Float64,
            parallelization: true,
            gpu_acceleration: false,
        }
    }

    /// Get backend name for logging
    pub fn name(&self) -> String {
        match self {
            Self::Simulator { .. } => "Quantum Simulator".to_string(),
            Self::IBM { device_name, .. } => format!("IBM {}", device_name),
            Self::Google { processor_id, .. } => format!("Google {}", processor_id),
            Self::IonQ { backend_type, .. } => format!("IonQ {}", backend_type),
            Self::Rigetti {
                quantum_processor_id,
                ..
            } => format!("Rigetti {}", quantum_processor_id),
            Self::Azure { workspace_name, .. } => format!("Azure {}", workspace_name),
            Self::Hybrid { .. } => "Hybrid Backend".to_string(),
        }
    }

    /// Check if backend requires authentication
    pub fn requires_auth(&self) -> bool {
        match self {
            Self::Simulator { .. } => false,
            _ => true,
        }
    }

    /// Get maximum qubit count for backend
    pub fn max_qubits(&self) -> usize {
        match self {
            Self::Simulator { .. } => 30, // Practical limit for simulation
            Self::IBM { device_name, .. } => match device_name.as_str() {
                "ibmq_qasm_simulator" => 32,
                "ibm_nairobi" => 7,
                "ibm_lagos" => 7,
                "ibm_perth" => 7,
                "ibm_brisbane" => 127,
                _ => 5,
            },
            Self::Google { processor_id, .. } => match processor_id.as_str() {
                "sycamore" => 54,
                "rainbow" => 23,
                _ => 23,
            },
            Self::IonQ { .. } => 32,
            Self::Rigetti { .. } => 40,
            Self::Azure { .. } => 20,
            Self::Hybrid { primary, .. } => primary.max_qubits(),
        }
    }
}

impl Default for SimulationParameters {
    fn default() -> Self {
        Self {
            shot_count: 1024,
            noise_model: NoiseModel::default(),
            optimization_level: 2,
            basis_gates: vec![
                "u1".to_string(),
                "u2".to_string(),
                "u3".to_string(),
                "cx".to_string(),
            ],
            coupling_map: None,
        }
    }
}

impl Default for NoiseModel {
    fn default() -> Self {
        let mut gate_errors = HashMap::new();
        gate_errors.insert("u1".to_string(), 0.0);
        gate_errors.insert("u2".to_string(), 0.001);
        gate_errors.insert("u3".to_string(), 0.001);
        gate_errors.insert("cx".to_string(), 0.01);

        Self {
            gate_errors,
            readout_errors: vec![
                vec![0.985, 0.015], // P(0|0), P(1|0)
                vec![0.02, 0.98],   // P(0|1), P(1|1)
            ],
            thermal_relaxation: true,
            dephasing: true,
            depolarizing: false,
            amplitude_damping: true,
        }
    }
}

impl NoiseModel {
    /// Create a noiseless model for testing
    pub fn noiseless() -> Self {
        Self {
            gate_errors: HashMap::new(),
            readout_errors: vec![vec![1.0, 0.0], vec![0.0, 1.0]],
            thermal_relaxation: false,
            dephasing: false,
            depolarizing: false,
            amplitude_damping: false,
        }
    }

    /// Create a realistic noise model based on current hardware
    pub fn realistic() -> Self {
        let mut gate_errors = HashMap::new();
        gate_errors.insert("u1".to_string(), 0.0);
        gate_errors.insert("u2".to_string(), 0.00095);
        gate_errors.insert("u3".to_string(), 0.00095);
        gate_errors.insert("cx".to_string(), 0.00729);
        gate_errors.insert("cz".to_string(), 0.00845);

        Self {
            gate_errors,
            readout_errors: vec![vec![0.9925, 0.0075], vec![0.0118, 0.9882]],
            thermal_relaxation: true,
            dephasing: true,
            depolarizing: true,
            amplitude_damping: true,
        }
    }

    /// Get average gate error rate
    pub fn average_gate_error(&self) -> f64 {
        if self.gate_errors.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.gate_errors.values().sum();
        sum / self.gate_errors.len() as f64
    }

    /// Get readout error rate
    pub fn readout_error_rate(&self) -> f64 {
        if self.readout_errors.len() >= 2 {
            // Average of misclassification probabilities
            (self.readout_errors[0][1] + self.readout_errors[1][0]) / 2.0
        } else {
            0.0
        }
    }
}

impl CouplingMap {
    /// Create a linear coupling map
    pub fn linear(num_qubits: usize) -> Self {
        let mut edges = Vec::new();
        for i in 0..num_qubits - 1 {
            edges.push((i, i + 1));
        }

        Self {
            edges,
            topology: TopologyType::Linear,
        }
    }

    /// Create a grid coupling map
    pub fn grid(rows: usize, cols: usize) -> Self {
        let mut edges = Vec::new();

        // Horizontal connections
        for r in 0..rows {
            for c in 0..cols - 1 {
                let idx1 = r * cols + c;
                let idx2 = r * cols + c + 1;
                edges.push((idx1, idx2));
            }
        }

        // Vertical connections
        for r in 0..rows - 1 {
            for c in 0..cols {
                let idx1 = r * cols + c;
                let idx2 = (r + 1) * cols + c;
                edges.push((idx1, idx2));
            }
        }

        Self {
            edges,
            topology: TopologyType::Grid { rows, cols },
        }
    }

    /// Check if two qubits are connected
    pub fn are_connected(&self, q1: usize, q2: usize) -> bool {
        self.edges.contains(&(q1, q2)) || self.edges.contains(&(q2, q1))
    }

    /// Get neighbors of a qubit
    pub fn neighbors(&self, qubit: usize) -> Vec<usize> {
        let mut neighbors = Vec::new();

        for &(a, b) in &self.edges {
            if a == qubit {
                neighbors.push(b);
            } else if b == qubit {
                neighbors.push(a);
            }
        }

        neighbors.sort();
        neighbors.dedup();
        neighbors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = QuantumConfig::default();
        assert_eq!(config.max_superposition_states, 1000);
        assert!(config.error_correction_enabled);
        assert!(matches!(
            config.hardware_backend,
            QuantumHardwareBackend::Simulator { .. }
        ));
    }

    #[test]
    fn test_backend_names() {
        let simulator = QuantumHardwareBackend::default_simulator();
        assert_eq!(simulator.name(), "Quantum Simulator");

        let ibm = QuantumHardwareBackend::IBM {
            device_name: "ibm_nairobi".to_string(),
            api_token: "token".to_string(),
            queue_priority: Priority::Normal,
        };
        assert_eq!(ibm.name(), "IBM ibm_nairobi");
    }

    #[test]
    fn test_noise_models() {
        let noiseless = NoiseModel::noiseless();
        assert_eq!(noiseless.average_gate_error(), 0.0);
        assert_eq!(noiseless.readout_error_rate(), 0.0);

        let realistic = NoiseModel::realistic();
        assert!(realistic.average_gate_error() > 0.0);
        assert!(realistic.readout_error_rate() > 0.0);
    }

    #[test]
    fn test_coupling_maps() {
        let linear = CouplingMap::linear(5);
        assert_eq!(linear.edges.len(), 4);
        assert!(linear.are_connected(0, 1));
        assert!(!linear.are_connected(0, 2));

        let grid = CouplingMap::grid(2, 3);
        assert_eq!(grid.edges.len(), 7); // 3 horizontal + 4 vertical

        let neighbors = linear.neighbors(2);
        assert_eq!(neighbors, vec![1, 3]);
    }
}
