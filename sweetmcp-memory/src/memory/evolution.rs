// src/memory/evolution.rs
//! Memory evolution tracking implementation for Rust-mem0.
//!
//! This module provides tracking of memory evolution over time,
//! including transitions, relationships between versions, and
//! analysis of memory development patterns.

use crate::graph::entity::{BaseEntity, Entity};
use crate::utils::Result;
use crate::utils::error::Error;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use surrealdb::sql::Value;

/// Evolution transition type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransitionType {
    /// Linear progression (simple update)
    Linear,
    /// Branching (creating multiple versions from one)
    Branching,
    /// Merging (combining multiple versions)
    Merging,
    /// Restoration (bringing back a deleted or previous version)
    Restoration,
    /// Deletion (removing a version)
    Deletion,
    /// Custom transition
    Custom(u8),
}

impl TransitionType {
    /// Convert to string
    pub fn as_str(&self) -> &'static str {
        match self {
            TransitionType::Linear => "linear",
            TransitionType::Branching => "branching",
            TransitionType::Merging => "merging",
            TransitionType::Restoration => "restoration",
            TransitionType::Deletion => "deletion",
            TransitionType::Custom(_) => "custom",
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "linear" => Ok(TransitionType::Linear),
            "branching" => Ok(TransitionType::Branching),
            "merging" => Ok(TransitionType::Merging),
            "restoration" => Ok(TransitionType::Restoration),
            "deletion" => Ok(TransitionType::Deletion),
            s if s.starts_with("custom") => {
                if let Some(code_str) = s.strip_prefix("custom") {
                    if let Ok(code) = code_str.trim().parse::<u8>() {
                        Ok(TransitionType::Custom(code))
                    } else {
                        Ok(TransitionType::Custom(0))
                    }
                } else {
                    Ok(TransitionType::Custom(0))
                }
            }
            _ => Err(Error::ValidationError(format!(
                "Invalid transition type: {}",
                s
            ))),
        }
    }

    /// Convert to value
    pub fn to_value(&self) -> Value {
        match self {
            TransitionType::Linear => Value::Strand("linear".into()),
            TransitionType::Branching => Value::Strand("branching".into()),
            TransitionType::Merging => Value::Strand("merging".into()),
            TransitionType::Restoration => Value::Strand("restoration".into()),
            TransitionType::Deletion => Value::Strand("deletion".into()),
            TransitionType::Custom(code) => Value::Strand(format!("custom{}", code).into()),
        }
    }

    /// Create from value
    pub fn from_value(value: &Value) -> Result<Self> {
        if let Value::Strand(s) = value {
            Self::from_str(&s.to_string())
        } else {
            Err(Error::ConversionError(
                "Invalid transition type value".to_string(),
            ))
        }
    }
}

/// Memory evolution transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionTransition {
    /// Transition ID
    pub id: String,
    /// Memory ID
    pub memory_id: String,
    /// Transition type
    pub transition_type: TransitionType,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Primary source version ID (for simple transitions)
    pub from_version_id: String,
    /// Primary target version ID (for simple transitions)
    pub to_version_id: String,
    /// Source version IDs (for complex transitions)
    pub source_version_ids: Vec<String>,
    /// Target version IDs (for complex transitions)
    pub target_version_ids: Vec<String>,
    /// Transition metadata
    pub metadata: HashMap<String, Value>,
    /// Transition description
    pub description: Option<String>,
}

impl EvolutionTransition {
    /// Create a new evolution transition
    pub fn new(
        id: &str,
        memory_id: &str,
        transition_type: TransitionType,
        source_version_ids: Vec<String>,
        target_version_ids: Vec<String>,
    ) -> Self {
        let now = Utc::now();

        // Set primary version IDs from first elements of vectors
        let from_version_id = source_version_ids.first().unwrap_or(&String::new()).clone();
        let to_version_id = target_version_ids.first().unwrap_or(&String::new()).clone();

        Self {
            id: id.to_string(),
            memory_id: memory_id.to_string(),
            transition_type,
            timestamp: now,
            from_version_id,
            to_version_id,
            source_version_ids,
            target_version_ids,
            metadata: HashMap::new(),
            description: None,
        }
    }

    /// Create a linear transition
    pub fn linear(
        id: &str,
        memory_id: &str,
        source_version_id: &str,
        target_version_id: &str,
    ) -> Self {
        Self::new(
            id,
            memory_id,
            TransitionType::Linear,
            vec![source_version_id.to_string()],
            vec![target_version_id.to_string()],
        )
    }

    /// Create a branching transition
    pub fn branching(
        id: &str,
        memory_id: &str,
        source_version_id: &str,
        target_version_ids: Vec<String>,
    ) -> Self {
        Self::new(
            id,
            memory_id,
            TransitionType::Branching,
            vec![source_version_id.to_string()],
            target_version_ids,
        )
    }

