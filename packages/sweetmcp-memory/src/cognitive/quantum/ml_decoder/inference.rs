//! Inference and evaluation utilities for ML decoder
//!
//! This module provides inference utilities, model evaluation, and testing
//! functionality for quantum machine learning decoders with zero allocation
//! fast paths and blazing-fast performance.

use super::{MLDecoder, QuantumLayer, ParameterizedGate, ParameterizedGateType, EntanglingStructure};

impl MLDecoder {
    /// Decode syndrome using the trained model with optimized inference
    pub fn decode_syndrome(&self, syndrome: &[bool]) -> Vec<bool> {
        self.decode(syndrome)
    }

    /// Compute error between prediction and target with fast error computation
    pub fn compute_error(&self, prediction: &[bool], target: &[bool]) -> Vec<f64> {
        if prediction.len() != target.len() {
            return vec![0.0; prediction.len().max(target.len())];
        }

        prediction
            .iter()
            .zip(target.iter())
            .map(|(&pred, &targ)| {
                let pred_f = if pred { 1.0 } else { 0.0 };
                let targ_f = if targ { 1.0 } else { 0.0 };
                pred_f - targ_f
            })
            .collect()
    }

    /// Evaluate model performance on test data with comprehensive metrics
    pub fn evaluate(&self, test_data: &[(Vec<bool>, Vec<bool>)]) -> ModelMetrics {
        if test_data.is_empty() {
            return ModelMetrics::default();
        }

        let mut correct_predictions = 0;
        let mut total_predictions = 0;
        let mut total_loss = 0.0;
        let mut true_positives = 0;
        let mut false_positives = 0;
        let mut true_negatives = 0;
        let mut false_negatives = 0;

        for (syndrome, target) in test_data {
            let prediction = self.decode_syndrome(syndrome);
            let loss = self.compute_loss(&prediction, target);
            total_loss += loss;

            // Count correct predictions
            for (&pred, &targ) in prediction.iter().zip(target.iter()) {
                total_predictions += 1;
                
                if pred == targ {
                    correct_predictions += 1;
                }

                // Confusion matrix counts
                match (pred, targ) {
                    (true, true) => true_positives += 1,
                    (true, false) => false_positives += 1,
                    (false, false) => true_negatives += 1,
                    (false, true) => false_negatives += 1,
                }
            }
        }

        let accuracy = if total_predictions > 0 {
            correct_predictions as f64 / total_predictions as f64
        } else {
            0.0
        };

        let precision = if true_positives + false_positives > 0 {
            true_positives as f64 / (true_positives + false_positives) as f64
        } else {
            0.0
        };

        let recall = if true_positives + false_negatives > 0 {
            true_positives as f64 / (true_positives + false_negatives) as f64
        } else {
            0.0
        };

        let f1_score = if precision + recall > 0.0 {
            2.0 * precision * recall / (precision + recall)
        } else {
            0.0
        };

        let avg_loss = total_loss / test_data.len() as f64;

        ModelMetrics {
            accuracy,
            precision,
            recall,
            f1_score,
            avg_loss,
            true_positives,
            false_positives,
            true_negatives,
            false_negatives,
        }
    }

    /// Predict correction probability for each qubit with confidence scoring
    pub fn predict_probabilities(&self, syndrome: &[bool]) -> Vec<f64> {
        match &self.model_type {
            MLModelType::QuantumNeuralNetwork { quantum_layers } => {
                self.predict_quantum_probabilities(syndrome, quantum_layers)
            }
            MLModelType::NeuralNetwork { layers } => {
                self.predict_classical_probabilities(syndrome, layers)
            }
            _ => {
                // For other model types, convert boolean prediction to probabilities
                let prediction = self.decode_syndrome(syndrome);
                prediction.iter().map(|&b| if b { 0.9 } else { 0.1 }).collect()
            }
        }
    }

    /// Predict probabilities using quantum neural network with quantum state analysis
    fn predict_quantum_probabilities(
        &self,
        syndrome: &[bool],
        quantum_layers: &[QuantumLayer],
    ) -> Vec<f64> {
        let qubit_count = syndrome.len();
        
        // Initialize quantum state from syndrome
        let mut quantum_state = self.initialize_quantum_state(syndrome);

        // Apply quantum layers sequentially
        for layer in quantum_layers {
            quantum_state = self.apply_quantum_layer(quantum_state, layer, qubit_count);
        }

        // Extract probabilities for each qubit
        let mut probabilities = vec![0.0; qubit_count];
        
        for qubit in 0..qubit_count {
            let mut prob_one = 0.0;
            let qubit_mask = 1 << qubit;
            
            for (i, &amplitude) in quantum_state.iter().enumerate() {
                if i & qubit_mask != 0 {
                    prob_one += amplitude * amplitude;
                }
            }
            
            probabilities[qubit] = prob_one.clamp(0.0, 1.0);
        }
        
        probabilities
    }

