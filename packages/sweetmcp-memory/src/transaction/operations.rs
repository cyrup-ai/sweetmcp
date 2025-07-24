//! Transaction operations extracted from transaction manager

use serde::{Deserialize, Serialize};

/// Operation performed in a transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operation {
    /// Insert operation
    Insert {
        table: String,
        id: String,
        data: serde_json::Value,
    },

    /// Update operation
    Update {
        table: String,
        id: String,
        data: serde_json::Value,
    },

    /// Delete operation
    Delete { table: String, id: String },
}

impl Operation {
    /// Create a new insert operation
    pub fn insert(table: String, id: String, data: serde_json::Value) -> Self {
        Self::Insert { table, id, data }
    }

    /// Create a new update operation
    pub fn update(table: String, id: String, data: serde_json::Value) -> Self {
        Self::Update { table, id, data }
    }

    /// Create a new delete operation
    pub fn delete(table: String, id: String) -> Self {
        Self::Delete { table, id }
    }

    /// Get the table affected by this operation
    pub fn table(&self) -> &str {
        match self {
            Operation::Insert { table, .. } => table,
            Operation::Update { table, .. } => table,
            Operation::Delete { table, .. } => table,
        }
    }

    /// Get the ID affected by this operation
    pub fn id(&self) -> &str {
        match self {
            Operation::Insert { id, .. } => id,
            Operation::Update { id, .. } => id,
            Operation::Delete { id, .. } => id,
        }
    }

    /// Get the data for this operation (if applicable)
    pub fn data(&self) -> Option<&serde_json::Value> {
        match self {
            Operation::Insert { data, .. } => Some(data),
            Operation::Update { data, .. } => Some(data),
            Operation::Delete { .. } => None,
        }
    }

    /// Check if this is a write operation (insert/update/delete)
    pub fn is_write_operation(&self) -> bool {
        true // All operations are write operations
    }

    /// Get operation type as string
    pub fn operation_type(&self) -> &'static str {
        match self {
            Operation::Insert { .. } => "INSERT",
            Operation::Update { .. } => "UPDATE",
            Operation::Delete { .. } => "DELETE",
        }
    }
}