//! Optimization algorithms for ML decoder training
//!
//! This module provides training algorithms including Adam, SGD, and other
//! optimization methods with zero allocation fast paths and blazing-fast performance.

use super::{MLDecoder, TrainingData};
use super::gradients::GradientMethod;
use smallvec::SmallVec;

/// Optimization backend algorithms for ML training
#[derive(Debug, Clone)]
pub enum OptimizationBackend {
    Adam { learning_rate: f64, beta1: f64, beta2: f64 },
    SGD { learning_rate: f64, momentum: f64 },
    LBFGS { max_iterations: usize },
    RMSprop { learning_rate: f64, decay_rate: f64 },
}

impl MLDecoder {
    /// Train the model using the specified training data and optimizer
    pub fn train(&mut self, training_data: &[(Vec<bool>, Vec<bool>)]) -> Result<f64, String> {
        match &self.inference_engine.optimization_backend {
            OptimizationBackend::Adam { learning_rate, beta1, beta2 } => {
                self.train_with_adam(training_data, *learning_rate, *beta1, *beta2)
            }
            OptimizationBackend::SGD { learning_rate, momentum } => {
                self.train_with_sgd(training_data, *learning_rate, *momentum)
            }
            OptimizationBackend::LBFGS { .. } => {
                self.train_with_lbfgs(training_data)
            }
            OptimizationBackend::RMSprop { learning_rate, decay_rate } => {
                self.train_with_rmsprop(training_data, *learning_rate, *decay_rate)
            }
        }
    }

    /// Train with Adam optimizer with adaptive learning rates and optimized Adam
    pub(super) fn train_with_adam(
        &mut self,
        training_data: &[(Vec<bool>, Vec<bool>)],
        lr: f64,
        beta1: f64,
        beta2: f64,
    ) -> Result<f64, String> {
        let param_count = self.trained_parameters.len();
        
        // Use SmallVec for small parameter sets
        let mut m: SmallVec<[f64; 256]> = if param_count <= 256 {
            SmallVec::new()
        } else {
            SmallVec::from_vec(vec![0.0; param_count])
        };
        m.resize(param_count, 0.0);
        
        let mut v: SmallVec<[f64; 256]> = if param_count <= 256 {
            SmallVec::new()
        } else {
            SmallVec::from_vec(vec![0.0; param_count])
        };
        v.resize(param_count, 0.0);

        let epsilon = 1e-8;
        let mut total_loss = 0.0;

        // Multiple epochs for better convergence
        for epoch in 0..50 {
            let mut epoch_loss = 0.0;
            
            for (t, (syndrome, target)) in training_data.iter().enumerate() {
                // Forward pass
                let prediction = self.decode(syndrome);
                
                // Compute loss (binary cross-entropy)
                let sample_loss = self.compute_loss(&prediction, target);
                epoch_loss += sample_loss;
                
                // Compute gradients
                let gradients = self.compute_gradients(syndrome, target, &prediction)?;
                
                // Update parameters with Adam
                for i in 0..param_count {
                    if i < gradients.len() {
                        let g = gradients[i];
                        
                        // Update biased first moment estimate
                        m[i] = beta1 * m[i] + (1.0 - beta1) * g;
                        
                        // Update biased second raw moment estimate
                        v[i] = beta2 * v[i] + (1.0 - beta2) * g * g;
                        
                        // Compute bias-corrected first moment estimate
                        let m_hat = m[i] / (1.0 - beta1.powi((t + 1) as i32));
                        
                        // Compute bias-corrected second raw moment estimate
                        let v_hat = v[i] / (1.0 - beta2.powi((t + 1) as i32));
                        
                        // Update parameter
                        self.trained_parameters[i] -= lr * m_hat / (v_hat.sqrt() + epsilon);
                    }
                }
            }
            
            total_loss = epoch_loss / training_data.len() as f64;
            
            // Early stopping if loss is very small
            if total_loss < 1e-6 {
                break;
            }
        }

        Ok(total_loss)
    }

