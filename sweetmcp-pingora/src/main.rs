//! SweetMCP Server - Sugora Gateway
//!
//! A production-grade, multi-protocol edge proxy built on Pingora 0.5 that normalizes
//! GraphQL, JSON-RPC 2.0, and Cap'n Proto into Model Context Protocol (MCP) requests.

mod auth;
mod circuit_breaker;
mod config;
mod crypto;
mod dns_discovery;
mod edge;
mod load;
mod mcp_bridge;
mod mdns_discovery;
mod metric_picker;
mod metrics;
mod normalize;
mod peer_discovery;
mod rate_limit;
mod shutdown;
mod tls;

use anyhow::Result;
use config::Config;
use opentelemetry::global;
use opentelemetry_prometheus::PrometheusExporter;
use pingora::server::Server;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    env_logger::init();
    
    if let Err(e) = run_server() {
        eprintln!("🚫 SweetMCP Server failed to start: {}", e);
        std::process::exit(1);
    }
}

fn run_server() -> Result<()> {
    // Initialize structured logging
    init_logging()?;

    tracing::info!("🍬 Starting SweetMCP Server with Sugora Gateway");

    // Load configuration
    let cfg = Arc::new(Config::from_env()?);
    tracing::info!("✅ Configuration loaded successfully");

    // Initialize OpenTelemetry
    let _exporter = init_otel()?;
    tracing::info!("📊 OpenTelemetry initialized");

    // Setup MCP bridge
    let (bridge_tx, bridge_rx) = mpsc::channel::<mcp_bridge::BridgeMsg>(1024);

    // Create server with default options
    let mut server =
        Server::new(None).map_err(|e| anyhow::anyhow!("Failed to create Pingora server: {}", e))?;
    server.bootstrap();

    // Create peer registry
    let peer_registry = peer_discovery::PeerRegistry::new();

    // Extract port from TCP bind address
    let local_port = cfg
        .tcp_bind
        .split(':')
        .last()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8443);

    // Create background services
    let mcp_bridge = background_service("mcp-bridge", McpBridgeService { rx: Some(bridge_rx) });
    
    // Create discovery services based on configuration
    if let Some(service_name) = dns_discovery::should_use_dns_discovery() {
        let dns_discovery = dns_discovery::DnsDiscovery::new(
            service_name.clone(),
            peer_registry.clone(),
            None, // Use default DoH servers
        );
        let dns_service = background_service("dns-discovery", DnsDiscoveryService {
            service_name,
            discovery: dns_discovery,
        });
        server.add_service(dns_service);
    } else {
        // Fallback: mDNS for local network discovery
        let mdns_discovery = mdns_discovery::MdnsDiscovery::new(peer_registry.clone(), local_port);
        let mdns_service = background_service("mdns-discovery", MdnsDiscoveryService {
            discovery: mdns_discovery,
        });
        server.add_service(mdns_service);
    }

    // Always start HTTP-based peer exchange for mesh formation
    let discovery_service = peer_discovery::DiscoveryService::new(peer_registry.clone());
    let peer_service = background_service("peer-discovery", PeerDiscoveryService {
        service: discovery_service,
    });
    
    // Add background services
    server.add_service(mcp_bridge);
    server.add_service(peer_service);

    // Create HTTP proxy service
    let edge_service =
        edge::EdgeService::new(cfg.clone(), bridge_tx.clone(), peer_registry.clone());
    let mut proxy_service = pingora_proxy::http_proxy_service(&server.configuration, edge_service);

    // Add TCP listener
    proxy_service.add_tcp(&cfg.tcp_bind);

    // Add Unix socket listener
    // Ensure directory exists
    if let Some(parent) = std::path::Path::new(&cfg.uds_path).parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            tracing::warn!("Failed to create UDS directory {:?}: {}", parent, e);
        } else {
            tracing::info!("Created UDS directory {:?}", parent);
        }
    }
    
    // Remove old socket file if it exists
    if std::path::Path::new(&cfg.uds_path).exists() {
        if let Err(e) = std::fs::remove_file(&cfg.uds_path) {
            tracing::warn!("Failed to remove old socket file: {}", e);
        }
    }
    
    proxy_service.add_uds(&cfg.uds_path, None);

    // Add the proxy service to server
    server.add_service(proxy_service);

    // Setup Prometheus metrics service
    let mut prometheus_service = pingora::services::listening::Service::prometheus_http_service();
    prometheus_service.add_tcp(&cfg.metrics_bind);
    server.add_service(prometheus_service);

    // The exporter automatically registers with the default prometheus registry

    tracing::info!("🚀 Sugora Gateway ready!");
    tracing::info!("  TCP: {}", cfg.tcp_bind);
    tracing::info!("  UDS: {}", cfg.uds_path);
    tracing::info!("  Metrics: http://{}/metrics", cfg.metrics_bind);

    // Run the server - this never returns
    server.run_forever();
}

