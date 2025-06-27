use pingora::Result;
use pingora_load_balancing::{discovery::ServiceDiscovery, Backend};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::time::interval;
use tracing::{debug, info, warn};

/// The build ID of this binary, set at compile time
pub const BUILD_ID: &str = env!("BUILD_ID");

/// Information about a discovered peer
#[derive(Debug, Clone)]
pub struct PeerInfo {
    /// Socket address of the peer
    pub addr: SocketAddr,
    /// Last time we successfully contacted this peer
    pub last_seen: Instant,
    /// Whether the peer is currently healthy
    pub healthy: bool,
    /// Number of consecutive failures
    pub failure_count: u32,
    /// Next time we should retry if the peer is failing
    pub next_retry: Instant,
}

impl PeerInfo {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            last_seen: Instant::now(),
            healthy: true,
            failure_count: 0,
            next_retry: Instant::now(),
        }
    }

    /// Calculate exponential backoff for retries
    pub fn calculate_backoff(&self) -> Duration {
        let base_delay = Duration::from_secs(1);
        let _max_delay = Duration::from_secs(60);
        // Use failure_count - 1 so first failure gets 1 second delay
        let exponent = self.failure_count.saturating_sub(1).min(6);
        let multiplier = 2u32.saturating_pow(exponent);
        base_delay * multiplier.min(60)
    }

    /// Mark this peer as failed and calculate next retry time
    pub fn mark_failed(&mut self) {
        self.healthy = false;
        self.failure_count = self.failure_count.saturating_add(1);
        let backoff = self.calculate_backoff();
        self.next_retry = Instant::now() + backoff;
        debug!("Peer {} failed, retry in {:?}", self.addr, backoff);
    }

    /// Mark this peer as successful
    pub fn mark_success(&mut self) {
        self.healthy = true;
        self.failure_count = 0;
        self.last_seen = Instant::now();
        self.next_retry = Instant::now();
    }

    /// Check if we should retry this peer now
    fn should_retry(&self) -> bool {
        !self.healthy && Instant::now() >= self.next_retry
    }
}

/// Thread-safe registry of discovered peers
#[derive(Clone)]
pub struct PeerRegistry {
    inner: Arc<RwLock<HashMap<SocketAddr, PeerInfo>>>,
}

impl PeerRegistry {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add or update a peer in the registry
    pub fn add_peer(&self, addr: SocketAddr) -> bool {
        let mut peers = match self.inner.write() {
            Ok(peers) => peers,
            Err(poisoned) => {
                tracing::warn!("Peer registry write lock poisoned during add_peer, recovering");
                poisoned.into_inner()
            }
        };
        if peers.contains_key(&addr) {
            false
        } else {
            info!("Discovered new peer: {}", addr);
            peers.insert(addr, PeerInfo::new(addr));
            true
        }
    }

    /// Mark a peer as successfully contacted
    pub fn mark_peer_success(&self, addr: &SocketAddr) {
        let mut peers = match self.inner.write() {
            Ok(peers) => peers,
            Err(poisoned) => {
                tracing::warn!(
                    "Peer registry write lock poisoned during mark_peer_success, recovering"
                );
                poisoned.into_inner()
            }
        };
        if let Some(peer) = peers.get_mut(addr) {
            peer.mark_success();
        }
    }

    /// Mark a peer as failed
    pub fn mark_peer_failed(&self, addr: &SocketAddr) {
        let mut peers = match self.inner.write() {
            Ok(peers) => peers,
            Err(poisoned) => {
                tracing::warn!(
                    "Peer registry write lock poisoned during mark_peer_failed, recovering"
                );
                poisoned.into_inner()
            }
        };
        if let Some(peer) = peers.get_mut(addr) {
            peer.mark_failed();
        }
    }

    /// Get all healthy peers
    pub fn get_healthy_peers(&self) -> Vec<SocketAddr> {
        let peers = match self.inner.read() {
            Ok(peers) => peers,
            Err(poisoned) => {
                tracing::warn!(
                    "Peer registry read lock poisoned during get_healthy_peers, recovering"
                );
                poisoned.into_inner()
            }
        };
        peers
            .values()
            .filter(|p| p.healthy)
            .map(|p| p.addr)
            .collect()
    }

