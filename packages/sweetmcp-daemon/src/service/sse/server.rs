//! SSE HTTP server implementation
//!
//! Implements the dual-endpoint SSE server with /sse and /messages endpoints
//! as specified in the MCP SSE transport protocol.

use crate::service::sse::{
    bridge::{create_invalid_request_response, validate_json_rpc_request, McpBridge},
    encoder::SseEncoder,
    events::SseEvent,
    session::{ClientInfo, SessionManager},
    SseConfig,
};
use anyhow::{Context, Result};
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::Sse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{convert::Infallible, net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::oneshot;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{debug, info, warn};

/// SSE server state shared across handlers
#[derive(Debug, Clone)]
struct ServerState {
    /// Session manager for tracking SSE connections
    session_manager: Arc<SessionManager>,
    /// Bridge for communicating with MCP server
    mcp_bridge: Arc<McpBridge>,
    /// SSE encoder for formatting events
    encoder: SseEncoder,
    /// Server configuration
    config: SseConfig,
}

/// Query parameters for the messages endpoint
#[derive(Debug, Deserialize)]
struct MessagesQuery {
    session_id: String,
}

/// SSE server implementation
#[derive(Debug)]
pub struct SseServer {
    config: SseConfig,
}

impl SseServer {
    /// Create a new SSE server with given configuration
    pub fn new(config: SseConfig) -> Self {
        Self { config }
    }

    /// Start serving on the given address
    pub async fn serve(self, addr: SocketAddr, shutdown_rx: oneshot::Receiver<()>) -> Result<()> {
        // Initialize components
        let session_manager = Arc::new(SessionManager::new(
            self.config.max_connections,
            Duration::from_secs(self.config.session_timeout),
        ));

        let mcp_bridge = Arc::new(
            McpBridge::new(self.config.mcp_server_url.clone(), Duration::from_secs(30))
                .context("Failed to create MCP bridge")?,
        );

        let encoder = SseEncoder::new();

        let state = ServerState {
            session_manager: session_manager.clone(),
            mcp_bridge,
            encoder,
            config: self.config.clone(),
        };

        // Start background cleanup task
        let _cleanup_task = session_manager.start_cleanup_task(Duration::from_secs(60));

        // Build the router
        let app = self.build_router(state);

        // Start the server
        info!("Starting SSE server on {}", addr);

        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .context("Failed to bind to address")?;

        // Run server with graceful shutdown
        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                shutdown_rx.await.ok();
                info!("SSE server shutting down gracefully");
            })
            .await
            .context("SSE server error")?;

        info!("SSE server stopped");
        Ok(())
    }

    /// Build the axum router with all endpoints
    fn build_router(&self, state: ServerState) -> Router {
        Router::new()
            .route("/sse", get(handle_sse_endpoint))
            .route("/messages", post(handle_messages_endpoint))
            .route("/health", get(handle_health_endpoint))
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(self.build_cors_layer())
                    .into_inner(),
            )
            .with_state(state)
    }

    /// Build CORS layer based on configuration
    fn build_cors_layer(&self) -> CorsLayer {
        let mut cors = CorsLayer::new()
            .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
            .allow_headers([
                axum::http::header::CONTENT_TYPE,
                axum::http::header::AUTHORIZATION,
                axum::http::header::ACCEPT,
            ]);

        // Configure allowed origins
        if self.config.cors_origins.contains(&"*".to_string()) {
            cors = cors.allow_origin(Any);
        } else {
            for origin in &self.config.cors_origins {
                if let Ok(origin_header) = origin.parse::<axum::http::HeaderValue>() {
                    cors = cors.allow_origin(origin_header);
                }
            }
        }

        cors
    }
}

/// Handle GET /sse endpoint - establish SSE connection
async fn handle_sse_endpoint(
    State(state): State<ServerState>,
    headers: HeaderMap,
) -> Result<
    Sse<impl tokio_stream::Stream<Item = Result<axum::response::sse::Event, Infallible>>>,
    StatusCode,
