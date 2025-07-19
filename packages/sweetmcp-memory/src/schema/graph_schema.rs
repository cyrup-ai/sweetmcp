//! Graph database schema definitions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Graph node schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    /// Node ID
    pub id: String,

    /// Node type
    pub node_type: String,

    /// Node properties
    pub properties: HashMap<String, serde_json::Value>,

    /// Labels/tags
    pub labels: Vec<String>,

    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Update timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Graph edge schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    /// Edge ID
    pub id: String,

    /// Source node ID
    pub source_id: String,

    /// Target node ID
    pub target_id: String,

    /// Edge type/label
    pub edge_type: String,

    /// Edge properties
    pub properties: HashMap<String, serde_json::Value>,

    /// Edge weight
    pub weight: Option<f32>,

    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Graph schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphSchema {
    /// Schema version
    pub version: String,

    /// Node type definitions
    pub node_types: HashMap<String, NodeTypeDefinition>,

    /// Edge type definitions
    pub edge_types: HashMap<String, EdgeTypeDefinition>,

    /// Constraints
    pub constraints: Vec<GraphConstraint>,
}

/// Node type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeTypeDefinition {
    /// Type name
    pub name: String,

    /// Required properties
    pub required_properties: Vec<PropertyDefinition>,

    /// Optional properties
    pub optional_properties: Vec<PropertyDefinition>,

    /// Allowed labels
    pub allowed_labels: Vec<String>,
}

/// Edge type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeTypeDefinition {
    /// Type name
    pub name: String,

    /// Allowed source node types
    pub source_types: Vec<String>,

    /// Allowed target node types
    pub target_types: Vec<String>,

    /// Required properties
    pub required_properties: Vec<PropertyDefinition>,

    /// Optional properties
    pub optional_properties: Vec<PropertyDefinition>,

    /// Whether the edge is directed
    pub directed: bool,
}

/// Property definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyDefinition {
    /// Property name
    pub name: String,

    /// Property type
    pub property_type: PropertyType,

    /// Default value
    pub default_value: Option<serde_json::Value>,

    /// Validation rules
    pub validation: Option<PropertyValidation>,
}

/// Property types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyType {
    String,
    Integer,
    Float,
    Boolean,
    DateTime,
    UUID,
    Array(Box<PropertyType>),
    Object,
}

/// Property validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyValidation {
    /// Minimum value (for numeric types)
    pub min: Option<f64>,

    /// Maximum value (for numeric types)
    pub max: Option<f64>,

    /// Pattern (for string types)
    pub pattern: Option<String>,

    /// Allowed values
    pub enum_values: Option<Vec<serde_json::Value>>,

    /// Minimum length (for strings and arrays)
    pub min_length: Option<usize>,

    /// Maximum length (for strings and arrays)
    pub max_length: Option<usize>,
}

/// Graph constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphConstraint {
    /// Unique constraint on node property
    UniqueNode { node_type: String, property: String },

    /// Unique constraint on edge
    UniqueEdge {
        edge_type: String,
        source_property: Option<String>,
        target_property: Option<String>,
    },

    /// Cardinality constraint
    Cardinality {
        edge_type: String,
        source_type: String,
        target_type: String,
        min: Option<usize>,
        max: Option<usize>,
    },
}

impl GraphSchema {
    /// Create a new graph schema
    pub fn new(version: String) -> Self {
        Self {
            version,
            node_types: HashMap::new(),
            edge_types: HashMap::new(),
            constraints: Vec::new(),
        }
    }

    /// Add a node type definition
    pub fn add_node_type(&mut self, definition: NodeTypeDefinition) {
        self.node_types.insert(definition.name.clone(), definition);
    }

    /// Add an edge type definition
    pub fn add_edge_type(&mut self, definition: EdgeTypeDefinition) {
        self.edge_types.insert(definition.name.clone(), definition);
    }

    /// Add a constraint
    pub fn add_constraint(&mut self, constraint: GraphConstraint) {
        self.constraints.push(constraint);
    }

    /// Validate a node against the schema
    pub fn validate_node(&self, node: &GraphNode) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if let Some(type_def) = self.node_types.get(&node.node_type) {
            // Check required properties
            for prop_def in &type_def.required_properties {
                if !node.properties.contains_key(&prop_def.name) {
                    errors.push(format!(
                        "Missing required property '{}' for node type '{}'",
                        prop_def.name, node.node_type
                    ));
                }
            }

            // Check labels
            for label in &node.labels {
                if !type_def.allowed_labels.contains(label) {
                    errors.push(format!(
                        "Label '{}' not allowed for node type '{}'",
                        label, node.node_type
                    ));
                }
            }
        } else {
            errors.push(format!("Unknown node type '{}'", node.node_type));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate an edge against the schema
    pub fn validate_edge(
        &self,
        edge: &GraphEdge,
        source_node: &GraphNode,
        target_node: &GraphNode,
    ) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if let Some(type_def) = self.edge_types.get(&edge.edge_type) {
            // Check source and target types
            if !type_def.source_types.contains(&source_node.node_type) {
                errors.push(format!(
                    "Source node type '{}' not allowed for edge type '{}'",
                    source_node.node_type, edge.edge_type
                ));
            }

            if !type_def.target_types.contains(&target_node.node_type) {
                errors.push(format!(
                    "Target node type '{}' not allowed for edge type '{}'",
                    target_node.node_type, edge.edge_type
                ));
            }

            // Check required properties
            for prop_def in &type_def.required_properties {
                if !edge.properties.contains_key(&prop_def.name) {
                    errors.push(format!(
                        "Missing required property '{}' for edge type '{}'",
                        prop_def.name, edge.edge_type
                    ));
                }
            }
        } else {
            errors.push(format!("Unknown edge type '{}'", edge.edge_type));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
