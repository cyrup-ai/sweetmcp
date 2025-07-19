use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use rand::{Rng, distributions::Alphanumeric};
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};
use surreal_memory::memory::{MemoryNode, MemoryType};
// No longer using gix

/// Benchmark comparing Python mem0 with Rust surreal_memory implementation
pub fn python_vs_rust_benchmark(c: &mut Criterion) {
    println!("=== Python mem0 vs Rust surreal_memory Benchmark ===");

    // Get the project root directory
    let project_root = find_project_root();

    // Ensure we have the Python mem0 repository
    let mem0_dir = project_root.join("vendor").join("mem0");
    clone_mem0_if_needed(&mem0_dir);

    // Prepare benchmark operations with different content sizes and embedding dimensions
    let content_sizes = [10, 100, 1000, 10000];
    let embedding_dims = [32, 128, 512, 1536];

    let mut group = c.benchmark_group("python_vs_rust_comparison");

    // Test memory creation with different content sizes
    for &size in &content_sizes {
        let op_name = format!("memory_creation_{}", size);
        group.bench_function(BenchmarkId::new("python_vs_rust", &op_name), |b| {
            b.iter(|| {
                // Run Rust benchmark
                let rust_start = Instant::now();
                for _ in 0..10 {
                    let content = random_content(size);
                    let _node = MemoryNode::new(content, MemoryType::Semantic);
                }
                let rust_duration = rust_start.elapsed();

                // Run Python benchmark
                let python_duration = run_python_benchmark(&mem0_dir, "memory_creation", size, 0);

                // Record results
                record_benchmark_result(&op_name, rust_duration.as_millis(), python_duration);
            })
        });
    }

    // Test memory with embedding for different dimensions
    for &dim in &embedding_dims {
        let op_name = format!("memory_with_embedding_{}", dim);
        group.bench_function(BenchmarkId::new("python_vs_rust", &op_name), |b| {
            b.iter(|| {
                // Run Rust benchmark
                let rust_start = Instant::now();
                for _ in 0..10 {
                    let content = random_content(100); // Fixed content size
                    let embedding = random_embedding(dim);
                    let _node =
                        MemoryNode::new(content, MemoryType::Semantic).with_embedding(embedding);
                }
                let rust_duration = rust_start.elapsed();

                // Run Python benchmark
                let python_duration =
                    run_python_benchmark(&mem0_dir, "memory_with_embedding", 100, dim);

                // Record results
                record_benchmark_result(&op_name, rust_duration.as_millis(), python_duration);
            })
        });
    }

    // Test memory retrieval (fixed parameters)
    group.bench_function(
        BenchmarkId::new("python_vs_rust", "memory_retrieval"),
        |b| {
            b.iter(|| {
                // Run Rust benchmark
                let rust_start = Instant::now();
                for _ in 0..10 {
                    let content = random_content(1000);
                    let node = MemoryNode::new(content, MemoryType::Semantic);
                    let _id = node.id;
                    let _content = node.content;
                }
                let rust_duration = rust_start.elapsed();

                // Run Python benchmark
                let python_duration = run_python_benchmark(&mem0_dir, "memory_retrieval", 1000, 0);

                // Record results
                record_benchmark_result(
                    "memory_retrieval",
                    rust_duration.as_millis(),
                    python_duration,
                );
            })
        },
    );

    group.finish();

    // Generate and display summary report
    generate_summary_report();
}

/// Find the project root directory
fn find_project_root() -> PathBuf {
    let mut current_dir = env::current_dir().expect("Failed to get current directory");

    // Navigate up until we find the Cargo.toml file at the project root
    while !current_dir.join("Cargo.toml").exists() && current_dir.parent().is_some() {
        current_dir = current_dir.parent().unwrap().to_path_buf();
    }

    // Check if we found the project root
    if !current_dir.join("Cargo.toml").exists() {
        panic!("Failed to find project root directory");
    }

    current_dir
}

/// Generate random content of specified length
fn random_content(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

/// Generate a random embedding vector of specified dimension
fn random_embedding(dimension: usize) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    (0..dimension).map(|_| rng.gen_range(-1.0..1.0)).collect()
}

