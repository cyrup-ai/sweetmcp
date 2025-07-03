//! DNS-based service discovery for SweetMCP using DoH (DNS over HTTPS)
//!
//! This module implements secure, zero-configuration service discovery using
//! DNS SRV records - the industry standard approach used by Consul, Kubernetes, etc.

use crate::peer_discovery::PeerRegistry;
// Temporarily disable hickory-resolver until API compatibility is resolved
// use hickory_resolver::config::{ResolverConfig, ResolverOpts, ResolveHosts};
// use hickory_resolver::TokioResolver;
use std::time::Duration;
use tokio::time::interval;
use tracing::{debug, info, warn};

const DISCOVERY_INTERVAL: Duration = Duration::from_secs(60);
// DoH servers for future enhancement when hickory-resolver DoH API stabilizes
// const DOH_SERVERS: &[&str] = &[
//     "https://cloudflare-dns.com/dns-query",
//     "https://dns.google/dns-query",
//     "https://dns.quad9.net/dns-query",
// ];

/// DNS-based discovery service using SRV records
/// NOTE: Temporarily disabled due to hickory-resolver API compatibility issues
pub struct DnsDiscovery {
    // resolver: TokioResolver,  // Temporarily disabled
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
        // NOTE: Temporarily disabled due to hickory-resolver API compatibility issues
        warn!("DNS discovery is temporarily disabled due to hickory-resolver API compatibility");

        Self {
            // resolver,  // Temporarily disabled
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
        debug!(
            "DNS discovery is temporarily disabled - skipping lookup for: {}",
            self.service_name
        );

        // NOTE: Temporarily disabled due to hickory-resolver API compatibility issues
        // When re-enabled, this will perform DNS SRV lookups and populate the peer registry

        warn!(
            "DNS discovery temporarily disabled - no peers discovered for {}",
            self.service_name
        );
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
