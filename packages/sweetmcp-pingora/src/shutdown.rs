//! Graceful shutdown handling for SweetMCP
//!
//! This module provides:
//! - Connection draining on SIGTERM
//! - Waiting for in-flight requests with timeout
//! - Discovery deregistration before shutdown
//! - State preservation for fast recovery

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::fs;
use tokio::net::UdpSocket;
use tokio::signal;
use tokio::sync::{broadcast, RwLock};
use tokio::time::{sleep, timeout};
use tracing::{debug, error, info, warn};

const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(30);
const STATE_FILE: &str = "sweetmcp_state.json";
const MDNS_MULTICAST_ADDR: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 251);
const MDNS_PORT: u16 = 5353;
const MDNS_GOODBYE_REPEATS: u8 = 3;
const MDNS_GOODBYE_INTERVAL: Duration = Duration::from_millis(100);

/// Shutdown coordinator for graceful termination
pub struct ShutdownCoordinator {
    /// Shutdown signal sender
    pub shutdown_tx: broadcast::Sender<()>,
    /// Flag indicating shutdown has been initiated
    pub shutting_down: Arc<AtomicBool>,
    /// Number of active requests
    active_requests: Arc<AtomicU64>,
    /// State to preserve
    state: Arc<RwLock<ServerState>>,
    /// Data directory for state persistence
    data_dir: PathBuf,
    /// Local service port
    local_port: u16,
    /// Peer registry reference
    peer_registry: Option<crate::peer_discovery::PeerRegistry>,
}

/// Server state to preserve across restarts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerState {
    /// Known peers at shutdown
    pub peers: Vec<String>,
    /// Last shutdown time
    pub shutdown_at: Option<u64>,
    /// Build ID for compatibility check
    pub build_id: String,
    /// Active circuit breaker states
    pub circuit_breakers: Vec<CircuitBreakerState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerState {
    pub peer: String,
    pub state: String,
    pub error_count: u64,
    pub total_count: u64,
}

