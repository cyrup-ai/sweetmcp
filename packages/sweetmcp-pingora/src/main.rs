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

fn main() {
    env_logger::init();

    if let Err(e) = run_server() {
        eprintln!("🚫 SweetMCP Server failed to start: {}", e);
        std::process::exit(1);
    }
}

fn run_server() -> Result<()> {
    log::info!("🍬 Starting SweetMCP Server with Sugora Gateway");

    // Load configuration
    let cfg = Arc::new(Config::from_env()?);
    log::info!("✅ Configuration loaded successfully");

    // Initialize OpenTelemetry
    let _exporter = init_otel()?;
    log::info!("📊 OpenTelemetry initialized");

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
    let mcp_bridge = background_service(
        "mcp-bridge",
        McpBridgeService {
            rx: Some(bridge_rx),
        },
    );

    // Create discovery services based on configuration
    if let Some(service_name) = dns_discovery::should_use_dns_discovery() {
        let dns_discovery = dns_discovery::DnsDiscovery::new(
            service_name.clone(),
            peer_registry.clone(),
            None, // Use default DoH servers
        );
        let dns_service = background_service(
            "dns-discovery",
            DnsDiscoveryService {
                service_name,
                discovery: dns_discovery,
            },
        );
        server.add_service(dns_service);
    } else {
        // Fallback: mDNS for local network discovery
        let mdns_discovery = mdns_discovery::MdnsDiscovery::new(peer_registry.clone(), local_port);
        let mdns_service = background_service(
            "mdns-discovery",
            MdnsDiscoveryService {
                discovery: mdns_discovery,
            },
        );
        server.add_service(mdns_service);
    }

    // Always start HTTP-based peer exchange for mesh formation
    let discovery_service = peer_discovery::DiscoveryService::new(peer_registry.clone());
    let peer_service = background_service(
        "peer-discovery",
        PeerDiscoveryService {
            service: discovery_service,
        },
    );

    // Add background services
    server.add_service(mcp_bridge);
    server.add_service(peer_service);

    // Create HTTP proxy service
    let edge_service =
        edge::EdgeService::new(cfg.clone(), bridge_tx.clone(), peer_registry.clone());

    // Add rate limit cleanup service
    let rate_limit_service = background_service(
        "rate-limit-cleanup",
        RateLimitCleanupService {
            rate_limiter: edge_service.rate_limiter(),
        },
    );
    server.add_service(rate_limit_service);

    // Add metrics collector service
    let metrics_service = background_service(
        "metrics-collector",
        MetricsCollectorService {
            metric_picker: edge_service.metric_picker(),
        },
    );
    server.add_service(metrics_service);

    let mut proxy_service = pingora_proxy::http_proxy_service(&server.configuration, edge_service);

    // Add TCP listeners
    proxy_service.add_tcp(&cfg.tcp_bind);
    proxy_service.add_tcp(&cfg.mcp_bind);

    // Add Unix socket listener
    // Ensure directory exists
    if let Some(parent) = std::path::Path::new(&cfg.uds_path).parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            log::warn!("Failed to create UDS directory {:?}: {}", parent, e);
        } else {
            log::info!("Created UDS directory {:?}", parent);
        }
    }

    // Remove old socket file if it exists
    if std::path::Path::new(&cfg.uds_path).exists() {
        if let Err(e) = std::fs::remove_file(&cfg.uds_path) {
            log::warn!("Failed to remove old socket file: {}", e);
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

    log::info!("🚀 Sugora Gateway ready!");
    log::info!("  TCP: {}", cfg.tcp_bind);
    log::info!("  MCP HTTP: {}", cfg.mcp_bind);
    log::info!("  UDS: {}", cfg.uds_path);
    log::info!("  Metrics: http://{}/metrics", cfg.metrics_bind);

    // Run the server - this never returns
    server.run_forever();
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
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

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
            log::info!("🔌 Starting MCP bridge");
            tokio::select! {
                _ = mcp_bridge::run(rx) => {
                    log::info!("MCP bridge stopped");
                }
                _ = shutdown.changed() => {
                    log::info!("MCP bridge shutting down");
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
        let discovery =
            unsafe { std::ptr::read(&self.discovery as *const dns_discovery::DnsDiscovery) };

        Box::pin(async move {
            log::info!("🌍 Starting DNS discovery for: {}", service_name);
            tokio::select! {
                _ = discovery.run() => {
                    log::info!("DNS discovery stopped");
                }
                _ = shutdown.changed() => {
                    log::info!("DNS discovery shutting down");
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
        let discovery =
            unsafe { std::ptr::read(&self.discovery as *const mdns_discovery::MdnsDiscovery) };

        Box::pin(async move {
            log::info!("🔍 Starting mDNS local discovery");
            tokio::select! {
                _ = discovery.run() => {
                    log::info!("mDNS discovery stopped");
                }
                _ = shutdown.changed() => {
                    log::info!("mDNS discovery shutting down");
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
        let service =
            unsafe { std::ptr::read(&self.service as *const peer_discovery::DiscoveryService) };

        Box::pin(async move {
            log::info!("🔄 Starting HTTP peer exchange");
            tokio::select! {
                _ = service.run() => {
                    log::info!("Peer discovery stopped");
                }
                _ = shutdown.changed() => {
                    log::info!("Peer discovery shutting down");
                }
            }
        })
    }
}

struct RateLimitCleanupService {
    rate_limiter: Arc<rate_limit::AdvancedRateLimitManager>,
}

impl BackgroundService for RateLimitCleanupService {
    fn start<'life0, 'async_trait>(
        &'life0 self,
        mut shutdown: ShutdownWatch,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        let rate_limiter = self.rate_limiter.clone();

        Box::pin(async move {
            log::info!("🧹 Starting rate limit cleanup service");
            let mut cleanup_interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes

            loop {
                tokio::select! {
                    _ = cleanup_interval.tick() => {
                        rate_limiter.cleanup_unused_limiters();
                    }
                    _ = shutdown.changed() => {
                        log::info!("Rate limit cleanup shutting down");
                        break;
                    }
                }
            }
        })
    }
}

struct MetricsCollectorService {
    metric_picker: Arc<metric_picker::MetricPicker>,
}

impl BackgroundService for MetricsCollectorService {
    fn start<'life0, 'async_trait>(
        &'life0 self,
        mut shutdown: ShutdownWatch,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        let metric_picker = self.metric_picker.clone();

        Box::pin(async move {
            log::info!("📊 Starting metrics collector service");
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(2))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new());

            let mut scrape_interval = tokio::time::interval(Duration::from_secs(5));

            loop {
                tokio::select! {
                    _ = scrape_interval.tick() => {
                        let targets = metric_picker.get_metrics_targets();
                        for (idx, url) in targets {
                            let client_clone = client.clone();
                            let picker_clone = metric_picker.clone();

                            // Spawn individual metric fetches to run concurrently
                            tokio::spawn(async move {
                                if let Ok(response) = client_clone.get(&url).send().await {
                                    if let Ok(text) = response.text().await {
                                        // Parse prometheus metrics for node_load1
                                        for line in text.lines() {
                                            if line.starts_with("node_load1 ") {
                                                if let Some(value_str) = line.split_whitespace().nth(1) {
                                                    if let Ok(value) = value_str.parse::<f64>() {
                                                        picker_clone.update_load(idx, value);
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            });
                        }
                    }
                    _ = shutdown.changed() => {
                        log::info!("Metrics collector shutting down");
                        break;
                    }
                }
            }
        })
    }
}
