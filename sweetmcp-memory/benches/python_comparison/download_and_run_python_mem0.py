#!/usr/bin/env python3
"""
Download and run mem0 benchmarks for comparison with Rust implementation.
This script clones the latest mem0, runs benchmarks, and outputs results.
"""

import os
import sys
import json
import time
import shutil
import subprocess
import statistics
from datetime import datetime
from pathlib import Path

def setup_mem0():
    """Clone mem0 and set up the environment."""
    vendor_dir = Path(__file__).parent.parent.parent / "vendor"
    vendor_dir.mkdir(exist_ok=True)
    
    mem0_dir = vendor_dir / "mem0"
    
    # Remove existing mem0 directory
    if mem0_dir.exists():
        print(f"Removing existing mem0 directory at {mem0_dir}")
        shutil.rmtree(mem0_dir)
    
    # Clone latest mem0
    print("Cloning latest mem0 repository...")
    subprocess.run([
        "git", "clone", "https://github.com/mem0ai/mem0.git", str(mem0_dir)
    ], check=True)
    
    # Create virtual environment
    venv_dir = mem0_dir / "venv"
    print("Creating virtual environment...")
    subprocess.run([sys.executable, "-m", "venv", str(venv_dir)], check=True)
    
    # Install mem0
    pip_path = venv_dir / "bin" / "pip"
    print("Installing mem0...")
    subprocess.run([str(pip_path), "install", "-e", str(mem0_dir)], check=True)
    subprocess.run([str(pip_path), "install", "numpy"], check=True)
    
    return mem0_dir, venv_dir

def run_benchmarks(mem0_dir, venv_dir):
    """Run mem0 benchmarks matching Rust benchmark operations."""
    python_path = venv_dir / "bin" / "python"
    
    # Create benchmark script
    benchmark_script = mem0_dir / "run_benchmarks.py"
    with open(benchmark_script, 'w') as f:
        f.write('''
import time
import json
import random
import string
from mem0 import Memory

# Initialize mem0
m = Memory()

def random_content(size):
    """Generate random content of specified size."""
    return ''.join(random.choices(string.ascii_letters + string.digits + ' ', k=size))

def random_embedding(dim):
    """Generate random embedding of specified dimension."""
    return [random.random() for _ in range(dim)]

def benchmark_memory_creation(sizes, iterations=10):
    """Benchmark memory creation with different content sizes."""
    results = {}
    
    for size in sizes:
        times = []
        for _ in range(iterations):
            content = random_content(size)
            start = time.time()
            result = m.add(content, user_id="benchmark_user")
            end = time.time()
            times.append((end - start) * 1000)  # Convert to ms
        
        results[f"memory_creation_{size}"] = {
            "mean_ms": sum(times) / len(times),
            "median_ms": sorted(times)[len(times) // 2],
            "min_ms": min(times),
            "max_ms": max(times)
        }
    
    return results

def benchmark_memory_with_embedding(dims, iterations=10):
    """Benchmark memory with embeddings of different dimensions."""
    results = {}
    
    for dim in dims:
        times = []
        for _ in range(iterations):
            content = random_content(100)  # Fixed content size
            embedding = random_embedding(dim)
            start = time.time()
            # mem0 doesn't directly support custom embeddings in add()
            # Just benchmark the memory creation
            result = m.add(content, user_id="benchmark_user", metadata={"embedding_dim": dim})
            end = time.time()
            times.append((end - start) * 1000)  # Convert to ms
        
        results[f"memory_with_embedding_{dim}"] = {
            "mean_ms": sum(times) / len(times),
            "median_ms": sorted(times)[len(times) // 2],
            "min_ms": min(times),
            "max_ms": max(times)
        }
    
    return results

def benchmark_memory_retrieval(iterations=10):
    """Benchmark memory retrieval."""
    times = []
    
    # First add some memories
    memory_ids = []
    for i in range(10):
        content = random_content(1000)
        result = m.add(content, user_id="benchmark_user")
        if result and 'id' in result:
            memory_ids.append(result['id'])
    
    # Benchmark retrieval
    for _ in range(iterations):
        start = time.time()
        memories = m.get_all(user_id="benchmark_user")
        end = time.time()
        times.append((end - start) * 1000)  # Convert to ms
    
    return {
        "memory_retrieval": {
            "mean_ms": sum(times) / len(times),
            "median_ms": sorted(times)[len(times) // 2],
            "min_ms": min(times),
            "max_ms": max(times)
        }
    }

# Run benchmarks
print("Running mem0 benchmarks...")

content_sizes = [10, 100, 1000, 10000]
embedding_dims = [32, 128, 512, 1536]

creation_results = benchmark_memory_creation(content_sizes)
embedding_results = benchmark_memory_with_embedding(embedding_dims)
retrieval_results = benchmark_memory_retrieval()

# Combine results
all_results = {
    "memory_creation": creation_results,
    "memory_with_embedding": embedding_results,
    "memory_retrieval": retrieval_results["memory_retrieval"]
}

# Save results
output_file = "mem0_benchmark_results.json"
with open(output_file, 'w') as f:
    json.dump(all_results, f, indent=2)

print(f"Results saved to: {output_file}")
print(json.dumps(all_results, indent=2))
''')
    
    # Run the benchmark script
    print("Running mem0 benchmarks...")
    result = subprocess.run([
        str(python_path), str(benchmark_script)
    ], capture_output=True, text=True, cwd=str(mem0_dir))
    
    if result.returncode != 0:
        print(f"Error running benchmarks: {result.stderr}")
        raise Exception("Benchmark failed")
    
    # Read the results
    results_file = mem0_dir / "mem0_benchmark_results.json"
    with open(results_file, 'r') as f:
        results = json.load(f)
    
    return results

def format_comparison_results(mem0_results):
    """Format results for comparison with Rust benchmarks."""
    # This matches what the Rust benchmark expects
    benchmark_results = {
        "python_results": {
            "memory_creation": mem0_results["memory_creation"],
            "memory_with_embedding": mem0_results["memory_with_embedding"],
        },
        "rust_results": {
            # Placeholder - will be filled by Rust benchmark
            "memory_creation": {},
            "memory_with_embedding": {},
        },
        "comparison": {
            "memory_creation": {},
            "memory_with_embedding": {},
            "summary": {
                "memory_creation": {"python_avg_ms": 0, "rust_avg_ms": 0, "speedup_factor": 0},
                "memory_with_embedding": {"python_avg_ms": 0, "rust_avg_ms": 0, "speedup_factor": 0},
                "overall": {"python_avg_ms": 0, "rust_avg_ms": 0, "speedup_factor": 0}
            }
        }
    }
    
    # Save results
    output_dir = Path(__file__).parent.parent / "benchmark_results"
    output_dir.mkdir(exist_ok=True)
    
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    output_file = output_dir / f"python_mem0_results_{timestamp}.json"
    
    with open(output_file, 'w') as f:
        json.dump(benchmark_results, f, indent=2)
    
    print(f"JSON results saved to: {output_file}")
    return output_file

def main():
    try:
        # Check if mem0 is already installed to speed up benchmarks
        mem0_dir = Path(__file__).parent.parent.parent / "vendor" / "mem0"
        venv_dir = mem0_dir / "venv"
        
        if not (mem0_dir.exists() and venv_dir.exists()):
            mem0_dir, venv_dir = setup_mem0()
        else:
            print(f"Using existing mem0 installation at {mem0_dir}")
        
        # Run benchmarks
        results = run_benchmarks(mem0_dir, venv_dir)
        
        # Format and save results
        output_file = format_comparison_results(results)
        
        print("\nBenchmark completed successfully!")
        
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()