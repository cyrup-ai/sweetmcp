//! Example demonstrating the async patterns in mem0-rs
//!
//! This shows how the sync trait methods return concrete Future/Stream types
//! that can be awaited, following the pattern of no async_trait or Box<dyn Future>

use futures::StreamExt;
use mem0_rs::memory::{
    memory_manager::{MemoryManager, SurrealDBMemoryManager},
    memory_node::{MemoryNode, MemoryType},
};
use surrealdb::{
    Surreal,
    engine::{
        any::Any,
        local::{Db, Mem},
    },
}; // For stream operations

async fn create_mem_db() -> Result<Surreal<Any>, Box<dyn std::error::Error>> {
    // Create in-memory database
    let db = Surreal::new::<Mem>(()).await?;
    db.use_ns("memory_ns").use_db("memory_db").await?;

    // SAFETY: We know this is an in-memory database, so we can unsafely transmute
    // This is a workaround for the lack of From<Surreal<Mem>> for Surreal<Any>
    Ok(unsafe { std::mem::transmute::<Surreal<Db>, Surreal<Any>>(db) })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 Mem0-rs Async Patterns Example\n");

    // Setup
    let db = create_mem_db().await?;
    let memory_manager = SurrealDBMemoryManager::new(db);
    memory_manager.initialize().await?;

    // 1. Demonstrating PendingMemory (Future type)
    println!("1. PendingMemory - Awaitable Future");
    println!("{}", "=".repeat(40));

    let memory = MemoryNode::with_id(
        "async_demo".to_string(),
        "This is an async demo".to_string(),
        MemoryType::Semantic,
    );

    // create_memory returns PendingMemory, which implements Future
    let pending_create = memory_manager.create_memory(memory.clone());

    // We can await it directly
    let created = pending_create.await?;
    println!("✅ Created memory: {}", created.id);

    // 2. Demonstrating MemoryQuery (Future type)
    println!("\n2. MemoryQuery - Awaitable Future");
    println!("{}", "=".repeat(40));

    // get_memory returns MemoryQuery, which implements Future
    let query = memory_manager.get_memory(&created.id);

    // We can await it
    match query.await? {
        Some(retrieved) => println!("✅ Retrieved: {}", retrieved.content),
        None => println!("❌ Memory not found"),
    }

    // 3. Demonstrating MemoryStream (Stream type)
    println!("\n3. MemoryStream - Async Stream");
    println!("{}", "=".repeat(40));

    // Create multiple memories
    for i in 1..=5 {
        let mem = MemoryNode::with_id(
            format!("stream_demo_{}", i),
            format!("Stream demo memory #{}", i),
            MemoryType::Semantic,
        );
        memory_manager.create_memory(mem).await?;
    }

    // query_by_type returns MemoryStream, which implements Stream
    let mut stream = memory_manager.query_by_type(MemoryType::Semantic);

    // We can iterate over the stream
    println!("Streaming memories:");
    let mut count = 0;
    while let Some(result) = stream.next().await {
        match result {
            Ok(memory) => {
                println!("  📄 {}: {}", memory.id, memory.content);
                count += 1;
            }
            Err(e) => println!("  ❌ Error: {}", e),
        }
    }
    println!("Total streamed: {} memories", count);

    // 4. Demonstrating concurrent operations
    println!("\n4. Concurrent Operations");
    println!("{}", "=".repeat(40));

    // We can run multiple operations concurrently
    let (mem1, mem2, mem3) = tokio::join!(
        memory_manager.create_memory(MemoryNode::with_id(
            "concurrent_1".to_string(),
            "First concurrent".to_string(),
            MemoryType::Semantic
        )),
        memory_manager.create_memory(MemoryNode::with_id(
            "concurrent_2".to_string(),
            "Second concurrent".to_string(),
            MemoryType::Episodic
        )),
        memory_manager.create_memory(MemoryNode::with_id(
            "concurrent_3".to_string(),
            "Third concurrent".to_string(),
            MemoryType::Procedural
        )),
    );

    let mem1 = mem1?;
    let mem2 = mem2?;
    let mem3 = mem3?;

    println!("✅ Created concurrently:");
    println!("   - {}: {:?}", mem1.id, mem1.memory_type);
    println!("   - {}: {:?}", mem2.id, mem2.memory_type);
    println!("   - {}: {:?}", mem3.id, mem3.memory_type);

    // 5. Demonstrating stream collection
    println!("\n5. Stream Collection Patterns");
    println!("{}", "=".repeat(40));

    // Collect first N items from a stream
    let search_results: Vec<_> = memory_manager
        .search_by_content("concurrent")
        .take(3) // Take only first 3
        .collect()
        .await;

    println!("First 3 search results:");
    for result in search_results {
        if let Ok(mem) = result {
            println!("   - {}: {}", mem.id, mem.content);
        }
    }

    // Filter stream results
    let filtered: Vec<_> = memory_manager
        .query_by_type(MemoryType::Semantic)
        .filter_map(|result| async move { result.ok().filter(|m| m.content.contains("demo")) })
        .collect()
        .await;

    println!(
        "\nFiltered semantic memories containing 'demo': {}",
        filtered.len()
    );

    // 6. Demonstrating error handling
    println!("\n6. Error Handling");
    println!("{}", "=".repeat(40));

    // Try to get a non-existent memory
    match memory_manager.get_memory("non_existent").await {
        Ok(Some(mem)) => println!("Found: {}", mem.id),
        Ok(None) => println!("✅ Correctly returned None for non-existent memory"),
        Err(e) => println!("Error: {}", e),
    }

    // 7. Demonstrating RelationshipStream
    println!("\n7. RelationshipStream");
    println!("{}", "=".repeat(40));

    // Create a relationship
    let rel = mem0_rs::memory::memory_relationship::MemoryRelationship::new(
        mem1.id.clone(),
        mem2.id.clone(),
        "relates_to".to_string(),
    );

    let created_rel = memory_manager.create_relationship(rel).await?;
    println!("✅ Created relationship: {}", created_rel.id);

    // Stream relationships
    let relationships: Vec<_> = memory_manager.get_relationships(&mem1.id).collect().await;

    println!("Found {} relationships", relationships.len());

    // 8. Cleanup demonstration
    println!("\n8. Cleanup with PendingDeletion");
    println!("{}", "=".repeat(40));

    // Delete operations return PendingDeletion futures
    let delete_futures = vec![
        memory_manager.delete_memory("concurrent_1"),
        memory_manager.delete_memory("concurrent_2"),
        memory_manager.delete_memory("concurrent_3"),
    ];

    // Await all deletions concurrently
    let results = futures::future::join_all(delete_futures).await;
    let deleted_count = results
        .iter()
        .filter(|r| r.as_ref().unwrap_or(&false) == &true)
        .count();

    println!("✅ Deleted {} memories", deleted_count);

    println!("\n✨ Async patterns example completed!");

    Ok(())
}
