// src/migration/converter.rs
//! Data conversion utilities for import/export.

use serde_json;
use std::collections::HashMap;
use std::sync::Arc;

use crate::migration::Result;

/// Import data structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImportData {
    pub version: String,
    pub metadata: ImportMetadata,
    pub data: serde_json::Value,
    /// Memory records
    pub memories: Vec<serde_json::Value>,
    /// Relationship records
    pub relationships: Vec<serde_json::Value>,
}

/// Import metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImportMetadata {
    pub source: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub format: String,
    /// Format version
    pub format_version: String,
}

/// Data converter for migrating between different format versions
pub struct DataConverter {
    /// Source version
    source_version: String,
    /// Target version
    target_version: String,
    /// Custom conversion rules
    custom_rules:
        HashMap<(String, String), Arc<dyn Fn(&ImportData) -> Result<ImportData> + Send + Sync>>,
}

impl DataConverter {
    /// Create a new data converter
    pub fn new(source_version: impl Into<String>, target_version: impl Into<String>) -> Self {
        Self {
            source_version: source_version.into(),
            target_version: target_version.into(),
            custom_rules: HashMap::new(),
        }
    }

    /// Convert data from source version to target version
    pub fn convert(&self, data: &ImportData) -> Result<ImportData> {
        // Check if we have a custom rule for this conversion
        let key = (self.source_version.clone(), self.target_version.clone());
        if let Some(rule) = self.custom_rules.get(&key) {
            return rule(data);
        }

        // Otherwise, use built-in conversion logic
        match (self.source_version.as_str(), self.target_version.as_str()) {
            ("0.1.0", "0.2.0") => self.convert_0_1_0_to_0_2_0(data),
            ("0.2.0", "0.1.0") => self.convert_0_2_0_to_0_1_0(data),
            _ => self.apply_generic_upgrade(data),
        }
    }

    /// Convert from version 0.1.0 to 0.2.0
    fn convert_0_1_0_to_0_2_0(&self, data: &ImportData) -> Result<ImportData> {
        // Create a new ImportData with updated metadata
        let mut new_data = data.clone();
        new_data.metadata.format_version = "0.2.0".to_string();

        // Update memories
        new_data.memories = data
            .memories
            .iter()
            .map(|memory| {
                let mut new_memory = memory.clone();

                // Add any new fields or transform existing ones
                // For example, add a new field to metadata if it exists
                if let Some(serde_json::Value::Object(metadata_obj)) = new_memory.get("metadata") {
                    let mut new_metadata = metadata_obj.clone();
                    new_metadata.insert("schema_version".to_string(), serde_json::json!("0.2.0"));
                    if let serde_json::Value::Object(obj) = &mut new_memory {
                        obj.insert(
                            "metadata".to_string(),
                            serde_json::Value::Object(new_metadata),
                        );
                    }
                } else {
                    // Create new metadata object if none exists
                    let mut new_metadata = serde_json::Map::new();
                    new_metadata.insert("schema_version".to_string(), serde_json::json!("0.2.0"));
                    if let serde_json::Value::Object(obj) = &mut new_memory {
                        obj.insert(
                            "metadata".to_string(),
                            serde_json::Value::Object(new_metadata),
                        );
                    }
                }

                new_memory
            })
            .collect();

        // Update relationships
        new_data.relationships = data
            .relationships
            .iter()
            .map(|relationship| {
                let mut new_relationship = relationship.clone();

                // Update relationship metadata with schema version
                if let Some(serde_json::Value::Object(metadata_obj)) =
                    new_relationship.get("metadata")
                {
                    // Process metadata fields
                    let mut new_metadata = metadata_obj.clone();
                    new_metadata.insert(
                        "schema_version".to_string(),
                        serde_json::json!(self.target_version),
                    );

                    // Update relationship with new metadata
                    if let serde_json::Value::Object(obj) = &mut new_relationship {
                        obj.insert(
                            "metadata".to_string(),
                            serde_json::Value::Object(new_metadata),
                        );
                    }
                } else {
                    // Create new metadata object if none exists
                    let mut new_metadata = serde_json::Map::new();
                    new_metadata.insert(
                        "schema_version".to_string(),
                        serde_json::json!(self.target_version),
                    );
                    if let serde_json::Value::Object(obj) = &mut new_relationship {
                        obj.insert(
                            "metadata".to_string(),
                            serde_json::Value::Object(new_metadata),
                        );
                    }
                }

                new_relationship
            })
            .collect();

        Ok(new_data)
    }

