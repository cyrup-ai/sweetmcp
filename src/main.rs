//! SweetMCP Server - Sugora Gateway
//! 
//! A production-grade, multi-protocol edge proxy built on Pingora 0.5 that normalizes
//! GraphQL, JSON-RPC 2.0, and Cap'n Proto into Model Context Protocol (MCP) requests.

mod config;
mod auth;
mod normalize;
mod load;
mod metric_picker;
mod mcp_bridge;
mod edge;
mod peer_discovery;
mod dns_discovery;
mod mdns_discovery;
mod metrics;
mod tls;
mod crypto;
mod circuit_breaker;
mod shutdown;
mod rate_limit;

use anyhow::Result;
use config::Config;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use opentelemetry::global;
use opentelemetry_prometheus::PrometheusExporter;
use pingora::server::Server;
use std::sync::Arc;
use tokio::sync::mpsc;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    if let Err(e) = run_server().await {
        eprintln!("🚫 SweetMCP Server failed to start: {}", e);
        std::process::exit(1);
    }
}

async fn run_server() -> Result<()> {
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
    tokio::spawn(async move { 
        tracing::info!("🔌 Starting MCP bridge");
        mcp_bridge::run(bridge_rx).await 
    });
    
    // Create server with default options
    let mut server = Server::new(None)
        .map_err(|e| anyhow::anyhow!("Failed to create Pingora server: {}", e))?;
    server.bootstrap();
    
    // Create peer registry
    let peer_registry = peer_discovery::PeerRegistry::new();
    
    // Extract port from TCP bind address
    let local_port = cfg.tcp_bind.split(':')
        .last()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8443);
    
    // Primary discovery: DNS-based (if configured)
    if let Some(service_name) = dns_discovery::should_use_dns_discovery() {
        let dns_discovery = dns_discovery::DnsDiscovery::new(
            service_name.clone(),
            peer_registry.clone(),
            None, // Use default DoH servers
        );
        tokio::spawn(async move {
            tracing::info!("🌍 Starting DNS discovery for: {}", service_name);
            dns_discovery.run().await;
        });
    } else {
        // Fallback: mDNS for local network discovery
        let mdns_discovery = mdns_discovery::MdnsDiscovery::new(peer_registry.clone(), local_port);
        tokio::spawn(async move {
            tracing::info!("🔍 Starting mDNS local discovery");
            mdns_discovery.run().await;
        });
    }
    
    // Always start HTTP-based peer exchange for mesh formation
    let discovery_service = peer_discovery::DiscoveryService::new(peer_registry.clone());
    tokio::spawn(async move {
        tracing::info!("🔄 Starting HTTP peer exchange");
        discovery_service.run().await;
    });
    
    // Create HTTP proxy service
    let edge_service = edge::EdgeService::new(cfg.clone(), bridge_tx.clone(), peer_registry.clone());
    let mut proxy_service = pingora_proxy::http_proxy_service(&server.configuration, edge_service);
    
    // Add TCP listener
    proxy_service.add_tcp(&cfg.tcp_bind);
    
    // Add Unix socket listener
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
                .json()
        )
        .with(
            tracing_subscriber::filter::EnvFilter::from_default_env()
                .add_directive(format!("sweetmcp_server={}", log_level).parse()?)
        );
    
    subscriber.init();
    Ok(())
}

fn init_otel() -> Result<PrometheusExporter> {
    let exporter = opentelemetry_prometheus::exporter()
        .build()?;
    
    // Set up trace propagation
    global::set_text_map_propagator(
        opentelemetry_sdk::propagation::TraceContextPropagator::new(),
    );
    
    Ok(exporter)
}