    /// Create a merging transition
    pub fn merging(
        id: &str,
        memory_id: &str,
        source_version_ids: Vec<String>,
        target_version_id: &str,
    ) -> Self {
        Self::new(
            id,
            memory_id,
            TransitionType::Merging,
            source_version_ids,
            vec![target_version_id.to_string()],
        )
    }

    /// Create a restoration transition
    pub fn restoration(
        id: &str,
        memory_id: &str,
        source_version_id: &str,
        target_version_id: &str,
    ) -> Self {
        Self::new(
            id,
            memory_id,
            TransitionType::Restoration,
            vec![source_version_id.to_string()],
            vec![target_version_id.to_string()],
        )
    }

    /// Create a deletion transition
    pub fn deletion(id: &str, memory_id: &str, source_version_id: &str) -> Self {
        Self::new(
            id,
            memory_id,
            TransitionType::Deletion,
            vec![source_version_id.to_string()],
            vec![],
        )
    }

    /// Add metadata
    pub fn with_metadata<T: Into<Value>>(mut self, key: &str, value: T) -> Self {
        self.metadata.insert(key.to_string(), value.into());
        self
    }

    /// Set description
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Convert to entity
    pub fn to_entity(&self) -> BaseEntity {
        let mut entity = BaseEntity::new(&self.id, "memory_evolution_transition");

        // Add basic attributes
        entity = entity.with_attribute("memory_id", Value::Strand(self.memory_id.clone().into()));
        entity = entity.with_attribute("transition_type", self.transition_type.to_value());
        entity = entity.with_attribute("timestamp", Value::Datetime(self.timestamp.into()));

        // Add source version IDs
        if !self.source_version_ids.is_empty() {
            let source_ids = self
                .source_version_ids
                .iter()
                .map(|id| Value::Strand(id.clone().into()))
                .collect::<Vec<_>>();
            entity = entity.with_attribute("source_version_ids", Value::Array(source_ids.into()));
        }

        // Add target version IDs
        if !self.target_version_ids.is_empty() {
            let target_ids = self
                .target_version_ids
                .iter()
                .map(|id| Value::Strand(id.clone().into()))
                .collect::<Vec<_>>();
            entity = entity.with_attribute("target_version_ids", Value::Array(target_ids.into()));
        }

        // Add description if present
        if let Some(ref description) = self.description {
            entity =
                entity.with_attribute("description", Value::Strand(description.clone().into()));
        }

        // Add metadata
        for (key, value) in &self.metadata {
            entity = entity.with_attribute(&format!("metadata_{}", key), value.clone());
        }

        entity
    }

    /// Create from entity
    pub fn from_entity(entity: &dyn Entity) -> Result<Self> {
        let id = entity.id().to_string();

        let memory_id = if let Some(Value::Strand(s)) = entity.get_attribute("memory_id") {
            s.to_string()
        } else {
            return Err(Error::ConversionError(
                "Missing memory_id attribute".to_string(),
            ));
        };

        let transition_type = if let Some(value) = entity.get_attribute("transition_type") {
            TransitionType::from_value(value)?
        } else {
            return Err(Error::ConversionError(
                "Missing transition_type attribute".to_string(),
            ));
        };

        let timestamp = if let Some(Value::Datetime(dt)) = entity.get_attribute("timestamp") {
            DateTime::<Utc>::from(dt.clone())
        } else {
            return Err(Error::ConversionError(
                "Missing timestamp attribute".to_string(),
            ));
        };

        let source_version_ids =
            if let Some(Value::Array(arr)) = entity.get_attribute("source_version_ids") {
                arr.iter()
                    .filter_map(|value| {
                        if let Value::Strand(s) = value {
                            Some(s.to_string())
                        } else {
                            None
                        }
                    })
                    .collect()
            } else {
                Vec::new()
            };

        let target_version_ids =
            if let Some(Value::Array(arr)) = entity.get_attribute("target_version_ids") {
                arr.iter()
                    .filter_map(|value| {
                        if let Value::Strand(s) = value {
                            Some(s.to_string())
                        } else {
                            None
                        }
                    })
                    .collect()
            } else {
                Vec::new()
            };

        let description = if let Some(Value::Strand(s)) = entity.get_attribute("description") {
            Some(s.to_string())
        } else {
            None
        };

        // Extract metadata
        let mut metadata = HashMap::new();
        for (key, value) in entity.attributes() {
            if key.starts_with("metadata_") {
                let metadata_key = key.strip_prefix("metadata_").unwrap().to_string();
                metadata.insert(metadata_key, value.clone());
            }
        }

        Ok(Self {
            id,
            memory_id,
            transition_type,
            timestamp,
            source_version_ids,
            target_version_ids,
            from_version_id: String::new(),
            to_version_id: String::new(),
            metadata,
            description,
        })
    }
}

/// Memory evolution graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionGraph {
    /// Memory ID
    pub memory_id: String,
    /// Transitions
    pub transitions: Vec<EvolutionTransition>,
    /// Version IDs
    pub version_ids: HashSet<String>,
}

