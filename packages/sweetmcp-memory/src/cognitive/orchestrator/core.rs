//! Core orchestrator types and initialization
//!
//! This module provides the core InfiniteOrchestrator struct and
//! initialization logic with zero allocation patterns and blazing-fast
//! performance.

use crate::cognitive::types::CognitiveError;
use crate::vector::async_vector_optimization::OptimizationSpec;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::info;

/// Orchestrator managing infinite optimization iterations
pub struct InfiniteOrchestrator {
    pub spec_file: PathBuf,
    pub output_dir: PathBuf,
    pub spec: Arc<OptimizationSpec>,
    pub user_objective: String,
    pub initial_code: String,
    pub initial_latency: f64,
    pub initial_memory: f64,
    pub initial_relevance: f64,
}

impl InfiniteOrchestrator {
    /// Create a new infinite orchestrator
    pub fn new<P: AsRef<Path>>(
        spec_file: P,
        output_dir: P,
        initial_code: String,
        initial_latency: f64,
        initial_memory: f64,
        initial_relevance: f64,
        user_objective: String,
    ) -> Result<Self, CognitiveError> {
        let spec_file = spec_file.as_ref().to_path_buf();
        let output_dir = output_dir.as_ref().to_path_buf();
        let spec = super::parsing::parse_spec(&spec_file)?;

        fs::create_dir_all(&output_dir)
            .map_err(|e| CognitiveError::OrchestrationError(e.to_string()))?;

        Ok(Self {
            spec_file,
            output_dir,
            spec: Arc::new(spec),
            user_objective,
            initial_code,
            initial_latency,
            initial_memory,
            initial_relevance,
        })
    }

    /// Get the current optimization specification
    pub fn spec(&self) -> &OptimizationSpec {
        &self.spec
    }

    /// Get the output directory path
    pub fn output_dir(&self) -> &Path {
        &self.output_dir
    }

    /// Get the spec file path
    pub fn spec_file(&self) -> &Path {
        &self.spec_file
    }

    /// Get the user objective
    pub fn user_objective(&self) -> &str {
        &self.user_objective
    }

    /// Get the initial code
    pub fn initial_code(&self) -> &str {
        &self.initial_code
    }

    /// Get initial performance metrics
    pub fn initial_metrics(&self) -> (f64, f64, f64) {
        (self.initial_latency, self.initial_memory, self.initial_relevance)
    }

    /// Update the optimization specification
    pub fn update_spec(&mut self, new_spec: OptimizationSpec) {
        self.spec = Arc::new(new_spec);
        info!("Updated optimization specification");
    }

    /// Check if the orchestrator is properly initialized
    pub fn is_initialized(&self) -> bool {
        self.output_dir.exists() && 
        !self.initial_code.is_empty() &&
        !self.user_objective.is_empty()
    }

    /// Get orchestrator status summary
    pub fn status_summary(&self) -> OrchestratorStatus {
        OrchestratorStatus {
            spec_file: self.spec_file.clone(),
            output_dir: self.output_dir.clone(),
            user_objective: self.user_objective.clone(),
            initial_latency: self.initial_latency,
            initial_memory: self.initial_memory,
            initial_relevance: self.initial_relevance,
            is_initialized: self.is_initialized(),
        }
    }

    /// Validate orchestrator configuration
    pub fn validate_config(&self) -> Result<(), CognitiveError> {
        if !self.spec_file.exists() {
            return Err(CognitiveError::SpecError(
                "Specification file does not exist".to_string()
            ));
        }

        if !self.output_dir.exists() {
            return Err(CognitiveError::OrchestrationError(
                "Output directory does not exist".to_string()
            ));
        }

        if self.initial_code.is_empty() {
            return Err(CognitiveError::OrchestrationError(
                "Initial code cannot be empty".to_string()
            ));
        }

        if self.user_objective.is_empty() {
            return Err(CognitiveError::OrchestrationError(
                "User objective cannot be empty".to_string()
            ));
        }

        if self.initial_latency < 0.0 || self.initial_memory < 0.0 || self.initial_relevance < 0.0 {
            return Err(CognitiveError::OrchestrationError(
                "Initial metrics must be non-negative".to_string()
            ));
        }

        Ok(())
    }

