//! MCP bridge module
//!
//! This module provides comprehensive MCP server communication bridge
//! functionality including HTTP client setup, request forwarding, response
//! handling, and JSON-RPC validation with zero allocation patterns and
//! blazing-fast performance.

pub mod core;
pub mod forwarding;
pub mod validation;

// Re-export key types and functions for ergonomic usage
pub use core::{
    McpBridge, McpBridgeBuilder, ConnectionStats, BridgeConfig,
};

pub use forwarding::{
    ForwardingStats,
};

pub use validation::{
    validate_json_rpc_request, validate_json_rpc_response, validate_batch_requests,
    validate_request_size, validate_security, extract_request_id, sanitize_error_message,
    create_parse_error_response, create_invalid_request_response,
    create_method_not_found_response, create_invalid_params_response,
    create_internal_error_response, create_server_error_response,
    get_error_code_name,
};

/// Create a new MCP bridge with default settings
pub fn bridge(server_url: impl Into<String>) -> anyhow::Result<McpBridge> {
    McpBridge::new(server_url.into(), std::time::Duration::from_secs(30))
}

/// Create a new MCP bridge builder
pub fn bridge_builder() -> McpBridgeBuilder {
    McpBridgeBuilder::new()
}

/// Create a new MCP bridge with custom timeout
pub fn bridge_with_timeout(
    server_url: impl Into<String>,
    timeout: std::time::Duration,
) -> anyhow::Result<McpBridge> {
    McpBridge::new(server_url.into(), timeout)
}

/// Validate and forward a JSON-RPC request
pub async fn validate_and_forward(
    bridge: &McpBridge,
    request: serde_json::Value,
) -> serde_json::Value {
    // Validate request first
    if let Err(validation_error) = validate_json_rpc_request(&request) {
        return create_invalid_request_response(request.get("id").cloned());
    }

    // Forward the validated request
    bridge.forward_request(request).await
}

/// Batch process multiple requests
pub async fn batch_process(
    bridge: &McpBridge,
    requests: Vec<serde_json::Value>,
) -> Vec<serde_json::Value> {
    if requests.is_empty() {
        return vec![create_invalid_request_response(None)];
    }

    // Validate all requests first
    let validation_results = validate_batch_requests(&requests);
    let mut responses = Vec::with_capacity(requests.len());

    for (request, validation_result) in requests.iter().zip(validation_results.iter()) {
        if let Err(_) = validation_result {
            responses.push(create_invalid_request_response(request.get("id").cloned()));
        } else {
            responses.push(bridge.forward_request(request.clone()).await);
        }
    }

    responses
}