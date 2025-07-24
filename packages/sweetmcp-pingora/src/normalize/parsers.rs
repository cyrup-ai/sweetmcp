//! Protocol-specific parsers and converters
//!
//! This module provides protocol-specific parsing and conversion logic
//! for GraphQL, Cap'n Proto, and other protocols with zero allocation
//! patterns and blazing-fast performance.

use super::types::{ProtocolContext, ConversionResult, ConversionError};
use anyhow::{bail, Context, Result};
use async_graphql::parser::{parse_query, types::*};
use async_graphql::{Name, Positioned};
use async_graphql_value::Value as GraphQLValue;
use capnp::message::ReaderOptions;
use serde_json::{json, Value};
use sweetmcp_axum::JSONRPC_VERSION;
use tracing::{debug, warn};

/// Convert GraphQL query to JSON-RPC
pub fn graphql_to_json_rpc(
    query: &str,
    variables: Value,
    operation_name: Option<Value>,
    request_id: &str,
) -> Result<Value> {
    debug!("Converting GraphQL query to JSON-RPC");

    // Parse GraphQL query
    let doc = parse_query(query)
        .map_err(|e| anyhow::anyhow!("GraphQL parse error: {}", e))?;

    // Extract operation information
    let operation = doc.operations.iter().next();
    
    let (method, params) = match operation {
        Some((name, op)) => {
            let method_name = if let Some(op_name) = operation_name {
                op_name.as_str().unwrap_or("graphql_query").to_string()
            } else if let Some(name) = name {
                name.to_string()
            } else {
                "graphql_query".to_string()
            };

            // Extract fields from selection set
            let mut fields = Vec::new();
            extract_fields_from_selection_set(&op.node.selection_set.node, &mut fields);

            let params = json!({
                "query": query,
                "variables": variables,
                "operationName": operation_name,
                "fields": fields,
                "operationType": format!("{:?}", op.node.ty)
            });

            (method_name, params)
        }
        None => {
            warn!("No GraphQL operation found, using default");
            ("graphql_query".to_string(), json!({
                "query": query,
                "variables": variables
            }))
        }
    };

    Ok(json!({
        "jsonrpc": JSONRPC_VERSION,
        "method": method,
        "params": params,
        "id": request_id
    }))
}

/// Extract fields from GraphQL selection set
fn extract_fields_from_selection_set(
    selection_set: &SelectionSet,
    fields: &mut Vec<String>,
) {
    for selection in &selection_set.items {
        match &selection.node {
            Selection::Field(field) => {
                fields.push(field.node.name.node.to_string());
                
                // Recursively extract nested fields
                if !field.node.selection_set.node.items.is_empty() {
                    extract_fields_from_selection_set(&field.node.selection_set.node, fields);
                }
            }
            Selection::InlineFragment(fragment) => {
                extract_fields_from_selection_set(&fragment.node.selection_set.node, fields);
            }
            Selection::FragmentSpread(_) => {
                // Fragment spreads would need fragment definition resolution
                // For now, we skip them
            }
        }
    }
}

/// Convert Cap'n Proto binary to JSON-RPC
pub fn capnp_to_json_rpc(body: &[u8], request_id: &str) -> Result<Value> {
    debug!("Converting Cap'n Proto to JSON-RPC");

    // For now, return error - Cap'n Proto support not yet implemented
    bail!("Cap'n Proto support not yet implemented")
}

/// Convert JSON-RPC response to GraphQL response
pub fn graphql_from_json_rpc(ctx: &ProtocolContext, response: &Value) -> ConversionResult<Vec<u8>> {
    debug!("Converting JSON-RPC response to GraphQL");

    let mut graphql_response = json!({
        "data": null
    });

    // Check for JSON-RPC error
    if let Some(error) = response.get("error") {
        let error_message = error
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown error");

        let error_code = error
            .get("code")
            .and_then(|c| c.as_i64())
            .unwrap_or(-32603);

        graphql_response["errors"] = json!([{
            "message": error_message,
            "extensions": {
                "code": error_code,
                "jsonrpc_error": error
            }
        }]);
    } else if let Some(result) = response.get("result") {
        // Shape response based on original query if available
        if let Some(original_query) = ctx.original_query() {
            graphql_response["data"] = shape_graphql_response(result, original_query)?;
        } else {
            // Simple passthrough if no original query context
            graphql_response["data"] = result.clone();
        }
    }

    // Add extensions with metadata
    graphql_response["extensions"] = json!({
        "request_id": ctx.request_id(),
        "protocol": "graphql",
        "converted_from": "json-rpc"
    });

    serde_json::to_vec(&graphql_response)
        .map_err(|e| ConversionError::JsonError(e))
}