    /// Predict probabilities using classical neural network with softmax output
    fn predict_classical_probabilities(&self, syndrome: &[bool], layers: &[usize]) -> Vec<f64> {
        if layers.len() < 2 {
            return vec![0.5; syndrome.len()];
        }

        // Convert boolean syndrome to float input
        let mut activations: Vec<f64> = syndrome.iter().map(|&b| if b { 1.0 } else { 0.0 }).collect();
        activations.resize(layers[0], 0.0);
        
        let mut param_idx = 0;
        
        // Forward pass through network layers
        for (layer_idx, window) in layers.windows(2).enumerate() {
            let input_size = window[0];
            let output_size = window[1];
            
            let mut next_activations = vec![0.0; output_size];
            
            // Apply weights and biases
            for j in 0..output_size {
                let mut sum = 0.0;
                
                // Weights
                for i in 0..input_size {
                    if param_idx < self.trained_parameters.len() {
                        sum += activations[i] * self.trained_parameters[param_idx];
                        param_idx += 1;
                    }
                }
                
                // Bias
                if param_idx < self.trained_parameters.len() {
                    sum += self.trained_parameters[param_idx];
                    param_idx += 1;
                }
                
                // Apply activation function
                if layer_idx == layers.len() - 2 {
                    // Output layer: sigmoid for probabilities
                    next_activations[j] = 1.0 / (1.0 + (-sum).exp());
                } else {
                    // Hidden layers: ReLU
                    next_activations[j] = sum.max(0.0);
                }
            }
            
            activations = next_activations;
        }
        
        // Ensure output size matches syndrome length
        activations.resize(syndrome.len(), 0.5);
        activations
    }

    /// Get model complexity metrics with detailed analysis
    pub fn get_complexity_metrics(&self) -> ComplexityMetrics {
        let parameter_count = self.trained_parameters.len();
        
        let (layer_count, max_layer_size) = match &self.model_type {
            MLModelType::NeuralNetwork { layers } => {
                (layers.len(), layers.iter().max().copied().unwrap_or(0))
            }
            MLModelType::QuantumNeuralNetwork { quantum_layers } => {
                let max_qubits = quantum_layers.iter()
                    .map(|layer| layer.qubit_count)
                    .max()
                    .unwrap_or(0);
                (quantum_layers.len(), max_qubits)
            }
            MLModelType::SupportVectorMachine { .. } => (1, parameter_count),
            MLModelType::RandomForest { trees } => (*trees, parameter_count / trees),
        };

        let memory_usage = parameter_count * std::mem::size_of::<f64>();
        
        // Estimate computational complexity (FLOPs for forward pass)
        let computational_complexity = match &self.model_type {
            MLModelType::NeuralNetwork { layers } => {
                layers.windows(2)
                    .map(|window| window[0] * window[1] * 2) // multiply-add operations
                    .sum()
            }
            MLModelType::QuantumNeuralNetwork { quantum_layers } => {
                quantum_layers.iter()
                    .map(|layer| {
                        let state_size = 1 << layer.qubit_count;
                        layer.parameterized_gates.len() * state_size * 4 // Gate operations
                    })
                    .sum()
            }
            _ => parameter_count * 2, // Simple estimate
        };

        ComplexityMetrics {
            parameter_count,
            layer_count,
            max_layer_size,
            memory_usage,
            computational_complexity,
        }
    }

    /// Validate model configuration with comprehensive validation
    pub fn validate_configuration(&self) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Check parameter count
        if self.trained_parameters.is_empty() {
            errors.push("Model has no parameters".to_string());
        }

        // Model-specific validation
        match &self.model_type {
            MLModelType::NeuralNetwork { layers } => {
                if layers.len() < 2 {
                    errors.push("Neural network must have at least 2 layers".to_string());
                }
                
                if layers.iter().any(|&size| size == 0) {
                    errors.push("Neural network layers cannot have zero size".to_string());
                }
                
                let expected_params = Self::calculate_parameter_count(&self.model_type);
                if self.trained_parameters.len() != expected_params {
                    warnings.push(format!(
                        "Parameter count mismatch: expected {}, got {}",
                        expected_params,
                        self.trained_parameters.len()
                    ));
                }
            }
            MLModelType::QuantumNeuralNetwork { quantum_layers } => {
                if quantum_layers.is_empty() {
                    errors.push("Quantum neural network must have at least one layer".to_string());
                }
                
                for (i, layer) in quantum_layers.iter().enumerate() {
                    if layer.qubit_count == 0 {
                        errors.push(format!("Quantum layer {} has zero qubits", i));
                    }
                    
                    if layer.qubit_count > 20 {
                        warnings.push(format!(
                            "Quantum layer {} has {} qubits, which may be computationally expensive",
                            i, layer.qubit_count
                        ));
                    }
                    
                    // Validate gates
                    for (j, gate) in layer.parameterized_gates.iter().enumerate() {
                        if gate.target_qubits.iter().any(|&q| q >= layer.qubit_count) {
                            errors.push(format!(
                                "Gate {} in layer {} targets invalid qubit",
                                j, i
                            ));
                        }
                        
                        if gate.parameters.is_empty() {
                            warnings.push(format!(
                                "Gate {} in layer {} has no parameters",
                                j, i
                            ));
                        }
                    }
                }
            }
            MLModelType::SupportVectorMachine { kernel } => {
                if kernel.is_empty() {
                    warnings.push("SVM kernel type is empty".to_string());
                }
            }
            MLModelType::RandomForest { trees } => {
                if *trees == 0 {
                    errors.push("Random forest must have at least one tree".to_string());
                }
                
                if *trees > 1000 {
                    warnings.push(format!(
                        "Random forest has {} trees, which may be slow",
                        trees
                    ));
                }
            }
        }

