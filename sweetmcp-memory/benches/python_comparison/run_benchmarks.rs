use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};
use std::env;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use chrono::Local;

#[derive(Debug, Serialize, Deserialize)]
struct BenchmarkResults {
    memory_creation: std::collections::HashMap<String, BenchmarkStats>,
    memory_with_embedding: std::collections::HashMap<String, BenchmarkStats>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BenchmarkStats {
    mean_ms: f64,
    median_ms: f64,
    min_ms: f64,
    max_ms: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct ComparisonResult {
    python_results: BenchmarkResults,
    rust_results: BenchmarkResults,
    comparison: ComparisonSummary,
}

#[derive(Debug, Serialize, Deserialize)]
struct ComparisonSummary {
    memory_creation: std::collections::HashMap<String, ComparisonStats>,
    memory_with_embedding: std::collections::HashMap<String, ComparisonStats>,
    summary: SummaryStats,
}

#[derive(Debug, Serialize, Deserialize)]
struct ComparisonStats {
    python_mean_ms: f64,
    rust_mean_ms: f64,
    speedup_factor: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct SummaryStats {
    memory_creation: OperationSummary,
    memory_with_embedding: OperationSummary,
    overall: OperationSummary,
}

#[derive(Debug, Serialize, Deserialize)]
struct OperationSummary {
    python_avg_ms: f64,
    rust_avg_ms: f64,
    speedup_factor: f64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Python mem0 vs Rust benchmark comparison...");

    // Create results directory
    let results_dir = get_results_dir()?;
    fs::create_dir_all(&results_dir)?;

    // Run Python benchmarks
    println!("Running Python mem0 benchmarks...");
    let python_script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("benches")
        .join("python_comparison")
        .join("download_and_run_python_mem0.py");

    let output = Command::new("python")
        .arg(&python_script)
        .output()?;

    if !output.status.success() {
        eprintln!("Error running Python benchmarks:");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        return Err("Python benchmark failed".into());
    }

    // Get the results file path from the output
    let output_str = String::from_utf8_lossy(&output.stdout);
    let results_file_line = output_str
        .lines()
        .find(|line| line.contains("JSON results saved to:"))
        .ok_or("Could not find results file path in output")?;

    let results_file_path = results_file_line
        .split(":")
        .nth(1)
        .ok_or("Could not parse results file path")?
        .trim();

    // Read and display the results
    println!("Reading benchmark results from: {}", results_file_path);
    let results_json = fs::read_to_string(results_file_path)?;
    let results: ComparisonResult = serde_json::from_str(&results_json)?;

    // Generate and print a simple summary
    print_summary(&results);

    // Open the markdown report
    let report_path = results_dir.join(format!(
        "python_vs_rust_report_{}.md",
        Local::now().format("%Y%m%d_%H%M%S")
    ));
    
    if let Ok(report_content) = fs::read_to_string(&report_path) {
        println!("\nDetailed report saved to: {}", report_path.display());
        println!("\nSummary of results:");
        println!("{}", report_content.lines().take(15).collect::<Vec<_>>().join("\n"));
    }

    Ok(())
}

fn get_results_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let bench_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("benches")
        .join("benchmark_results");
    
    fs::create_dir_all(&bench_dir)?;
    Ok(bench_dir)
}

fn print_summary(results: &ComparisonResult) {
    println!("\n=== BENCHMARK COMPARISON SUMMARY ===\n");
    
    println!("MEMORY CREATION:");
    println!("Python avg: {:.4} ms", results.comparison.summary.memory_creation.python_avg_ms);
    println!("Rust avg:   {:.4} ms", results.comparison.summary.memory_creation.rust_avg_ms);
    println!("Speedup:    {:.2}x\n", results.comparison.summary.memory_creation.speedup_factor);
    
    println!("MEMORY WITH EMBEDDING:");
    println!("Python avg: {:.4} ms", results.comparison.summary.memory_with_embedding.python_avg_ms);
    println!("Rust avg:   {:.4} ms", results.comparison.summary.memory_with_embedding.rust_avg_ms);
    println!("Speedup:    {:.2}x\n", results.comparison.summary.memory_with_embedding.speedup_factor);
    
    println!("OVERALL PERFORMANCE:");
    println!("Python avg: {:.4} ms", results.comparison.summary.overall.python_avg_ms);
    println!("Rust avg:   {:.4} ms", results.comparison.summary.overall.rust_avg_ms);
    println!("Speedup:    {:.2}x\n", results.comparison.summary.overall.speedup_factor);
}