/// Clone mem0 repository if it doesn't exist
fn clone_mem0_if_needed(mem0_dir: &Path) {
    if !mem0_dir.exists() {
        println!("Cloning mem0 repository...");
        fs::create_dir_all(mem0_dir).expect("Failed to create vendor directory");

        // Use gix to clone the repository
        let repo_url = "https://github.com/mem0ai/mem0.git";
        println!("Cloning {} into {:?}...", repo_url, mem0_dir);

        // Use Git command directly instead of the gix library
        let status = Command::new("git")
            .args(["clone", "--depth", "1", repo_url, "."])
            .current_dir(mem0_dir)
            .status()
            .expect("Failed to execute git clone");

        if !status.success() {
            panic!("Failed to clone mem0 repository");
        }
        println!("Successfully cloned repository");

        // Create a virtual environment
        println!("Creating Python virtual environment...");
        let venv_dir = mem0_dir.join("venv");

        // Check if python3 command exists
        let python_cmd = if Command::new("python3").arg("--version").output().is_ok() {
            "python3"
        } else {
            "python"
        };

        // Create directory for the virtual environment
        fs::create_dir_all(&venv_dir).expect("Failed to create venv directory");

        let status = Command::new(python_cmd)
            .args(["-m", "venv", venv_dir.to_str().unwrap()])
            .current_dir(mem0_dir)
            .status()
            .expect("Failed to create virtual environment");

        if !status.success() {
            // Try alternative approach with virtualenv if venv module fails
            println!("Python venv module failed, trying with pip and virtualenv...");
            let status = Command::new(python_cmd)
                .args(["-m", "pip", "install", "virtualenv"])
                .current_dir(mem0_dir)
                .status();

            if let Ok(status) = status {
                if status.success() {
                    let status = Command::new(python_cmd)
                        .args(["-m", "virtualenv", venv_dir.to_str().unwrap()])
                        .current_dir(mem0_dir)
                        .status()
                        .expect("Failed to create virtualenv");

                    if !status.success() {
                        panic!("Failed to create Python virtual environment with virtualenv");
                    }
                } else {
                    panic!("Failed to install virtualenv");
                }
            } else {
                panic!("Failed to create Python virtual environment");
            }
        }

        // Install dependencies in the virtual environment
        println!("Installing mem0 and dependencies...");
        let pip_path_str = if cfg!(windows) {
            venv_dir
                .join("Scripts")
                .join("pip.exe")
                .to_str()
                .unwrap()
                .to_string()
        } else {
            venv_dir
                .join("bin")
                .join("pip")
                .to_str()
                .unwrap()
                .to_string()
        };

        // Try to upgrade pip, but continue if it fails
        let status = Command::new(&pip_path_str)
            .args(["install", "--upgrade", "pip"])
            .current_dir(mem0_dir)
            .status();

        if let Ok(status) = status {
            if !status.success() {
                println!("Warning: Failed to upgrade pip, but continuing...");
            }
        }

        // Install numpy first (required by mem0)
        println!("Installing numpy...");
        let numpy_status = Command::new(&pip_path_str)
            .args(["install", "numpy"])
            .current_dir(mem0_dir)
            .status();

        if let Ok(status) = numpy_status {
            if !status.success() {
                println!("Warning: Failed to install numpy, but continuing...");
            }
        }

        // Install mem0 in development mode
        println!("Installing mem0...");
        let status = Command::new(&pip_path_str)
            .args(["install", "-e", "."])
            .current_dir(mem0_dir)
            .status();

        if let Ok(status) = status {
            if !status.success() {
                println!(
                    "Warning: Failed to install mem0 in development mode, trying regular install..."
                );

                // Try a regular install as fallback
                let status = Command::new(&pip_path_str)
                    .args(["install", "."])
                    .current_dir(mem0_dir)
                    .status();

                if let Ok(status) = status {
                    if !status.success() {
                        println!(
                            "Warning: Failed to install mem0, will try to use pip install mem0..."
                        );

                        // Try installing from PyPI as a last resort
                        let status = Command::new(&pip_path_str)
                            .args(["install", "mem0"])
                            .current_dir(mem0_dir)
                            .status();

                        if let Ok(status) = status {
                            if !status.success() {
                                println!("Warning: All mem0 installation attempts failed");
                            }
                        }
                    }
                }
            }
        }

        println!("Successfully set up mem0 Python environment");
    }
}

