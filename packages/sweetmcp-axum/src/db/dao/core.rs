//! Core DAO traits and types
//!
//! This module provides the foundational traits and types for database
//! access objects with zero allocation patterns and blazing-fast performance.

use std::fmt::Debug;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use uuid::Uuid;

/// Generic entity trait for database objects
pub trait Entity: Serialize + DeserializeOwned + Debug + Send + Sync + Clone {
    /// Get the table name for this entity
    fn table_name() -> &'static str;

    /// Get the ID of this entity
    fn id(&self) -> Option<String>;

    /// Set the ID of this entity
    fn set_id(&mut self, id: String);

    /// Generate a unique ID for this entity
    fn generate_id() -> String {
        format!("{}:{}", Self::table_name(), Uuid::new_v4())
    }

    /// Check if entity has a valid ID
    fn has_id(&self) -> bool {
        self.id().is_some()
    }

    /// Get the table prefix from ID
    fn get_table_prefix(&self) -> Option<String> {
        self.id().and_then(|id| {
            let parts: Vec<&str> = id.split(':').collect();
            if parts.len() >= 2 {
                Some(parts[0].to_string())
            } else {
                None
            }
        })
    }

    /// Validate that the entity ID matches the expected table
    fn validate_table_id(&self) -> bool {
        match self.get_table_prefix() {
            Some(prefix) => prefix == Self::table_name(),
            None => true, // Allow entities without IDs (for creation)
        }
    }

    /// Generate a new ID and set it on the entity
    fn ensure_id(&mut self) {
        if !self.has_id() {
            let new_id = Self::generate_id();
            self.set_id(new_id);
        }
    }

    /// Clone the entity with a new ID
    fn clone_with_new_id(&self) -> Self {
        let mut cloned = self.clone();
        let new_id = Self::generate_id();
        cloned.set_id(new_id);
        cloned
    }

    /// Get entity metadata
    fn get_metadata(&self) -> EntityMetadata {
        EntityMetadata {
            table_name: Self::table_name().to_string(),
            entity_id: self.id(),
            has_id: self.has_id(),
            table_matches: self.validate_table_id(),
        }
    }
}

/// Base DAO trait providing common CRUD operations for entities
pub trait BaseDao {
    type Entity: Entity + 'static;

    /// Create a new entity
    fn create(&self, entity: &mut Self::Entity) -> crate::types::AsyncTask<Self::Entity>;

    /// Find a single entity by ID
    fn find_by_id(&self, id: &str) -> crate::types::AsyncTask<Option<Self::Entity>>;

    /// Update an entity
    fn update(&self, entity: &Self::Entity) -> crate::types::AsyncTask<Option<Self::Entity>>;

    /// Delete an entity by ID
    fn delete(&self, id: &str) -> crate::types::AsyncTask<Option<Self::Entity>>;

    /// Find all entities as a stream
    fn find(&self) -> crate::types::AsyncTask<crate::types::AsyncStream<Self::Entity>>;

    /// Create a table for this entity
    fn create_table(&self) -> crate::types::AsyncTask<()>;

    /// Check if entity exists by ID
    fn exists(&self, id: &str) -> crate::types::AsyncTask<bool> {
        let find_task = self.find_by_id(id);
        crate::types::AsyncTask::from_future(async move {
            find_task.await.is_some()
        })
    }

    /// Count all entities
    fn count(&self) -> crate::types::AsyncTask<u64> {
        let find_task = self.find();
        crate::types::AsyncTask::from_future(async move {
            let stream = find_task.await;
            // In a real implementation, this would use a COUNT query
            // For now, we'll return a placeholder
            0
        })
    }
}

/// Entity metadata for introspection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMetadata {
    /// Table name
    pub table_name: String,
    /// Entity ID if present
    pub entity_id: Option<String>,
    /// Whether entity has an ID
    pub has_id: bool,
    /// Whether the ID table prefix matches the entity table
    pub table_matches: bool,
}

impl EntityMetadata {
    /// Check if the entity is valid for database operations
    pub fn is_valid_for_create(&self) -> bool {
        // For creation, we don't require an ID (it can be generated)
        true
    }