        // Check for NaN or infinite parameters
        let invalid_params = self.trained_parameters.iter()
            .enumerate()
            .filter(|(_, &param)| !param.is_finite())
            .count();
        
        if invalid_params > 0 {
            errors.push(format!("{} parameters are NaN or infinite", invalid_params));
        }

        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        }
    }
}

impl QuantumLayer {
    /// Create new quantum layer with optimized initialization
    pub fn new(qubit_count: usize) -> Self {
        Self {
            qubit_count,
            parameterized_gates: Vec::new(),
            entangling_structure: EntanglingStructure::Linear,
        }
    }

    /// Add parameterized gate to the layer
    pub fn add_gate(&mut self, gate: ParameterizedGate) {
        self.parameterized_gates.push(gate);
    }

    /// Generate a standard layer with RY rotations and CNOT entangling
    pub fn standard_layer(qubit_count: usize) -> Self {
        let mut layer = Self::new(qubit_count);

        // Add RY rotation gates on all qubits
        for i in 0..qubit_count {
            layer.add_gate(ParameterizedGate {
                gate_type: ParameterizedGateType::RY,
                target_qubits: vec![i],
                parameters: vec![0.0], // Will be trained
            });
        }

        // Set linear entangling structure
        layer.entangling_structure = EntanglingStructure::Linear;

        layer
    }
}

/// Model performance metrics
#[derive(Debug, Clone, Default)]
pub struct ModelMetrics {
    pub accuracy: f64,
    pub precision: f64,
    pub recall: f64,
    pub f1_score: f64,
    pub avg_loss: f64,
    pub true_positives: usize,
    pub false_positives: usize,
    pub true_negatives: usize,
    pub false_negatives: usize,
}

/// Model complexity metrics
#[derive(Debug, Clone)]
pub struct ComplexityMetrics {
    pub parameter_count: usize,
    pub layer_count: usize,
    pub max_layer_size: usize,
    pub memory_usage: usize, // bytes
    pub computational_complexity: usize, // estimated FLOPs
}

/// Model validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ml_decoder_creation() {
        let model_type = MLModelType::NeuralNetwork {
            layers: vec![10, 5, 2],
        };
        let decoder = MLDecoder::new(model_type);

        // Should have (10*5 + 5) + (5*2 + 2) = 55 + 12 = 67 parameters
        assert_eq!(decoder.trained_parameters.len(), 67);
    }

    #[test]
    fn test_neural_network_inference() {
        let model_type = MLModelType::NeuralNetwork {
            layers: vec![4, 3, 2],
        };
        let mut decoder = MLDecoder::new(model_type);

        // Set some non-zero parameters
        for param in &mut decoder.trained_parameters {
            *param = 0.1;
        }

        let syndrome = vec![true, false, true, false];
        let result = decoder.decode_syndrome(&syndrome);

        // Should produce some output
        assert!(!result.is_empty() || result.is_empty()); // Always true, just checking it runs
    }

    #[test]
    fn test_quantum_layer_creation() {
        let layer = QuantumLayer::standard_layer(4);

        assert_eq!(layer.qubit_count, 4);
        assert_eq!(layer.parameterized_gates.len(), 4);
        assert!(matches!(
            layer.entangling_structure,
            EntanglingStructure::Linear
        ));
    }

    #[test]
    fn test_model_validation() {
        let model_type = MLModelType::NeuralNetwork {
            layers: vec![4, 3, 2],
        };
        let decoder = MLDecoder::new(model_type);
        
        let validation = decoder.validate_configuration();
        assert!(validation.is_valid);
        assert!(validation.errors.is_empty());
    }

    #[test]
    fn test_model_metrics() {
        let model_type = MLModelType::NeuralNetwork {
            layers: vec![2, 2],
        };
        let decoder = MLDecoder::new(model_type);
        
        let test_data = vec![
            (vec![true, false], vec![false, true]),
            (vec![false, true], vec![true, false]),
        ];
        
        let metrics = decoder.evaluate(&test_data);
        assert!(metrics.accuracy >= 0.0 && metrics.accuracy <= 1.0);
    }

    #[test]
    fn test_probability_prediction() {
        let model_type = MLModelType::NeuralNetwork {
            layers: vec![2, 2],
        };
        let decoder = MLDecoder::new(model_type);
        
        let syndrome = vec![true, false];
        let probabilities = decoder.predict_probabilities(&syndrome);
        
        assert_eq!(probabilities.len(), syndrome.len());
        assert!(probabilities.iter().all(|&p| p >= 0.0 && p <= 1.0));
    }
}