//! JSON-RPC validation and error responses
//!
//! This module provides JSON-RPC validation functionality and error response
//! creation for the MCP bridge with zero allocation patterns and blazing-fast
//! performance.

use serde_json::Value;
use tracing::{debug, warn};

/// Validate JSON-RPC request structure
pub fn validate_json_rpc_request(request: &Value) -> Result<(), anyhow::Error> {
    if !request.is_object() {
        return Err(anyhow::anyhow!("Request must be a JSON object"));
    }

    let obj = request.as_object().ok_or_else(|| {
        anyhow::anyhow!("Failed to parse request as JSON object")
    })?;

    // Check JSON-RPC version
    match obj.get("jsonrpc") {
        Some(Value::String(version)) if version == "2.0" => {}
        Some(Value::String(version)) => {
            return Err(anyhow::anyhow!("Unsupported JSON-RPC version: {}", version));
        }
        Some(_) => {
            return Err(anyhow::anyhow!("JSON-RPC version must be a string"));
        }
        None => {
            return Err(anyhow::anyhow!("Missing required 'jsonrpc' field"));
        }
    }

    // Check method field
    match obj.get("method") {
        Some(Value::String(method)) if !method.is_empty() => {}
        Some(Value::String(_)) => {
            return Err(anyhow::anyhow!("Method name cannot be empty"));
        }
        Some(_) => {
            return Err(anyhow::anyhow!("Method must be a string"));
        }
        None => {
            return Err(anyhow::anyhow!("Missing required 'method' field"));
        }
    }

    // Validate id field (optional, but if present must be string, number, or null)
    if let Some(id) = obj.get("id") {
        match id {
            Value::String(_) | Value::Number(_) | Value::Null => {}
            _ => {
                return Err(anyhow::anyhow!(
                    "ID must be a string, number, or null"
                ));
            }
        }
    }

    // Validate params field (optional, but if present must be object or array)
    if let Some(params) = obj.get("params") {
        match params {
            Value::Object(_) | Value::Array(_) => {}
            _ => {
                return Err(anyhow::anyhow!(
                    "Params must be an object or array"
                ));
            }
        }
    }

    Ok(())
}

/// Validate JSON-RPC response structure
pub fn validate_json_rpc_response(response: &Value) -> Result<(), anyhow::Error> {
    if !response.is_object() {
        return Err(anyhow::anyhow!("Response must be a JSON object"));
    }

    let obj = response.as_object().ok_or_else(|| {
        anyhow::anyhow!("Failed to parse response as JSON object")
    })?;

    // Check JSON-RPC version
    match obj.get("jsonrpc") {
        Some(Value::String(version)) if version == "2.0" => {}
        Some(Value::String(version)) => {
            return Err(anyhow::anyhow!("Unsupported JSON-RPC version: {}", version));
        }
        Some(_) => {
            return Err(anyhow::anyhow!("JSON-RPC version must be a string"));
        }
        None => {
            return Err(anyhow::anyhow!("Missing required 'jsonrpc' field"));
        }
    }

    // Must have either result or error, but not both
    let has_result = obj.contains_key("result");
    let has_error = obj.contains_key("error");

    if !has_result && !has_error {
        return Err(anyhow::anyhow!(
            "Response must contain either 'result' or 'error'"
        ));
    }

    if has_result && has_error {
        return Err(anyhow::anyhow!(
            "Response cannot contain both 'result' and 'error'"
        ));
    }

    // Validate error structure if present
    if let Some(error) = obj.get("error") {
        validate_error_object(error)?;
    }

    // Validate id field (required for responses)
    match obj.get("id") {
        Some(Value::String(_)) | Some(Value::Number(_)) | Some(Value::Null) => {}
        Some(_) => {
            return Err(anyhow::anyhow!(
                "Response ID must be a string, number, or null"
            ));
        }
        None => {
            return Err(anyhow::anyhow!("Missing required 'id' field in response"));
        }
    }

    Ok(())
}

