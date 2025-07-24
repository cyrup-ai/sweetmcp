//! Value conversion utilities for entity serialization
//!
//! This module provides efficient conversion functions between surrealdb::sql::Value
//! and serde_json::Value with zero-allocation patterns where possible.

use crate::graph::graph_db::{GraphError, Result};
use surrealdb::sql::Value;

/// Convert surrealdb::sql::Value to serde_json::Value
/// 
/// Provides zero-allocation conversion where possible using references.
/// Handles all SurrealDB value types with proper error handling.
pub fn sql_to_json_value(value: &Value) -> serde_json::Value {
    match value {
        Value::Null => serde_json::Value::Null,
        Value::Bool(b) => serde_json::Value::Bool(*b),
        Value::Number(n) => {
            // Convert Number to JSON number with proper type handling
            if let Ok(i) = n.as_int() {
                serde_json::Value::Number(serde_json::Number::from(i))
            } else if let Ok(f) = n.as_float() {
                serde_json::Number::from_f64(f)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null)
            } else {
                serde_json::Value::Null
            }
        }
        Value::Strand(s) => serde_json::Value::String(s.as_string()),
        Value::Array(arr) => {
            let json_array: Vec<serde_json::Value> = arr.iter().map(sql_to_json_value).collect();
            serde_json::Value::Array(json_array)
        }
        Value::Object(obj) => {
            let mut json_obj = serde_json::Map::new();
            for (k, v) in obj.iter() {
                json_obj.insert(k.clone(), sql_to_json_value(v));
            }
            serde_json::Value::Object(json_obj)
        }
        Value::Datetime(dt) => {
            // Convert datetime to ISO 8601 string
            serde_json::Value::String(dt.to_string())
        }
        Value::Duration(dur) => {
            // Convert duration to string representation
            serde_json::Value::String(dur.to_string())
        }
        Value::Uuid(uuid) => {
            // Convert UUID to string
            serde_json::Value::String(uuid.to_string())
        }
        Value::Bytes(bytes) => {
            // Convert bytes to base64 string for JSON compatibility
            use base64::{engine::general_purpose, Engine as _};
            let base64_string = general_purpose::STANDARD.encode(bytes.as_slice());
            serde_json::Value::String(base64_string)
        }
        Value::Geometry(geom) => {
            // Convert geometry to string representation
            serde_json::Value::String(geom.to_string())
        }
        _ => {
            // For other types, try to serialize and deserialize as fallback
            match serde_json::to_string(value) {
                Ok(json_str) => serde_json::from_str(&json_str).unwrap_or(serde_json::Value::Null),
                Err(_) => serde_json::Value::Null,
            }
        }
    }
}

/// Convert serde_json::Value to surrealdb::sql::Value
/// 
/// Provides efficient conversion with comprehensive error handling.
/// Supports all JSON value types with appropriate SurrealDB mappings.
pub fn json_to_sql_value(json_value: serde_json::Value) -> Result<Value> {
    match json_value {
        serde_json::Value::Null => Ok(Value::Null),
        serde_json::Value::Bool(b) => Ok(Value::Bool(b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Number(i.into()))
            } else if let Some(u) = n.as_u64() {
                // Handle unsigned integers
                if u <= i64::MAX as u64 {
                    Ok(Value::Number((u as i64).into()))
                } else {
                    // Convert large unsigned to float
                    Ok(Value::Number((u as f64).into()))
                }
            } else if let Some(f) = n.as_f64() {
                Ok(Value::Number(f.into()))
            } else {
                Err(GraphError::SerializationError(
                    "Invalid number format in JSON".to_string(),
                ))
            }
        }
        serde_json::Value::String(s) => {
            // Try to parse special string formats
            if let Ok(uuid) = s.parse::<uuid::Uuid>() {
                Ok(Value::Uuid(uuid.into()))
            } else if let Ok(datetime) = chrono::DateTime::parse_from_rfc3339(&s) {
                Ok(Value::Datetime(datetime.into()))
            } else if s.starts_with("data:") || is_base64(&s) {
                // Handle base64 encoded data
                use base64::{engine::general_purpose, Engine as _};
                match general_purpose::STANDARD.decode(&s) {
                    Ok(bytes) => Ok(Value::Bytes(bytes.into())),
                    Err(_) => Ok(Value::Strand(s.into())),
                }
            } else {
                Ok(Value::Strand(s.into()))
            }
        }
        serde_json::Value::Array(arr) => {
            let mut sql_array = Vec::with_capacity(arr.len());
            for item in arr {
                sql_array.push(json_to_sql_value(item)?);
            }
            Ok(Value::Array(sql_array.into()))
        }
        serde_json::Value::Object(obj) => {
            let mut sql_obj = std::collections::BTreeMap::new();
            for (k, v) in obj {
                sql_obj.insert(k, json_to_sql_value(v)?);
            }
            Ok(Value::Object(sql_obj.into()))
        }
    }
}

