// src/memory/episodic.rs
//! Episodic memory implementation for the memory system.
//!
//! Episodic memory stores sequences of events with temporal information,
//! allowing for time-based queries and context-aware retrieval.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::memory::memory_metadata::MemoryMetadata;
use crate::memory::memory_node::MemoryNode;
use crate::memory::memory_type::{BaseMemory, Memory, MemoryContent, MemoryTypeEnum};
use crate::memory::repository::MemoryRepository;
use crate::utils::Result;
use crate::utils::error::Error;

/// Context for an episodic memory event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodicContext {
    /// Unique identifier for the context
    pub id: String,

    /// Type of context (e.g., "location", "person", "object")
    pub context_type: String,

    /// Value of the context
    pub value: String,

    /// Additional metadata for the context
    pub metadata: HashMap<String, Value>,
}

impl EpisodicContext {
    /// Create a new episodic context
    pub fn new(id: &str, context_type: &str, value: &str) -> Self {
        Self {
            id: id.to_string(),
            context_type: context_type.to_string(),
            value: value.to_string(),
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the context
    pub fn with_metadata(mut self, key: &str, value: Value) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }

    /// Convert the context to a SurrealDB value
    pub fn to_value(&self) -> Result<Value> {
        let mut obj = serde_json::Map::new();
        obj.insert("id".to_string(), Value::String(self.id.clone()));
        obj.insert("type".to_string(), Value::String(self.context_type.clone()));
        obj.insert("value".to_string(), Value::String(self.value.clone()));

        if !self.metadata.is_empty() {
            obj.insert(
                "metadata".to_string(),
                serde_json::to_value(&self.metadata)?,
            );
        }

        Ok(Value::Object(obj))
    }

    /// Convert from a SurrealDB value to a context
    pub fn from_value(value: &Value) -> Result<Self> {
        if let Value::Object(obj) = value {
            let id = if let Some(Value::String(s)) = obj.get("id") {
                s.clone()
            } else {
                return Err(Error::ConversionError("Missing id in context".to_string()));
            };

            let context_type = if let Some(Value::String(s)) = obj.get("type") {
                s.clone()
            } else {
                return Err(Error::ConversionError(
                    "Missing type in context".to_string(),
                ));
            };

            let value = if let Some(Value::String(s)) = obj.get("value") {
                s.clone()
            } else {
                return Err(Error::ConversionError(
                    "Missing value in context".to_string(),
                ));
            };

            let mut metadata = HashMap::new();
            if let Some(Value::Object(meta_obj)) = obj.get("metadata") {
                for (key, val) in meta_obj.iter() {
                    metadata.insert(key.to_string(), val.clone());
                }
            }

            Ok(Self {
                id,
                context_type,
                value,
                metadata,
            })
        } else {
            Err(Error::ConversionError("Invalid context value".to_string()))
        }
    }
}

/// Event in an episodic memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodicEvent {
    /// Unique identifier for the event
    pub id: String,

    /// Description of the event
    pub description: String,

    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,

    /// Importance of the event (0-100)
    pub importance: u8,

    /// Contexts associated with the event
    pub contexts: Vec<EpisodicContext>,

    /// Additional metadata for the event
    pub metadata: HashMap<String, Value>,
}

impl EpisodicEvent {
    /// Create a new episodic event
    pub fn new(id: &str, description: &str, timestamp: DateTime<Utc>, importance: u8) -> Self {
        Self {
            id: id.to_string(),
            description: description.to_string(),
            timestamp,
            importance,
            contexts: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a context to the event
    pub fn with_context(mut self, context: EpisodicContext) -> Self {
        self.contexts.push(context);
        self
    }

    /// Add metadata to the event
    pub fn with_metadata(mut self, key: &str, value: Value) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }

    /// Convert the event to a SurrealDB value
    pub fn to_value(&self) -> Result<Value> {
        let mut obj = serde_json::Map::new();
        obj.insert("id".to_string(), Value::String(self.id.clone()));
        obj.insert(
            "description".to_string(),
            Value::String(self.description.clone()),
        );
        obj.insert(
            "timestamp".to_string(),
            Value::String(self.timestamp.to_rfc3339()),
        );
        obj.insert(
            "importance".to_string(),
            Value::Number(self.importance.into()),
        );

        let mut contexts = Vec::new();
        for context in &self.contexts {
            contexts.push(context.to_value()?);
        }
        obj.insert("contexts".to_string(), Value::Array(contexts));

        if !self.metadata.is_empty() {
            obj.insert(
                "metadata".to_string(),
                serde_json::to_value(&self.metadata)?,
            );
        }

        Ok(Value::Object(obj))
    }

    /// Convert from a SurrealDB value to an event
    pub fn from_value(value: &Value) -> Result<Self> {
        if let Value::Object(obj) = value {
            let id = if let Some(Value::String(s)) = obj.get("id") {
                s.clone()
            } else {
                return Err(Error::ConversionError("Missing id in event".to_string()));
            };

            let description = if let Some(Value::String(s)) = obj.get("description") {
                s.clone()
            } else {
                return Err(Error::ConversionError(
                    "Missing description in event".to_string(),
                ));
            };

            let timestamp = if let Some(Value::String(s)) = obj.get("timestamp") {
                DateTime::parse_from_rfc3339(s)
                    .map_err(|_| Error::ConversionError("Invalid timestamp format".to_string()))?
                    .with_timezone(&Utc)
            } else {
                return Err(Error::ConversionError(
                    "Missing timestamp in event".to_string(),
                ));
            };

            let importance = if let Some(Value::Number(n)) = obj.get("importance") {
                n.as_u64()
                    .ok_or_else(|| Error::ConversionError("Invalid importance value".to_string()))?
                    as u8
            } else {
                return Err(Error::ConversionError(
                    "Missing importance in event".to_string(),
                ));
            };

            let mut contexts = Vec::new();
            if let Some(Value::Array(arr)) = obj.get("contexts") {
                for value in arr.iter() {
                    contexts.push(EpisodicContext::from_value(value)?);
                }
            }

            let mut metadata = HashMap::new();
            if let Some(Value::Object(meta_obj)) = obj.get("metadata") {
                for (key, val) in meta_obj.iter() {
                    metadata.insert(key.to_string(), val.clone());
                }
            }

            Ok(Self {
                id,
                description,
                timestamp,
                importance,
                contexts,
                metadata,
            })
        } else {
            Err(Error::ConversionError("Invalid event value".to_string()))
        }
    }
}

/// Episodic memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodicMemory {
    /// Base memory fields
    pub base: BaseMemory,

