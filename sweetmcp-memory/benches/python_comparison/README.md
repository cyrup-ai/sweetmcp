# Python vs Rust Memory Benchmark

This benchmark compares the performance of the Python `mem0` implementation with our Rust `surreal_memory` implementation.

## Overview

The benchmark tests two key operations:
1. **Memory Creation**: Creating memory nodes with varying content sizes
2. **Memory with Embedding**: Creating memory nodes with embeddings of varying dimensions

## Running the Benchmark

### Prerequisites

- Python 3.8+ with `venv` module
- Rust toolchain with Cargo

### Option 1: Run the Python script directly

```bash
cd cyrun/crates/memory
python benches/python_comparison/download_and_run_python_mem0.py
```

This will:
1. Create a temporary Python virtual environment
2. Install the Python `mem0` package and dependencies
3. Run benchmarks for both Python and Rust implementations
4. Generate comparison results and reports

### Option 2: Use the Rust benchmark harness

```bash
cd cyrun/crates/memory
cargo bench --bench python_vs_rust
```

## Output

The benchmark will produce:
1. JSON file with detailed results in `benches/benchmark_results/`
2. Markdown report with comparison summary in `benches/benchmark_results/`

## Benchmark Parameters

- Memory creation is tested with content sizes: 10, 100, 1000, 10000 characters
- Memory with embedding is tested with dimensions: 32, 128, 512, 1536
- Each benchmark is run multiple times and averaged for statistical significance

## Methodology

The benchmark ensures a fair comparison by:
1. Using equivalent operations in both implementations
2. Running benchmarks in isolated environments
3. Using the same random data generation logic
4. Performing multiple runs to account for variance

## Interpreting Results

The speedup factor in the results represents how many times faster the Rust implementation is compared to the Python implementation. For example, a speedup factor of 10x means the Rust implementation is 10 times faster.