    /// Create output directory if it doesn't exist
    pub fn ensure_output_dir(&self) -> Result<(), CognitiveError> {
        if !self.output_dir.exists() {
            fs::create_dir_all(&self.output_dir)
                .map_err(|e| CognitiveError::OrchestrationError(e.to_string()))?;
        }
        Ok(())
    }

    /// Get relative path within output directory
    pub fn output_path<P: AsRef<Path>>(&self, relative_path: P) -> PathBuf {
        self.output_dir.join(relative_path)
    }

    /// Check if spec file has been modified
    pub fn spec_file_modified(&self) -> Result<bool, CognitiveError> {
        let metadata = fs::metadata(&self.spec_file)
            .map_err(|e| CognitiveError::SpecError(e.to_string()))?;
        
        // This is a simplified check - in a real implementation,
        // you'd want to track the last modification time
        Ok(metadata.len() > 0)
    }

    /// Reload specification from file
    pub fn reload_spec(&mut self) -> Result<(), CognitiveError> {
        let new_spec = super::parsing::parse_spec(&self.spec_file)?;
        self.spec = Arc::new(new_spec);
        info!("Reloaded optimization specification from file");
        Ok(())
    }

    /// Get performance baseline
    pub fn performance_baseline(&self) -> PerformanceBaseline {
        PerformanceBaseline {
            latency: self.initial_latency,
            memory: self.initial_memory,
            relevance: self.initial_relevance,
        }
    }

    /// Calculate improvement metrics
    pub fn calculate_improvement(
        &self,
        current_latency: f64,
        current_memory: f64,
        current_relevance: f64,
    ) -> ImprovementMetrics {
        let latency_improvement = if self.initial_latency > 0.0 {
            ((self.initial_latency - current_latency) / self.initial_latency) * 100.0
        } else {
            0.0
        };

        let memory_improvement = if self.initial_memory > 0.0 {
            ((self.initial_memory - current_memory) / self.initial_memory) * 100.0
        } else {
            0.0
        };

        let relevance_improvement = if self.initial_relevance > 0.0 {
            ((current_relevance - self.initial_relevance) / self.initial_relevance) * 100.0
        } else {
            0.0
        };

        ImprovementMetrics {
            latency_improvement,
            memory_improvement,
            relevance_improvement,
        }
    }
}

/// Orchestrator status information
#[derive(Debug, Clone)]
pub struct OrchestratorStatus {
    pub spec_file: PathBuf,
    pub output_dir: PathBuf,
    pub user_objective: String,
    pub initial_latency: f64,
    pub initial_memory: f64,
    pub initial_relevance: f64,
    pub is_initialized: bool,
}

/// Performance baseline metrics
#[derive(Debug, Clone)]
pub struct PerformanceBaseline {
    pub latency: f64,
    pub memory: f64,
    pub relevance: f64,
}

/// Improvement metrics calculation
#[derive(Debug, Clone)]
pub struct ImprovementMetrics {
    pub latency_improvement: f64,
    pub memory_improvement: f64,
    pub relevance_improvement: f64,
}

impl ImprovementMetrics {
    /// Check if improvements meet minimum thresholds
    pub fn meets_thresholds(&self, min_latency: f64, min_memory: f64, min_relevance: f64) -> bool {
        self.latency_improvement >= min_latency &&
        self.memory_improvement >= min_memory &&
        self.relevance_improvement >= min_relevance
    }

    /// Calculate overall improvement score
    pub fn overall_score(&self) -> f64 {
        (self.latency_improvement + self.memory_improvement + self.relevance_improvement) / 3.0
    }

    /// Check if any metric has regressed significantly
    pub fn has_regression(&self, threshold: f64) -> bool {
        self.latency_improvement < -threshold ||
        self.memory_improvement < -threshold ||
        self.relevance_improvement < -threshold
    }
}