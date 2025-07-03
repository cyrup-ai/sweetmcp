//! Operation monitoring and tracking

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;


/// Operation types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationType {
    /// Memory creation
    CreateMemory,
    /// Memory update
    UpdateMemory,
    /// Memory deletion
    DeleteMemory,
    /// Memory search
    SearchMemory,
    /// Relationship creation
    CreateRelationship,
    /// Relationship deletion
    DeleteRelationship,
    /// Batch operation
    BatchOperation,
    /// Custom operation
    Custom(String),
}

/// Operation status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OperationStatus {
    /// Operation is pending
    Pending,
    /// Operation is in progress
    InProgress,
    /// Operation completed successfully
    Success,
    /// Operation failed
    Failed,
    /// Operation was cancelled
    Cancelled,
}

/// Operation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    /// Operation ID
    pub id: String,
    
    /// Operation type
    pub operation_type: OperationType,
    
    /// Operation status
    pub status: OperationStatus,
    
    /// Start time
    pub started_at: DateTime<Utc>,
    
    /// End time
    pub ended_at: Option<DateTime<Utc>>,
    
    /// Duration
    pub duration: Option<Duration>,
    
    /// User ID
    pub user_id: Option<String>,
    
    /// Error message if failed
    pub error: Option<String>,
    
    /// Additional metadata
    pub metadata: serde_json::Value,
}

impl Operation {
    /// Create a new operation
    pub fn new(operation_type: OperationType, user_id: Option<String>) -> Self {
        Self {
            id: String::new_v4(),
            operation_type,
            status: OperationStatus::Pending,
            started_at: Utc::now(),
            ended_at: None,
            duration: None,
            user_id,
            error: None,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
    
    /// Start the operation
    pub fn start(&mut self) {
        self.status = OperationStatus::InProgress;
        self.started_at = Utc::now();
    }
    
    /// Complete the operation successfully
    pub fn complete(&mut self) {
        let now = Utc::now();
        self.status = OperationStatus::Success;
        self.ended_at = Some(now);
        self.duration = Some(
            now.signed_duration_since(self.started_at)
                .to_std()
                .unwrap_or(Duration::from_secs(0))
        );
    }
    
    /// Fail the operation
    pub fn fail(&mut self, error: String) {
        let now = Utc::now();
        self.status = OperationStatus::Failed;
        self.ended_at = Some(now);
        self.duration = Some(
            now.signed_duration_since(self.started_at)
                .to_std()
                .unwrap_or(Duration::from_secs(0))
        );
        self.error = Some(error);
    }
}

/// Operation tracker
pub struct OperationTracker {
    /// Active operations
    active: std::sync::RwLock<HashMap<String, Operation>>,
    
    /// Completed operations (limited history)
    completed: std::sync::RwLock<Vec<Operation>>,
    
    /// Maximum completed operations to keep
    max_history: usize,
}

impl OperationTracker {
    /// Create a new tracker
    pub fn new(max_history: usize) -> Self {
        Self {
            active: std::sync::RwLock::new(HashMap::new()),
            completed: std::sync::RwLock::new(Vec::new()),
            max_history,
        }
    }
    
    /// Start tracking an operation
    pub fn start_operation(
        &self,
        operation_type: OperationType,
        user_id: Option<String>,
    ) -> String {
        let mut operation = Operation::new(operation_type, user_id);
        operation.start();
        let id = operation.id;
        
        self.active.write().unwrap().insert(id, operation);
        id
    }
    
    /// Complete an operation
    pub fn complete_operation(&self, id: String) {
        if let Some(mut operation) = self.active.write().unwrap().remove(&id) {
            operation.complete();
            self.add_to_history(operation);
        }
    }
    
    /// Fail an operation
    pub fn fail_operation(&self, id: String, error: String) {
        if let Some(mut operation) = self.active.write().unwrap().remove(&id) {
            operation.fail(error);
            self.add_to_history(operation);
        }
    }
    
    /// Add operation to history
    fn add_to_history(&self, operation: Operation) {
        let mut completed = self.completed.write().unwrap();
        completed.push(operation);
        
        // Keep only the most recent operations
        if completed.len() > self.max_history {
            completed.drain(0..completed.len() - self.max_history);
        }
    }
    
    /// Get active operations
    pub fn active_operations(&self) -> Vec<Operation> {
        self.active.read().unwrap().values().cloned().collect()
    }
    
    /// Get operation history
    pub fn operation_history(&self) -> Vec<Operation> {
        self.completed.read().unwrap().clone()
    }
}

impl Default for OperationTracker {
    fn default() -> Self {
        Self::new(1000)
    }
}

use std::collections::HashMap;