    /// Train with SGD optimizer with momentum and optimized gradient descent
    pub(super) fn train_with_sgd(
        &mut self,
        training_data: &[(Vec<bool>, Vec<bool>)],
        lr: f64,
        momentum: f64,
    ) -> Result<f64, String> {
        let param_count = self.trained_parameters.len();
        
        // Use SmallVec for small parameter sets
        let mut velocity: SmallVec<[f64; 256]> = if param_count <= 256 {
            SmallVec::new()
        } else {
            SmallVec::from_vec(vec![0.0; param_count])
        };
        velocity.resize(param_count, 0.0);

        let mut total_loss = 0.0;

        // Multiple epochs for better convergence
        for _epoch in 0..100 {
            let mut epoch_loss = 0.0;

            for (syndrome, target) in training_data {
                // Forward pass
                let prediction = self.decode(syndrome);
                
                // Compute loss
                let sample_loss = self.compute_loss(&prediction, target);
                epoch_loss += sample_loss;
                
                // Compute gradients
                let gradients = self.compute_gradients(syndrome, target, &prediction)?;
                
                // Update parameters with momentum
                for i in 0..param_count {
                    if i < gradients.len() {
                        // Update velocity with momentum
                        velocity[i] = momentum * velocity[i] - lr * gradients[i];
                        
                        // Update parameter
                        self.trained_parameters[i] += velocity[i];
                    }
                }
            }
            
            total_loss = epoch_loss / training_data.len() as f64;
            
            // Early stopping if loss is very small
            if total_loss < 1e-6 {
                break;
            }
        }

        Ok(total_loss)
    }

    /// Train with L-BFGS optimizer with quasi-Newton optimization
    pub(super) fn train_with_lbfgs(&mut self, training_data: &[(Vec<bool>, Vec<bool>)]) -> Result<f64, String> {
        // Simplified L-BFGS implementation
        // In practice, this would use proper L-BFGS with history vectors
        let param_count = self.trained_parameters.len();
        let lr = 0.01; // Fixed learning rate for simplified implementation
        
        let mut total_loss = 0.0;

        for _epoch in 0..50 {
            let mut epoch_loss = 0.0;
            let mut accumulated_gradients = vec![0.0; param_count];

            // Accumulate gradients over all samples
            for (syndrome, target) in training_data {
                let prediction = self.decode(syndrome);
                let sample_loss = self.compute_loss(&prediction, target);
                epoch_loss += sample_loss;
                
                let gradients = self.compute_gradients(syndrome, target, &prediction)?;
                
                for i in 0..param_count {
                    if i < gradients.len() {
                        accumulated_gradients[i] += gradients[i];
                    }
                }
            }

            // Average gradients
            for gradient in &mut accumulated_gradients {
                *gradient /= training_data.len() as f64;
            }

            // Simple gradient descent update (simplified L-BFGS)
            for i in 0..param_count {
                self.trained_parameters[i] -= lr * accumulated_gradients[i];
            }

            total_loss = epoch_loss / training_data.len() as f64;
            
            if total_loss < 1e-6 {
                break;
            }
        }

        Ok(total_loss)
    }

    /// Train with RMSprop optimizer with adaptive learning rates
    pub(super) fn train_with_rmsprop(
        &mut self,
        training_data: &[(Vec<bool>, Vec<bool>)],
        lr: f64,
        decay_rate: f64,
    ) -> Result<f64, String> {
        let param_count = self.trained_parameters.len();
        
        // Use SmallVec for small parameter sets
        let mut squared_gradients: SmallVec<[f64; 256]> = if param_count <= 256 {
            SmallVec::new()
        } else {
            SmallVec::from_vec(vec![0.0; param_count])
        };
        squared_gradients.resize(param_count, 0.0);

        let epsilon = 1e-8;
        let mut total_loss = 0.0;

        // Multiple epochs for better convergence
        for _epoch in 0..50 {
            let mut epoch_loss = 0.0;

            for (syndrome, target) in training_data {
                // Forward pass
                let prediction = self.decode(syndrome);
                
                // Compute loss
                let sample_loss = self.compute_loss(&prediction, target);
                epoch_loss += sample_loss;
                
                // Compute gradients
                let gradients = self.compute_gradients(syndrome, target, &prediction)?;
                
                // Update parameters with RMSprop
                for i in 0..param_count {
                    if i < gradients.len() {
                        let g = gradients[i];
                        
                        // Update squared gradient accumulator
                        squared_gradients[i] = decay_rate * squared_gradients[i] + (1.0 - decay_rate) * g * g;
                        
                        // Update parameter
                        self.trained_parameters[i] -= lr * g / (squared_gradients[i].sqrt() + epsilon);
                    }
                }
            }
            
            total_loss = epoch_loss / training_data.len() as f64;
            
            // Early stopping if loss is very small
            if total_loss < 1e-6 {
                break;
            }
        }

        Ok(total_loss)
    }
}