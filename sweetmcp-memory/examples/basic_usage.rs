//! Basic usage example for mem0-rs
//!
//! This example demonstrates:
//! - Creating and connecting to SurrealDB
//! - Initializing the memory manager
//! - Creating, retrieving, updating, and deleting memories
//! - Creating relationships between memories
//! - Searching memories by content and vector similarity

use futures::StreamExt;
use mem0_rs::memory::{
    memory_manager::{MemoryManager, SurrealDBMemoryManager},
    memory_metadata::MemoryMetadata,
    memory_node::{MemoryNode, MemoryType},
    memory_relationship::MemoryRelationship,
};
use surrealdb::{
    Surreal,
    engine::{
        any::Any,
        local::{Db, Mem},
    },
};

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
    println!("🧠 Mem0-rs Basic Usage Example\n");

    // 1. Create a SurrealDB instance (in-memory for this example)
    println!("1. Setting up SurrealDB...");
    let db = create_mem_db().await?;
    println!("   ✓ Database created successfully");

    // 2. Create and initialize the memory manager
    println!("2. Initializing memory manager...");
    let memory_manager = SurrealDBMemoryManager::new(db);
    match memory_manager.initialize().await {
        Ok(_) => println!("   ✓ Memory manager initialized"),
        Err(e) => {
            println!("   ✗ Failed to initialize: {:?}", e);
            return Err(e.into());
        }
    }

    // 3. Create a memory node
    println!("\n3. Creating memories...");

    // Create a semantic memory about Rust
    let mut rust_memory = MemoryNode::with_id(
        "rust_intro".to_string(),
        "Rust is a systems programming language focused on safety, speed, and concurrency."
            .to_string(),
        MemoryType::Semantic,
    );

    // Add metadata
    rust_memory.metadata.importance = 0.9;
    rust_memory.metadata.tags = vec!["programming".to_string(), "rust".to_string()];
    rust_memory.metadata.category = "technical_knowledge".to_string();

    // Add a dummy embedding (in real usage, this would come from an embedding model)
    rust_memory.metadata.embedding = Some(vec![0.1, 0.2, 0.3, 0.4, 0.5]);

    // Create the memory
    println!("   Creating rust_intro memory...");
    let created_rust = match memory_manager.create_memory(rust_memory).await {
        Ok(created) => {
            println!("✅ Created memory: {}", created.id);
            created
        }
        Err(e) => {
            println!("❌ Failed to create memory: {:?}", e);
            return Err(e.into());
        }
    };

    // Create an episodic memory
    let mut meeting_memory = MemoryNode::with_id(
        "meeting_2024".to_string(),
        "Had a productive meeting about the new Rust project architecture".to_string(),
        MemoryType::Episodic,
    );
    meeting_memory.metadata.importance = 0.7;
    meeting_memory.metadata.tags = vec!["meeting".to_string(), "project".to_string()];
    meeting_memory.metadata.user_id = Some("user123".to_string());
    meeting_memory.metadata.embedding = Some(vec![0.2, 0.3, 0.4, 0.5, 0.6]);

    let created_meeting = memory_manager.create_memory(meeting_memory).await?;
    println!("✅ Created memory: {}", created_meeting.id);

    // 5. Retrieve a memory
    println!("\n5. Retrieving memory...");
    if let Some(retrieved) = memory_manager.get_memory(&created_rust.id).await? {
        println!(
            "✅ Retrieved memory: {} - {}",
            retrieved.id, retrieved.content
        );
        println!(
            "   Type: {:?}, Importance: {}",
            retrieved.memory_type, retrieved.metadata.importance
        );
    }

    // 6. Update a memory
    println!("\n6. Updating memory...");
    let mut updated_rust = created_rust.clone();
    updated_rust.content =
        "Rust is a modern systems programming language that guarantees memory safety.".to_string();
    updated_rust.metadata.importance = 0.95;

    let updated = memory_manager.update_memory(updated_rust).await?;
    println!("✅ Updated memory: {}", updated.content);

    // 7. Create a relationship between memories
    println!("\n7. Creating relationship...");
    let relationship = MemoryRelationship::new(
        created_rust.id.clone(),
        created_meeting.id.clone(),
        "related_to".to_string(),
    );

    let created_rel = memory_manager.create_relationship(relationship).await?;
    println!(
        "✅ Created relationship: {} -> {} ({})",
        created_rel.source_id, created_rel.target_id, created_rel.relationship_type
    );

    // 8. Get relationships for a memory
    println!("\n8. Getting relationships...");
    let relationships = memory_manager
        .get_relationships(&created_rust.id)
        .collect::<Vec<_>>()
        .await;

    for rel_result in relationships {
        if let Ok(rel) = rel_result {
            println!(
                "   Found relationship: {} -> {} ({})",
                rel.source_id, rel.target_id, rel.relationship_type
            );
        }
    }

    // 9. Search memories by type
    println!("\n9. Searching by type...");
    let semantic_memories = memory_manager
        .query_by_type(MemoryType::Semantic)
        .collect::<Vec<_>>()
        .await;

    println!("   Found {} semantic memories", semantic_memories.len());
    for mem_result in semantic_memories {
        if let Ok(mem) = mem_result {
            println!(
                "   - {}: {}",
                mem.id,
                &mem.content[..50.min(mem.content.len())]
            );
        }
    }

    // 10. Search memories by content
    println!("\n10. Searching by content...");
    let search_results = memory_manager
        .search_by_content("Rust")
        .collect::<Vec<_>>()
        .await;

    println!(
        "   Found {} memories containing 'Rust'",
        search_results.len()
    );
    for mem_result in search_results {
        if let Ok(mem) = mem_result {
            println!(
                "   - {}: {}",
                mem.id,
                &mem.content[..50.min(mem.content.len())]
            );
        }
    }

    // 11. Search by vector similarity (if embeddings are available)
    println!("\n11. Searching by vector similarity...");
    let query_vector = vec![0.15, 0.25, 0.35, 0.45, 0.55]; // Similar to rust_memory embedding
    let similar_memories = memory_manager
        .search_by_vector(query_vector, 5)
        .collect::<Vec<_>>()
        .await;

    println!("   Found {} similar memories", similar_memories.len());
    for mem_result in similar_memories {
        if let Ok(mem) = mem_result {
            println!(
                "   - {}: {}",
                mem.id,
                &mem.content[..50.min(mem.content.len())]
            );
        }
    }

    // 12. Delete a relationship
    println!("\n12. Deleting relationship...");
    let deleted_rel = memory_manager.delete_relationship(&created_rel.id).await?;
    println!("✅ Deleted relationship: {}", deleted_rel);

    // 13. Delete memories
    println!("\n13. Cleaning up memories...");
    let deleted1 = memory_manager.delete_memory(&created_rust.id).await?;
    let deleted2 = memory_manager.delete_memory(&created_meeting.id).await?;
    println!("✅ Deleted memories: {} and {}", deleted1, deleted2);

    println!("\n✨ Example completed successfully!");

    Ok(())
}