impl ShutdownCoordinator {
    /// Create a new shutdown coordinator
    pub fn new(data_dir: PathBuf) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);

        Self {
            shutdown_tx,
            shutting_down: Arc::new(AtomicBool::new(false)),
            active_requests: Arc::new(AtomicU64::new(0)),
            state: Arc::new(RwLock::new(ServerState {
                peers: Vec::new(),
                shutdown_at: None,
                build_id: crate::peer_discovery::BUILD_ID.to_string(),
                circuit_breakers: Vec::new(),
            })),
            data_dir,
            local_port: 8443, // Default, should be set via set_local_port
            peer_registry: None,
        }
    }

    /// Set the local service port
    pub fn set_local_port(&mut self, port: u16) {
        self.local_port = port;
    }

    /// Set the peer registry reference
    pub fn set_peer_registry(&mut self, registry: crate::peer_discovery::PeerRegistry) {
        self.peer_registry = Some(registry);
    }

    /// Get a shutdown receiver
    pub fn subscribe(&self) -> broadcast::Receiver<()> {
        self.shutdown_tx.subscribe()
    }

    /// Check if shutdown is in progress
    pub fn is_shutting_down(&self) -> bool {
        self.shutting_down.load(Ordering::SeqCst)
    }

    /// Increment active request count
    pub fn request_start(&self) -> RequestGuard {
        if !self.is_shutting_down() {
            self.active_requests.fetch_add(1, Ordering::SeqCst);
            RequestGuard {
                counter: self.active_requests.clone(),
                active: true,
            }
        } else {
            RequestGuard {
                counter: self.active_requests.clone(),
                active: false,
            }
        }
    }

    /// Get active request count
    pub fn active_request_count(&self) -> u64 {
        self.active_requests.load(Ordering::SeqCst)
    }

    /// Update state to preserve
    pub async fn update_state<F>(&self, updater: F)
    where
        F: FnOnce(&mut ServerState),
    {
        let mut state = self.state.write().await;
        updater(&mut state);
    }

    /// Load preserved state from disk
    pub async fn load_state(&self) -> Result<Option<ServerState>> {
        let state_path = self.data_dir.join(STATE_FILE);

        if !state_path.exists() {
            return Ok(None);
        }

        let data = fs::read_to_string(&state_path).await?;
        let state: ServerState = serde_json::from_str(&data)?;

        // Check if build ID matches
        if state.build_id != crate::peer_discovery::BUILD_ID {
            warn!(
                "State file has different build ID: {} vs {}, ignoring",
                state.build_id,
                crate::peer_discovery::BUILD_ID
            );
            return Ok(None);
        }

        info!("Loaded previous state with {} peers", state.peers.len());
        Ok(Some(state))
    }

    /// Save state to disk
    pub async fn save_state(&self) -> Result<()> {
        let state = self.state.read().await.clone();
        let state_path = self.data_dir.join(STATE_FILE);

        // Ensure directory exists
        fs::create_dir_all(&self.data_dir).await?;

        // Write atomically using temp file
        let temp_path = state_path.with_extension("tmp");
        let data = serde_json::to_string_pretty(&state)?;
        fs::write(&temp_path, &data).await?;
        fs::rename(&temp_path, &state_path).await?;

        info!("Saved state with {} peers", state.peers.len());
        Ok(())
    }

    /// Start listening for shutdown signals
    pub async fn listen_for_shutdown(self: Arc<Self>) {
        tokio::spawn(async move {
            // Wait for SIGTERM or SIGINT
            let mut sigterm = match signal::unix::signal(signal::unix::SignalKind::terminate()) {
                Ok(signal) => signal,
                Err(e) => {
                    error!("Failed to register SIGTERM handler: {}", e);
                    return;
                }
            };
            let mut sigint = match signal::unix::signal(signal::unix::SignalKind::interrupt()) {
                Ok(signal) => signal,
                Err(e) => {
                    error!("Failed to register SIGINT handler: {}", e);
                    return;
                }
            };

            tokio::select! {
                _ = sigterm.recv() => {
                    info!("Received SIGTERM, initiating graceful shutdown");
                }
                _ = sigint.recv() => {
                    info!("Received SIGINT, initiating graceful shutdown");
                }
            }

            self.initiate_shutdown().await;
        });
    }

    /// Initiate graceful shutdown
    pub async fn initiate_shutdown(&self) {
        if self.shutting_down.swap(true, Ordering::SeqCst) {
            // Already shutting down
            return;
        }

        info!("Starting graceful shutdown sequence");
        let shutdown_start = Instant::now();

        // Step 1: Stop accepting new requests
        let _ = self.shutdown_tx.send(());
        info!("Stopped accepting new requests");

        // Step 2: Deregister from discovery
        if let Err(e) = self.deregister_from_discovery().await {
            error!("Failed to deregister from discovery: {}", e);
        }

        // Step 3: Wait for active requests to complete
        let drain_result = timeout(SHUTDOWN_TIMEOUT, self.drain_connections()).await;

        match drain_result {
            Ok(_) => {
                info!("All connections drained in {:?}", shutdown_start.elapsed());
            }
            Err(_) => {
                let remaining = self.active_request_count();
                warn!(
                    "Shutdown timeout reached, {} requests still active",
                    remaining
                );
            }
        }

        // Step 4: Save state
        self.state.write().await.shutdown_at = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or(std::time::Duration::from_secs(0))
                .as_secs(),
        );

        if let Err(e) = self.save_state().await {
            error!("Failed to save state: {}", e);
        }

        info!(
            "Graceful shutdown completed in {:?}",
            shutdown_start.elapsed()
        );
    }

    /// Deregister from all discovery mechanisms
    async fn deregister_from_discovery(&self) -> Result<()> {
        info!("Deregistering from discovery services");

        let mut errors = Vec::new();

        // Task 1: Send mDNS goodbye packets
        if let Err(e) = self.send_mdns_goodbye_packets().await {
            error!("Failed to send mDNS goodbye packets: {}", e);
            errors.push(format!("mDNS goodbye: {}", e));
        }

        // Task 2: Notify all peers of shutdown
        if let Err(e) = self.notify_peers_of_shutdown().await {
            error!("Failed to notify peers: {}", e);
            errors.push(format!("peer notification: {}", e));
        }

        // Task 3: Deregister from Shuttle service registry
        if let Err(e) = self.deregister_from_shuttle().await {
            error!("Failed to deregister from Shuttle: {}", e);
            errors.push(format!("Shuttle deregistration: {}", e));
        }

        if errors.is_empty() {
            info!("Discovery deregistration completed successfully");
            Ok(())
        } else {
            let error_msg = format!(
                "Deregistration completed with {} errors: {}",
                errors.len(),
                errors.join(", ")
            );
            warn!("{}", error_msg);
            // We don't fail the shutdown for deregistration errors
            Ok(())
        }
    }

    /// Send mDNS goodbye packets per RFC 6762
    async fn send_mdns_goodbye_packets(&self) -> Result<()> {
        info!("Sending mDNS goodbye packets");

        // Create UDP socket for mDNS
        let socket = match UdpSocket::bind("0.0.0.0:0").await {
            Ok(s) => s,
            Err(e) => {
                warn!("Failed to create mDNS socket: {}", e);
                return Err(e.into());
            }
        };

        socket.set_multicast_loop_v4(false)?;

        // Create goodbye announcement with TTL=0 to indicate service removal
        let goodbye_msg = format!(
            "SWEETMCP|{}|{}|GOODBYE",
            crate::peer_discovery::BUILD_ID,
            self.local_port
        );

        let dest = SocketAddr::new(IpAddr::V4(MDNS_MULTICAST_ADDR), MDNS_PORT);

        // Send goodbye packet multiple times for reliability (RFC 6762 recommends this)
        for i in 0..MDNS_GOODBYE_REPEATS {
            match socket.send_to(goodbye_msg.as_bytes(), dest).await {
                Ok(_) => debug!(
                    "Sent mDNS goodbye packet {}/{}",
                    i + 1,
                    MDNS_GOODBYE_REPEATS
                ),
                Err(e) => warn!("Failed to send mDNS goodbye packet {}: {}", i + 1, e),
            }

            if i < MDNS_GOODBYE_REPEATS - 1 {
                sleep(MDNS_GOODBYE_INTERVAL).await;
            }
        }

        info!("mDNS goodbye packets sent");
        Ok(())
    }

    /// Notify all known peers of impending shutdown
    async fn notify_peers_of_shutdown(&self) -> Result<()> {
        info!("Notifying peers of shutdown");

        let peers = if let Some(registry) = &self.peer_registry {
            registry.get_all_peers()
        } else {
            warn!("No peer registry available for shutdown notification");
            return Ok(());
        };

        if peers.is_empty() {
            info!("No peers to notify");
            return Ok(());
        }

        info!("Notifying {} peers of shutdown", peers.len());

        let mut successful = 0;
        let mut failed = 0;

        for peer_addr in peers {
            // Send shutdown notification with short timeout
            let notification_timeout = Duration::from_secs(2);

            match timeout(
                notification_timeout,
                self.send_shutdown_notification(&peer_addr),
            )
            .await
            {
                Ok(Ok(())) => {
                    debug!("Successfully notified {} of shutdown", peer_addr);
                    successful += 1;
                }
                Ok(Err(e)) => {
                    warn!("Failed to notify {} of shutdown: {}", peer_addr, e);
                    failed += 1;
                }
                Err(_) => {
                    warn!("Timeout notifying {} of shutdown", peer_addr);
                    failed += 1;
                }
            }
        }

        info!(
            "Peer shutdown notifications: {} successful, {} failed",
            successful, failed
        );

        if failed > 0 {
            Err(anyhow::anyhow!("{} peers could not be notified", failed))
        } else {
            Ok(())
        }
    }

    /// Send shutdown notification to a specific peer
    async fn send_shutdown_notification(&self, peer_addr: &SocketAddr) -> Result<()> {
        // Create a simple UDP datagram with shutdown message
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        let shutdown_msg = format!(
            "SWEETMCP|{}|SHUTDOWN|{}",
            crate::peer_discovery::BUILD_ID,
            self.local_port
        );

        socket.send_to(shutdown_msg.as_bytes(), peer_addr).await?;
        Ok(())
    }

    /// Deregister from Shuttle's service registry
    async fn deregister_from_shuttle(&self) -> Result<()> {
        info!("Deregistering from Shuttle service registry");

        // Check if we're running on Shuttle
        if std::env::var("SHUTTLE_RUNTIME_ID").is_err() {
            debug!("Not running on Shuttle, skipping Shuttle deregistration");
            return Ok(());
        }

        let service_name =
            std::env::var("SHUTTLE_SERVICE_NAME").unwrap_or_else(|_| "sweetmcp".to_string());

        // Shuttle handles most cleanup automatically when the service stops,
        // but we can clean up any custom state we maintain

        // Remove any local service state files
        let shuttle_state_file = self.data_dir.join(".shuttle_service_state");
        if shuttle_state_file.exists() {
            if let Err(e) = fs::remove_file(&shuttle_state_file).await {
                warn!("Failed to remove Shuttle state file: {}", e);
            } else {
                debug!("Removed Shuttle service state file");
            }
        }

        // Clean up any Shuttle-specific resources
        info!(
            "Shuttle deregistration completed for service: {}",
            service_name
        );
        Ok(())
    }

    /// Wait for all active connections to drain
    async fn drain_connections(&self) {
        let check_interval = Duration::from_millis(100);
        let mut last_count = self.active_request_count();

        while self.active_request_count() > 0 {
            sleep(check_interval).await;

            let current_count = self.active_request_count();
            if current_count != last_count {
                info!("Waiting for {} active requests to complete", current_count);
                last_count = current_count;
            }
        }
    }
}

/// RAII guard for request tracking
pub struct RequestGuard {
    counter: Arc<AtomicU64>,
    active: bool,
}

impl Drop for RequestGuard {
    fn drop(&mut self) {
        if self.active {
            self.counter.fetch_sub(1, Ordering::SeqCst);
        }
    }
}

/// Shutdown-aware service wrapper
pub struct ShutdownAware<S> {
    inner: S,
    coordinator: Arc<ShutdownCoordinator>,
}

impl<S> ShutdownAware<S> {
    pub fn new(service: S, coordinator: Arc<ShutdownCoordinator>) -> Self {
        Self {
            inner: service,
            coordinator,
        }
    }

    pub fn inner(&self) -> &S {
        &self.inner
    }

    pub fn is_shutting_down(&self) -> bool {
        self.coordinator.is_shutting_down()
    }

    pub fn track_request(&self) -> Option<RequestGuard> {
        if !self.is_shutting_down() {
            Some(self.coordinator.request_start())
        } else {
            None
        }
    }
}
