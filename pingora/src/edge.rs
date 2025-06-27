//! Sugora EdgeService: auth, overload, routing.

use crate::{
    auth::JwtAuth,
    config::Config,
    load::Load,
    metric_picker::MetricPicker,
    peer_discovery::{PeerRegistry, PeersResponse, RegisterRequest, BUILD_ID},
    metrics,
    rate_limit::AdvancedRateLimitManager,
};
use bytes::Bytes;
use pingora::upstreams::peer::HttpPeer;
use pingora::Result;
use pingora_proxy::{ProxyHttp, Session};
use pingora_load_balancing::Backend;
use std::collections::BTreeSet;
use std::{sync::{Arc, Mutex}};
use std::pin::Pin;
use std::future::Future;
use tokio::sync::mpsc::Sender;
use pingora::http::Method;
use rand::prelude::*;

pub struct EdgeService {
    cfg: Arc<Config>,
    auth: JwtAuth,
    picker: MetricPicker,
    load: Arc<Mutex<Load>>,
    #[allow(dead_code)]
    bridge_tx: Sender<crate::mcp_bridge::BridgeMsg>,
    peer_registry: PeerRegistry,
    rate_limit_manager: Arc<AdvancedRateLimitManager>,
}

impl EdgeService {
    pub fn new(cfg: Arc<Config>, bridge_tx: Sender<crate::mcp_bridge::BridgeMsg>, peer_registry: PeerRegistry) -> Self {
        // Create Backend objects from upstream URLs
        let backends: BTreeSet<Backend> = cfg.upstreams
            .iter()
            .filter_map(|url| {
                // Parse URL to extract host:port
                if let Ok(parsed) = url.parse::<url::Url>() {
                    if let Some(host) = parsed.host_str() {
                        let port = parsed.port().unwrap_or(80);
                        Backend::new(&format!("{}:{}", host, port)).ok()
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
        
        // Advanced rate limiting with token bucket and sliding window algorithms
        let rate_limit_manager = Arc::new(AdvancedRateLimitManager::new());
        
        // Start cleanup task for unused peer limiters
        rate_limit_manager.clone().start_cleanup_task();
            
        Self {
            auth: JwtAuth::new(cfg.jwt_secret.clone(), cfg.jwt_expiry),
            picker: MetricPicker::from_backends(&backends),
            load: Arc::new(Mutex::new(Load::new())),
            peer_registry,
            rate_limit_manager,
            cfg,
            bridge_tx,
        }
    }
}

impl EdgeService {
    fn validate_discovery_token(&self, token: &str) -> bool {
        if let Ok(expected_token) = std::env::var("SWEETMCP_DISCOVERY_TOKEN") {
            !expected_token.is_empty() && token == expected_token
        } else {
            false // No token = no discovery
        }
    }
    
    /// Record HTTP metrics and decrement active request counters
    fn record_http_metrics_and_cleanup(&self, ctx: &HttpMetricsContext, status_code: u16, response_size: usize) {
        if let (Some(start_time), Some(method), Some(endpoint)) = (&ctx.start_time, &ctx.method, &ctx.endpoint) {
            let duration = start_time.elapsed().as_secs_f64();
            
            // Record comprehensive HTTP metrics
            metrics::record_http_request(
                method,
                endpoint,
                status_code,
                duration,
                ctx.request_size,
                response_size,
            );
            
            // Decrement active request counters
            metrics::decrement_active_requests(method, endpoint);
        }
        
        // Decrement load counter
        match self.load.lock() {
            Ok(load) => load.dec(),
            Err(poisoned) => {
                tracing::warn!("Load counter mutex poisoned on decrement, recovering");
                poisoned.into_inner().dec();
            }
        }
    }
}

/// HTTP request context for metrics tracking and protocol conversion
#[derive(Default)]
pub struct HttpMetricsContext {
    pub start_time: Option<std::time::Instant>,
    pub request_size: usize,
    pub method: Option<String>,
    pub endpoint: Option<String>,
    pub protocol_context: Option<crate::normalize::ProtocolContext>,
}

impl ProxyHttp for EdgeService {
    type CTX = HttpMetricsContext;

    fn new_ctx(&self) -> Self::CTX {
        HttpMetricsContext::default()
    }

    fn request_filter<'life0, 'life1, 'life2, 'async_trait>(
        &'life0 self,
        session: &'life1 mut Session,
        ctx: &'life2 mut Self::CTX,
    ) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        'life2: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
        // HTTP Metrics Middleware - Capture request start and metadata
        ctx.start_time = Some(std::time::Instant::now());
        ctx.method = Some(session.req_header().method.to_string());
        ctx.endpoint = Some(session.req_header().uri.path().to_string());
        
        // Estimate request size from headers and body length
        let headers_size = session.req_header().headers.iter()
            .map(|(name, value)| name.as_str().len() + value.as_bytes().len() + 4) // +4 for ": " and "\r\n"
            .sum::<usize>();
        
        let body_size = if let Some(content_length) = session.req_header().headers.get("content-length") {
            content_length.to_str().unwrap_or("0").parse::<usize>().unwrap_or(0)
        } else {
            0
        };
        
        ctx.request_size = headers_size + body_size + session.req_header().uri.to_string().len();
        
        // Increment active request metrics
        if let (Some(method), Some(endpoint)) = (&ctx.method, &ctx.endpoint) {
            metrics::increment_active_requests(method, endpoint);
        }
        
        match self.load.lock() {
            Ok(load) => load.inc(),
            Err(poisoned) => {
                tracing::warn!("Load counter mutex poisoned on increment, recovering");
                poisoned.into_inner().inc();
            }
        }

        // Check for hop header to prevent infinite forwarding
        let _already_hopped = session
            .req_header()
            .headers
            .get("x-polygate-hop")
            .is_some();

        // Check if this is an API endpoint that doesn't require auth
        let path = session.req_header().uri.path();
        let method = session.req_header().method.clone();
        
        // Handle /api/peers GET endpoint
        if path == "/api/peers" && method == Method::GET {
            let start = std::time::Instant::now();
            
            // Advanced rate limiting with token bucket algorithm
            let client_ip = session.client_addr().and_then(|addr| {
                match addr {
                    pingora::protocols::l4::socket::SocketAddr::Inet(inet_addr) => {
                        Some(inet_addr.ip().to_string())
                    }
                    _ => None,
                }
            });
            
            if !self.rate_limit_manager.check_request("/api/peers", client_ip.as_deref(), 1) {
                let response_body = b"Rate limit exceeded";
                let _ = session
                    .respond_error_with_body(429, Bytes::from_static(response_body))
                    .await;
                self.record_http_metrics_and_cleanup(ctx, 429, response_body.len());
                return Ok(true);
            }
            
            // Check discovery token
            let discovery_token = session
                .req_header()
                .headers
                .get("x-discovery-token")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("");
                
            if !self.validate_discovery_token(discovery_token) {
                let response_body = b"Invalid discovery token";
                let _ = session
                    .respond_error_with_body(401, Bytes::from_static(response_body))
                    .await;
                self.record_http_metrics_and_cleanup(ctx, 401, response_body.len());
                return Ok(true);
            }
            let peers: Vec<String> = self.peer_registry.get_all_peers()
                .into_iter()
                .map(|addr| addr.to_string())
                .collect();
                
            let response = PeersResponse {
                build_id: BUILD_ID.to_string(),
                peers,
            };
            
            let body = serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string());
            let body_bytes = Bytes::from(body);
            
            let mut resp_header = pingora::http::ResponseHeader::build(200, None)?;
            resp_header.insert_header("Content-Type", "application/json")?;
            resp_header.insert_header("Content-Length", body_bytes.len().to_string())?;
            
            session.write_response_header(Box::new(resp_header), false).await?;
            session.write_response_body(Some(body_bytes.clone()), true).await?;
            
            crate::metrics::record_discovery("get_peers", true, start.elapsed().as_secs_f64());
            self.record_http_metrics_and_cleanup(ctx, 200, body_bytes.len());
            return Ok(true); // Response written
        }
        
        // Handle /api/register POST endpoint
        if path == "/api/register" && method == Method::POST {
            // Advanced rate limiting with sliding window algorithm (per-peer)
            let client_ip = session.client_addr().and_then(|addr| {
                match addr {
                    pingora::protocols::l4::socket::SocketAddr::Inet(inet_addr) => {
                        Some(inet_addr.ip().to_string())
                    }
                    _ => None,
                }
            });
            
            if !self.rate_limit_manager.check_request("/api/register", client_ip.as_deref(), 1) {
                let response_body = b"Rate limit exceeded";
                let _ = session
                    .respond_error_with_body(429, Bytes::from_static(response_body))
                    .await;
                self.record_http_metrics_and_cleanup(ctx, 429, response_body.len());
                return Ok(true);
            }
            
            // Check discovery token
            let discovery_token = session
                .req_header()
                .headers
                .get("x-discovery-token")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("");
                
            if !self.validate_discovery_token(discovery_token) {
                let response_body = b"Invalid discovery token";
                let _ = session
                    .respond_error_with_body(401, Bytes::from_static(response_body))
                    .await;
                self.record_http_metrics_and_cleanup(ctx, 401, response_body.len());
                return Ok(true);
            }
            // Read the request body
            let body = match session.read_request_body().await {
                Ok(Some(body)) => body,
                Ok(None) => Bytes::new(),
                Err(_) => {
                    let response_body = b"Failed to read body";
                    let _ = session
                        .respond_error_with_body(400, Bytes::from_static(response_body))
                        .await;
                    self.record_http_metrics_and_cleanup(ctx, 400, response_body.len());
                    return Ok(true);
                }
            };
            
            // Parse the JSON body
            let request: RegisterRequest = match serde_json::from_slice(&body) {
                Ok(req) => req,
                Err(_) => {
                    let response_body = b"Invalid JSON";
                    let _ = session
                        .respond_error_with_body(400, Bytes::from_static(response_body))
                        .await;
                    self.record_http_metrics_and_cleanup(ctx, 400, response_body.len());
                    return Ok(true);
                }
            };
            
            // Check if build_id matches
            if request.build_id != BUILD_ID {
                let error_msg = format!(
                    "Build ID mismatch: expected '{}', got '{}'", 
                    BUILD_ID, request.build_id
                );
                let response_bytes = Bytes::from(error_msg);
                let response_size = response_bytes.len();
                let _ = session
                    .respond_error_with_body(409, response_bytes)
                    .await;
                self.record_http_metrics_and_cleanup(ctx, 409, response_size);
                return Ok(true);
            }
            
            // Parse the peer address
            let peer_addr = match request.peer.parse::<std::net::SocketAddr>() {
                Ok(addr) => addr,
                Err(_) => {
                    let response_body = b"Invalid peer address";
                    let _ = session
                        .respond_error_with_body(400, Bytes::from_static(response_body))
                        .await;
                    self.record_http_metrics_and_cleanup(ctx, 400, response_body.len());
                    return Ok(true);
                }
            };
            
            // Add the peer to the registry
            let added = self.peer_registry.add_peer(peer_addr);
            
            // Return success response
            let response_body = if added {
                r#"{"status":"added"}"#
            } else {
                r#"{"status":"already_registered"}"#
            };
            
            let mut resp_header = pingora::http::ResponseHeader::build(200, None)?;
            resp_header.insert_header("Content-Type", "application/json")?;
            resp_header.insert_header("Content-Length", response_body.len().to_string())?;
            
            session.write_response_header(Box::new(resp_header), false).await?;
            session.write_response_body(Some(Bytes::from_static(response_body.as_bytes())), true).await?;
            self.record_http_metrics_and_cleanup(ctx, 200, response_body.len());
            return Ok(true);
        }
        
        if path == "/api/peers" || path == "/api/register" {
            // Wrong method for these endpoints
            let response_body = b"Method not allowed";
            let _ = session
                .respond_error_with_body(405, Bytes::from_static(response_body))
                .await;
            self.record_http_metrics_and_cleanup(ctx, 405, response_body.len());
            return Ok(true);
        }

        // Authentication check for other endpoints
        let auth_hdr = session
            .req_header()
            .headers
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("");
            
        let claims = match self.auth.verify(auth_hdr) {
            Ok(c) => c,
            Err(_) => {
                let response_body = b"Unauthorized";
                let _ = session
                    .respond_error_with_body(401, Bytes::from_static(response_body))
                    .await;
                self.record_http_metrics_and_cleanup(ctx, 401, response_body.len());
                return Ok(true); // Early return - response written
            }
        };

        // Check if this is an MCP request
        if is_mcp_request(session.req_header()) {
            // Read request body
            let body = match session.read_request_body().await {
                Ok(Some(body)) => body,
                Ok(None) => Bytes::new(),
                Err(e) => {
                    tracing::error!("Failed to read request body: {}", e);
                    let response_body = b"Failed to read request body";
                    let _ = session
                        .respond_error_with_body(400, Bytes::from_static(response_body))
                        .await;
                    self.record_http_metrics_and_cleanup(ctx, 400, response_body.len());
                    return Ok(true);
                }
            };

            // Normalize protocol to JSON-RPC
            let (protocol_ctx, json_rpc_request) = match crate::normalize::to_json_rpc(&claims.sub, &body) {
                Ok(result) => result,
                Err(e) => {
                    tracing::error!("Failed to normalize protocol: {}", e);
                    let error_msg = format!("Invalid protocol: {}", e);
                    let _ = session
                        .respond_error_with_body(400, Bytes::from(error_msg.as_bytes()))
                        .await;
                    self.record_http_metrics_and_cleanup(ctx, 400, error_msg.len());
                    return Ok(true);
                }
            };

            // Store protocol context for response conversion
            ctx.protocol_context = Some(protocol_ctx.clone());

            // Send to MCP bridge
            let (tx, rx) = tokio::sync::oneshot::channel();
            let bridge_msg = (json_rpc_request, protocol_ctx, tx);
            
            if let Err(e) = self.bridge_tx.send(bridge_msg).await {
                tracing::error!("Failed to send to MCP bridge: {}", e);
                let response_body = b"Internal server error";
                let _ = session
                    .respond_error_with_body(500, Bytes::from_static(response_body))
                    .await;
                self.record_http_metrics_and_cleanup(ctx, 500, response_body.len());
                return Ok(true);
            }

            // Await response from bridge
            match rx.await {
                Ok(json_rpc_response) => {
                    // Convert response back to original protocol
                    let response_bytes = match crate::normalize::from_json_rpc(
                        ctx.protocol_context.as_ref().unwrap(), 
                        &json_rpc_response
                    ) {
                        Ok(bytes) => bytes,
                        Err(e) => {
                            tracing::error!("Failed to convert response: {}", e);
                            let error_msg = format!("Response conversion error: {}", e);
                            let _ = session
                                .respond_error_with_body(500, Bytes::from(error_msg.as_bytes()))
                                .await;
                            self.record_http_metrics_and_cleanup(ctx, 500, error_msg.len());
                            return Ok(true);
                        }
                    };

                    // Determine content type based on protocol
                    let content_type = match &ctx.protocol_context.as_ref().unwrap().protocol {
                        crate::normalize::Proto::GraphQL => "application/json",
                        crate::normalize::Proto::JsonRpc => "application/json",
                        crate::normalize::Proto::Capnp => "application/octet-stream",
                    };

                    // Write response
                    let mut resp_header = pingora::http::ResponseHeader::build(200, None)?;
                    resp_header.insert_header("Content-Type", content_type)?;
                    resp_header.insert_header("Content-Length", response_bytes.len().to_string())?;
                    
                    session.write_response_header(Box::new(resp_header), false).await?;
                    session.write_response_body(Some(Bytes::from(response_bytes)), true).await?;
                    
                    self.record_http_metrics_and_cleanup(ctx, 200, response_bytes.len());
                    return Ok(true); // Request handled
                }
                Err(_) => {
                    tracing::error!("MCP bridge response channel closed");
                    let response_body = b"Internal server error";
                    let _ = session
                        .respond_error_with_body(500, Bytes::from_static(response_body))
                        .await;
                    self.record_http_metrics_and_cleanup(ctx, 500, response_body.len());
                    return Ok(true);
                }
            }
        }

        // Continue to upstream_peer for routing logic
        Ok(false) // Continue processing
        })
    }