    /// Convert from version 0.2.0 to 0.1.0
    fn convert_0_2_0_to_0_1_0(&self, data: &ImportData) -> Result<ImportData> {
        // Create a new ImportData with updated metadata
        let mut new_data = data.clone();
        new_data.metadata.format_version = "0.1.0".to_string();

        // Update memories
        new_data.memories = data
            .memories
            .iter()
            .map(|memory| {
                let mut new_memory = memory.clone();

                // Remove or transform fields
                if let Some(serde_json::Value::Object(metadata_obj)) = new_memory.get("metadata") {
                    let mut new_metadata = metadata_obj.clone();
                    // Remove schema version and other fields not compatible with older versions
                    new_metadata.remove("schema_version");
                    new_metadata.remove("advanced_features");
                    new_metadata.remove("version_specific_data");

                    if let serde_json::Value::Object(obj) = &mut new_memory {
                        obj.insert(
                            "metadata".to_string(),
                            serde_json::Value::Object(new_metadata),
                        );
                    }
                }

                new_memory
            })
            .collect();

        // Update relationships
        new_data.relationships = data
            .relationships
            .iter()
            .map(|relationship| {
                let mut new_relationship = relationship.clone();

                // Remove or transform fields
                if let Some(serde_json::Value::Object(metadata_obj)) =
                    new_relationship.get("metadata")
                {
                    let mut new_metadata = metadata_obj.clone();
                    // Remove schema version and other fields not compatible with older versions
                    new_metadata.remove("schema_version");

                    // Remove any fields that might not be compatible with older versions
                    new_metadata.remove("advanced_features");

                    // Remove version specific data
                    new_metadata.remove("version_specific_data");

                    if let serde_json::Value::Object(obj) = &mut new_relationship {
                        obj.insert(
                            "metadata".to_string(),
                            serde_json::Value::Object(new_metadata),
                        );
                    }
                }

                new_relationship
            })
            .collect();

        Ok(new_data)
    }

    /// Apply generic upgrade
    fn apply_generic_upgrade(&self, data: &ImportData) -> Result<ImportData> {
        // Create a new ImportData with updated metadata
        let mut new_data = data.clone();
        new_data.metadata.format_version = self.target_version.clone();

        // For generic upgrades, we keep all existing data and add version info to metadata
        new_data.memories = data
            .memories
            .iter()
            .map(|memory| {
                let mut new_memory = memory.clone();

                // Update metadata with schema version
                if let Some(serde_json::Value::Object(metadata_obj)) = new_memory.get("metadata") {
                    let mut new_metadata = metadata_obj.clone();
                    new_metadata.insert(
                        "schema_version".to_string(),
                        serde_json::json!(self.target_version),
                    );
                    if let serde_json::Value::Object(obj) = &mut new_memory {
                        obj.insert(
                            "metadata".to_string(),
                            serde_json::Value::Object(new_metadata),
                        );
                    }
                } else {
                    // If metadata is not an object, create a new object
                    let mut new_metadata = serde_json::Map::new();
                    new_metadata.insert(
                        "schema_version".to_string(),
                        serde_json::json!(self.target_version),
                    );
                    if let serde_json::Value::Object(obj) = &mut new_memory {
                        obj.insert(
                            "metadata".to_string(),
                            serde_json::Value::Object(new_metadata),
                        );
                    }
                }

                new_memory
            })
            .collect();

        // Update relationships
        new_data.relationships = data
            .relationships
            .iter()
            .map(|relationship| {
                let mut new_relationship = relationship.clone();

                // Update relationship metadata with schema version
                if let Some(serde_json::Value::Object(metadata_obj)) =
                    new_relationship.get("metadata")
                {
                    // Process metadata fields
                    let mut new_metadata = metadata_obj.clone();
                    new_metadata.insert(
                        "schema_version".to_string(),
                        serde_json::json!(self.target_version),
                    );

                    // Update relationship with new metadata
                    if let serde_json::Value::Object(obj) = &mut new_relationship {
                        obj.insert(
                            "metadata".to_string(),
                            serde_json::Value::Object(new_metadata),
                        );
                    }
                } else {
                    // Create new metadata object if none exists
                    let mut new_metadata = serde_json::Map::new();
                    new_metadata.insert(
                        "schema_version".to_string(),
                        serde_json::json!(self.target_version),
                    );
                    if let serde_json::Value::Object(obj) = &mut new_relationship {
                        obj.insert(
                            "metadata".to_string(),
                            serde_json::Value::Object(new_metadata),
                        );
                    }
                }

                new_relationship
            })
            .collect();

        Ok(new_data)
    }

