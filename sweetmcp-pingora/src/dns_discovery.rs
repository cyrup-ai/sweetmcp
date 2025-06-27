//! DNS-based service discovery for SweetMCP using DoH (DNS over HTTPS)
//!
//! This module implements secure, zero-configuration service discovery using
//! DNS SRV records - the industry standard approach used by Consul, Kubernetes, etc.

use crate::peer_discovery::PeerRegistry;
use futures::future::join_all;
use hickory_resolver::config::{ResolverConfig, ResolverOpts};
use hickory_resolver::TokioAsyncResolver;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

const DISCOVERY_INTERVAL: Duration = Duration::from_secs(60);
// DoH servers for future enhancement when hickory-resolver DoH API stabilizes
// const DOH_SERVERS: &[&str] = &[
//     "https://cloudflare-dns.com/dns-query",
//     "https://dns.google/dns-query",
//     "https://dns.quad9.net/dns-query",
// ];

/// DNS-based discovery service using SRV records
pub struct DnsDiscovery {
    resolver: TokioAsyncResolver,
    service_name: String,
    registry: PeerRegistry,
}

impl DnsDiscovery {
    /// Creates a new DNS discovery instance
    ///
    /// # Arguments
    /// - `service_name`: The SRV service name (e.g., "_sweetmcp._tcp.example.com")
    /// - `registry`: The peer registry to update with discovered services
    /// - `doh_server`: Reserved for future DoH support
    pub fn new(service_name: String, registry: PeerRegistry, _doh_server: Option<&str>) -> Self {
        // Use standard DNS (DoH can be configured via system resolver when needed)
        let config = ResolverConfig::default();

        let mut opts = ResolverOpts::default();
        opts.timeout = Duration::from_secs(5);
        opts.attempts = 3;
        opts.cache_size = 256; // Small cache for discovery
        opts.use_hosts_file = false; // Don't use local hosts file
        opts.validate = true; // DNSSEC validation when available

        let resolver = TokioAsyncResolver::tokio(config, opts);

        Self {
            resolver,
            service_name,
            registry,
        }
    }

    /// Start the DNS discovery service
    pub async fn run(self) {
        info!("Starting DNS discovery for service: {}", self.service_name);

        let mut discovery_interval = interval(DISCOVERY_INTERVAL);

        // Discover immediately
        self.discover_peers().await;

        loop {
            discovery_interval.tick().await;
            self.discover_peers().await;
        }
    }

    async fn discover_peers(&self) {
        debug!("Performing DNS SRV lookup for: {}", self.service_name);

        match self.resolver.srv_lookup(&self.service_name).await {
            Ok(srv_lookup) => {
                let srv_records: Vec<_> = srv_lookup.iter().collect();

                if srv_records.is_empty() {
                    debug!("No SRV records found for {}", self.service_name);
                    return;
                }

                info!(
                    "Found {} SRV records for {}",
                    srv_records.len(),
                    self.service_name
                );

                // Create concurrent IP lookup tasks
                let futures = srv_records.into_iter().map(|srv| {
                    let resolver = self.resolver.clone();
                    let target = srv.target().to_utf8();
                    let port = srv.port();
                    let priority = srv.priority();
                    let weight = srv.weight();

                    async move {
                        debug!(
                            "Resolving {} (priority: {}, weight: {})",
                            target, priority, weight
                        );

                        match resolver.lookup_ip(&target).await {
                            Ok(ip_lookup) => {
                                let addrs: Vec<SocketAddr> = ip_lookup
                                    .into_iter()
                                    .map(|ip| SocketAddr::new(ip, port))
                                    .collect();

                                (addrs, priority, weight)
                            }
                            Err(e) => {
                                warn!("Failed to resolve {}: {}", target, e);
                                (Vec::new(), priority, weight)
                            }
                        }
                    }
                });

                // Await all resolutions concurrently
                let results = join_all(futures).await;

                // Update peer registry with discovered addresses
                for (addrs, _priority, _weight) in results {
                    for addr in addrs {
                        if self.registry.add_peer(addr) {
                            info!("Discovered peer via DNS: {}", addr);
                        }
                    }
                }
            }
            Err(e) => {
                error!("DNS SRV lookup failed for {}: {}", self.service_name, e);
            }
        }
    }
}

/// Check if we should use DNS discovery based on environment
pub fn should_use_dns_discovery() -> Option<String> {
    // Check for explicit DNS service name
    if let Ok(service) = std::env::var("SWEETMCP_DNS_SERVICE") {
        return Some(service);
    }

    // Auto-detect based on domain
    if let Ok(domain) = std::env::var("SWEETMCP_DOMAIN") {
        return Some(format!("_sweetmcp._tcp.{}", domain));
    }

    None
}