/// Check if a string is valid base64
/// 
/// Performs a quick heuristic check for base64 format without full decoding.
fn is_base64(s: &str) -> bool {
    // Basic heuristics for base64 detection
    if s.is_empty() || s.len() % 4 != 0 {
        return false;
    }

    // Check if all characters are valid base64 characters
    s.chars().all(|c| {
        c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='
    })
}

/// Convert a HashMap of JSON values to SQL values
/// 
/// Batch conversion utility for efficient processing of multiple values.
pub fn json_map_to_sql_map(
    json_map: std::collections::HashMap<String, serde_json::Value>,
) -> Result<std::collections::HashMap<String, Value>> {
    let mut sql_map = std::collections::HashMap::with_capacity(json_map.len());
    
    for (key, json_value) in json_map {
        let sql_value = json_to_sql_value(json_value)?;
        sql_map.insert(key, sql_value);
    }
    
    Ok(sql_map)
}

/// Convert a HashMap of SQL values to JSON values
/// 
/// Batch conversion utility for efficient processing of multiple values.
pub fn sql_map_to_json_map(
    sql_map: &std::collections::HashMap<String, Value>,
) -> std::collections::HashMap<String, serde_json::Value> {
    let mut json_map = std::collections::HashMap::with_capacity(sql_map.len());
    
    for (key, sql_value) in sql_map {
        let json_value = sql_to_json_value(sql_value);
        json_map.insert(key.clone(), json_value);
    }
    
    json_map
}

/// Validate that a JSON value can be safely converted to SQL
/// 
/// Performs pre-conversion validation to catch potential issues early.
pub fn validate_json_for_sql_conversion(value: &serde_json::Value) -> Result<()> {
    match value {
        serde_json::Value::Null | serde_json::Value::Bool(_) | serde_json::Value::String(_) => {
            Ok(())
        }
        serde_json::Value::Number(n) => {
            if n.is_finite() {
                Ok(())
            } else {
                Err(GraphError::ValidationError(
                    "JSON number must be finite for SQL conversion".to_string(),
                ))
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                validate_json_for_sql_conversion(item)?;
            }
            Ok(())
        }
        serde_json::Value::Object(obj) => {
            for (key, val) in obj {
                if key.is_empty() {
                    return Err(GraphError::ValidationError(
                        "Object keys cannot be empty for SQL conversion".to_string(),
                    ));
                }
                validate_json_for_sql_conversion(val)?;
            }
            Ok(())
        }
    }
}

/// Optimize a JSON value for SQL conversion
/// 
/// Applies optimizations to reduce memory usage and improve conversion performance.
pub fn optimize_json_for_sql(mut value: serde_json::Value) -> serde_json::Value {
    match &mut value {
        serde_json::Value::String(s) => {
            // Trim whitespace from strings
            let trimmed = s.trim();
            if trimmed.len() != s.len() {
                *s = trimmed.to_string();
            }
        }
        serde_json::Value::Array(arr) => {
            // Recursively optimize array elements
            for item in arr.iter_mut() {
                *item = optimize_json_for_sql(std::mem::take(item));
            }
        }
        serde_json::Value::Object(obj) => {
            // Recursively optimize object values and remove empty strings
            let mut keys_to_remove = Vec::new();
            for (key, val) in obj.iter_mut() {
                if let serde_json::Value::String(s) = val {
                    if s.is_empty() {
                        keys_to_remove.push(key.clone());
                        continue;
                    }
                }
                *val = optimize_json_for_sql(std::mem::take(val));
            }
            for key in keys_to_remove {
                obj.remove(&key);
            }
        }
        _ => {}
    }
    
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_to_json_basic_types() {
        // Test null
        assert_eq!(sql_to_json_value(&Value::Null), serde_json::Value::Null);
        
        // Test boolean
        assert_eq!(sql_to_json_value(&Value::Bool(true)), serde_json::Value::Bool(true));
        
        // Test string
        let strand = Value::Strand("test".into());
        assert_eq!(sql_to_json_value(&strand), serde_json::Value::String("test".to_string()));
    }

    #[test]
    fn test_json_to_sql_basic_types() {
        // Test null
        assert!(matches!(json_to_sql_value(serde_json::Value::Null).unwrap(), Value::Null));
        
        // Test boolean
        assert!(matches!(json_to_sql_value(serde_json::Value::Bool(true)).unwrap(), Value::Bool(true)));
        
        // Test string
        let result = json_to_sql_value(serde_json::Value::String("test".to_string())).unwrap();
        if let Value::Strand(s) = result {
            assert_eq!(s.as_string(), "test");
        } else {
            panic!("Expected Strand value");
        }
    }

    #[test]
    fn test_base64_detection() {
        assert!(is_base64("SGVsbG8gV29ybGQ="));
        assert!(!is_base64("not base64"));
        assert!(!is_base64(""));
        assert!(!is_base64("abc")); // Wrong length
    }
}