// src/cognitive/orchestrator.rs
//! Infinite agentic orchestrator for committee-driven optimization

use crate::cognitive::evolution::{CodeEvolution, CognitiveCodeEvolution};
use crate::cognitive::types::{CognitiveError, OptimizationOutcome, OptimizationSpec};
use serde_json;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::task::JoinSet;
use tokio::time::{Duration, sleep};
use tracing::{error, info, warn};
use walkdir::WalkDir;

/// Orchestrator managing infinite optimization iterations
pub struct InfiniteOrchestrator {
    spec_file: PathBuf,
    output_dir: PathBuf,
    spec: Arc<OptimizationSpec>,
    user_objective: String,
    initial_code: String,
    initial_latency: f64,
    initial_memory: f64,
    initial_relevance: f64,
}

impl InfiniteOrchestrator {
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
        let spec = Self::parse_spec(&spec_file)?;

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

    fn parse_spec<P: AsRef<Path>>(spec_file: P) -> Result<OptimizationSpec, CognitiveError> {
        let mut file =
            File::open(spec_file).map_err(|e| CognitiveError::SpecError(e.to_string()))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| CognitiveError::SpecError(e.to_string()))?;

        // Try to parse as JSON first
        if let Ok(spec) = serde_json::from_str(&contents) {
            return Ok(spec);
        }

        // Otherwise convert Markdown to spec
        markdown_to_spec(&contents)
    }

    fn scan_output_dir(&self) -> Result<(u64, Vec<(PathBuf, u64)>, Vec<String>), CognitiveError> {
        let mut files = vec![];
        let mut max_iter = 0;
        let mut gaps = vec![];

        for entry in WalkDir::new(&self.output_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() && entry.file_name().to_string_lossy().ends_with(".json")
            {
                if let Some(file_name) = entry.file_name().to_str() {
                    if file_name.starts_with("iteration_") {
                        if let Some(iter_str) = file_name
                            .strip_prefix("iteration_")
                            .and_then(|s| s.strip_suffix(".json"))
                        {
                            if let Ok(iter) = iter_str.parse::<u64>() {
                                files.push((entry.path().to_path_buf(), iter));
                                max_iter = max_iter.max(iter);
                            }
                        }
                    }
                }
            }
        }

        // Check for missing iterations
        for i in 1..=max_iter {
            if !files.iter().any(|(_, iter)| *iter == i) {
                gaps.push(format!("Missing iteration {}", i));
            }
        }

        Ok((max_iter, files, gaps))
    }

    async fn create_evolution(
        &self,
        base_code: String,
        base_latency: f64,
        base_memory: f64,
        base_relevance: f64,
    ) -> Result<Arc<CognitiveCodeEvolution>, CognitiveError> {
        let evolution = CognitiveCodeEvolution::new(
            base_code,
            base_latency,
            base_memory,
            base_relevance,
            self.spec.clone(),
            self.user_objective.clone(),
        )?;

        Ok(Arc::new(evolution))
    }

    pub async fn run_infinite(&self) -> Result<(), CognitiveError> {
        let (max_iter, _, gaps) = self.scan_output_dir()?;
        let mut current_iter = max_iter + 1;
        let mut join_set = JoinSet::new();
        let mut outcomes: Vec<OptimizationOutcome> = vec![];

        if !gaps.is_empty() {
            warn!("Detected gaps in output: {:?}", gaps);
        }

        // Track current best state
        let mut best_code = self.initial_code.clone();
        let mut best_latency = self.initial_latency;
        let mut best_memory = self.initial_memory;
        let mut best_relevance = self.initial_relevance;

        loop {
            // Adaptive agent count based on recent success
            let agents_per_wave =
                if outcomes.len() > 10 && outcomes.iter().rev().take(5).all(|o| !o.applied) {
                    3 // Scale down if no recent progress
                } else {
                    5 // Default for infinite mode
                };

            // Wait if we have too many concurrent tasks
            while join_set.len() >= agents_per_wave {
                if let Some(res) = join_set.join_next().await {
                    match res {
                        Ok(Ok(mut outcome)) => {
                            outcome.iteration = current_iter;

                            // Update best state if improved
                            if outcome.applied {
                                let new_latency =
                                    best_latency * (1.0 - outcome.latency_improvement / 100.0);
                                let new_memory =
                                    best_memory * (1.0 - outcome.memory_improvement / 100.0);
                                let new_relevance =
                                    best_relevance * (1.0 + outcome.relevance_improvement / 100.0);

                                best_latency = new_latency;
                                best_memory = new_memory;
                                best_relevance = new_relevance;

                                // In a real system, we'd also update best_code here
                                info!(
                                    "New best state: latency={:.2}, memory={:.2}, relevance={:.2}",
                                    best_latency, best_memory, best_relevance
                                );
                            }

                            outcomes.push(outcome.clone());

                            // Save outcome
                            let output_path = self
                                .output_dir
                                .join(format!("iteration_{}.json", outcome.iteration));
                            fs::write(&output_path, serde_json::to_string_pretty(&outcome)?)
                                .map_err(|e| CognitiveError::OrchestrationError(e.to_string()))?;

                            info!("Saved outcome for iteration {}", outcome.iteration);
                        }
                        Ok(Err(e)) => error!("Evolution task failed: {}", e),
                        Err(e) => error!("Evolution task panicked: {}", e),
                    }
                    current_iter += 1;
                }
            }

            // Create evolution with current best state
            let evolution = self
                .create_evolution(best_code.clone(), best_latency, best_memory, best_relevance)
                .await?;

            // Spawn optimization task
            join_set.spawn(async move { evolution.evolve_routing_logic().await });

            // Brief pause to prevent CPU saturation
            sleep(Duration::from_millis(100)).await;

            // Log progress periodically
            if current_iter % 10 == 0 {
                let successful = outcomes.iter().filter(|o| o.applied).count();
                info!(
                    "Progress: {} iterations, {} successful optimizations",
                    current_iter, successful
                );
            }
        }
    }
}

