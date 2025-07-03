# Memory Benchmarks

This directory contains benchmarks for the `surreal_memory` crate, comparing performance between Rust and Python implementations.

## Available Benchmarks

### Memory Operations Benchmark

Standard benchmarks for memory operations using Criterion:

```bash
cargo bench --bench memory_benchmarks
```

### Python Comparison Benchmark

Compares our Rust implementation with the original Python `mem0` implementation:

```bash
cargo bench --bench python_comparison_bench
```

This benchmark:

1. Clones the Python `mem0` repository into `../../vendor/mem0` (if it doesn't exist)
2. Installs the Python package and its dependencies
3. Runs equivalent benchmarks on both implementations
4. Generates comparison reports

## Requirements for Python Comparison

- Python 3.8+ installed and available in PATH
- `pip` and `venv` modules available
- Git installed and available in PATH

## Benchmark Results

Results are stored in:

- `target/criterion` - Criterion benchmark results
- `target/python_comparison` - Python vs Rust comparison results

A summary report is generated at `target/python_comparison/benchmark_summary.md`.

## Measured Operations

- **Memory Creation**: Creating memory nodes with content
- **Memory with Embedding**: Creating memory nodes with embedding vectors
- **Memory Retrieval**: Retrieving information from memory nodes