    /// Events in the episodic memory
    pub events: Vec<EpisodicEvent>,
}

impl EpisodicMemory {
    /// Create a new episodic memory
    pub fn new(id: &str, name: &str, description: &str) -> Self {
        Self {
            base: BaseMemory::new(
                id,
                name,
                description,
                MemoryTypeEnum::Episodic,
                MemoryContent::json(Value::Array(vec![])),
            ),
            events: Vec::new(),
        }
    }

    /// Add an event to the episodic memory
    pub fn add_event(&mut self, event: EpisodicEvent) {
        self.events.push(event);
        self.base.updated_at = chrono::Utc::now();
    }

    /// Get events in a time range
    pub fn get_events_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<&EpisodicEvent> {
        self.events
            .iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .collect()
    }

    /// Get events by importance threshold
    pub fn get_events_by_importance(&self, min_importance: u8) -> Vec<&EpisodicEvent> {
        self.events
            .iter()
            .filter(|e| e.importance >= min_importance)
            .collect()
    }

    /// Get events by context type and value
    pub fn get_events_by_context(&self, context_type: &str, value: &str) -> Vec<&EpisodicEvent> {
        self.events
            .iter()
            .filter(|e| {
                e.contexts
                    .iter()
                    .any(|c| c.context_type == context_type && c.value == value)
            })
            .collect()
    }

    /// Convert to a Memory object
    pub fn to_memory(&self) -> Box<dyn Memory> {
        Box::new(self.base.clone())
    }

    /// Convert from a Memory object
    pub fn from_memory(memory: &dyn Memory) -> Result<Self> {
        let base = BaseMemory {
            id: memory.id().to_string(),
            name: "Episodic Memory".to_string(),
            description: "Converted from memory object".to_string(),
            updated_at: chrono::Utc::now(),
            metadata: memory.metadata().clone(),
            content: memory.content().clone(),
        };

        // Parse events from JSON
        let events = if let Value::Array(arr) = memory.data() {
            let mut events = Vec::new();
            for value in arr.iter() {
                if let Ok(event) = serde_json::from_value(value.clone()) {
                    events.push(event);
                }
            }
            events
        } else {
            Vec::new()
        };

        Ok(Self { base, events })
    }

    /// Generate a timeline summary of the episodic memory
    pub fn generate_timeline(&self) -> String {
        let mut events = self.events.clone();
        events.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        let mut timeline = String::new();
        timeline.push_str(&format!("Timeline for {}\n", self.base.name));
        timeline.push_str(&format!("Description: {}\n\n", self.base.description));

        for event in events {
            timeline.push_str(&format!(
                "[{}] {}\n",
                event.timestamp.format("%Y-%m-%d %H:%M:%S"),
                event.description
            ));

            if !event.contexts.is_empty() {
                timeline.push_str("  Contexts:\n");
                for context in event.contexts {
                    timeline.push_str(&format!(
                        "    - {} ({}): {}\n",
                        context.context_type, context.id, context.value
                    ));
                }
            }

            timeline.push('\n');
        }

        timeline
    }

    /// Find related events by context similarity
    pub fn find_related_events(&self, event_id: &str) -> Result<Vec<&EpisodicEvent>> {
        let event = self
            .events
            .iter()
            .find(|e| e.id == event_id)
            .ok_or_else(|| Error::NotFound(format!("Event with ID {} not found", event_id)))?;

        let mut related = Vec::new();
        for other in &self.events {
            if other.id == event_id {
                continue;
            }

            // Check for shared contexts
            for context in &event.contexts {
                if other
                    .contexts
                    .iter()
                    .any(|c| c.context_type == context.context_type && c.value == context.value)
                {
                    related.push(other);
                    break;
                }
            }
        }

        Ok(related)
    }