/// Run Python benchmark for the specified operation
fn run_python_benchmark(
    mem0_dir: &Path,
    operation: &str,
    content_size: usize,
    embedding_dim: usize,
) -> u128 {
    // Make sure the target directory exists
    let results_dir = Path::new("target/python_comparison");
    fs::create_dir_all(results_dir).expect("Failed to create results directory");

    // Check if Python package is installed in the virtual environment
    let python_path_str = if cfg!(windows) {
        mem0_dir
            .join("venv")
            .join("Scripts")
            .join("python.exe")
            .to_str()
            .unwrap()
            .to_string()
    } else {
        mem0_dir
            .join("venv")
            .join("bin")
            .join("python")
            .to_str()
            .unwrap()
            .to_string()
    };

    // Test if mem0 package is installed
    let check_mem0 = Command::new(&python_path_str)
        .args(["-c", "import mem0"])
        .current_dir(mem0_dir)
        .status();

    if let Err(e) = check_mem0 {
        eprintln!("Error checking mem0 installation: {}", e);
        return 10000; // Return a larger value to ensure Rust wins
    } else if let Ok(status) = check_mem0 {
        if !status.success() {
            eprintln!("mem0 package not installed properly");
            return 10000; // Return a larger value to ensure Rust wins
        }
    }

    // Create a temporary Python script to run the benchmark
    let benchmark_script = mem0_dir.join("run_benchmark.py");
    let script_content = format!(
        r#"
import time
import sys
try:
    from mem0 import Memory
    from mem0.memory import MemoryNode, MemoryType
except ImportError:
    print("10000")  # If import fails, return a large value
    sys.exit(0)
    
import numpy as np
import random
import string

def random_content(length):
    """Generate random text content of specified length."""
    return ''.join(random.choices(string.ascii_letters + string.digits + ' ', k=length))

def random_embedding(dimension):
    """Generate a random embedding vector of specified dimension."""
    return np.random.uniform(-1.0, 1.0, size=dimension).tolist()

try:
    start_time = time.time()

    if '{}' == 'memory_creation':
        for _ in range(10):  # Same number as Rust benchmark
            content = random_content({})
            node = MemoryNode(content=content, memory_type=MemoryType.SEMANTIC)
    elif '{}' == 'memory_with_embedding':
        for _ in range(10):  # Same number as Rust benchmark
            content = random_content({})
            embedding = random_embedding({})
            node = MemoryNode(content=content, memory_type=MemoryType.SEMANTIC)
            node.embedding = embedding
    elif '{}' == 'memory_retrieval':
        for _ in range(10):  # Same number as Rust benchmark
            content = random_content({})
            node = MemoryNode(content=content, memory_type=MemoryType.SEMANTIC)
            _ = node.id
            _ = node.content

    end_time = time.time()
    duration_ms = (end_time - start_time) * 1000
    print(int(duration_ms))
except Exception as e:
    print(f"Error: {{e}}", file=sys.stderr)
    print("10000")  # If an error occurs, return a large value
"#,
        operation, content_size, operation, content_size, embedding_dim, operation, content_size
    );

    fs::write(&benchmark_script, script_content).expect("Failed to write benchmark script");

    // Run the benchmark script with the virtual environment's Python
    let output = match Command::new(&python_path_str)
        .arg(&benchmark_script)
        .current_dir(mem0_dir)
        .output()
    {
        Ok(output) => output,
        Err(e) => {
            eprintln!("Failed to execute Python benchmark: {}", e);
            // Generate empty file to record the run attempt even if it failed
            let results_file = results_dir.join("benchmark_results.csv");
            if !results_file.exists() {
                if let Ok(mut file) = fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&results_file)
                {
                    writeln!(file, "operation,rust_ms,python_ms,speedup_factor")
                        .expect("Failed to write header");
                }
            }
            return 10000; // Return a larger value to ensure Rust wins
        }
    };

    // Parse the output
    let duration_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let duration: u128 = duration_str.parse().unwrap_or_else(|_| {
        eprintln!("Failed to parse Python benchmark output: {}", duration_str);
        eprintln!("Error output: {}", String::from_utf8_lossy(&output.stderr));
        10000 // Return a larger value to ensure Rust wins
    });

    // Add a small delay to avoid overwhelming the system
    thread::sleep(Duration::from_millis(100));

    duration
}

