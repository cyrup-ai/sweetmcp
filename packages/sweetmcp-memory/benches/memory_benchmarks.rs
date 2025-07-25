//! Benchmark tests for memory operations

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, black_box};
use rand::{Rng, distributions::Alphanumeric};
use std::time::Duration;
use tokio::runtime::Runtime;
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

/// Benchmark memory serialization
fn bench_memory_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_serialization");
    
    let memory = MemoryNode::new("Test content".to_string(), MemoryType::Semantic);
    
    group.bench_function("serialize", |b| {
        b.iter(|| {
            let _ = serde_json::to_vec(&memory).unwrap();
        });
    });
    
    let serialized = serde_json::to_vec(&memory).unwrap();
    
    group.bench_function("deserialize", |b| {
        b.iter(|| {
            let _: MemoryNode = serde_json::from_slice(&serialized).unwrap();
        });
    });
    
    group.finish();
}

/// Benchmark memory storage
fn bench_memory_storage(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (graph_db, vector_store, _) = setup_benchmark_environment();
    
    let mut group = c.benchmark_group("memory_storage");
    
    group.bench_function("store", |b| {
        b.iter(|| {
            let memory = MemoryNode::new(random_content(100), MemoryType::Semantic);
            let entity = memory.to_entity().unwrap();
            let vector = random_embedding(1536);
            
            rt.block_on(async {
                graph_db.create_entity(entity).await.unwrap();
                vector_store.store_vector(&memory.id(), vector).await.unwrap();
            });
        });
    });
    
    group.finish();
}

/// Benchmark vector search
fn bench_vector_search(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (_, vector_store, _) = setup_benchmark_environment();
    
    // Pre-populate with some vectors
    rt.block_on(async {
        for i in 0..1000 {
            let vector = random_embedding(1536);
            vector_store.store_vector(&format!("test_{}", i), vector).await.unwrap();
        }
    });
    
    let mut group = c.benchmark_group("vector_search");
    
    group.bench_function("search_nearest_neighbors", |b| {
        b.iter(|| {
            let query = random_embedding(1536);
            rt.block_on(async {
                let _ = vector_store.search_vectors(query, 10).await.unwrap();
            });
        });
    });
    
    group.finish();
}

/// Benchmark memory cache operations
fn bench_memory_cache(c: &mut Criterion) {
    use lru::LruCache;
    use std::num::NonZeroUsize;
    
    let mut group = c.benchmark_group("memory_cache");
    
    for cache_size in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("cache_operations", cache_size), 
            cache_size, 
            |b, &size| {
                let mut cache = LruCache::new(NonZeroUsize::new(size).unwrap());
                let memories: Vec<_> = (0..size * 2)
                    .map(|i| MemoryNode::new(format!("content_{}", i), MemoryType::Semantic))
                    .collect();
                
                b.iter(|| {
                    // Mix of gets and puts
                    for i in 0..1000 {
                        let idx = black_box(rand::random::<usize>() % (size * 2));
                        if i % 4 == 0 {
                            // 25% of the time, update the cache
                            let memory = memories[idx].clone();
                            cache.put(memory.id().clone(), memory);
                        } else {
                            // 75% of the time, try to get from cache
                            let _ = cache.get(&memories[idx].id());
                        }
                    }
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark memory operations under concurrent load
fn bench_concurrent_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (graph_db, vector_store, _) = setup_benchmark_environment();
    
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
                    rt.block_on(async {
                        let mut handles = Vec::new();
                        
                        for i in 0..concurrency {
                            let graph_db = graph_db.clone();
                            let vector_store = vector_store.clone();
                            
                            let handle = tokio::spawn(async move {
                                let id = format!("concurrent_{}_{}", concurrency, i);
                                let memory = MemoryNode::new(random_content(100), MemoryType::Semantic);
                                let entity = memory.to_entity().unwrap();
                                let vector = random_embedding(1536);
                                
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
                    rt.block_on(async {
                        let mut handles = Vec::new();
                        
                        for i in 0..concurrency {
                            let graph_db = graph_db.clone();
                            let vector_store = vector_store.clone();
                            
                            let handle = tokio::spawn(async move {
                                let id = format!("concurrent_{}_{}", concurrency, i);
                                
                                let _ = graph_db.get_entity(&id).await.unwrap();
                                let _ = vector_store.get_vector(&id).await.unwrap();
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
                    rt.block_on(async {
                        let mut handles = Vec::new();
                        
                        for _ in 0..concurrency {
                            let vector_store = vector_store.clone();
                            let query_vector = random_embedding(1536);
                            
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

/// Setup benchmark environment
fn setup_benchmark_environment() -> (GraphDb, VectorStore, Runtime) {
    let rt = Runtime::new().unwrap();
    
    // Initialize in-memory graph database
    let graph_db = GraphDb::new_in_memory();
    
    // Initialize in-memory vector store
    let vector_store = VectorStore::new(VectorStoreConfig {
        dimension: 1536,
        distance: DistanceMetric::Cosine,
        ..Default::default()
    });
    
    (graph_db, vector_store, rt)
}

criterion_group!(
    benches,
    bench_memory_creation,
    bench_memory_with_embedding,
    bench_memory_serialization,
    bench_memory_storage,
    bench_vector_search,
    bench_memory_cache,
    bench_concurrent_operations,
);

criterion_main!(benches);