fn init_logging() -> Result<()> {
    // Get log level from environment or use INFO
    let log_level = std::env::var("SWEETMCP_LOG_LEVEL")
        .unwrap_or_else(|_| "info".to_string())
        .parse::<tracing::Level>()
        .unwrap_or(tracing::Level::INFO);

    let subscriber = tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
                .json(),
        )
        .with(
            tracing_subscriber::filter::EnvFilter::from_default_env()
                .add_directive(format!("sweetmcp_server={}", log_level).parse()?),
        );

    subscriber.init();
    Ok(())
}

fn init_otel() -> Result<PrometheusExporter> {
    let exporter = opentelemetry_prometheus::exporter().build()?;

    // Set up trace propagation
    global::set_text_map_propagator(opentelemetry_sdk::propagation::TraceContextPropagator::new());

    Ok(exporter)
}

// Background service implementations
use pingora::server::ShutdownWatch;
use pingora::services::background::{background_service, BackgroundService};
use std::pin::Pin;
use std::future::Future;

struct McpBridgeService {
    rx: Option<mpsc::Receiver<mcp_bridge::BridgeMsg>>,
}

impl BackgroundService for McpBridgeService {
    fn start<'life0, 'async_trait>(
        &'life0 self,
        mut shutdown: ShutdownWatch,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        // This is safe because we only call start once
        let rx = unsafe { 
            let this = self as *const Self as *mut Self;
            (*this).rx.take().expect("start called twice")
        };
        
        Box::pin(async move {
            tracing::info!("🔌 Starting MCP bridge");
            tokio::select! {
                _ = mcp_bridge::run(rx) => {
                    tracing::info!("MCP bridge stopped");
                }
                _ = shutdown.changed() => {
                    tracing::info!("MCP bridge shutting down");
                }
            }
        })
    }
}

struct DnsDiscoveryService {
    service_name: String,
    discovery: dns_discovery::DnsDiscovery,
}

impl BackgroundService for DnsDiscoveryService {
    fn start<'life0, 'async_trait>(
        &'life0 self,
        mut shutdown: ShutdownWatch,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        // We need to move the discovery out of self
        let service_name = self.service_name.clone();
        let discovery = unsafe {
            std::ptr::read(&self.discovery as *const dns_discovery::DnsDiscovery)
        };
        
        Box::pin(async move {
            tracing::info!("🌍 Starting DNS discovery for: {}", service_name);
            tokio::select! {
                _ = discovery.run() => {
                    tracing::info!("DNS discovery stopped");
                }
                _ = shutdown.changed() => {
                    tracing::info!("DNS discovery shutting down");
                }
            }
        })
    }
}

struct MdnsDiscoveryService {
    discovery: mdns_discovery::MdnsDiscovery,
}

impl BackgroundService for MdnsDiscoveryService {
    fn start<'life0, 'async_trait>(
        &'life0 self,
        mut shutdown: ShutdownWatch,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        // We need to move the discovery out of self
        let discovery = unsafe {
            std::ptr::read(&self.discovery as *const mdns_discovery::MdnsDiscovery)
        };
        
        Box::pin(async move {
            tracing::info!("🔍 Starting mDNS local discovery");
            tokio::select! {
                _ = discovery.run() => {
                    tracing::info!("mDNS discovery stopped");
                }
                _ = shutdown.changed() => {
                    tracing::info!("mDNS discovery shutting down");
                }
            }
        })
    }
}

struct PeerDiscoveryService {
    service: peer_discovery::DiscoveryService,
}

impl BackgroundService for PeerDiscoveryService {
    fn start<'life0, 'async_trait>(
        &'life0 self,
        mut shutdown: ShutdownWatch,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        // We need to move the service out of self
        let service = unsafe {
            std::ptr::read(&self.service as *const peer_discovery::DiscoveryService)
        };
        
        Box::pin(async move {
            tracing::info!("🔄 Starting HTTP peer exchange");
            tokio::select! {
                _ = service.run() => {
                    tracing::info!("Peer discovery stopped");
                }
                _ = shutdown.changed() => {
                    tracing::info!("Peer discovery shutting down");
                }
            }
        })
    }
}
