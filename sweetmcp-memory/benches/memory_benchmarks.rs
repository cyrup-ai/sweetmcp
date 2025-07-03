//! Benchmark tests for memory operations

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use rand::{Rng, distributions::Alphanumeric};
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

criterion_group!(benches, bench_memory_creation, bench_memory_with_embedding,);
criterion_main!(benches);