impl EvolutionGraph {
    /// Create a new evolution graph
    pub fn new(memory_id: &str) -> Self {
        Self {
            memory_id: memory_id.to_string(),
            transitions: Vec::new(),
            version_ids: HashSet::new(),
        }
    }

    /// Add transition
    pub fn add_transition(&mut self, transition: EvolutionTransition) -> Result<()> {
        // Validate transition
        if transition.memory_id != self.memory_id {
            return Err(Error::ValidationError(format!(
                "Transition memory ID {} does not match graph memory ID {}",
                transition.memory_id, self.memory_id
            )));
        }

        // Add transition
        self.transitions.push(transition.clone());

        // Add version IDs
        for id in &transition.source_version_ids {
            self.version_ids.insert(id.clone());
        }

        for id in &transition.target_version_ids {
            self.version_ids.insert(id.clone());
        }

        // Sort transitions by timestamp
        self.transitions
            .sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        Ok(())
    }

    /// Get transition by ID
    pub fn get_transition(&self, id: &str) -> Option<&EvolutionTransition> {
        self.transitions.iter().find(|t| t.id == id)
    }

    /// Get transitions by type
    pub fn get_transitions_by_type(
        &self,
        transition_type: TransitionType,
    ) -> Vec<&EvolutionTransition> {
        self.transitions
            .iter()
            .filter(|t| t.transition_type == transition_type)
            .collect()
    }

    /// Get transitions by time range
    pub fn get_transitions_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<&EvolutionTransition> {
        self.transitions
            .iter()
            .filter(|t| t.timestamp >= start && t.timestamp <= end)
            .collect()
    }

    /// Get transitions for version
    pub fn get_transitions_for_version(&self, version_id: &str) -> Vec<&EvolutionTransition> {
        self.transitions
            .iter()
            .filter(|t| {
                t.source_version_ids.contains(&version_id.to_string())
                    || t.target_version_ids.contains(&version_id.to_string())
            })
            .collect()
    }

    /// Get incoming transitions for version
    pub fn get_incoming_transitions(&self, version_id: &str) -> Vec<&EvolutionTransition> {
        self.transitions
            .iter()
            .filter(|t| t.target_version_ids.contains(&version_id.to_string()))
            .collect()
    }

    /// Get outgoing transitions for version
    pub fn get_outgoing_transitions(&self, version_id: &str) -> Vec<&EvolutionTransition> {
        self.transitions
            .iter()
            .filter(|t| t.source_version_ids.contains(&version_id.to_string()))
            .collect()
    }

    /// Get evolution path between versions
    pub fn get_evolution_path(
        &self,
        from_version_id: &str,
        to_version_id: &str,
    ) -> Result<Vec<&EvolutionTransition>> {
        // Check if versions exist
        if !self.version_ids.contains(from_version_id) {
            return Err(Error::NotFound(format!(
                "Version {} not found",
                from_version_id
            )));
        }

        if !self.version_ids.contains(to_version_id) {
            return Err(Error::NotFound(format!(
                "Version {} not found",
                to_version_id
            )));
        }

        // If same version, return empty path
        if from_version_id == to_version_id {
            return Ok(Vec::new());
        }

        // Build graph representation for path finding
        let mut graph: HashMap<String, Vec<(String, &EvolutionTransition)>> = HashMap::new();

        for transition in &self.transitions {
            for source_id in &transition.source_version_ids {
                for target_id in &transition.target_version_ids {
                    graph
                        .entry(source_id.clone())
                        .or_insert_with(Vec::new)
                        .push((target_id.clone(), transition));
                }
            }
        }

        // Perform breadth-first search
        let mut queue = std::collections::VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent_map: HashMap<String, String> = HashMap::new();

        queue.push_back(from_version_id.to_string());
        visited.insert(from_version_id.to_string());

        while let Some(current) = queue.pop_front() {
            if current == to_version_id {
                // Found path, reconstruct it
                let mut path = Vec::new();
                let mut current_id = to_version_id;

                while let Some(parent_id) = parent_map.get(current_id) {
                    if let Some(transition) = self
                        .transitions
                        .iter()
                        .find(|t| &t.from_version_id == parent_id && &t.to_version_id == current_id)
                    {
                        path.push(transition);
                    }
                    current_id = parent_id;
                    if current_id == from_version_id {
                        break;
                    }
                }

                path.reverse();
                return Ok(path);
            }

            // Explore neighbors
            for transition in &self.transitions {
                if transition.from_version_id == current
                    && !visited.contains(&transition.to_version_id)
                {
                    visited.insert(transition.to_version_id.clone());
                    parent_map.insert(transition.to_version_id.clone(), current.clone());
                    queue.push_back(transition.to_version_id.clone());
                }
            }
        }

        Err(Error::NotFound(format!(
            "No evolution path found from version {} to {}",
            from_version_id, to_version_id
        )))
    }
}