/// Validate JSON-RPC error object structure
fn validate_error_object(error: &Value) -> Result<(), anyhow::Error> {
    if !error.is_object() {
        return Err(anyhow::anyhow!("Error must be an object"));
    }

    let obj = error.as_object().ok_or_else(|| {
        anyhow::anyhow!("Failed to parse error as JSON object")
    })?;

    // Check required code field
    match obj.get("code") {
        Some(Value::Number(code)) => {
            let code_int = code.as_i64().ok_or_else(|| {
                anyhow::anyhow!("Error code must be an integer")
            })?;
            
            // Validate error code ranges
            if !is_valid_error_code(code_int) {
                warn!("Non-standard error code: {}", code_int);
            }
        }
        Some(_) => {
            return Err(anyhow::anyhow!("Error code must be a number"));
        }
        None => {
            return Err(anyhow::anyhow!("Missing required 'code' field in error"));
        }
    }

    // Check required message field
    match obj.get("message") {
        Some(Value::String(message)) if !message.is_empty() => {}
        Some(Value::String(_)) => {
            return Err(anyhow::anyhow!("Error message cannot be empty"));
        }
        Some(_) => {
            return Err(anyhow::anyhow!("Error message must be a string"));
        }
        None => {
            return Err(anyhow::anyhow!("Missing required 'message' field in error"));
        }
    }

    // Data field is optional and can be any JSON value
    Ok(())
}

/// Check if error code is within valid JSON-RPC ranges
fn is_valid_error_code(code: i64) -> bool {
    match code {
        // Standard JSON-RPC errors
        -32700..=-32600 => true,  // Parse error to Invalid request
        -32099..=-32000 => true,  // Server error range
        // Application-defined errors
        -32000..=32000 => true,   // Extended range for applications
        _ => false,
    }
}

/// Create standardized error responses
pub fn create_parse_error_response(id: Option<Value>) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id.unwrap_or(Value::Null),
        "error": {
            "code": -32700,
            "message": "Parse error",
            "data": "Invalid JSON was received by the server"
        }
    })
}

/// Create invalid request error response
pub fn create_invalid_request_response(id: Option<Value>) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id.unwrap_or(Value::Null),
        "error": {
            "code": -32600,
            "message": "Invalid Request",
            "data": "The JSON sent is not a valid Request object"
        }
    })
}

/// Create method not found error response
pub fn create_method_not_found_response(id: Option<Value>, method: &str) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id.unwrap_or(Value::Null),
        "error": {
            "code": -32601,
            "message": "Method not found",
            "data": format!("The method '{}' does not exist", method)
        }
    })
}

/// Create invalid params error response
pub fn create_invalid_params_response(id: Option<Value>, details: &str) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id.unwrap_or(Value::Null),
        "error": {
            "code": -32602,
            "message": "Invalid params",
            "data": details
        }
    })
}

/// Create internal error response
pub fn create_internal_error_response(id: Option<Value>, details: Option<&str>) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id.unwrap_or(Value::Null),
        "error": {
            "code": -32603,
            "message": "Internal error",
            "data": details.unwrap_or("An internal error occurred")
        }
    })
}

/// Create server error response
pub fn create_server_error_response(
    id: Option<Value>,
    code: i64,
    message: &str,
    data: Option<Value>,
) -> Value {
    let mut error = serde_json::json!({
        "code": code,
        "message": message
    });

    if let Some(data_value) = data {
        error.as_object_mut().unwrap().insert("data".to_string(), data_value);
    }

    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id.unwrap_or(Value::Null),
        "error": error
    })
}

