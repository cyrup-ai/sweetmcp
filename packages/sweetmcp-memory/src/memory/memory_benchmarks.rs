#![cfg(feature = "bench")]
//! Benchmark suite for memory operations and cache performance
use criterion::{
    black_box, criterion_group, criterion_main, measurement::WallTime, BenchmarkId, Criterion,
};
use rand::Rng;
use std::time::Duration;
use sweetmcp_memory::{
    cache::{CachePolicy, MemoryCache},
    memory::{create_procedural_memory, Memory},
};
use tokio::runtime::Runtime;
// Helper function to convert Memory to a simple string for benchmarking
fn memory_to_entity(_memory: &Memory) -> String {
    "test_entity".to_string()
}

// Placeholder implementations for benchmarking
#[derive(Clone)]
struct GraphDb;

impl GraphDb {
    fn new() -> Self {
        Self {}
    }
    
    async fn create_entity(&self, _entity: ()) -> Result<(), ()> {
        Ok(())
    }
    
    async fn get_entity(&self, _id: &str) -> Result<(), ()> {
        Ok(())
    }
}

#[derive(Clone)]
struct VectorStore;

impl VectorStore {
    fn new() -> Self {
        Self {}
    }
    
    async fn store_vector(&self, _id: &str, _vector: Vec<f32>) -> Result<(), ()> {
        Ok(())
    }
    
    async fn get_vector(&self, _id: &str) -> Result<Vec<f32>, ()> {
        Ok(vec![])
    }
    
    async fn search_vectors(&self, _query: Vec<f32>, _k: usize) -> Result<Vec<f32>, ()> {
        Ok(vec![])
    }
}

fn create_memory_vector(dim: usize) -> Vec<f32> {
    use rand::Rng;
    let mut rng = rand::rng();
    (0..dim).map(|_| rng.random_range(0.0..1.0)).collect()
}

/// Helper function to create a test runtime for async benchmarks
fn setup_benchmark_environment() -> (GraphDb, VectorStore, Runtime) {
    let runtime = Runtime::new().unwrap();
    let graph_db = GraphDb::new();
    let vector_store = VectorStore::new();
    (graph_db, vector_store, runtime)
}

// Benchmark memory cache operations
fn bench_memory_cache(c: &mut Criterion) {
    // Create a test cache with different eviction policies
    let policies = [
        CachePolicy::Lru(1000),
        CachePolicy::Lfu(1000),
        CachePolicy::Fifo(1000),
    ];
    
    // Create test memories
    let mut memories = Vec::with_capacity(1000);
    for i in 0..1000 {
        let memory = create_procedural_memory(&format!("test_{}", i), 5);
        memories.push(memory);
    }
    
    let mut group = c.benchmark_group("memory_cache");
    
    for policy in &policies {
        // Initialize cache with policy
        let mut cache = MemoryCache::new(policy.clone());
        
        // Pre-populate cache with some memories
        for memory in &memories[0..500] {
            cache.put(memory.clone()).unwrap();
        }
        
        // Benchmark cache hits
        
        // Benchmark cache misses
        group.bench_with_input(
            BenchmarkId::new("cache_miss", format!("{:?}", policy)),
            &policy,
            |b, _| {
                b.iter(|| {
                    let idx = black_box(500 + rand::random::<usize>() % 500);
                    let id = black_box(&memories[idx].id());
                    cache.get(id)
                });
            },
        );
        
        // Benchmark cache updates
        group.bench_with_input(
            BenchmarkId::new("cache_update", format!("{:?}", policy)),
            &policy,
            |b, _| {
                b.iter(|| {
                    let idx = black_box(rand::random::<usize>() % 1000);
                    let memory = black_box(memories[idx].clone());
                    cache.put(memory)
                });
            },
        );
    }
    
    group.finish();
}

