//! MCP bridge core implementation
//!
//! This module provides the core McpBridge struct and HTTP client setup
//! for communicating with the sweetmcp-axum MCP server with zero allocation
//! patterns and blazing-fast performance.

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
    pub(super) client: Client,
    /// Base URL of the MCP server
    pub(super) mcp_server_url: String,
    /// Request timeout
    pub(super) timeout: Duration,
}

impl McpBridge {
    /// Create a new MCP bridge with optimized HTTP client
    pub fn new(mcp_server_url: String, timeout: Duration) -> Result<Self> {
        let client = Client::builder()
            .timeout(timeout)
            .pool_idle_timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(10)
            .tcp_keepalive(Duration::from_secs(60))
            .tcp_nodelay(true)
            .http2_prior_knowledge()
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            mcp_server_url,
            timeout,
        })
    }

    /// Create a new MCP bridge with custom client configuration
    pub fn with_client_config(
        mcp_server_url: String,
        timeout: Duration,
        max_idle_connections: usize,
        keepalive_timeout: Duration,
    ) -> Result<Self> {
        let client = Client::builder()
            .timeout(timeout)
            .pool_idle_timeout(keepalive_timeout)
            .pool_max_idle_per_host(max_idle_connections)
            .tcp_keepalive(keepalive_timeout)
            .tcp_nodelay(true)
            .http2_prior_knowledge()
            .user_agent("sweetmcp-daemon/1.0")
            .build()
            .context("Failed to create HTTP client with custom config")?;

        Ok(Self {
            client,
            mcp_server_url,
            timeout,
        })
    }

    /// Get the MCP server URL
    pub fn server_url(&self) -> &str {
        &self.mcp_server_url
    }

    /// Get the request timeout
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Update the MCP server URL
    pub fn set_server_url(&mut self, url: String) {
        self.mcp_server_url = url;
    }

    /// Update the request timeout
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    /// Check if the MCP server is reachable
    pub async fn health_check(&self) -> Result<bool> {
        let health_url = format!("{}/health", self.mcp_server_url);
        
        match self.client.get(&health_url).send().await {
            Ok(response) => {
                let is_healthy = response.status().is_success();
                debug!("MCP server health check: {}", if is_healthy { "healthy" } else { "unhealthy" });
                Ok(is_healthy)
            }
            Err(e) => {
                warn!("MCP server health check failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Get connection statistics
    pub fn get_connection_stats(&self) -> ConnectionStats {
        // Note: reqwest doesn't expose detailed connection pool stats
        // This is a placeholder for potential future implementation
        ConnectionStats {
            active_connections: 0,
            idle_connections: 0,
            total_requests: 0,
            failed_requests: 0,
        }
    }

    /// Send raw HTTP request to MCP server
    pub(super) async fn send_request(&self, json_rpc_request: Value) -> Result<Value> {
        debug!(
            "Sending JSON-RPC request to MCP server: {}",
            json_rpc_request
        );

        let response = self
            .client
            .post(&self.mcp_server_url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&json_rpc_request)
            .send()
            .await
            .context("Failed to send request to MCP server")?;

        self.handle_http_response(response).await
    }

    /// Handle HTTP response from MCP server
    async fn handle_http_response(&self, response: Response) -> Result<Value> {
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

        let response_text = response
            .text()
            .await
            .context("Failed to read response body")?;

        if response_text.trim().is_empty() {
            return Err(anyhow::anyhow!("MCP server returned empty response"));
        }

        serde_json::from_str(&response_text)
            .context("Failed to parse JSON response from MCP server")
    }

    /// Create error response for failed requests
    pub(super) fn create_error_response(&self, original_request: &Value, error: anyhow::Error) -> Value {
        let request_id = original_request
            .get("id")
            .cloned()
            .unwrap_or(Value::Null);

        serde_json::json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "error": {
                "code": -32603,
                "message": "MCP server unavailable",
                "data": {
                    "original_error": error.to_string(),
                    "server_url": self.mcp_server_url,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }
            }
        })
    }

    /// Validate MCP server configuration
    pub fn validate_config(&self) -> Result<()> {
        // Validate URL format
        if self.mcp_server_url.is_empty() {
            return Err(anyhow::anyhow!("MCP server URL cannot be empty"));
        }

        let parsed_url = url::Url::parse(&self.mcp_server_url)
            .context("Invalid MCP server URL format")?;

        if !matches!(parsed_url.scheme(), "http" | "https") {
            return Err(anyhow::anyhow!("MCP server URL must use HTTP or HTTPS scheme"));
        }

        // Validate timeout
        if self.timeout.is_zero() {
            return Err(anyhow::anyhow!("Request timeout must be greater than zero"));
        }

        if self.timeout > Duration::from_secs(300) {
            warn!("Request timeout is very high: {:?}", self.timeout);
        }

        Ok(())
    }

    /// Get bridge configuration summary
    pub fn get_config_summary(&self) -> BridgeConfig {
        BridgeConfig {
            server_url: self.mcp_server_url.clone(),
            timeout_ms: self.timeout.as_millis() as u64,
            client_configured: true,
        }
    }
}

/// Connection statistics for monitoring
#[derive(Debug, Clone)]
pub struct ConnectionStats {
    /// Number of active connections
    pub active_connections: usize,
    /// Number of idle connections in pool
    pub idle_connections: usize,
    /// Total number of requests sent
    pub total_requests: u64,
    /// Number of failed requests
    pub failed_requests: u64,
}

/// Bridge configuration summary
#[derive(Debug, Clone)]
pub struct BridgeConfig {
    /// MCP server URL
    pub server_url: String,
    /// Request timeout in milliseconds
    pub timeout_ms: u64,
    /// Whether HTTP client is properly configured
    pub client_configured: bool,
}

impl Default for McpBridge {
    fn default() -> Self {
        Self::new(
            "http://localhost:8080".to_string(),
            Duration::from_secs(30),
        ).expect("Failed to create default MCP bridge")
    }
}

/// Builder for McpBridge with fluent API
pub struct McpBridgeBuilder {
    server_url: Option<String>,
    timeout: Option<Duration>,
    max_idle_connections: Option<usize>,
    keepalive_timeout: Option<Duration>,
    user_agent: Option<String>,
}

impl McpBridgeBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            server_url: None,
            timeout: None,
            max_idle_connections: None,
            keepalive_timeout: None,
            user_agent: None,
        }
    }

    /// Set the MCP server URL
    pub fn server_url(mut self, url: impl Into<String>) -> Self {
        self.server_url = Some(url.into());
        self
    }

    /// Set the request timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set maximum idle connections
    pub fn max_idle_connections(mut self, max: usize) -> Self {
        self.max_idle_connections = Some(max);
        self
    }

    /// Set keepalive timeout
    pub fn keepalive_timeout(mut self, timeout: Duration) -> Self {
        self.keepalive_timeout = Some(timeout);
        self
    }

    /// Set user agent string
    pub fn user_agent(mut self, agent: impl Into<String>) -> Self {
        self.user_agent = Some(agent.into());
        self
    }

    /// Build the McpBridge
    pub fn build(self) -> Result<McpBridge> {
        let server_url = self.server_url
            .ok_or_else(|| anyhow::anyhow!("Server URL is required"))?;
        
        let timeout = self.timeout.unwrap_or(Duration::from_secs(30));
        let max_idle = self.max_idle_connections.unwrap_or(10);
        let keepalive = self.keepalive_timeout.unwrap_or(Duration::from_secs(60));

        McpBridge::with_client_config(server_url, timeout, max_idle, keepalive)
    }
}

impl Default for McpBridgeBuilder {
    fn default() -> Self {
        Self::new()
    }
}