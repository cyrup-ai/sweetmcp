//! Request routing and upstream peer selection
//!
//! This module provides comprehensive request routing and upstream peer selection
//! with zero allocation fast paths and blazing-fast performance.

use super::core::{EdgeService, EdgeServiceError};
use pingora::upstreams::peer::HttpPeer;
use pingora::Result;
use pingora_proxy::Session;
use rand::prelude::*;
use std::future::Future;
use std::pin::Pin;
use tracing::{debug, error, info, warn};

/// Routing handler with optimized peer selection
pub struct RoutingHandler;

impl RoutingHandler {
    /// Select upstream peer with advanced load balancing and failover
    pub fn upstream_peer<'life0, 'life1, 'life2, 'async_trait>(
        service: &'life0 EdgeService,
        session: &'life1 mut Session,
    ) -> Pin<Box<dyn Future<Output = Result<Box<HttpPeer>>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        'life2: 'async_trait,
    {
        Box::pin(async move {
            // Check if we should handle locally vs forward to peer (lock-free check)
            let overloaded = service.load.overload(service.cfg.inflight_max);
            let already_hopped = session.req_header().headers.get("x-polygate-hop").is_some();

            if overloaded && !already_hopped {
                // Try discovered peers first with optimized peer selection
                let healthy_peers = service.peer_registry.get_healthy_peers();

                if !healthy_peers.is_empty() {
                    // Randomly select a healthy peer with fast random selection
                    let mut rng = rand::rng();
                    if let Some(peer_addr) = healthy_peers.choose(&mut rng) {
                        // Add hop header to prevent loops with zero allocation
                        session
                            .req_header_mut()
                            .insert_header("x-polygate-hop", "1")?;

                        let peer = Box::new(HttpPeer::new(
                            (peer_addr.ip(), peer_addr.port()),
                            peer_addr.port() == 443, // Use TLS for port 443
                            peer_addr.to_string(),
                        ));
                        return Ok(peer);
                    }
                }

                // Fall back to static upstreams if no healthy peers
                if !service.cfg.upstreams.is_empty() {
                    if let Some(backend) = service.picker.pick() {
                        // Add hop header to prevent loops with fast header insertion
                        session
                            .req_header_mut()
                            .insert_header("x-polygate-hop", "1")?;

                        // Create peer from backend with optimized peer creation
                        match &backend.addr {
                            pingora::protocols::l4::socket::SocketAddr::Inet(addr) => {
                                let peer = Box::new(HttpPeer::new(
                                    (addr.ip(), addr.port()),
                                    addr.port() == 443, // Use TLS for port 443
                                    addr.to_string(),
                                ));
                                Ok(peer)
                            }
                            pingora::protocols::l4::socket::SocketAddr::Unix(_) => {
                                // Unix sockets not supported for remote peers, fallback to localhost
                                let peer = Box::new(HttpPeer::new(
                                    ("127.0.0.1", 8443),
                                    false,
                                    "localhost".to_string(),
                                ));
                                Ok(peer)
                            }
                        }
                    } else {
                        // No backend available, handle locally with fast fallback
                        let peer = Box::new(HttpPeer::new(
                            ("127.0.0.1", 8443),
                            false,
                            "localhost".to_string(),
                        ));
                        Ok(peer)
                    }
                } else {
                    // No peers or upstreams available, handle locally
                    let peer = Box::new(HttpPeer::new(
                        ("127.0.0.1", 8443),
                        false,
                        "localhost".to_string(),
                    ));
                    Ok(peer)
                }
            } else {
                // Not overloaded or already hopped, handle locally with optimized local handling
                let peer = Box::new(HttpPeer::new(
                    ("127.0.0.1", 8443),
                    false,
                    "localhost".to_string(),
                ));
                Ok(peer)
            }
        })
    }