    /// Summarize the episodic memory
    pub fn summarize(&self) -> String {
        let mut summary = String::new();
        summary.push_str(&format!("Episodic Memory: {}\n", self.base.name));
        summary.push_str(&format!("Description: {}\n", self.base.description));
        summary.push_str(&format!("Events: {}\n", self.events.len()));

        if !self.events.is_empty() {
            let mut events = self.events.clone();
            events.sort_by(|a, b| b.importance.cmp(&a.importance));

            summary.push_str("\nKey Events:\n");
            for event in events.iter().take(5) {
                summary.push_str(&format!(
                    "- [{}] {} (Importance: {})\n",
                    event.timestamp.format("%Y-%m-%d"),
                    event.description,
                    event.importance
                ));
            }

            // Find most common contexts
            let mut context_counts: HashMap<(String, String), usize> = HashMap::new();
            for event in &self.events {
                for context in &event.contexts {
                    let key = (context.context_type.clone(), context.value.clone());
                    *context_counts.entry(key).or_insert(0) += 1;
                }
            }

            let mut context_vec: Vec<_> = context_counts.into_iter().collect();
            context_vec.sort_by(|a, b| b.1.cmp(&a.1));

            if !context_vec.is_empty() {
                summary.push_str("\nCommon Contexts:\n");
                for ((context_type, value), count) in context_vec.iter().take(5) {
                    summary.push_str(&format!(
                        "- {} ({}): {} occurrences\n",
                        context_type, value, count
                    ));
                }
            }
        }

        summary
    }
}

/// Episodic memory manager
pub struct EpisodicMemoryManager {
    /// Memory repository
    memory_repo: Arc<Mutex<MemoryRepository>>,
}

impl EpisodicMemoryManager {
    /// Create a new episodic memory manager
    pub fn new(memory_repo: Arc<Mutex<MemoryRepository>>) -> Self {
        Self { memory_repo }
    }

    /// Get episodic memory by ID
    pub fn get(
        &self,
        id: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<EpisodicMemory>>> + Send>> {
        let id_string = id.to_string();
        let memory_repo = Arc::clone(&self.memory_repo);

        Box::pin(async move {
            let memory_option = memory_repo.lock().await.get(&id_string);

            match memory_option {
                Some(memory) => {
                    // Convert MemoryNode to BaseMemory for from_memory
                    let mut metadata =
                        crate::memory::memory_type::MemoryMetadata::new(MemoryTypeEnum::Episodic);
                    metadata.created_at = memory.created_at;
                    metadata.updated_at = memory.updated_at;

                    let base_memory = BaseMemory {
                        id: memory.id.clone(),
                        name: "Episodic Memory".to_string(),
                        description: "Retrieved from storage".to_string(),
                        updated_at: memory.updated_at,
                        metadata,
                        content: MemoryContent::text(&memory.content),
                    };
                    let episodic = EpisodicMemory::from_memory(&base_memory)?;
                    Ok(Some(episodic))
                }
                None => Ok(None),
            }
        })
    }

    /// Create a new episodic memory
    pub fn create(
        &self,
        id: &str,
        name: &str,
        description: &str,
    ) -> Pin<Box<dyn Future<Output = Result<EpisodicMemory>> + Send>> {
        let id_string = id.to_string();
        let name_string = name.to_string();
        let description_string = description.to_string();
        let memory_repo = Arc::clone(&self.memory_repo);

        Box::pin(async move {
            let episodic = EpisodicMemory::new(&id_string, &name_string, &description_string);

            // Convert to MemoryNode for storage
            let mut metadata = MemoryMetadata::new();
            metadata.created_at = episodic.base.metadata.created_at;

            let memory_node = MemoryNode {
                id: episodic.base.id.clone(),
                content: serde_json::to_string(&episodic.base.content).unwrap_or_default(),
                memory_type: crate::memory::memory_node::MemoryType::Episodic,
                created_at: episodic.base.metadata.created_at,
                updated_at: episodic.base.updated_at,
                embedding: None,
                metadata,
            };

            let mut memory_repo_guard = memory_repo.lock().await;
            let created_memory = memory_repo_guard.create(&id_string, &memory_node)?;
            // Convert created MemoryNode to BaseMemory
            let mut metadata =
                crate::memory::memory_type::MemoryMetadata::new(MemoryTypeEnum::Episodic);
            metadata.created_at = created_memory.created_at;
            metadata.updated_at = created_memory.updated_at;

            let base_memory = BaseMemory {
                id: created_memory.id.clone(),
                name: name_string.clone(),
                description: description_string.clone(),
                updated_at: created_memory.updated_at,
                metadata,
                content: MemoryContent::text(&created_memory.content),
            };
            let created_episodic = EpisodicMemory::from_memory(&base_memory)?;

            Ok(created_episodic)
        })
    }
}
