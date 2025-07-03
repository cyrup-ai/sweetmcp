//! Example demonstrating different memory types in mem0-rs
//!
//! This example shows how to work with:
//! - Semantic memories (facts and concepts)
//! - Episodic memories (personal experiences)
//! - Procedural memories (how-to knowledge)

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
    println!("🧠 Mem0-rs Memory Types Example\n");

    // Setup
    let db = create_mem_db().await?;
    let memory_manager = SurrealDBMemoryManager::new(db);
    memory_manager.initialize().await?;

    // 1. Semantic Memory Example
    println!("1. Semantic Memory (Facts & Concepts)");
    println!("{}", "=".repeat(40));

    // Create semantic memories about programming languages
    let python_fact = create_semantic_memory(
        "python_overview",
        "Python Programming Language",
        "Python is a high-level, interpreted programming language known for its simplicity and readability.",
        vec!["programming", "python", "interpreted"],
        vec!["Java", "JavaScript", "Ruby"], // related concepts
    );

    let rust_fact = create_semantic_memory(
        "rust_overview",
        "Rust Programming Language",
        "Rust is a systems programming language that guarantees memory safety without garbage collection.",
        vec!["programming", "rust", "systems", "memory-safety"],
        vec!["C++", "Go", "Python"],
    );

    // Store semantic memories
    let python_mem = memory_manager.create_memory(python_fact).await?;
    let rust_mem = memory_manager.create_memory(rust_fact).await?;

    println!("✅ Created semantic memories:");
    println!("   - Python: {}", python_mem.id);
    println!("   - Rust: {}", rust_mem.id);

    // 2. Episodic Memory Example
    println!("\n2. Episodic Memory (Personal Experiences)");
    println!("{}", "=".repeat(40));

    // Create episodic memories about learning experiences
    let learning_rust = create_episodic_memory(
        "learning_rust_2024",
        "Started Learning Rust",
        "Today I started learning Rust. The ownership concept was challenging but fascinating.",
        "online_course",
        Some("user123".to_string()),
        vec!["learning", "rust", "programming"],
    );

    let first_rust_project = create_episodic_memory(
        "first_rust_project",
        "Built First Rust Project",
        "Successfully built my first Rust project - a CLI todo application. Felt accomplished!",
        "personal_project",
        Some("user123".to_string()),
        vec!["project", "rust", "achievement"],
    );

    // Store episodic memories
    let episode1 = memory_manager.create_memory(learning_rust).await?;
    let episode2 = memory_manager.create_memory(first_rust_project).await?;

    println!("✅ Created episodic memories:");
    println!("   - Learning: {}", episode1.id);
    println!("   - Project: {}", episode2.id);

    // 3. Procedural Memory Example
    println!("\n3. Procedural Memory (How-To Knowledge)");
    println!("{}", "=".repeat(40));

    // Create a procedural memory for "How to create a Rust project"
    let mut procedure_node = MemoryNode::with_id(
        "create_rust_project".to_string(),
        "How to Create a Rust Project: Step-by-step guide to create a new Rust project\n\
         Prerequisites: Rust must be installed (check with: which rustc)\n\
         1. Install Rust (if needed): curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh\n\
         2. Create new project: cargo new my_project\n\
         3. Navigate to project: cd my_project\n\
         4. Build the project: cargo build\n\
         5. Run the project: cargo run".to_string(),
        MemoryType::Procedural,
    );

    // Add metadata for each step
    procedure_node
        .metadata
        .set_custom("step_1", "Install Rust")
        .unwrap();
    procedure_node
        .metadata
        .set_custom("step_2", "Create new project")
        .unwrap();
    procedure_node
        .metadata
        .set_custom("step_3", "Navigate to project")
        .unwrap();
    procedure_node
        .metadata
        .set_custom("step_4", "Build the project")
        .unwrap();
    procedure_node
        .metadata
        .set_custom("step_5", "Run the project")
        .unwrap();

    let stored_procedure = memory_manager.create_memory(procedure_node).await?;

    println!("✅ Created procedural memory: {}", stored_procedure.id);
    println!("   Steps: Install → Create → Navigate → Build → Run");

    // 4. Create relationships between memories
    println!("\n4. Creating Memory Relationships");
    println!("{}", "=".repeat(40));

    // Link semantic memory to episodic memory
    let rel1 = memory_manager
        .create_relationship(
            mem0_rs::memory::memory_relationship::MemoryRelationship::new(
                rust_mem.id.clone(),
                episode1.id.clone(),
                "triggered".to_string(),
            ),
        )
        .await?;

    // Link episodic memories in sequence
    let rel2 = memory_manager
        .create_relationship(
            mem0_rs::memory::memory_relationship::MemoryRelationship::new(
                episode1.id.clone(),
                episode2.id.clone(),
                "led_to".to_string(),
            ),
        )
        .await?;

    // Link procedural memory to semantic memory
    let rel3 = memory_manager
        .create_relationship(
            mem0_rs::memory::memory_relationship::MemoryRelationship::new(
                stored_procedure.id.clone(),
                rust_mem.id.clone(),
                "implements".to_string(),
            ),
        )
        .await?;

    println!("✅ Created relationships:");
    println!("   - Rust concept → triggered → Learning experience");
    println!("   - Learning → led_to → First project");
    println!("   - How-to guide → implements → Rust concept");

    // 5. Query memories by type
    println!("\n5. Querying Memories by Type");
    println!("{}", "=".repeat(40));

    // Get all episodic memories
    let episodes = memory_manager
        .query_by_type(MemoryType::Episodic)
        .collect::<Vec<_>>()
        .await;

    println!("📚 Episodic memories ({}):", episodes.len());
    for mem_result in episodes {
        if let Ok(mem) = mem_result {
            println!(
                "   - {}: {}",
                mem.id,
                &mem.content[..60.min(mem.content.len())]
            );
        }
    }

    // Get all semantic memories
    let semantics = memory_manager
        .query_by_type(MemoryType::Semantic)
        .collect::<Vec<_>>()
        .await;

    println!("\n📖 Semantic memories ({}):", semantics.len());
    for mem_result in semantics {
        if let Ok(mem) = mem_result {
            println!(
                "   - {}: {}",
                mem.id,
                &mem.content[..60.min(mem.content.len())]
            );
        }
    }

    println!("\n✨ Memory types example completed!");

    Ok(())
}

