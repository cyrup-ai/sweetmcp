//! Sugora EdgeService: auth, overload, routing.

use crate::{
    auth::JwtAuth,
    config::Config,
    load::Load,
    metric_picker::MetricPicker,
};
use bytes::Bytes;
use pingora::upstreams::peer::HttpPeer;
use pingora::Result;
use pingora_proxy::{ProxyHttp, Session};
use pingora_load_balancing::Backend;
use std::collections::BTreeSet;
use std::{sync::{Arc, Mutex}, time::Instant};
use std::pin::Pin;
use std::future::Future;
use tokio::sync::{mpsc::Sender, oneshot};

pub struct EdgeService {
    cfg: Arc<Config>,
    auth: JwtAuth,
    picker: MetricPicker,
    load: Arc<Mutex<Load>>,
    bridge_tx: Sender<crate::mcp_bridge::BridgeMsg>,
}

impl EdgeService {
    pub fn new(cfg: Arc<Config>, bridge_tx: Sender<crate::mcp_bridge::BridgeMsg>) -> Self {
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
            
        Self {
            auth: JwtAuth::new(cfg.jwt_secret.clone(), cfg.jwt_expiry),
            picker: MetricPicker::from_backends(&backends),
            load: Arc::new(Mutex::new(Load::new())),
            cfg,
            bridge_tx,
        }
    }
}

impl ProxyHttp for EdgeService {
    type CTX = ();

    fn new_ctx(&self) -> Self::CTX {}

    fn request_filter<'life0, 'life1, 'life2, 'async_trait>(
        &'life0 self,
        session: &'life1 mut Session,
        _ctx: &'life2 mut Self::CTX,
    ) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        'life2: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
        self.load.lock().unwrap().inc();

        // Check for hop header to prevent infinite forwarding
        let _already_hopped = session
            .req_header()
            .headers
            .get("x-polygate-hop")
            .is_some();

        // Authentication check
        let auth_hdr = session
            .req_header()
            .headers
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("");
            
        let _claims = match self.auth.verify(auth_hdr) {
            Ok(c) => c,
            Err(_) => {
                let _ = session
                    .respond_error_with_body(401, Bytes::from_static(b"Unauthorized"))
                    .await;
                self.load.lock().unwrap().dec();
                return Ok(true); // Early return - response written
            }
        };

        // For now, let this continue to upstream_peer for routing logic
        Ok(false) // Continue processing
        })
    }

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
        let overloaded = self.load.lock().unwrap().overload(self.cfg.inflight_max);
        let already_hopped = session
            .req_header()
            .headers
            .get("x-polygate-hop")
            .is_some();

        if overloaded && !already_hopped && !self.cfg.upstreams.is_empty() {
            // Forward to a peer - use the metric picker
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