> {
    // Extract client information
    let remote_addr = headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let client_info = ClientInfo {
        remote_addr: remote_addr.clone(),
        user_agent,
        connection_id: None,
    };

    // Create new session
    let session = match state.session_manager.create_session(client_info).await {
        Some(session) => session,
        None => {
            warn!(
                "Rejected SSE connection from {} (session limit reached)",
                remote_addr
            );
            return Err(StatusCode::SERVICE_UNAVAILABLE);
        }
    };

    info!("Established SSE connection for session {}", session.id);

    // Create event stream using futures stream
    let session_id = session.id.clone();
    let session_manager = state.session_manager.clone();
    let encoder = state.encoder.clone();
    let config = state.config.clone();

    use futures_util::stream::{self, StreamExt};

    // Send initial endpoint event
    let base_url = format!("http://127.0.0.1:{}", config.port);
    let endpoint_event = SseEvent::endpoint(&session_id, &base_url);
    let initial_data = encoder.encode(&endpoint_event);
    let initial_event = axum::response::sse::Event::default().data(initial_data.trim_end());

    // Create ping stream
    let ping_stream = {
        let session_id = session_id.clone();
        let session_manager = session_manager.clone();
        let encoder = encoder.clone();
        let ping_interval = config.ping_interval;

        stream::unfold(0u64, move |event_counter| {
            let session_id = session_id.clone();
            let session_manager = session_manager.clone();
            let encoder = encoder.clone();

            async move {
                // Wait for ping interval
                tokio::time::sleep(Duration::from_secs(ping_interval)).await;

                // Create ping event
                let timestamp = chrono::Utc::now().to_rfc3339();
                let ping_event =
                    SseEvent::ping(timestamp).with_id(format!("ping-{}", event_counter));
                let encoded = encoder.encode(&ping_event);
                let event = axum::response::sse::Event::default().data(encoded.trim_end());

                // Touch session to keep it alive
                session_manager.touch_session(&session_id).await;

                Some((Ok::<_, Infallible>(event), event_counter + 1))
            }
        })
    };

    // Combine initial event with ping stream
    let combined_stream =
        stream::once(async { Ok::<_, Infallible>(initial_event) }).chain(ping_stream);

    Ok(Sse::new(combined_stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(config.ping_interval))
            .text("keep-alive"),
    ))
}

/// Handle POST /messages endpoint - process JSON-RPC requests
async fn handle_messages_endpoint(
    State(state): State<ServerState>,
    Query(query): Query<MessagesQuery>,
    Json(request): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    debug!(
        "Received message for session {}: {}",
        query.session_id, request
    );

    // Validate session exists
    let session = match state.session_manager.get_session(&query.session_id).await {
        Some(session) => session,
        None => {
            warn!("Message for unknown session: {}", query.session_id);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    // Touch session to update activity
    state.session_manager.touch_session(&session.id).await;

    // Validate JSON-RPC format
    if let Err(error) = validate_json_rpc_request(&request) {
        warn!("Invalid JSON-RPC request: {}", error);
        let error_response = create_invalid_request_response(request.get("id").cloned());
        return Ok(Json(error_response));
    }

    // Forward request to MCP server
    let response = state.mcp_bridge.forward_request(request).await;

    debug!("Returning response for session {}", session.id);
    Ok(Json(response))
}

/// Handle GET /health endpoint - health check
async fn handle_health_endpoint(
    State(state): State<ServerState>,
) -> Result<(StatusCode, Json<HealthResponse>), StatusCode> {
    let session_count = state.session_manager.session_count().await;
    let mcp_healthy = state.mcp_bridge.health_check().await.unwrap_or(false);

    let response = HealthResponse {
        status: if mcp_healthy { "healthy" } else { "degraded" }.to_string(),
        session_count,
        mcp_server_url: state.mcp_bridge.server_url().to_string(),
        mcp_server_healthy: mcp_healthy,
    };

    let status_code = if mcp_healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    Ok((status_code, Json(response)))
}

/// Health check response
#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    session_count: usize,
    mcp_server_url: String,
    mcp_server_healthy: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::sse::SseConfig;

    #[test]
    fn test_server_creation() {
        let config = SseConfig::default();
        let server = SseServer::new(config);
        assert_eq!(server.config.port, 8080);
    }

    #[test]
    fn test_cors_configuration() {
        let config = SseConfig {
            cors_origins: vec!["*".to_string()],
            ..Default::default()
        };
        let server = SseServer::new(config);
        let _cors_layer = server.build_cors_layer();
        // CORS layer creation should not panic
    }

    #[test]
    fn test_messages_query_parsing() {
        use serde_urlencoded;

        let query_str = "session_id=abc123";
        let query: MessagesQuery = serde_urlencoded::from_str(query_str).unwrap();
        assert_eq!(query.session_id, "abc123");
    }

    #[tokio::test]
    async fn test_server_state_creation() {
        let config = SseConfig::default();
        let session_manager = Arc::new(SessionManager::default());
        let mcp_bridge = Arc::new(
            McpBridge::new("http://localhost:3000".to_string(), Duration::from_secs(30)).unwrap(),
        );
        let encoder = SseEncoder::new();

        let state = ServerState {
            session_manager,
            mcp_bridge,
            encoder,
            config,
        };

        assert_eq!(state.config.port, 8080);
    }
}
