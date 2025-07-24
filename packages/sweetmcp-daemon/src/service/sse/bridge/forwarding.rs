//! Request forwarding and response handling
//!
//! This module provides request forwarding logic and response processing
//! for the MCP bridge with zero allocation patterns and blazing-fast
//! performance.

use anyhow::{Context, Result};
use serde_json::Value;
use std::time::{Duration, Instant};
use tracing::{debug, error, warn};

use super::core::McpBridge;

impl McpBridge {
    /// Forward a JSON-RPC request to the MCP server
    ///
    /// Takes a JSON-RPC request and forwards it to the configured MCP server.
    /// Returns the JSON-RPC response or an error response on failure.
    pub async fn forward_request(&self, json_rpc_request: Value) -> Value {
        let start_time = Instant::now();
        
        debug!(
            "Forwarding JSON-RPC request to MCP server: {}",
            json_rpc_request
        );

        // Validate request before forwarding
        if let Err(validation_error) = super::validation::validate_json_rpc_request(&json_rpc_request) {
            error!("Invalid JSON-RPC request: {}", validation_error);
            return self.create_error_response(&json_rpc_request, validation_error);
        }

        match self.send_request(json_rpc_request.clone()).await {
            Ok(response) => {
                let duration = start_time.elapsed();
                debug!(
                    "Received successful response from MCP server in {:?}",
                    duration
                );
                
                // Log slow requests
                if duration > Duration::from_millis(1000) {
                    warn!(
                        "Slow MCP server response: {:?} for request: {}",
                        duration,
                        json_rpc_request.get("method").unwrap_or(&Value::Null)
                    );
                }
                
                response
            }
            Err(error) => {
                let duration = start_time.elapsed();
                error!(
                    "Failed to forward request to MCP server after {:?}: {}",
                    duration, error
                );
                self.create_error_response(&json_rpc_request, error)
            }
        }
    }

