use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use rand::{Rng, distributions::Alphanumeric};
use std::env;
use std::path::Path;
use std::process::Command;
use surreal_memory::memory::{MemoryNode, MemoryType};

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

/// Benchmark memory creation
fn bench_memory_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_creation");

    for size in [10, 100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let content = random_content(size);
                MemoryNode::new(content, MemoryType::Semantic)
            });
        });
    }

    group.finish();
}

/// Benchmark memory creation with embedding
fn bench_memory_with_embedding(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_with_embedding");

    for dim in [32, 128, 512, 1536].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(dim), dim, |b, &dim| {
            b.iter(|| {
                let content = random_content(100);
                let embedding = random_embedding(dim);
                MemoryNode::new(content, MemoryType::Semantic).with_embedding(embedding)
            });
        });
    }

    group.finish();
}

/// Run Python comparison benchmarks (if Python is available)
fn run_python_comparison() {
    // Check if we can run the Python comparison
    let has_python = Command::new("python")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

    if has_python {
        println!("Python detected! Running Python vs Rust comparison benchmarks...");

        let python_script = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("benches")
            .join("python_comparison")
            .join("download_and_run_python_mem0.py");

        if python_script.exists() {
            let status = Command::new("python").arg(&python_script).status();

            match status {
                Ok(exit_status) if exit_status.success() => {
                    println!("Python comparison benchmarks completed successfully!");
                }
                Ok(_) => {
                    eprintln!("Python comparison benchmarks failed to run.");
                }
                Err(e) => {
                    eprintln!("Failed to run Python comparison benchmarks: {}", e);
                }
            }
        } else {
            eprintln!("Python comparison script not found at: {:?}", python_script);
        }
    } else {
        println!("Python not found. Skipping Python vs Rust comparison benchmarks.");
    }
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = bench_memory_creation, bench_memory_with_embedding
);
criterion_main!(benches);
