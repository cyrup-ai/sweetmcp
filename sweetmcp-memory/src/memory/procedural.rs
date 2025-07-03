// src/memory/procedural.rs
//! Procedural memory implementation for Rust-mem0.
//!
//! This module provides a specialized memory type for storing action sequences
//! and procedural knowledge, with support for steps, conditions, and execution.

use crate::graph::graph_db::{GraphError, Result};
use crate::memory::memory_type::{BaseMemory, MemoryContent, MemoryTypeEnum};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use surrealdb::sql::{Object, Value};

/// Step status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepStatus {
    /// Step is pending execution
    Pending,
    /// Step is currently executing
    Executing,
    /// Step has completed successfully
    Completed,
    /// Step has failed
    Failed,
    /// Step has been skipped
    Skipped,
}

impl StepStatus {
    /// Convert to string
    pub fn as_str(&self) -> &'static str {
        match self {
            StepStatus::Pending => "pending",
            StepStatus::Executing => "executing",
            StepStatus::Completed => "completed",
            StepStatus::Failed => "failed",
            StepStatus::Skipped => "skipped",
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(StepStatus::Pending),
            "executing" => Ok(StepStatus::Executing),
            "completed" => Ok(StepStatus::Completed),
            "failed" => Ok(StepStatus::Failed),
            "skipped" => Ok(StepStatus::Skipped),
            _ => Err(GraphError::ValidationError(format!(
                "Invalid step status: {}",
                s
            ))),
        }
    }
}

/// Condition type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConditionType {
    /// Condition is a prerequisite
    Prerequisite,
    /// Condition is a postcondition
    Postcondition,
    /// Condition is an invariant
    Invariant,
}

impl ConditionType {
    /// Convert to string
    pub fn as_str(&self) -> &'static str {
        match self {
            ConditionType::Prerequisite => "prerequisite",
            ConditionType::Postcondition => "postcondition",
            ConditionType::Invariant => "invariant",
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "prerequisite" => Ok(ConditionType::Prerequisite),
            "postcondition" => Ok(ConditionType::Postcondition),
            "invariant" => Ok(ConditionType::Invariant),
            _ => Err(GraphError::ValidationError(format!(
                "Invalid condition type: {}",
                s
            ))),
        }
    }
}

/// Condition struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    /// Condition ID
    pub id: String,

    /// Condition type
    pub condition_type: ConditionType,

    /// Condition description
    pub description: String,

    /// Condition expression
    pub expression: Value,

    /// Is the condition required
    pub required: bool,
}

impl Condition {
    /// Create a new condition
    pub fn new(
        id: &str,
        condition_type: ConditionType,
        description: &str,
        expression: Value,
        required: bool,
    ) -> Self {
        Self {
            id: id.to_string(),
            condition_type,
            description: description.to_string(),
            expression,
            required,
        }
    }

    /// Create a new prerequisite
    pub fn prerequisite(id: &str, description: &str, expression: Value, required: bool) -> Self {
        Self::new(
            id,
            ConditionType::Prerequisite,
            description,
            expression,
            required,
        )
    }

    /// Create a new postcondition
    pub fn postcondition(id: &str, description: &str, expression: Value, required: bool) -> Self {
        Self::new(
            id,
            ConditionType::Postcondition,
            description,
            expression,
            required,
        )
    }

    /// Create a new invariant
    pub fn invariant(id: &str, description: &str, expression: Value, required: bool) -> Self {
        Self::new(
            id,
            ConditionType::Invariant,
            description,
            expression,
            required,
        )
    }

    /// Convert to Value
    pub fn to_value(&self) -> Value {
        let mut obj = Object::default();
        obj.insert("id".into(), Value::Strand(self.id.clone().into()));
        obj.insert(
            "condition_type".into(),
            Value::Strand(self.condition_type.as_str().into()),
        );
        obj.insert(
            "description".into(),
            Value::Strand(self.description.clone().into()),
        );
        obj.insert("expression".into(), self.expression.clone());
        obj.insert("required".into(), Value::Bool(self.required));
        Value::Object(obj)
    }