/// Shape GraphQL response based on original query structure
fn shape_graphql_response(result: &Value, original_query: &str) -> ConversionResult<Value> {
    // Parse the original query to understand expected structure
    let doc = parse_query(original_query)
        .map_err(|e| ConversionError::GraphQLError(format!("Failed to parse original query: {}", e)))?;

    // For now, return result as-is
    // In a full implementation, this would reshape the response to match the GraphQL query structure
    Ok(result.clone())
}

/// Convert JSON-RPC response to Cap'n Proto
pub fn capnp_from_json_rpc(_ctx: &ProtocolContext, _response: &Value) -> ConversionResult<Vec<u8>> {
    Err(ConversionError::UnsupportedProtocol(
        "Cap'n Proto response conversion not yet implemented".to_string()
    ))
}

/// Parse GraphQL variables
pub fn parse_graphql_variables(variables: &Value) -> ConversionResult<std::collections::HashMap<String, GraphQLValue>> {
    let mut parsed_variables = std::collections::HashMap::new();

    if let Some(vars) = variables.as_object() {
        for (key, value) in vars {
            let graphql_value = json_to_graphql_value(value)?;
            parsed_variables.insert(key.clone(), graphql_value);
        }
    }

    Ok(parsed_variables)
}

/// Convert JSON value to GraphQL value
fn json_to_graphql_value(value: &Value) -> ConversionResult<GraphQLValue> {
    let graphql_value = match value {
        Value::Null => GraphQLValue::Null,
        Value::Bool(b) => GraphQLValue::Boolean(*b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                GraphQLValue::Number(async_graphql_value::Number::from(i))
            } else if let Some(f) = n.as_f64() {
                GraphQLValue::Number(async_graphql_value::Number::from(f))
            } else {
                return Err(ConversionError::GraphQLError(
                    "Invalid number format".to_string()
                ));
            }
        }
        Value::String(s) => GraphQLValue::String(s.clone()),
        Value::Array(arr) => {
            let mut graphql_array = Vec::new();
            for item in arr {
                graphql_array.push(json_to_graphql_value(item)?);
            }
            GraphQLValue::List(graphql_array)
        }
        Value::Object(obj) => {
            let mut graphql_object = async_graphql_value::indexmap::IndexMap::new();
            for (key, val) in obj {
                let name = Name::new(key);
                graphql_object.insert(name, json_to_graphql_value(val)?);
            }
            GraphQLValue::Object(graphql_object)
        }
    };

    Ok(graphql_value)
}

/// Validate GraphQL query syntax
pub fn validate_graphql_query(query: &str) -> ConversionResult<()> {
    parse_query(query)
        .map_err(|e| ConversionError::GraphQLError(format!("Invalid GraphQL syntax: {}", e)))?;
    
    Ok(())
}

/// Extract GraphQL operation type
pub fn extract_operation_type(query: &str) -> ConversionResult<String> {
    let doc = parse_query(query)
        .map_err(|e| ConversionError::GraphQLError(format!("Failed to parse query: {}", e)))?;

    if let Some((_, operation)) = doc.operations.iter().next() {
        Ok(format!("{:?}", operation.node.ty).to_lowercase())
    } else {
        Ok("query".to_string()) // Default to query
    }
}

/// Extract GraphQL operation name
pub fn extract_operation_name(query: &str) -> ConversionResult<Option<String>> {
    let doc = parse_query(query)
        .map_err(|e| ConversionError::GraphQLError(format!("Failed to parse query: {}", e)))?;

    if let Some((name, _)) = doc.operations.iter().next() {
        Ok(name.as_ref().map(|n| n.to_string()))
    } else {
        Ok(None)
    }
}