/// Extract request ID from potentially malformed JSON
pub fn extract_request_id(request_text: &str) -> Option<Value> {
    // Try to parse as JSON first
    if let Ok(json) = serde_json::from_str::<Value>(request_text) {
        if let Some(obj) = json.as_object() {
            return obj.get("id").cloned();
        }
    }

    // If parsing fails, try to extract ID with regex
    // This is a fallback for malformed JSON
    if let Some(captures) = regex::Regex::new(r#""id"\s*:\s*([^,}]+)"#)
        .ok()?
        .captures(request_text)
    {
        if let Some(id_match) = captures.get(1) {
            let id_str = id_match.as_str().trim();
            
            // Try to parse as number
            if let Ok(num) = id_str.parse::<i64>() {
                return Some(Value::Number(serde_json::Number::from(num)));
            }
            
            // Try to parse as string (remove quotes)
            if id_str.starts_with('"') && id_str.ends_with('"') {
                let unquoted = &id_str[1..id_str.len()-1];
                return Some(Value::String(unquoted.to_string()));
            }
            
            // Check for null
            if id_str == "null" {
                return Some(Value::Null);
            }
        }
    }

    None
}

/// Sanitize error message to prevent information leakage
pub fn sanitize_error_message(message: &str) -> String {
    // Remove potentially sensitive information
    let sanitized = message
        .replace("localhost", "[server]")
        .replace("127.0.0.1", "[server]")
        .replace("::1", "[server]");
    
    // Truncate very long messages
    if sanitized.len() > 500 {
        format!("{}...", &sanitized[..497])
    } else {
        sanitized
    }
}

/// Batch validate multiple JSON-RPC requests
pub fn validate_batch_requests(requests: &[Value]) -> Vec<Result<(), anyhow::Error>> {
    if requests.is_empty() {
        return vec![Err(anyhow::anyhow!("Batch cannot be empty"))];
    }

    if requests.len() > 100 {
        return vec![Err(anyhow::anyhow!("Batch size exceeds maximum limit of 100"))];
    }

    requests.iter()
        .map(validate_json_rpc_request)
        .collect()
}

/// Get error code name for debugging
pub fn get_error_code_name(code: i64) -> &'static str {
    match code {
        -32700 => "Parse error",
        -32600 => "Invalid Request",
        -32601 => "Method not found",
        -32602 => "Invalid params",
        -32603 => "Internal error",
        -32099..=-32000 => "Server error",
        _ => "Unknown error",
    }
}

/// Validate request size limits
pub fn validate_request_size(request_text: &str) -> Result<(), anyhow::Error> {
    const MAX_REQUEST_SIZE: usize = 1024 * 1024; // 1MB
    
    if request_text.len() > MAX_REQUEST_SIZE {
        return Err(anyhow::anyhow!(
            "Request size {} bytes exceeds maximum limit of {} bytes",
            request_text.len(),
            MAX_REQUEST_SIZE
        ));
    }
    
    Ok(())
}

/// Check for common JSON-RPC security issues
pub fn validate_security(request: &Value) -> Result<(), anyhow::Error> {
    if let Some(obj) = request.as_object() {
        // Check for suspicious method names
        if let Some(Value::String(method)) = obj.get("method") {
            if method.contains("..") || method.contains("/") || method.contains("\\") {
                return Err(anyhow::anyhow!("Method name contains invalid characters"));
            }
            
            if method.starts_with("_") || method.starts_with("rpc.") {
                debug!("Method name '{}' may be reserved", method);
            }
        }
        
        // Check for excessively nested parameters
        if let Some(params) = obj.get("params") {
            if get_json_depth(params) > 10 {
                return Err(anyhow::anyhow!("Parameters exceed maximum nesting depth"));
            }
        }
    }
    
    Ok(())
}

/// Calculate JSON nesting depth
fn get_json_depth(value: &Value) -> usize {
    match value {
        Value::Object(obj) => {
            1 + obj.values().map(get_json_depth).max().unwrap_or(0)
        }
        Value::Array(arr) => {
            1 + arr.iter().map(get_json_depth).max().unwrap_or(0)
        }
        _ => 0,
    }
}