// Helper functions

fn create_semantic_memory(
    id: &str,
    name: &str,
    content: &str,
    tags: Vec<&str>,
    related: Vec<&str>,
) -> MemoryNode {
    let mut memory = MemoryNode::with_id(id.to_string(), content.to_string(), MemoryType::Semantic);
    memory.metadata.tags = tags.into_iter().map(String::from).collect();
    memory.metadata.category = "knowledge".to_string();
    memory.metadata.importance = 0.8;

    // Add related concepts as custom metadata
    let mut custom = serde_json::Map::new();
    custom.insert(
        "name".to_string(),
        serde_json::Value::String(name.to_string()),
    );
    custom.insert(
        "related_concepts".to_string(),
        serde_json::Value::Array(
            related
                .into_iter()
                .map(|s| serde_json::Value::String(s.to_string()))
                .collect(),
        ),
    );
    memory.metadata.custom = serde_json::Value::Object(custom);

    memory
}

fn create_episodic_memory(
    id: &str,
    title: &str,
    content: &str,
    context: &str,
    user_id: Option<String>,
    tags: Vec<&str>,
) -> MemoryNode {
    let mut memory = MemoryNode::with_id(id.to_string(), content.to_string(), MemoryType::Episodic);
    memory.metadata.tags = tags.into_iter().map(String::from).collect();
    memory.metadata.category = "experience".to_string();
    memory.metadata.importance = 0.7;
    memory.metadata.user_id = user_id;
    memory.metadata.context = context.to_string();

    // Add episodic-specific metadata
    let mut custom = serde_json::Map::new();
    custom.insert(
        "title".to_string(),
        serde_json::Value::String(title.to_string()),
    );
    custom.insert(
        "emotion".to_string(),
        serde_json::Value::String("positive".to_string()),
    );
    memory.metadata.custom = serde_json::Value::Object(custom);

    memory
}