    // Note: response_filter would be ideal for capturing proxied response metrics,
    // but the exact Pingora trait signature needs verification. Metrics
    // capture is complete for locally handled requests (API endpoints).

    fn upstream_peer<'life0, 'life1, 'life2, 'async_trait>(
        &'life0 self,
        session: &'life1 mut Session,
        _ctx: &'life2 mut Self::CTX,
    ) -> Pin<Box<dyn Future<Output = Result<Box<HttpPeer>>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        'life2: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
        // Check if we should handle locally vs forward to peer
        let overloaded = match self.load.lock() {
            Ok(mut load) => load.overload(self.cfg.inflight_max),
            Err(poisoned) => {
                tracing::warn!("Load counter mutex poisoned on overload check, recovering");
                poisoned.into_inner().overload(self.cfg.inflight_max)
            }
        };
        let already_hopped = session
            .req_header()
            .headers
            .get("x-polygate-hop")
            .is_some();

        if overloaded && !already_hopped {
            // Try discovered peers first
            let healthy_peers = self.peer_registry.get_healthy_peers();
            
            if !healthy_peers.is_empty() {
                // Randomly select a healthy peer
                let mut rng = rand::rng();
                if let Some(peer_addr) = healthy_peers.choose(&mut rng) {
                    // Add hop header to prevent loops
                    session.req_header_mut().insert_header("x-polygate-hop", "1")?;
                    
                    let peer = Box::new(HttpPeer::new(
                        (peer_addr.ip(), peer_addr.port()),
                        peer_addr.port() == 443, // Use TLS for port 443
                        peer_addr.to_string(),
                    ));
                    return Ok(peer);
                }
            }
            
            // Fall back to static upstreams if no healthy peers
            if !self.cfg.upstreams.is_empty() {
                if let Some(backend) = self.picker.pick() {
                    // Add hop header to prevent loops
                    session.req_header_mut().insert_header("x-polygate-hop", "1")?;
                    
                    // Create peer from backend
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
                    // No backend available, handle locally
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
            // Handle locally - return localhost peer
            let peer = Box::new(HttpPeer::new(
                ("127.0.0.1", 8443),
                false,
                "localhost".to_string(),
            ));
            Ok(peer)
        }
        })
    }
}