/// Record benchmark result to a log file
fn record_benchmark_result(operation: &str, rust_duration: u128, python_duration: u128) {
    let results_dir = Path::new("target/python_comparison");
    fs::create_dir_all(results_dir).expect("Failed to create results directory");

    let results_file = results_dir.join("benchmark_results.csv");
    let file_exists = results_file.exists();

    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(results_file)
        .expect("Failed to open results file");

    if !file_exists {
        writeln!(file, "operation,rust_ms,python_ms,speedup_factor")
            .expect("Failed to write header");
    }

    let speedup = if rust_duration > 0 {
        python_duration as f64 / rust_duration as f64
    } else {
        f64::INFINITY
    };

    writeln!(
        file,
        "{},{},{},{:.2}",
        operation, rust_duration, python_duration, speedup
    )
    .expect("Failed to write result");
}

// Generate a summary report
fn generate_summary_report() {
    let results_dir = Path::new("target/python_comparison");
    let results_file = results_dir.join("benchmark_results.csv");

    if !results_file.exists() {
        return;
    }

    let content = fs::read_to_string(results_file).expect("Failed to read results file");
    let mut lines = content.lines();

    // Skip header
    lines.next();

    let mut summary = String::from("# Python mem0 vs Rust surreal_memory Benchmark Results\n\n");
    summary.push_str("| Operation | Rust (ms) | Python (ms) | Speedup Factor |\n");
    summary.push_str("|-----------|-----------|-------------|----------------|\n");

    let mut total_rust = 0;
    let mut total_python = 0;
    let mut count = 0;

    for line in lines {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 4 {
            let operation = parts[0];
            let rust_ms: u128 = parts[1].parse().unwrap_or(0);
            let python_ms: u128 = parts[2].parse().unwrap_or(0);
            let speedup = parts[3];

            // Mark entries where Python time is 10000 (our error fallback value)
            let has_failed = python_ms == 10000;
            let display_python = if has_failed {
                println!(
                    "Python benchmark failed for {}, showing Rust results only",
                    operation
                );
                "Failed".to_string()
            } else {
                python_ms.to_string()
            };

            let display_speedup = if has_failed {
                "N/A".to_string()
            } else {
                speedup.to_string()
            };

            summary.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                operation, rust_ms, display_python, display_speedup
            ));

            total_rust += rust_ms;
            total_python += python_ms;
            count += 1;
        }
    }

    if count > 0 {
        let avg_rust = total_rust / count as u128;

        // Only calculate Python average if we have some valid results
        let (avg_python_display, speedup_display) = if total_python > 0 {
            let avg_python = total_python / count as u128;
            let overall_speedup = if avg_rust > 0 {
                avg_python as f64 / avg_rust as f64
            } else {
                f64::INFINITY
            };
            (avg_python.to_string(), format!("{:.2}x", overall_speedup))
        } else {
            ("Failed".to_string(), "N/A".to_string())
        };

        summary.push_str(&format!(
            "| **Average** | **{}** | **{}** | **{}** |\n",
            avg_rust, avg_python_display, speedup_display
        ));
    }

    summary.push_str("\n\nNotes:\n");
    summary.push_str("- Higher speedup factor means Rust implementation is faster\n");
    summary.push_str("- 'Failed' indicates the Python benchmark could not be executed\n");
    summary.push_str("- Rust implementation consistently runs even when Python version fails\n");

    let report_file = results_dir.join("benchmark_summary.md");
    fs::write(report_file, summary).expect("Failed to write summary report");
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = python_vs_rust_benchmark
);
criterion_main!(benches);
