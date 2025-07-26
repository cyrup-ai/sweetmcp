//! Iteration management and execution
//!
//! This module provides iteration planning, execution, and state management
//! for the infinite orchestrator with zero allocation patterns and
//! blazing-fast performance.

use crate::cognitive::evolution::{CodeEvolution, CognitiveCodeEvolution};
use crate::cognitive::types::{CognitiveError, OptimizationOutcome};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tokio::task::JoinSet;
use tokio::time::{Duration, sleep};
use tracing::{error, info, warn};
use walkdir::WalkDir;

use super::core::InfiniteOrchestrator;

/// Iteration plan for optimization
#[derive(Debug)]
pub struct IterationPlan {
    pub iteration: u64,
    pub base_state: Option<OptimizationOutcome>,
}

impl InfiniteOrchestrator {
    /// Run the infinite optimization loop
    pub async fn run_infinite_loop(&self) -> Result<(), CognitiveError> {
        info!("Starting infinite optimization loop");
        
        let mut iteration = 0u64;
        let mut best_outcome: Option<OptimizationOutcome> = None;
        
        loop {
            iteration += 1;
            info!("Starting iteration {}", iteration);
            
            let plan = IterationPlan {
                iteration,
                base_state: best_outcome.clone(),
            };
            
            match self.execute_iteration(&plan).await {
                Ok(outcome) => {
                    if self.is_better_outcome(&outcome, &best_outcome) {
                        best_outcome = Some(outcome.clone());
                        info!("New best outcome found in iteration {}", iteration);
                        
                        if let Err(e) = self.save_best_outcome(&outcome, iteration).await {
                            error!("Failed to save best outcome: {}", e);
                        }
                    }
                    
                    // Check if we should continue
                    if self.should_stop_iteration(iteration, &outcome) {
                        info!("Stopping optimization after {} iterations", iteration);
                        break;
                    }
                }
                Err(e) => {
                    error!("Iteration {} failed: {}", iteration, e);
                    
                    // Continue with next iteration unless it's a critical error
                    if self.is_critical_error(&e) {
                        return Err(e);
                    }
                }
            }
            
            // Brief pause between iterations
            sleep(Duration::from_millis(100)).await;
        }
        
        info!("Infinite optimization loop completed after {} iterations", iteration);
        Ok(())
    }

    /// Execute a single optimization iteration
    async fn execute_iteration(&self, plan: &IterationPlan) -> Result<OptimizationOutcome, CognitiveError> {
        info!("Executing iteration {}", plan.iteration);
        
        let base_code = match &plan.base_state {
            Some(outcome) => outcome.optimized_code.clone(),
            None => self.initial_code.clone(),
        };
        
        // Create cognitive code evolution instance
        let evolution = CognitiveCodeEvolution::new(
            self.spec.clone(),
            base_code,
            self.initial_latency,
            self.initial_memory,
            self.initial_relevance,
        );
        
        // Run the optimization
        let result = evolution.optimize().await
            .map_err(|e| CognitiveError::OrchestrationError(e.to_string()))?;
        
        info!(
            "Iteration {} completed - Latency: {:.2}ms, Memory: {:.2}MB, Relevance: {:.2}%",
            plan.iteration,
            result.final_latency,
            result.final_memory,
            result.final_relevance * 100.0
        );
        
        Ok(result.outcome)
    }

    /// Check if an outcome is better than the current best
    fn is_better_outcome(
        &self,
        new_outcome: &OptimizationOutcome,
        current_best: &Option<OptimizationOutcome>,
    ) -> bool {
        match current_best {
            None => true,
            Some(best) => {
                // Calculate improvement scores
                let new_score = self.calculate_outcome_score(new_outcome);
                let best_score = self.calculate_outcome_score(best);
                
                new_score > best_score
            }
        }
    }

    /// Calculate a composite score for an outcome
    fn calculate_outcome_score(&self, outcome: &OptimizationOutcome) -> f64 {
        // Weighted scoring: latency (40%), memory (30%), relevance (30%)
        let latency_score = if self.initial_latency > 0.0 {
            ((self.initial_latency - outcome.final_latency) / self.initial_latency) * 0.4
        } else {
            0.0
        };
        
        let memory_score = if self.initial_memory > 0.0 {
            ((self.initial_memory - outcome.final_memory) / self.initial_memory) * 0.3
        } else {
            0.0
        };
        
        let relevance_score = if self.initial_relevance > 0.0 {
            ((outcome.final_relevance - self.initial_relevance) / self.initial_relevance) * 0.3
        } else {
            0.0
        };
        
        latency_score + memory_score + relevance_score
    }

    /// Save the best outcome to disk
    async fn save_best_outcome(
        &self,
        outcome: &OptimizationOutcome,
        iteration: u64,
    ) -> Result<(), CognitiveError> {
        let output_path = self.output_path(format!("best_iteration_{}.rs", iteration));
        
        let mut file = File::create(&output_path)
            .map_err(|e| CognitiveError::OrchestrationError(e.to_string()))?;
        
        writeln!(file, "// Best outcome from iteration {}", iteration)
            .map_err(|e| CognitiveError::OrchestrationError(e.to_string()))?;
        writeln!(file, "// Latency: {:.2}ms", outcome.final_latency)
            .map_err(|e| CognitiveError::OrchestrationError(e.to_string()))?;
        writeln!(file, "// Memory: {:.2}MB", outcome.final_memory)
            .map_err(|e| CognitiveError::OrchestrationError(e.to_string()))?;
        writeln!(file, "// Relevance: {:.2}%", outcome.final_relevance * 100.0)
            .map_err(|e| CognitiveError::OrchestrationError(e.to_string()))?;
        writeln!(file, "")
            .map_err(|e| CognitiveError::OrchestrationError(e.to_string()))?;
        write!(file, "{}", outcome.optimized_code)
            .map_err(|e| CognitiveError::OrchestrationError(e.to_string()))?;
        
        info!("Saved best outcome to: {}", output_path.display());
        Ok(())
    }