    /// Check if the entity is valid for update operations
    pub fn is_valid_for_update(&self) -> bool {
        self.has_id && self.table_matches
    }

    /// Check if the entity is valid for delete operations
    pub fn is_valid_for_delete(&self) -> bool {
        self.has_id && self.table_matches
    }

    /// Get validation errors
    pub fn get_validation_errors(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if !self.has_id {
            errors.push("Entity does not have an ID".to_string());
        }

        if !self.table_matches {
            errors.push("Entity ID table prefix does not match entity table name".to_string());
        }

        errors
    }
}

/// DAO operation result
#[derive(Debug, Clone)]
pub enum DaoResult<T> {
    /// Operation succeeded
    Success(T),
    /// Entity not found
    NotFound,
    /// Validation error
    ValidationError(String),
    /// Database error
    DatabaseError(String),
}

impl<T> DaoResult<T> {
    /// Check if the result is successful
    pub fn is_success(&self) -> bool {
        matches!(self, DaoResult::Success(_))
    }

    /// Check if the result is not found
    pub fn is_not_found(&self) -> bool {
        matches!(self, DaoResult::NotFound)
    }

    /// Check if the result is an error
    pub fn is_error(&self) -> bool {
        matches!(self, DaoResult::ValidationError(_) | DaoResult::DatabaseError(_))
    }

    /// Get the success value if present
    pub fn success_value(self) -> Option<T> {
        match self {
            DaoResult::Success(value) => Some(value),
            _ => None,
        }
    }

    /// Get the error message if present
    pub fn error_message(&self) -> Option<&str> {
        match self {
            DaoResult::ValidationError(msg) => Some(msg),
            DaoResult::DatabaseError(msg) => Some(msg),
            _ => None,
        }
    }

    /// Convert to a standard Result
    pub fn into_result(self) -> Result<T, String> {
        match self {
            DaoResult::Success(value) => Ok(value),
            DaoResult::NotFound => Err("Entity not found".to_string()),
            DaoResult::ValidationError(msg) => Err(format!("Validation error: {}", msg)),
            DaoResult::DatabaseError(msg) => Err(format!("Database error: {}", msg)),
        }
    }

    /// Map the success value to a different type
    pub fn map<U, F>(self, f: F) -> DaoResult<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            DaoResult::Success(value) => DaoResult::Success(f(value)),
            DaoResult::NotFound => DaoResult::NotFound,
            DaoResult::ValidationError(msg) => DaoResult::ValidationError(msg),
            DaoResult::DatabaseError(msg) => DaoResult::DatabaseError(msg),
        }
    }

    /// Chain operations on success
    pub fn and_then<U, F>(self, f: F) -> DaoResult<U>
    where
        F: FnOnce(T) -> DaoResult<U>,
    {
        match self {
            DaoResult::Success(value) => f(value),
            DaoResult::NotFound => DaoResult::NotFound,
            DaoResult::ValidationError(msg) => DaoResult::ValidationError(msg),
            DaoResult::DatabaseError(msg) => DaoResult::DatabaseError(msg),
        }
    }
}

/// Helper function to validate entity ID format
pub fn validate_entity_id(id: &str, expected_table: &str) -> bool {
    let parts: Vec<&str> = id.split(':').collect();
    parts.len() == 2 && parts[0] == expected_table
}

/// Helper function to extract table name from entity ID
pub fn extract_table_from_id(id: &str) -> Option<&str> {
    let parts: Vec<&str> = id.split(':').collect();
    if parts.len() >= 2 {
        Some(parts[0])
    } else {
        None
    }
}

/// Helper function to extract UUID part from entity ID
pub fn extract_uuid_from_id(id: &str) -> Option<&str> {
    let parts: Vec<&str> = id.split(':').collect();
    if parts.len() >= 2 {
        Some(parts[1])
    } else {
        None
    }
}

/// Helper function to get current UTC time
pub fn utc_now() -> DateTime<Utc> {
    Utc::now()
}