/// Convert Markdown spec to OptimizationSpec
fn markdown_to_spec(md: &str) -> Result<OptimizationSpec, CognitiveError> {
    use crate::cognitive::types::*;

    // Default values
    let mut max_latency_increase = 20.0;
    let mut max_memory_increase = 30.0;
    let mut min_relevance_improvement = 40.0;
    let mut baseline_latency = 10.0;
    let mut baseline_memory = 100.0;
    let mut baseline_relevance = 50.0;

    // Parse markdown for key values
    for line in md.lines() {
        if line.contains("latency increase") {
            if let Some(num) = extract_percentage(line) {
                max_latency_increase = num;
            }
        } else if line.contains("memory increase") {
            if let Some(num) = extract_percentage(line) {
                max_memory_increase = num;
            }
        } else if line.contains("relevance improvement") {
            if let Some(num) = extract_percentage(line) {
                min_relevance_improvement = num;
            }
        } else if line.contains("Latency:") {
            if let Some(num) = extract_number(line) {
                baseline_latency = num;
            }
        } else if line.contains("Memory:") {
            if let Some(num) = extract_number(line) {
                baseline_memory = num;
            }
        } else if line.contains("Relevance:") {
            if let Some(num) = extract_percentage(line) {
                baseline_relevance = num;
            }
        }
    }

    Ok(OptimizationSpec {
        content_type: ContentType {
            format: "Rust source code".to_string(),
            restrictions: Restrictions {
                compiler: "rustc 1.82.0".to_string(),
                max_latency_increase,
                max_memory_increase,
                min_relevance_improvement,
            },
        },
        constraints: Constraints {
            size: "Single function or module".to_string(),
            style: "Idiomatic Rust".to_string(),
            schemas: vec!["syn::Item".to_string()],
        },
        evolution_rules: EvolutionRules {
            build_on_previous: true,
            new_axis_per_iteration: true,
            max_cumulative_latency_increase: max_latency_increase,
            min_action_diversity: 30.0,
            validation_required: true,
        },
        baseline_metrics: BaselineMetrics {
            latency: baseline_latency,
            memory: baseline_memory,
            relevance: baseline_relevance,
        },
    })
}

fn extract_percentage(line: &str) -> Option<f64> {
    line.split_whitespace()
        .find(|word| word.ends_with('%'))
        .and_then(|word| word.trim_end_matches('%').parse().ok())
}

fn extract_number(line: &str) -> Option<f64> {
    line.split_whitespace()
        .find_map(|word| word.parse::<f64>().ok())
}

#[derive(Debug)]
struct IterationPlan {
    iteration: u64,
    base_state: Option<OptimizationOutcome>,
}