    /// Add custom conversion rule
    pub fn add_custom_rule<F>(
        &mut self,
        source: impl Into<String>,
        target: impl Into<String>,
        rule: F,
    ) where
        F: Fn(&ImportData) -> Result<ImportData> + Send + Sync + 'static,
    {
        let key = (source.into(), target.into());
        self.custom_rules.insert(key, Arc::new(rule));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_0_1_0_to_0_2_0() {
        let converter = DataConverter::new("0.1.0", "0.2.0");

        let metadata = ImportMetadata {
            source: "test".to_string(),
            timestamp: chrono::Utc::now(),
            format: "memory_export".to_string(),
            format_version: "0.1.0".to_string(),
        };

        let mut data = ImportData {
            version: "0.1.0".to_string(),
            metadata,
            data: serde_json::Value::Object(serde_json::Map::new()),
            memories: Vec::new(),
            relationships: Vec::new(),
        };

        // Add a test memory
        let memory = crate::schema::memory_schema::Memory {
            id: "test-memory".to_string(),
            content: "Test".to_string(),
            memory_type: "semantic".to_string(),
            created_at: chrono::Utc::now(),
            last_accessed_at: chrono::Utc::now(),
            importance: 0.5,
            tags: vec![],
            metadata: Some(serde_json::Value::Object(serde_json::Map::new())),
        };
        data.memories.push(serde_json::to_value(memory).unwrap());

        // Add a test relationship
        let relationship =
            crate::schema::relationship_schema::Relationship::new("source", "target", "related_to");
        data.relationships
            .push(serde_json::to_value(relationship).unwrap());

        // Convert the data
        let result = converter.convert(&data).unwrap();

        // Check that the version was updated
        assert_eq!(result.metadata.format_version, "0.2.0");

        // Check that memories were updated
        assert!(!result.memories.is_empty());
        let memory = &result.memories[0];
        if let serde_json::Value::Object(memory_obj) = memory {
            if let Some(serde_json::Value::Object(metadata)) = memory_obj.get("metadata") {
                assert_eq!(
                    metadata.get("schema_version").unwrap(),
                    &serde_json::json!("0.2.0")
                );
            } else {
                panic!("Expected memory metadata to be an object");
            }
        } else {
            panic!("Expected memory to be an object");
        }

        // Check that relationships were updated
        assert!(!result.relationships.is_empty());
        let relationship = &result.relationships[0];
        if let serde_json::Value::Object(rel_obj) = relationship {
            if let Some(serde_json::Value::Object(metadata)) = rel_obj.get("metadata") {
                assert_eq!(
                    metadata.get("schema_version").unwrap(),
                    &serde_json::json!("0.2.0")
                );
            } else {
                panic!("Expected relationship metadata to be an object");
            }
        } else {
            panic!("Expected relationship to be an object");
        }
    }

    #[test]
    fn test_convert_0_2_0_to_0_1_0() {
        let converter = DataConverter::new("0.2.0", "0.1.0");

        let metadata = ImportMetadata {
            source: "test".to_string(),
            timestamp: chrono::Utc::now(),
            format: "memory_export".to_string(),
            format_version: "0.2.0".to_string(),
        };

        let mut data = ImportData {
            version: "0.2.0".to_string(),
            metadata,
            data: serde_json::Value::Object(serde_json::Map::new()),
            memories: Vec::new(),
            relationships: Vec::new(),
        };

        // Add a test memory with schema version
        let mut memory_metadata = serde_json::Map::new();
        memory_metadata.insert("schema_version".to_string(), serde_json::json!("0.2.0"));
        memory_metadata.insert("advanced_features".to_string(), serde_json::json!(true));

        let memory = crate::schema::memory_schema::Memory {
            id: "test-memory".to_string(),
            content: "Test".to_string(),
            memory_type: "semantic".to_string(),
            created_at: chrono::Utc::now(),
            last_accessed_at: chrono::Utc::now(),
            importance: 0.5,
            tags: vec![],
            metadata: Some(serde_json::Value::Object(memory_metadata)),
        };
        data.memories.push(serde_json::to_value(memory).unwrap());

        // Add a test relationship with schema version
        let mut relationship =
            crate::schema::relationship_schema::Relationship::new("source", "target", "related_to");
        let mut rel_metadata = serde_json::Map::new();
        rel_metadata.insert("schema_version".to_string(), serde_json::json!("0.2.0"));
        rel_metadata.insert("advanced_features".to_string(), serde_json::json!(true));
        relationship.metadata = serde_json::Value::Object(rel_metadata);
        data.relationships
            .push(serde_json::to_value(relationship).unwrap());

        // Convert the data
        let result = converter.convert(&data).unwrap();

        // Check that the version was updated
        assert_eq!(result.metadata.format_version, "0.1.0");

        // Check that memories were updated
        assert!(!result.memories.is_empty());
        let memory = &result.memories[0];
        if let serde_json::Value::Object(memory_obj) = memory {
            if let Some(serde_json::Value::Object(metadata)) = memory_obj.get("metadata") {
                assert!(metadata.get("schema_version").is_none());
                assert!(metadata.get("advanced_features").is_none());
            } else {
                panic!("Expected memory metadata to be an object");
            }
        } else {
            panic!("Expected memory to be an object");
        }

        // Check that relationships were updated
        assert!(!result.relationships.is_empty());
        let relationship = &result.relationships[0];
        if let serde_json::Value::Object(rel_obj) = relationship {
            if let Some(serde_json::Value::Object(metadata)) = rel_obj.get("metadata") {
                assert!(metadata.get("schema_version").is_none());
                assert!(metadata.get("advanced_features").is_none());
            } else {
                panic!("Expected relationship metadata to be an object");
            }
        } else {
            panic!("Expected relationship to be an object");
        }
    }

    #[test]
    fn test_custom_conversion_rule() {
        let mut converter = DataConverter::new("custom", "target");

        converter.add_custom_rule("custom", "target", |data| {
            let mut new_data = data.clone();
            new_data.metadata.format_version = "custom-converted".to_string();
            Ok(new_data)
        });

        let metadata = ImportMetadata {
            source: "test".to_string(),
            timestamp: chrono::Utc::now(),
            format: "memory_export".to_string(),
            format_version: "custom".to_string(),
        };

        let data = ImportData {
            version: "custom".to_string(),
            metadata,
            data: serde_json::Value::Object(serde_json::Map::new()),
            memories: Vec::new(),
            relationships: Vec::new(),
        };

        // Convert using custom rule
        let result = converter.convert(&data).unwrap();

        // Check that the custom rule was applied
        assert_eq!(result.metadata.format_version, "custom-converted");
    }
}
