//! Unit tests for memory schema

use sweetmcp_memory::MemoryType;
use sweetmcp_memory::memory::memory_schema::*;

#[test]
fn test_new_memory() {
    let content = "Test memory content".to_string();
    let memory_type = MemoryType::Semantic;
    let memory = Memory::new(content.clone(), memory_type);

    assert!(!memory.id.is_empty());
    assert_eq!(memory.content, content);
    assert_eq!(memory.r#type, memory_type);
    assert!(memory.embedding.is_none());
    assert!(memory.metadata.is_empty());
    assert!(memory.created_at > 0);
    assert_eq!(memory.created_at, memory.updated_at);
    assert_eq!(memory.created_at, memory.last_accessed_at);
}

#[test]
fn test_touch_memory() {
    let mut memory = Memory::new("Test".to_string(), MemoryType::Semantic);
    let initial_accessed_at = memory.last_accessed_at;
    std::thread::sleep(std::time::Duration::from_millis(10)); // Ensure time changes
    memory.touch();
    assert!(memory.last_accessed_at > initial_accessed_at);
}

#[test]
fn test_set_embedding() {
    let mut memory = Memory::new("Test".to_string(), MemoryType::Semantic);
    let embedding = vec![0.1, 0.2, 0.3];
    memory.set_embedding(embedding.clone());
    assert_eq!(memory.embedding, Some(embedding));
    assert!(memory.updated_at >= memory.created_at);
}

#[test]
fn test_metadata_operations() {
    let mut memory = Memory::new("Test".to_string(), MemoryType::Semantic);
    let key = "source".to_string();
    let value = serde_json::json!("web");

    memory.add_metadata(key.clone(), value.clone());

    if let serde_json::Value::Object(map) = &memory.metadata {
        assert_eq!(map.get(&key), Some(&value));
    } else {
        panic!("Expected metadata to be an object");
    }

    memory.remove_metadata(&key);

    if let serde_json::Value::Object(map) = &memory.metadata {
        assert!(map.get(&key).is_none());
    } else {
        panic!("Expected metadata to be an object");
    }
}