    /// Get all peers that should be retried
    pub fn get_peers_to_retry(&self) -> Vec<SocketAddr> {
        let peers = match self.inner.read() {
            Ok(peers) => peers,
            Err(poisoned) => {
                tracing::warn!("Peer registry read lock poisoned, recovering");
                poisoned.into_inner()
            }
        };
        peers
            .values()
            .filter(|p| p.should_retry())
            .map(|p| p.addr)
            .collect()
    }

    /// Get all known peers (healthy and unhealthy)
    pub fn get_all_peers(&self) -> Vec<SocketAddr> {
        let peers = match self.inner.read() {
            Ok(peers) => peers,
            Err(poisoned) => {
                tracing::warn!("Peer registry read lock poisoned, recovering");
                poisoned.into_inner()
            }
        };
        peers.keys().cloned().collect()
    }

    /// Remove peers that haven't been seen in the given duration
    pub fn remove_stale_peers(&self, max_age: Duration) {
        let mut peers = match self.inner.write() {
            Ok(peers) => peers,
            Err(poisoned) => {
                tracing::warn!("Peer registry write lock poisoned, recovering");
                poisoned.into_inner()
            }
        };
        let now = Instant::now();
        peers.retain(|addr, info| {
            let age = now.duration_since(info.last_seen);
            if age > max_age && !info.healthy {
                warn!("Removing stale peer: {}", addr);
                false
            } else {
                true
            }
        });
    }

    /// Get the number of healthy peers
    pub fn healthy_peer_count(&self) -> usize {
        let peers = match self.inner.read() {
            Ok(peers) => peers,
            Err(poisoned) => {
                tracing::warn!("Peer registry read lock poisoned, recovering");
                poisoned.into_inner()
            }
        };
        peers.values().filter(|p| p.healthy).count()
    }
}

/// Response from the /api/peers endpoint
#[derive(Debug, Serialize, Deserialize)]
pub struct PeersResponse {
    pub build_id: String,
    pub peers: Vec<String>,
}

/// Request body for /api/register endpoint
#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub peer: String,
    pub build_id: String,
}

/// Pingora ServiceDiscovery implementation for dynamic peer discovery
#[allow(dead_code)]
pub struct PeerDiscovery {
    registry: PeerRegistry,
}

#[allow(dead_code)]
impl PeerDiscovery {
    pub fn new(registry: PeerRegistry) -> Self {
        Self { registry }
    }
}

impl ServiceDiscovery for PeerDiscovery {
    fn discover<'life0, 'async_trait>(
        &'life0 self,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<(BTreeSet<Backend>, HashMap<u64, bool>)>>
                + Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let peers = self.registry.get_healthy_peers();
            let mut backends = BTreeSet::new();

            for addr in peers {
                let backend = Backend::new(&addr.to_string())?;
                backends.insert(backend);
            }

            // All peers returned are healthy, so no readiness overrides needed
            let readiness = HashMap::new();

            Ok((backends, readiness))
        })
    }
}

/// Background service that discovers peers by polling /api/peers endpoints
#[derive(Clone)]
pub struct DiscoveryService {
    registry: PeerRegistry,
    client: reqwest::Client,
    poll_interval: Duration,
}

impl DiscoveryService {
    pub fn new(registry: PeerRegistry) -> Self {
        let mut headers = reqwest::header::HeaderMap::new();

        // Add discovery token if configured
        if let Ok(token) = std::env::var("SWEETMCP_DISCOVERY_TOKEN") {
            if !token.is_empty() {
                if let Ok(header_value) = reqwest::header::HeaderValue::from_str(&token) {
                    headers.insert("x-discovery-token", header_value);
                }
            }
        }

        Self {
            registry,
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .default_headers(headers)
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            poll_interval: Duration::from_secs(30),
        }
    }

    /// Initialize the registry with seed peers
    #[allow(dead_code)]
    pub fn add_seed_peers(&self, seed_peers: &[String]) {
        for peer_str in seed_peers {
            match peer_str.parse::<SocketAddr>() {
                Ok(addr) => {
                    if self.registry.add_peer(addr) {
                        info!("Added seed peer: {}", addr);
                    }
                }
                Err(e) => {
                    warn!("Invalid seed peer address '{}': {}", peer_str, e);
                }
            }
        }
    }