    /// Create from Value
    pub fn from_value(value: &Value) -> Result<Self> {
        if let Value::Object(obj) = value {
            let id = if let Some(Value::Strand(s)) = obj.get("id") {
                s.to_string()
            } else {
                return Err(GraphError::ConversionError(
                    "Missing id in condition".to_string(),
                ));
            };

            let condition_type = if let Some(Value::Strand(s)) = obj.get("condition_type") {
                ConditionType::from_str(&s.to_string())?
            } else {
                return Err(GraphError::ConversionError(
                    "Missing condition_type in condition".to_string(),
                ));
            };

            let description = if let Some(Value::Strand(s)) = obj.get("description") {
                s.to_string()
            } else {
                return Err(GraphError::ConversionError(
                    "Missing description in condition".to_string(),
                ));
            };

            let expression = if let Some(expr) = obj.get("expression") {
                expr.clone()
            } else {
                return Err(GraphError::ConversionError(
                    "Missing expression in condition".to_string(),
                ));
            };

            let required = if let Some(Value::Bool(b)) = obj.get("required") {
                *b
            } else {
                false
            };

            Ok(Self {
                id,
                condition_type,
                description,
                expression,
                required,
            })
        } else {
            Err(GraphError::ConversionError(
                "Value is not an object".to_string(),
            ))
        }
    }
}

/// Step struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    /// Step ID
    pub id: String,

    /// Step name
    pub name: String,

    /// Step description
    pub description: String,

    /// Step order
    pub order: u32,

    /// Step action
    pub action: Value,

    /// Step status
    pub status: StepStatus,

    /// Step conditions
    pub conditions: Vec<Condition>,

    /// Step dependencies
    pub dependencies: Vec<String>,

    /// Step result
    pub result: Option<Value>,

    /// Step error
    pub error: Option<String>,

    /// Step metadata
    pub metadata: HashMap<String, Value>,
}

impl Step {
    /// Create a new step
    pub fn new(id: &str, name: &str, description: &str, order: u32, action: Value) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            order,
            action,
            status: StepStatus::Pending,
            conditions: Vec::new(),
            dependencies: Vec::new(),
            result: None,
            error: None,
            metadata: HashMap::new(),
        }
    }

    /// Add a condition
    pub fn with_condition(mut self, condition: Condition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// Add a dependency
    pub fn with_dependency(mut self, dependency_id: &str) -> Self {
        self.dependencies.push(dependency_id.to_string());
        self
    }

    /// Set status
    pub fn with_status(mut self, status: StepStatus) -> Self {
        self.status = status;
        self
    }

    /// Set result
    pub fn with_result(mut self, result: Value) -> Self {
        self.result = Some(result);
        self
    }

    /// Set error
    pub fn with_error(mut self, error: &str) -> Self {
        self.error = Some(error.to_string());
        self
    }

    /// Add metadata
    pub fn with_metadata<T: Into<Value>>(mut self, key: &str, value: T) -> Self {
        self.metadata.insert(key.to_string(), value.into());
        self
    }

    /// Convert to Value
    pub fn to_value(&self) -> Value {
        let mut obj = Object::default();
        obj.insert("id".into(), Value::Strand(self.id.clone().into()));
        obj.insert("name".into(), Value::Strand(self.name.clone().into()));
        obj.insert(
            "description".into(),
            Value::Strand(self.description.clone().into()),
        );
        obj.insert("order".into(), Value::Number(self.order.into()));
        obj.insert("action".into(), self.action.clone());
        obj.insert("status".into(), Value::Strand(self.status.as_str().into()));

        let conditions = self
            .conditions
            .iter()
            .map(|c| c.to_value())
            .collect::<Vec<_>>();
        obj.insert("conditions".into(), Value::Array(conditions.into()));

        let dependencies = self
            .dependencies
            .iter()
            .map(|d| Value::Strand(d.clone().into()))
            .collect::<Vec<_>>();
        obj.insert("dependencies".into(), Value::Array(dependencies.into()));

        if let Some(result) = &self.result {
            obj.insert("result".into(), result.clone());
        }

        if let Some(error) = &self.error {
            obj.insert("error".into(), Value::Strand(error.clone().into()));
        }

        let mut metadata_obj = Object::default();
        for (key, value) in &self.metadata {
            metadata_obj.insert(key.clone().into(), value.clone());
        }
        obj.insert("metadata".into(), Value::Object(metadata_obj));

        Value::Object(obj)
    }

    /// Create from Value
    pub fn from_value(value: &Value) -> Result<Self> {
        if let Value::Object(obj) = value {
            let id = if let Some(Value::Strand(s)) = obj.get("id") {
                s.to_string()
            } else {
                return Err(GraphError::ConversionError(
                    "Missing id in step".to_string(),
                ));
            };

            let name = if let Some(Value::Strand(s)) = obj.get("name") {
                s.to_string()
            } else {
                return Err(GraphError::ConversionError(
                    "Missing name in step".to_string(),
                ));
            };

            let description = if let Some(Value::Strand(s)) = obj.get("description") {
                s.to_string()
            } else {
                return Err(GraphError::ConversionError(
                    "Missing description in step".to_string(),
                ));
            };

            let order = if let Some(Value::Number(n)) = obj.get("order") {
                n.as_int() as u32
            } else {
                return Err(GraphError::ConversionError(
                    "Missing order in step".to_string(),
                ));
            };

            let action = if let Some(a) = obj.get("action") {
                a.clone()
            } else {
                return Err(GraphError::ConversionError(
                    "Missing action in step".to_string(),
                ));
            };

            let status = if let Some(Value::Strand(s)) = obj.get("status") {
                StepStatus::from_str(&s.to_string())?
            } else {
                StepStatus::Pending
            };

            let mut conditions = Vec::new();
            if let Some(Value::Array(arr)) = obj.get("conditions") {
                for value in arr.iter() {
                    conditions.push(Condition::from_value(value)?);
                }
            }

            let mut dependencies = Vec::new();
            if let Some(Value::Array(arr)) = obj.get("dependencies") {
                for value in arr.iter() {
                    if let Value::Strand(s) = value {
                        dependencies.push(s.to_string());
                    }
                }
            }

            let result = obj.get("result").cloned();

            let error = if let Some(Value::Strand(s)) = obj.get("error") {
                Some(s.to_string())
            } else {
                None
            };

            let mut metadata = HashMap::new();
            if let Some(Value::Object(meta_obj)) = obj.get("metadata") {
                for (key, value) in meta_obj.iter() {
                    metadata.insert(key.to_string(), value.clone());
                }
            }

            Ok(Self {
                id,
                name,
                description,
                order,
                action,
                status,
                conditions,
                dependencies,
                result,
                error,
                metadata,
            })
        } else {
            Err(GraphError::ConversionError(
                "Value is not an object".to_string(),
            ))
        }
    }
}

