//! SSE (Server-Sent Events) transport module for MCP Claude Code integration
//!
//! This module provides a complete SSE server implementation that enables
//! Claude Code to connect to the SweetMCP daemon via the SSE transport protocol.
//!
//! ## Architecture
//!
//! The SSE transport uses a dual-endpoint architecture:
//! - GET /sse - Establishes persistent SSE connection with session management
//! - POST /messages - Handles JSON-RPC requests routed by session ID
//!
//! ## Components
//!
//! - `events` - SSE event types and wire format encoding
//! - `session` - Session management and lifecycle
//! - `server` - HTTP server with SSE and messages endpoints
//! - `bridge` - Communication bridge to sweetmcp-axum MCP server
//! - `encoder` - SSE wire format encoding per RFC 6455

pub mod bridge;
pub mod encoder;
pub mod events;
pub mod server;
pub mod session;

pub use bridge::McpBridge;
pub use encoder::SseEncoder;
pub use events::{EventType, SseEvent};
pub use server::SseServer;
pub use session::{SessionManager, SseSession};

use anyhow::Result;
use std::net::SocketAddr;
use tokio::sync::oneshot;

/// SSE server configuration
#[derive(Debug, Clone)]
pub struct SseConfig {
    /// Port to bind SSE server to (default: 8080)
    pub port: u16,
    /// MCP server URL to bridge requests to
    pub mcp_server_url: String,
    /// Maximum number of concurrent connections
    pub max_connections: usize,
    /// Ping interval for keep-alive (seconds)
    pub ping_interval: u64,
    /// Session timeout (seconds)
    pub session_timeout: u64,
    /// CORS allowed origins
    pub cors_origins: Vec<String>,
}

impl Default for SseConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            mcp_server_url: "http://127.0.0.1:3000".to_string(),
            max_connections: 100,
            ping_interval: 30,
            session_timeout: 300,
            cors_origins: vec!["*".to_string()],
        }
    }
}

/// Start the SSE server with given configuration
pub async fn start_sse_server(config: SseConfig, shutdown_rx: oneshot::Receiver<()>) -> Result<()> {
    let addr: SocketAddr = ([127, 0, 0, 1], config.port).into();
    let server = SseServer::new(config);
    server.serve(addr, shutdown_rx).await
}