// Benchmark memory operations under concurrent load
fn bench_concurrent_operations(c: &mut Criterion) {
    let (graph_db, vector_store, runtime) = setup_benchmark_environment();
    
    let mut group = c.benchmark_group("concurrent_operations");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(20);
    
    // Concurrent memory creation and storage
    let concurrency_levels = [1, 4, 8, 16, 32];
    
    for &concurrency in &concurrency_levels {
        group.bench_with_input(
            BenchmarkId::new("concurrent_store", concurrency),
            &concurrency,
            |b, &concurrency| {
                b.iter(|| {
                    runtime.block_on(async {
                        let mut handles = Vec::new();
                        
                        for i in 0..concurrency {
                            let graph_db = graph_db.clone();
                            let vector_store = vector_store.clone();
                            
                            let handle = tokio::spawn(async move {
                                let id = format!("concurrent_{}_{}", concurrency, i);
                                let memory = create_procedural_memory(&id, 10);
                                let entity = memory_to_entity(&memory);
                                let vector = create_memory_vector(128);
                                
                                graph_db.create_entity(entity).await.unwrap();
                                vector_store.store_vector(&id, vector).await.unwrap();
                            });
                            
                            handles.push(handle);
                        }
                        
                        for handle in handles {
                            handle.await.unwrap();
                        }
                    });
                });
            },
        );
        
        // Concurrent memory retrieval
        group.bench_with_input(
            BenchmarkId::new("concurrent_retrieve", concurrency),
            &concurrency,
            |b, &concurrency| {
                b.iter(|| {
                    runtime.block_on(async {
                        let mut handles = Vec::new();
                        
                        for i in 0..concurrency {
                            let graph_db = graph_db.clone();
                            let vector_store = vector_store.clone();
                            
                            let handle = tokio::spawn(async move {
                                let id = format!("concurrent_{}_{}", concurrency, i);
                                
                                let entity = graph_db.get_entity(&id).await.unwrap();
                                let vector = vector_store.get_vector(&id).await.unwrap();
                                
                                (entity, vector)
                            });
                            
                            handles.push(handle);
                        }
                        
                        for handle in handles {
                            handle.await.unwrap();
                        }
                    });
                });
            },
        );
        
        // Concurrent vector search
        group.bench_with_input(
            BenchmarkId::new("concurrent_search", concurrency),
            &concurrency,
            |b, &concurrency| {
                b.iter(|| {
                    runtime.block_on(async {
                        let mut handles = Vec::new();
                        
                        for _ in 0..concurrency {
                            let vector_store = vector_store.clone();
                            let query_vector = create_memory_vector(128);
                            
                            let handle = tokio::spawn(async move {
                                vector_store.search_vectors(query_vector, 10).await.unwrap()
                            });
                            
                            handles.push(handle);
                        }
                        
                        for handle in handles {
                            handle.await.unwrap();
                        }
                    });
                });
            },
        );
    }
    
    group.finish();
}

// Placeholder benchmark functions
fn bench_memory_creation(c: &mut Criterion) {
    c.bench_function("memory_creation", |b| {
        b.iter(|| {
            black_box(create_procedural_memory("test", 10));
        });
    });
}

fn bench_memory_serialization(c: &mut Criterion) {
    let memory = create_procedural_memory("test", 10);
    c.bench_function("memory_serialization", |b| {
        b.iter(|| {
            black_box(serde_json::to_string(&memory).unwrap());
        });
    });
}

fn bench_memory_storage(c: &mut Criterion) {
    let (graph_db, vector_store, runtime) = setup_benchmark_environment();
    let memory = create_procedural_memory("test", 10);
    
    c.bench_function("memory_storage", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let entity = memory_to_entity(&memory);
                let vector = create_memory_vector(128);
                graph_db.create_entity(entity).await.unwrap();
                vector_store.store_vector("test", vector).await.unwrap();
            });
        });
    });
}

fn bench_vector_search(c: &mut Criterion) {
    let (_, vector_store, runtime) = setup_benchmark_environment();
    let query = create_memory_vector(128);
    
    c.bench_function("vector_search", |b| {
        b.iter(|| {
            runtime.block_on(async {
                black_box(vector_store.search_vectors(query.clone(), 10).await.unwrap());
            });
        });
    });
}

criterion_group!(
    benches,
    bench_memory_creation,
    bench_memory_serialization,
    bench_memory_storage,
    bench_vector_search,
    bench_memory_cache,
    bench_concurrent_operations,
);
criterion_main!(benches);