/// Procedural memory struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProceduralMemory {
    /// Base memory
    pub base: BaseMemory,

    /// Procedure name
    pub name: String,

    /// Procedure description
    pub description: String,

    /// Procedure steps
    pub steps: Vec<Step>,

    /// Procedure conditions
    pub conditions: Vec<Condition>,

    /// Current step index
    pub current_step: Option<usize>,

    /// Execution status
    pub status: StepStatus,

    /// Execution result
    pub result: Option<Value>,

    /// Execution error
    pub error: Option<String>,
}

impl ProceduralMemory {
    /// Create a new procedural memory
    pub fn new(id: &str, name: &str, description: &str) -> Self {
        let content = MemoryContent::json(serde_json::Value::Object(serde_json::Map::new()));
        let base = BaseMemory::new(
            id,
            "Procedural Memory",
            "Auto-generated procedural memory",
            MemoryTypeEnum::Procedural,
            content,
        );

        Self {
            base,
            name: name.to_string(),
            description: description.to_string(),
            steps: Vec::new(),
            conditions: Vec::new(),
            current_step: None,
            status: StepStatus::Pending,
            result: None,
            error: None,
        }
    }

    /// Add a step
    pub fn add_step(&mut self, step: Step) {
        self.steps.push(step);
        // Sort steps by order
        self.steps.sort_by_key(|s| s.order);
    }

    /// Add a condition
    pub fn add_condition(&mut self, condition: Condition) {
        self.conditions.push(condition);
    }

    /// Get step by ID
    pub fn get_step(&self, id: &str) -> Option<&Step> {
        self.steps.iter().find(|s| s.id == id)
    }

    /// Get mutable step by ID
    pub fn get_step_mut(&mut self, id: &str) -> Option<&mut Step> {
        self.steps.iter_mut().find(|s| s.id == id)
    }

    /// Get condition by ID
    pub fn get_condition(&self, id: &str) -> Option<&Condition> {
        self.conditions.iter().find(|c| c.id == id)
    }

    /// Get mutable condition by ID
    pub fn get_condition_mut(&mut self, id: &str) -> Option<&mut Condition> {
        self.conditions.iter_mut().find(|c| c.id == id)
    }

    /// Get current step
    pub fn current_step(&self) -> Option<&Step> {
        self.current_step.and_then(|i| self.steps.get(i))
    }

    /// Get next step
    pub fn next_step(&self) -> Option<&Step> {
        match self.current_step {
            Some(i) if i + 1 < self.steps.len() => self.steps.get(i + 1),
            None if !self.steps.is_empty() => self.steps.first(),
            _ => None,
        }
    }
}