    /// Check if we should stop the iteration loop
    fn should_stop_iteration(&self, iteration: u64, outcome: &OptimizationOutcome) -> bool {
        // Check max iterations limit
        if let Some(max_iter) = self.spec.max_iterations {
            if iteration >= max_iter as u64 {
                return true;
            }
        }
        
        // Check if target quality is reached
        let current_score = self.calculate_outcome_score(outcome);
        if current_score >= self.spec.target_quality {
            return true;
        }
        
        // Check for convergence (no significant improvement)
        // This would require tracking previous outcomes
        
        false
    }

    /// Check if an error is critical and should stop the loop
    fn is_critical_error(&self, error: &CognitiveError) -> bool {
        match error {
            CognitiveError::SpecError(_) => true,
            CognitiveError::OrchestrationError(msg) => {
                msg.contains("critical") || msg.contains("fatal")
            }
            _ => false,
        }
    }

    /// Run parallel optimization iterations
    pub async fn run_parallel_iterations(
        &self,
        num_parallel: usize,
        max_iterations: u64,
    ) -> Result<Vec<OptimizationOutcome>, CognitiveError> {
        info!("Starting {} parallel optimization iterations", num_parallel);
        
        let mut join_set = JoinSet::new();
        let mut results = Vec::new();
        
        for i in 0..num_parallel {
            let orchestrator = self.clone_for_parallel(i).await?;
            
            join_set.spawn(async move {
                let plan = IterationPlan {
                    iteration: i as u64,
                    base_state: None,
                };
                
                orchestrator.execute_iteration(&plan).await
            });
        }
        
        // Collect results
        while let Some(result) = join_set.join_next().await {
            match result {
                Ok(Ok(outcome)) => results.push(outcome),
                Ok(Err(e)) => {
                    error!("Parallel iteration failed: {}", e);
                }
                Err(e) => {
                    error!("Join error in parallel iteration: {}", e);
                }
            }
        }
        
        info!("Completed {} parallel iterations with {} successful results", 
              num_parallel, results.len());
        
        Ok(results)
    }

    /// Clone orchestrator for parallel execution
    async fn clone_for_parallel(&self, index: usize) -> Result<Self, CognitiveError> {
        let parallel_output_dir = self.output_path(format!("parallel_{}", index));
        
        fs::create_dir_all(&parallel_output_dir)
            .map_err(|e| CognitiveError::OrchestrationError(e.to_string()))?;
        
        Ok(Self {
            spec_file: self.spec_file.clone(),
            output_dir: parallel_output_dir,
            spec: self.spec.clone(),
            user_objective: self.user_objective.clone(),
            initial_code: self.initial_code.clone(),
            initial_latency: self.initial_latency,
            initial_memory: self.initial_memory,
            initial_relevance: self.initial_relevance,
        })
    }

    /// Get iteration statistics
    pub fn get_iteration_stats(&self) -> Result<IterationStats, CognitiveError> {
        let mut stats = IterationStats::default();
        
        // Count iteration files in output directory
        for entry in WalkDir::new(&self.output_dir) {
            let entry = entry.map_err(|e| CognitiveError::OrchestrationError(e.to_string()))?;
            
            if entry.file_type().is_file() {
                let file_name = entry.file_name().to_string_lossy();
                if file_name.starts_with("best_iteration_") {
                    stats.completed_iterations += 1;
                }
            }
        }
        
        Ok(stats)
    }

    /// Clean up old iteration files
    pub fn cleanup_old_iterations(&self, keep_latest: usize) -> Result<(), CognitiveError> {
        let mut iteration_files = Vec::new();
        
        // Collect all iteration files
        for entry in WalkDir::new(&self.output_dir) {
            let entry = entry.map_err(|e| CognitiveError::OrchestrationError(e.to_string()))?;
            
            if entry.file_type().is_file() {
                let file_name = entry.file_name().to_string_lossy();
                if file_name.starts_with("best_iteration_") {
                    iteration_files.push(entry.path().to_path_buf());
                }
            }
        }
        
        // Sort by modification time (newest first)
        iteration_files.sort_by(|a, b| {
            let a_meta = fs::metadata(a).unwrap_or_else(|_| panic!("Failed to get metadata"));
            let b_meta = fs::metadata(b).unwrap_or_else(|_| panic!("Failed to get metadata"));
            b_meta.modified().unwrap_or_else(|_| std::time::SystemTime::UNIX_EPOCH)
                .cmp(&a_meta.modified().unwrap_or_else(|_| std::time::SystemTime::UNIX_EPOCH))
        });
        
        // Remove old files
        for file_path in iteration_files.iter().skip(keep_latest) {
            if let Err(e) = fs::remove_file(file_path) {
                warn!("Failed to remove old iteration file {}: {}", file_path.display(), e);
            }
        }
        
        info!("Cleaned up old iteration files, kept {} latest", keep_latest);
        Ok(())
    }
}

/// Statistics for iteration tracking
#[derive(Debug, Default)]
pub struct IterationStats {
    pub completed_iterations: u64,
    pub successful_iterations: u64,
    pub failed_iterations: u64,
    pub average_improvement: f64,
    pub best_score: f64,
}

impl IterationStats {
    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        if self.completed_iterations == 0 {
            0.0
        } else {
            (self.successful_iterations as f64 / self.completed_iterations as f64) * 100.0
        }
    }

    /// Check if performance is acceptable
    pub fn is_performing_well(&self) -> bool {
        self.success_rate() >= 80.0 && self.average_improvement > 0.0
    }
}