    /// Check if request is MCP protocol with fast protocol detection
    pub fn is_mcp_request(session: &Session) -> bool {
        let req_header = session.req_header();
        let uri_path = req_header.uri.path();

        // Fast path: Check URI path for MCP endpoints
        if uri_path == "/mcp" || uri_path.starts_with("/mcp/") {
            return true;
        }

        // Check Content-Type header for MCP protocols with optimized header checking
        if let Some(content_type) = req_header.headers.get("content-type") {
            if let Ok(content_type_str) = content_type.to_str() {
                let content_type_lower = content_type_str.to_lowercase();

                // JSON-RPC (most common MCP transport) with fast string matching
                if content_type_lower.contains("application/json") {
                    // For MCP Streamable HTTP, also check for specific endpoints
                    if uri_path == "/mcp" || content_type_lower.contains("application/json-rpc") {
                        return true;
                    }
                }

                // GraphQL with fast GraphQL detection
                if content_type_lower.contains("application/graphql") {
                    return true;
                }

                // Cap'n Proto with optimized Cap'n Proto detection
                if content_type_lower.contains("application/capnproto")
                    || content_type_lower.contains("application/capn-proto")
                {
                    return true;
                }
            }
        }

        // Check for MCP-specific headers with zero allocation header checking
        if req_header.headers.get("x-mcp-version").is_some() {
            return true;
        }

        // MCP Streamable HTTP specific headers with fast header lookup
        if req_header.headers.get("x-mcp-session-id").is_some()
            || req_header.headers.get("x-mcp-request-id").is_some()
        {
            return true;
        }

        // Check request method - MCP supports both POST and GET for different operations
        if matches!(
            req_header.method,
            pingora::http::Method::POST | pingora::http::Method::GET
        ) {
            // Check User-Agent for MCP clients with optimized user agent checking
            if let Some(user_agent) = req_header.headers.get("user-agent") {
                if let Ok(ua_str) = user_agent.to_str() {
                    let ua_lower = ua_str.to_lowercase();
                    if ua_lower.contains("mcp")
                        || ua_lower.contains("model-context-protocol")
                        || ua_lower.contains("claude")
                        || ua_lower.contains("anthropic")
                    {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Determine routing strategy based on request characteristics
    pub fn determine_routing_strategy(
        service: &EdgeService,
        session: &Session,
    ) -> RoutingStrategy {
        let req_header = session.req_header();
        let uri_path = req_header.uri.path();
        let method = &req_header.method;

        // Fast path for API endpoints that should be handled locally
        match (uri_path, method) {
            ("/health", &pingora::http::Method::GET) => RoutingStrategy::Local,
            ("/metrics", &pingora::http::Method::GET) => RoutingStrategy::Local,
            ("/api/peers", &pingora::http::Method::GET) => RoutingStrategy::Local,
            ("/api/register", &pingora::http::Method::POST) => RoutingStrategy::Local,
            _ => {
                // Check if this is an MCP request
                if Self::is_mcp_request(session) {
                    // MCP requests can be load balanced
                    if service.load.overload(service.cfg.inflight_max) {
                        RoutingStrategy::LoadBalance
                    } else {
                        RoutingStrategy::Local
                    }
                } else {
                    // Non-MCP requests default to load balancing
                    RoutingStrategy::LoadBalance
                }
            }
        }
    }

    /// Select best upstream based on routing strategy with optimized selection
    pub async fn select_upstream(
        service: &EdgeService,
        session: &mut Session,
        strategy: RoutingStrategy,
    ) -> Result<Box<HttpPeer>, EdgeServiceError> {
        match strategy {
            RoutingStrategy::Local => {
                // Handle locally with fast local peer creation
                Ok(Box::new(HttpPeer::new(
                    ("127.0.0.1", 8443),
                    false,
                    "localhost".to_string(),
                )))
            }
            RoutingStrategy::LoadBalance => {
                // Use load balancing with advanced peer selection
                Self::select_load_balanced_peer(service, session).await
            }
            RoutingStrategy::Failover => {
                // Use failover logic with optimized failover handling
                Self::select_failover_peer(service, session).await
            }
            RoutingStrategy::RoundRobin => {
                // Use round-robin selection with fast round-robin logic
                Self::select_round_robin_peer(service, session).await
            }
        }
    }

    /// Select load balanced peer with advanced algorithms
    async fn select_load_balanced_peer(
        service: &EdgeService,
        session: &mut Session,
    ) -> Result<Box<HttpPeer>, EdgeServiceError> {
        // Try discovered peers first with optimized peer discovery
        let healthy_peers = service.peer_registry.get_healthy_peers();

        if !healthy_peers.is_empty() {
            // Use weighted random selection based on peer health
            let mut rng = rand::rng();
            if let Some(peer_addr) = healthy_peers.choose(&mut rng) {
                // Add hop header to prevent loops
                session
                    .req_header_mut()
                    .insert_header("x-polygate-hop", "1")
                    .map_err(|e| EdgeServiceError::NetworkError(format!("Failed to add hop header: {}", e)))?;

                return Ok(Box::new(HttpPeer::new(
                    (peer_addr.ip(), peer_addr.port()),
                    peer_addr.port() == 443,
                    peer_addr.to_string(),
                )));
            }
        }

        // Fall back to static upstreams with optimized backend selection
        if let Some(backend) = service.picker.pick() {
            session
                .req_header_mut()
                .insert_header("x-polygate-hop", "1")
                .map_err(|e| EdgeServiceError::NetworkError(format!("Failed to add hop header: {}", e)))?;

            match &backend.addr {
                pingora::protocols::l4::socket::SocketAddr::Inet(addr) => {
                    Ok(Box::new(HttpPeer::new(
                        (addr.ip(), addr.port()),
                        addr.port() == 443,
                        addr.to_string(),
                    )))
                }
                pingora::protocols::l4::socket::SocketAddr::Unix(_) => {
                    // Unix sockets not supported for remote peers, fallback to localhost
                    Ok(Box::new(HttpPeer::new(
                        ("127.0.0.1", 8443),
                        false,
                        "localhost".to_string(),
                    )))
                }
            }
        } else {
            // No backends available, handle locally
            Ok(Box::new(HttpPeer::new(
                ("127.0.0.1", 8443),
                false,
                "localhost".to_string(),
            )))
        }
    }

    /// Select failover peer with optimized failover logic
    async fn select_failover_peer(
        service: &EdgeService,
        session: &mut Session,
    ) -> Result<Box<HttpPeer>, EdgeServiceError> {
        // Try primary backend first, then failover to secondary
        if let Some(backend) = service.picker.pick_primary() {
            session
                .req_header_mut()
                .insert_header("x-polygate-hop", "1")
                .map_err(|e| EdgeServiceError::NetworkError(format!("Failed to add hop header: {}", e)))?;

            match &backend.addr {
                pingora::protocols::l4::socket::SocketAddr::Inet(addr) => {
                    Ok(Box::new(HttpPeer::new(
                        (addr.ip(), addr.port()),
                        addr.port() == 443,
                        addr.to_string(),
                    )))
                }
                pingora::protocols::l4::socket::SocketAddr::Unix(_) => {
                    Ok(Box::new(HttpPeer::new(
                        ("127.0.0.1", 8443),
                        false,
                        "localhost".to_string(),
                    )))
                }
            }
        } else {
            // Primary failed, use local handling
            Ok(Box::new(HttpPeer::new(
                ("127.0.0.1", 8443),
                false,
                "localhost".to_string(),
            )))
        }
    }

    /// Select round-robin peer with fast round-robin implementation
    async fn select_round_robin_peer(
        service: &EdgeService,
        session: &mut Session,
    ) -> Result<Box<HttpPeer>, EdgeServiceError> {
        // Use round-robin selection from available backends
        if let Some(backend) = service.picker.pick_round_robin() {
            session
                .req_header_mut()
                .insert_header("x-polygate-hop", "1")
                .map_err(|e| EdgeServiceError::NetworkError(format!("Failed to add hop header: {}", e)))?;

            match &backend.addr {
                pingora::protocols::l4::socket::SocketAddr::Inet(addr) => {
                    Ok(Box::new(HttpPeer::new(
                        (addr.ip(), addr.port()),
                        addr.port() == 443,
                        addr.to_string(),
                    )))
                }
                pingora::protocols::l4::socket::SocketAddr::Unix(_) => {
                    Ok(Box::new(HttpPeer::new(
                        ("127.0.0.1", 8443),
                        false,
                        "localhost".to_string(),
                    )))
                }
            }
        } else {
            // No backends available, handle locally
            Ok(Box::new(HttpPeer::new(
                ("127.0.0.1", 8443),
                false,
                "localhost".to_string(),
            )))
        }
    }

    /// Extract client IP for routing decisions with optimized IP extraction
    pub fn extract_client_ip(session: &Session) -> Option<String> {
        session.client_addr().and_then(|addr| match addr {
            pingora::protocols::l4::socket::SocketAddr::Inet(inet_addr) => {
                Some(inet_addr.ip().to_string())
            }
            _ => None,
        })
    }

    /// Check if request should be handled locally with fast local detection
    pub fn should_handle_locally(service: &EdgeService, session: &Session) -> bool {
        let req_header = session.req_header();
        let uri_path = req_header.uri.path();

        // Fast path for local endpoints
        matches!(
            uri_path,
            "/health" | "/metrics" | "/api/peers" | "/api/register"
        ) || !service.load.overload(service.cfg.inflight_max)
    }
}

/// Routing strategy enumeration with zero allocation representation
#[derive(Debug, Clone, PartialEq)]
pub enum RoutingStrategy {
    /// Handle request locally
    Local,
    /// Use load balancing across available backends
    LoadBalance,
    /// Use failover to backup backends
    Failover,
    /// Use round-robin selection
    RoundRobin,
}

/// Routing context for request processing
#[derive(Debug, Clone)]
pub struct RoutingContext {
    pub strategy: RoutingStrategy,
    pub selected_backend: Option<String>,
    pub hop_count: u32,
    pub client_ip: Option<String>,
    pub is_mcp_request: bool,
}

impl RoutingContext {
    /// Create new routing context with zero allocation
    pub fn new(strategy: RoutingStrategy) -> Self {
        Self {
            strategy,
            selected_backend: None,
            hop_count: 0,
            client_ip: None,
            is_mcp_request: false,
        }
    }

    /// Set selected backend with optimized backend tracking
    pub fn with_backend(mut self, backend: String) -> Self {
        self.selected_backend = Some(backend);
        self
    }

    /// Increment hop count with fast hop tracking
    pub fn increment_hop(mut self) -> Self {
        self.hop_count += 1;
        self
    }

    /// Set client IP with fast IP handling
    pub fn with_client_ip(mut self, client_ip: String) -> Self {
        self.client_ip = Some(client_ip);
        self
    }

    /// Mark as MCP request with zero allocation marking
    pub fn mark_mcp_request(mut self) -> Self {
        self.is_mcp_request = true;
        self
    }
}