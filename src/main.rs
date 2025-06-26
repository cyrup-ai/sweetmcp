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

use anyhow::Result;
use config::Config;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use opentelemetry::{global, KeyValue};
use opentelemetry_sdk::Resource;
use opentelemetry_prometheus::PrometheusExporter;
use pingora::server::{configuration::Opt, Server};
use std::sync::Arc;
use tokio::sync::mpsc;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
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
    let mut server = Server::new(None).unwrap();
    server.bootstrap();
    
    // Create HTTP proxy service
    let edge_service = edge::EdgeService::new(cfg.clone(), bridge_tx.clone());
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
    
    // Run the server
    server.run_forever();
    
    tracing::info!("🛑 SweetMCP Server shutdown complete");
    Ok(())
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