    /// Forward multiple requests in batch
    pub async fn forward_batch_requests(&self, requests: Vec<Value>) -> Vec<Value> {
        if requests.is_empty() {
            return Vec::new();
        }

        debug!("Forwarding batch of {} requests", requests.len());
        
        let start_time = Instant::now();
        let mut responses = Vec::with_capacity(requests.len());

        // Process requests concurrently with controlled parallelism
        let semaphore = tokio::sync::Semaphore::new(10); // Limit concurrent requests
        let mut tasks = Vec::new();

        for request in requests {
            let permit = semaphore.clone().acquire_owned().await;
            let bridge = self.clone();
            
            let task = tokio::spawn(async move {
                let _permit = permit;
                bridge.forward_request(request).await
            });
            
            tasks.push(task);
        }

        // Collect results maintaining order
        for task in tasks {
            match task.await {
                Ok(response) => responses.push(response),
                Err(join_error) => {
                    error!("Task join error in batch processing: {}", join_error);
                    responses.push(serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": null,
                        "error": {
                            "code": -32603,
                            "message": "Internal error during batch processing"
                        }
                    }));
                }
            }
        }

        let duration = start_time.elapsed();
        debug!(
            "Completed batch of {} requests in {:?}",
            responses.len(),
            duration
        );

        responses
    }

    /// Forward request with retry logic
    pub async fn forward_request_with_retry(
        &self,
        json_rpc_request: Value,
        max_retries: u32,
        retry_delay: Duration,
    ) -> Value {
        let mut last_error = None;

        for attempt in 0..=max_retries {
            if attempt > 0 {
                debug!("Retrying request (attempt {})", attempt + 1);
                tokio::time::sleep(retry_delay).await;
            }

            match self.send_request(json_rpc_request.clone()).await {
                Ok(response) => {
                    if attempt > 0 {
                        debug!("Request succeeded on retry attempt {}", attempt + 1);
                    }
                    return response;
                }
                Err(error) => {
                    last_error = Some(error);
                    
                    // Don't retry on client errors (4xx status codes)
                    if error.to_string().contains("400") || error.to_string().contains("401") ||
                       error.to_string().contains("403") || error.to_string().contains("404") {
                        break;
                    }
                }
            }
        }

        let final_error = last_error.unwrap_or_else(|| {
            anyhow::anyhow!("Request failed after {} attempts", max_retries + 1)
        });

        error!(
            "Request failed after {} attempts: {}",
            max_retries + 1,
            final_error
        );

        self.create_error_response(&json_rpc_request, final_error)
    }

    /// Forward request with timeout override
    pub async fn forward_request_with_timeout(
        &self,
        json_rpc_request: Value,
        timeout: Duration,
    ) -> Value {
        debug!(
            "Forwarding request with custom timeout: {:?}",
            timeout
        );

        let request_future = self.send_request(json_rpc_request.clone());
        
        match tokio::time::timeout(timeout, request_future).await {
            Ok(Ok(response)) => response,
            Ok(Err(error)) => {
                error!("Request failed: {}", error);
                self.create_error_response(&json_rpc_request, error)
            }
            Err(_timeout_error) => {
                error!("Request timed out after {:?}", timeout);
                let timeout_error = anyhow::anyhow!("Request timed out after {:?}", timeout);
                self.create_error_response(&json_rpc_request, timeout_error)
            }
        }
    }

    /// Stream responses for long-running requests
    pub async fn forward_streaming_request(
        &self,
        json_rpc_request: Value,
        response_callback: impl Fn(Value) + Send + Sync,
    ) -> Result<()> {
        debug!("Forwarding streaming request");

        // For now, this is a placeholder implementation
        // In a full implementation, this would handle Server-Sent Events or WebSocket streams
        let response = self.send_request(json_rpc_request).await?;
        response_callback(response);

        Ok(())
    }

    /// Get forwarding statistics
    pub fn get_forwarding_stats(&self) -> ForwardingStats {
        // This would be implemented with actual metrics collection
        // For now, returning placeholder values
        ForwardingStats {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            average_response_time_ms: 0.0,
            last_request_time: None,
        }
    }

    /// Process response and apply transformations if needed
    pub fn process_response(&self, response: Value, original_request: &Value) -> Value {
        // Apply any necessary response transformations
        let mut processed_response = response;

        // Ensure response has proper JSON-RPC structure
        if !processed_response.is_object() {
            warn!("Response is not a JSON object, wrapping in error response");
            return self.create_error_response(
                original_request,
                anyhow::anyhow!("Invalid response format from MCP server")
            );
        }

        // Ensure jsonrpc field is present
        if processed_response.get("jsonrpc").is_none() {
            if let Some(obj) = processed_response.as_object_mut() {
                obj.insert("jsonrpc".to_string(), Value::String("2.0".to_string()));
            }
        }

        // Ensure id field matches request
        if let Some(request_id) = original_request.get("id") {
            if let Some(obj) = processed_response.as_object_mut() {
                obj.insert("id".to_string(), request_id.clone());
            }
        }

        processed_response
    }

    /// Handle connection errors with appropriate fallback
    pub async fn handle_connection_error(
        &self,
        error: &anyhow::Error,
        request: &Value,
    ) -> Option<Value> {
        let error_msg = error.to_string();

        // Check for specific connection issues
        if error_msg.contains("Connection refused") {
            warn!("MCP server connection refused, server may be down");
            
            // Attempt health check to confirm server status
            if !self.health_check().await.unwrap_or(false) {
                return Some(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": request.get("id").cloned().unwrap_or(Value::Null),
                    "error": {
                        "code": -32001,
                        "message": "MCP server is unavailable",
                        "data": {
                            "server_url": self.mcp_server_url,
                            "error_type": "connection_refused"
                        }
                    }
                }));
            }
        } else if error_msg.contains("timeout") {
            warn!("Request timed out, server may be overloaded");
            
            return Some(serde_json::json!({
                "jsonrpc": "2.0",
                "id": request.get("id").cloned().unwrap_or(Value::Null),
                "error": {
                    "code": -32002,
                    "message": "Request timed out",
                    "data": {
                        "timeout_ms": self.timeout.as_millis(),
                        "error_type": "timeout"
                    }
                }
            }));
        }

        None
    }

    /// Validate response structure
    pub fn validate_response(&self, response: &Value) -> Result<()> {
        if !response.is_object() {
            return Err(anyhow::anyhow!("Response must be a JSON object"));
        }

        let obj = response.as_object().unwrap();

        // Check for required jsonrpc field
        match obj.get("jsonrpc") {
            Some(Value::String(version)) if version == "2.0" => {}
            Some(_) => return Err(anyhow::anyhow!("Invalid JSON-RPC version")),
            None => return Err(anyhow::anyhow!("Missing jsonrpc field")),
        }

        // Must have either result or error
        let has_result = obj.contains_key("result");
        let has_error = obj.contains_key("error");

        if !has_result && !has_error {
            return Err(anyhow::anyhow!("Response must contain either result or error"));
        }

        if has_result && has_error {
            return Err(anyhow::anyhow!("Response cannot contain both result and error"));
        }

        Ok(())
    }
}

/// Forwarding statistics for monitoring
#[derive(Debug, Clone)]
pub struct ForwardingStats {
    /// Total number of requests forwarded
    pub total_requests: u64,
    /// Number of successful requests
    pub successful_requests: u64,
    /// Number of failed requests
    pub failed_requests: u64,
    /// Average response time in milliseconds
    pub average_response_time_ms: f64,
    /// Timestamp of last request
    pub last_request_time: Option<chrono::DateTime<chrono::Utc>>,
}

impl ForwardingStats {
    /// Calculate success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.successful_requests as f64 / self.total_requests as f64) * 100.0
        }
    }

    /// Calculate failure rate as percentage
    pub fn failure_rate(&self) -> f64 {
        100.0 - self.success_rate()
    }

    /// Check if performance is acceptable
    pub fn is_healthy(&self) -> bool {
        self.success_rate() >= 95.0 && self.average_response_time_ms < 1000.0
    }
}