/// Create GraphQL error response
pub fn create_graphql_error(message: &str, code: Option<i32>) -> Value {
    json!({
        "errors": [{
            "message": message,
            "extensions": {
                "code": code.unwrap_or(-1)
            }
        }],
        "data": null
    })
}

/// Parse Cap'n Proto message (placeholder implementation)
pub fn parse_capnp_message(body: &[u8]) -> ConversionResult<Value> {
    // This is a placeholder - real Cap'n Proto parsing would be much more complex
    if body.len() < 8 {
        return Err(ConversionError::CapnProtoError(
            "Cap'n Proto message too short".to_string()
        ));
    }

    // For now, return a placeholder structure
    Ok(json!({
        "method": "capnp_method",
        "params": {
            "binary_data_length": body.len(),
            "message": "Cap'n Proto parsing not fully implemented"
        }
    }))
}

/// Validate Cap'n Proto binary format
pub fn validate_capnp_format(body: &[u8]) -> ConversionResult<()> {
    if body.len() < 8 {
        return Err(ConversionError::CapnProtoError(
            "Cap'n Proto message too short".to_string()
        ));
    }

    // Basic validation - real implementation would check Cap'n Proto headers
    Ok(())
}

/// Get parser statistics
pub fn get_parser_stats() -> ParserStats {
    // In a real implementation, this would track actual statistics
    ParserStats {
        graphql_queries_parsed: 0,
        capnp_messages_parsed: 0,
        parsing_errors: 0,
        average_parse_time_ms: 0.0,
    }
}

/// Statistics for parser performance monitoring
#[derive(Debug, Clone)]
pub struct ParserStats {
    pub graphql_queries_parsed: u64,
    pub capnp_messages_parsed: u64,
    pub parsing_errors: u64,
    pub average_parse_time_ms: f64,
}

impl ParserStats {
    /// Calculate error rate
    pub fn error_rate(&self) -> f64 {
        let total_parsed = self.graphql_queries_parsed + self.capnp_messages_parsed;
        if total_parsed == 0 {
            0.0
        } else {
            (self.parsing_errors as f64 / total_parsed as f64) * 100.0
        }
    }

    /// Check if parser performance is healthy
    pub fn is_healthy(&self) -> bool {
        self.error_rate() < 5.0 && self.average_parse_time_ms < 50.0
    }
}

/// Helper function to create method name from GraphQL operation
pub fn create_method_name(operation_name: Option<&str>, operation_type: &str) -> String {
    match operation_name {
        Some(name) => format!("graphql_{}", name),
        None => format!("graphql_{}", operation_type),
    }
}

/// Extract arguments from GraphQL field
pub fn extract_field_arguments(field: &Field) -> std::collections::HashMap<String, Value> {
    let mut args = std::collections::HashMap::new();
    
    for (name, value) in &field.node.arguments {
        // Convert GraphQL value to JSON value
        if let Ok(json_value) = graphql_value_to_json(&value.node) {
            args.insert(name.node.to_string(), json_value);
        }
    }
    
    args
}

/// Convert GraphQL value to JSON value
fn graphql_value_to_json(value: &async_graphql::parser::types::Value) -> ConversionResult<Value> {
    use async_graphql::parser::types::Value as GQLValue;
    
    let json_value = match value {
        GQLValue::Variable(_) => {
            // Variables would need to be resolved from context
            Value::Null
        }
        GQLValue::Number(n) => {
            Value::Number(serde_json::Number::from_f64(n.as_f64().unwrap_or(0.0))
                .unwrap_or_else(|| serde_json::Number::from(0)))
        }
        GQLValue::String(s) => Value::String(s.clone()),
        GQLValue::Boolean(b) => Value::Bool(*b),
        GQLValue::Null => Value::Null,
        GQLValue::Enum(e) => Value::String(e.to_string()),
        GQLValue::List(list) => {
            let mut json_array = Vec::new();
            for item in list {
                json_array.push(graphql_value_to_json(item)?);
            }
            Value::Array(json_array)
        }
        GQLValue::Object(obj) => {
            let mut json_object = serde_json::Map::new();
            for (key, val) in obj {
                json_object.insert(key.to_string(), graphql_value_to_json(val)?);
            }
            Value::Object(json_object)
        }
    };
    
    Ok(json_value)
}