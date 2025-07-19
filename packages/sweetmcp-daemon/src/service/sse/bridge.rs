//! MCP server communication bridge
//!
//! Handles communication between the SSE server and the sweetmcp-axum MCP server.
//! Provides request forwarding, response handling, and error management.

use anyhow::{Context, Result};
use reqwest::{Client, Response};
use serde_json::Value;
use std::time::Duration;
use tracing::{debug, error, warn};

/// Bridge for communicating with the MCP server
///
/// Handles HTTP communication with the sweetmcp-axum server,
/// including request forwarding, response processing, and error handling.
#[derive(Debug, Clone)]
pub struct McpBridge {
    /// HTTP client for making requests
    client: Client,
    /// Base URL of the MCP server
    mcp_server_url: String,
    /// Request timeout
    timeout: Duration,
}

impl McpBridge {
    /// Create a new MCP bridge
    pub fn new(mcp_server_url: String, timeout: Duration) -> Result<Self> {
        let client = Client::builder()
            .timeout(timeout)
            .pool_idle_timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(10)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            mcp_server_url,
            timeout,
        })
    }

    /// Forward a JSON-RPC request to the MCP server
    ///
    /// Takes a JSON-RPC request and forwards it to the configured MCP server.
    /// Returns the JSON-RPC response or an error response on failure.
    pub async fn forward_request(&self, json_rpc_request: Value) -> Value {
        debug!(
            "Forwarding JSON-RPC request to MCP server: {}",
            json_rpc_request
        );

        match self.send_request(json_rpc_request.clone()).await {
            Ok(response) => {
                debug!("Received successful response from MCP server");
                response
            }
            Err(error) => {
                error!("Failed to forward request to MCP server: {}", error);
                self.create_error_response(&json_rpc_request, error)
            }
        }
    }

    /// Send a request to the MCP server
    async fn send_request(&self, request: Value) -> Result<Value> {
        let url = format!("{}/rpc", self.mcp_server_url);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to MCP server")?;

        self.handle_response(response).await
    }

    /// Handle the HTTP response from the MCP server
    async fn handle_response(&self, response: Response) -> Result<Value> {
        let status = response.status();

        if !status.is_success() {
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            return Err(anyhow::anyhow!(
                "MCP server returned error status {}: {}",
                status,
                error_body
            ));
        }

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if !content_type.starts_with("application/json") {
            warn!(
                "MCP server returned unexpected content type: {}",
                content_type
            );
        }

        let response_text = response
            .text()
            .await
            .context("Failed to read response body from MCP server")?;

        serde_json::from_str(&response_text)
            .context("Failed to parse JSON response from MCP server")
    }

    /// Create a JSON-RPC error response
    fn create_error_response(&self, original_request: &Value, error: anyhow::Error) -> Value {
        let request_id = original_request.get("id").cloned().unwrap_or(Value::Null);

        serde_json::json!({
            "jsonrpc": "2.0",
            "error": {
                "code": -32603,
                "message": "Internal error: MCP server unavailable",
                "data": {
                    "details": error.to_string(),
                    "mcp_server_url": self.mcp_server_url
                }
            },
            "id": request_id
        })
    }

    /// Check if the MCP server is healthy
    ///
    /// Sends a ping request to verify the MCP server is responsive.
    pub async fn health_check(&self) -> bool {
        let ping_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "health-check",
            "method": "ping",
            "params": {}
        });

        match self.send_request(ping_request).await {
            Ok(response) => {
                if let Some(error) = response.get("error") {
                    warn!("MCP server health check returned error: {}", error);
                    false
                } else {
                    debug!("MCP server health check passed");
                    true
                }
            }
            Err(error) => {
                warn!("MCP server health check failed: {}", error);
                false
            }
        }
    }

    /// Get the MCP server URL
    pub fn server_url(&self) -> &str {
        &self.mcp_server_url
    }

    /// Get the configured timeout
    pub fn timeout(&self) -> Duration {
        self.timeout
    }
}

/// Helper function to validate JSON-RPC request format
pub fn validate_json_rpc_request(request: &Value) -> Result<()> {
    // Check required fields
    if !request.is_object() {
        return Err(anyhow::anyhow!("Request must be a JSON object"));
    }

    let jsonrpc = request.get("jsonrpc");
    if jsonrpc != Some(&Value::String("2.0".to_string())) {
        return Err(anyhow::anyhow!("Invalid or missing jsonrpc field"));
    }

    if request.get("method").is_none() {
        return Err(anyhow::anyhow!("Missing method field"));
    }

    // ID field is optional for notifications
    Ok(())
}

/// Helper function to create a JSON-RPC parse error response
pub fn create_parse_error_response(id: Option<Value>) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "error": {
            "code": -32700,
            "message": "Parse error"
        },
        "id": id.unwrap_or(Value::Null)
    })
}

/// Helper function to create a JSON-RPC invalid request error response
pub fn create_invalid_request_response(id: Option<Value>) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "error": {
            "code": -32600,
            "message": "Invalid Request"
        },
        "id": id.unwrap_or(Value::Null)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validate_json_rpc_request() {
        // Valid request
        let valid_request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "ping",
            "params": {}
        });
        assert!(validate_json_rpc_request(&valid_request).is_ok());

        // Valid notification (no id)
        let valid_notification = json!({
            "jsonrpc": "2.0",
            "method": "notify",
            "params": {}
        });
        assert!(validate_json_rpc_request(&valid_notification).is_ok());

        // Invalid: not an object
        let invalid_array = json!([1, 2, 3]);
        assert!(validate_json_rpc_request(&invalid_array).is_err());

        // Invalid: wrong jsonrpc version
        let invalid_version = json!({
            "jsonrpc": "1.0",
            "id": 1,
            "method": "ping"
        });
        assert!(validate_json_rpc_request(&invalid_version).is_err());

        // Invalid: missing method
        let missing_method = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "params": {}
        });
        assert!(validate_json_rpc_request(&missing_method).is_err());
    }

    #[test]
    fn test_error_response_creation() {
        let parse_error = create_parse_error_response(Some(json!(1)));
        assert_eq!(parse_error["jsonrpc"], "2.0");
        assert_eq!(parse_error["error"]["code"], -32700);
        assert_eq!(parse_error["id"], 1);

        let invalid_request = create_invalid_request_response(None);
        assert_eq!(invalid_request["jsonrpc"], "2.0");
        assert_eq!(invalid_request["error"]["code"], -32600);
        assert_eq!(invalid_request["id"], Value::Null);
    }

    #[tokio::test]
    async fn test_bridge_creation() {
        let bridge = McpBridge::new("http://localhost:3000".to_string(), Duration::from_secs(30));

        assert!(bridge.is_ok());
        let bridge = bridge.unwrap();
        assert_eq!(bridge.server_url(), "http://localhost:3000");
        assert_eq!(bridge.timeout(), Duration::from_secs(30));
    }

    #[test]
    fn test_create_error_response() {
        let bridge =
            McpBridge::new("http://localhost:3000".to_string(), Duration::from_secs(30)).unwrap();

        let original_request = json!({
            "jsonrpc": "2.0",
            "id": 42,
            "method": "test"
        });

        let error = anyhow::anyhow!("Test error");
        let error_response = bridge.create_error_response(&original_request, error);

        assert_eq!(error_response["jsonrpc"], "2.0");
        assert_eq!(error_response["id"], 42);
        assert_eq!(error_response["error"]["code"], -32603);
        assert!(error_response["error"]["message"]
            .as_str()
            .unwrap()
            .contains("MCP server unavailable"));
    }
}
