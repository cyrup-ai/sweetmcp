        }
        
        // Benchmark cache hits
        group.bench_with_input(
            BenchmarkId::new("cache_hit", format!("{:?}", policy)),
            &policy,
            |b, _| {
                b.iter(|| {
                    let idx = black_box(rand::random::<usize>() % 500);
                    let id = black_box(&memories[idx].id());
                    cache.get(id)
                });
            },
        );
        
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
                                let entity = memory.to_entity().unwrap();
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
