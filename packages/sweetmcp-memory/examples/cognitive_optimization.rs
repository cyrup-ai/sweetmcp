// examples/cognitive_optimization.rs
//! Example of using the cognitive optimization system

use cyrup_memory::cognitive::{orchestrator::InfiniteOrchestrator, types::OptimizationSpec};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Example spec file content
    let spec_content = r#"
# Optimization Specification for Quantum Memory System

## Content Type
- **Format**: Rust source code modifications
- **Restrictions**: Must compile with `rustc 1.82.0`, target `<20%` latency increase, `<30%` memory increase, `>40%` relevance improvement

## Generation Constraints
- **Size**: Single function or module per modification
- **Style**: Idiomatic Rust, adhering to `clippy` lints
- **Schemas**: Modifications must be valid `syn::Item` (e.g., functions, structs)

## Evolution Rules
- **Iteration N+1**:
  - Build on N's best modification
  - Introduce one new optimization axis (e.g., caching, parallelization, memory reduction)
  - Ensure cumulative latency `<20%` increase from baseline
- **Diversity**: At least 30% of actions must differ from prior iterations
- **Validation**: All modifications must pass compilation and benchmarks

## Baseline Metrics
- **Latency**: 10.0 ms
- **Memory**: 100.0 MB
- **Relevance**: 50.0%
"#;

    // Create spec file
    std::fs::create_dir_all("output")?;
    std::fs::write("output/spec.md", spec_content)?;

    // Initial code to optimize
    let initial_code = r#"
use std::collections::HashMap;

pub struct QuantumMemory {
    cache: HashMap<String, Vec<f64>>,
}

impl QuantumMemory {
    pub fn search(&self, query: &str) -> Vec<f64> {
        // Unoptimized linear search
        for (key, value) in &self.cache {
            if key.contains(query) {
                return value.clone();
            }
        }
        vec![]
    }
}
"#;

    // User objective - this drives the committee evaluation
    let user_objective = "Optimize the quantum memory search function for better relevance while maintaining performance constraints. Focus on improving search accuracy and reducing false positives.";

    // Create orchestrator
    let orchestrator = InfiniteOrchestrator::new(
        "output/spec.md",
        "output/iterations",
        initial_code.to_string(),
        10.0,  // initial latency
        100.0, // initial memory
        50.0,  // initial relevance
        user_objective.to_string(),
    )?;

    println!("Starting infinite optimization with objective:");
    println!("{}", user_objective);
    println!("\nThe committee will evaluate each modification based on how well it:");
    println!("1. Achieves the user objective");
    println!("2. Stays within performance constraints");
    println!("3. Improves search relevance");
    println!("\nPress Ctrl+C to stop...\n");

    // Run infinite optimization
    orchestrator.run_infinite().await?;

    Ok(())
}