    /// Start the discovery service
    pub async fn run(self) {
        let mut discovery_interval = interval(self.poll_interval);
        let mut health_check_interval = interval(Duration::from_secs(10));

        // Do initial discovery immediately
        info!("Starting peer discovery service");
        self.discover_peers().await;

        loop {
            tokio::select! {
                _ = discovery_interval.tick() => {
                    self.discover_peers().await;
                }
                _ = health_check_interval.tick() => {
                    self.health_check_peers().await;
                }
            }
        }
    }

    /// Discover peers by polling known peers and fetching their peer lists
    async fn discover_peers(&self) {
        // Get all peers to poll (healthy and those ready for retry)
        let mut peers_to_poll = self.registry.get_healthy_peers();
        peers_to_poll.extend(self.registry.get_peers_to_retry());

        // Track peers we've already seen in this discovery round
        let mut discovered = HashSet::new();
        let mut to_process: Vec<SocketAddr> = peers_to_poll;

        // Process peers in rounds to handle recursive discovery
        while !to_process.is_empty() {
            let current_round = to_process.clone();
            to_process.clear();

            for peer_addr in current_round {
                if discovered.contains(&peer_addr) {
                    continue;
                }
                discovered.insert(peer_addr);

                match self.fetch_peers_from(&peer_addr).await {
                    Ok(new_peers) => {
                        self.registry.mark_peer_success(&peer_addr);

                        for new_peer in new_peers {
                            if self.registry.add_peer(new_peer) {
                                // New peer discovered, add to processing queue
                                to_process.push(new_peer);
                            }
                        }
                    }
                    Err(e) => {
                        debug!("Failed to fetch peers from {}: {}", peer_addr, e);
                        self.registry.mark_peer_failed(&peer_addr);
                    }
                }
            }
        }

        // Clean up stale peers that haven't been seen in 5 minutes
        self.registry.remove_stale_peers(Duration::from_secs(300));
    }

    /// Fetch the peer list from a specific peer
    async fn fetch_peers_from(&self, addr: &SocketAddr) -> anyhow::Result<Vec<SocketAddr>> {
        let url = format!("http://{}/api/peers", addr);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            anyhow::bail!("HTTP error: {}", response.status());
        }

        let peers_response: PeersResponse = response.json().await?;

        // Check build ID match
        if peers_response.build_id != BUILD_ID {
            warn!(
                "Build ID mismatch from {}: expected '{}', got '{}'",
                addr, BUILD_ID, peers_response.build_id
            );
            anyhow::bail!("Build ID mismatch");
        }

        // Parse peer addresses
        let mut peer_addrs = Vec::new();
        for peer_str in peers_response.peers {
            match peer_str.parse::<SocketAddr>() {
                Ok(peer_addr) => peer_addrs.push(peer_addr),
                Err(e) => {
                    warn!("Invalid peer address '{}' from {}: {}", peer_str, addr, e);
                }
            }
        }

        Ok(peer_addrs)
    }

    /// Perform TCP health checks on all peers
    async fn health_check_peers(&self) {
        let peers_to_check = self.registry.get_all_peers();
        let start = std::time::Instant::now();

        for peer_addr in peers_to_check {
            let registry = self.registry.clone();
            tokio::spawn(async move {
                // Production TCP health check with timeout
                match tokio::time::timeout(
                    Duration::from_secs(2),
                    tokio::net::TcpStream::connect(peer_addr),
                )
                .await
                {
                    Ok(Ok(_stream)) => {
                        registry.mark_peer_success(&peer_addr);
                    }
                    _ => {
                        registry.mark_peer_failed(&peer_addr);
                    }
                }
            });
        }

        // Update metrics
        let healthy_count = self.registry.healthy_peer_count();
        crate::metrics::update_peer_count(healthy_count as i64);
        crate::metrics::record_discovery("health_check", true, start.elapsed().as_secs_f64());
    }
}
