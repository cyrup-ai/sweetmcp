//! mDNS-based auto-discovery for SweetMCP nodes
//!
//! This module implements true auto-discovery using multicast DNS (mDNS).
//! Nodes announce themselves on the local network and discover peers automatically.

use crate::peer_discovery::{PeerRegistry, BUILD_ID};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

pub const MDNS_PORT: u16 = 5353;
pub const MDNS_MULTICAST_ADDR: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 251);
#[allow(dead_code)]
const SERVICE_NAME: &str = "_sweetmcp._tcp.local";
pub const ANNOUNCE_INTERVAL: Duration = Duration::from_secs(30);

/// mDNS discovery service that announces our presence and discovers peers
pub struct MdnsDiscovery {
    registry: PeerRegistry,
    local_port: u16,
    socket: Option<UdpSocket>,
}

impl MdnsDiscovery {
    pub fn new(registry: PeerRegistry, local_port: u16) -> Self {
        Self {
            registry,
            local_port,
            socket: None,
        }
    }

    /// Start the mDNS discovery service
    pub async fn run(mut self) {
        info!("Starting mDNS discovery service on port {}", MDNS_PORT);

        // Bind to mDNS multicast address
        match self.setup_socket().await {
            Ok(socket) => {
                self.socket = Some(socket);
                self.run_discovery_loop().await;
            }
            Err(e) => {
                error!("Failed to setup mDNS socket: {}", e);
            }
        }
    }

    async fn setup_socket(&self) -> std::io::Result<UdpSocket> {
        let socket = UdpSocket::bind(("0.0.0.0", MDNS_PORT)).await?;

        // Join multicast group
        socket.join_multicast_v4(MDNS_MULTICAST_ADDR, Ipv4Addr::new(0, 0, 0, 0))?;
        socket.set_multicast_loop_v4(false)?;

        Ok(socket)
    }

    async fn run_discovery_loop(&mut self) {
        let socket = match self.socket.as_ref() {
            Some(socket) => socket,
            None => {
                error!("Socket not initialized for mDNS discovery loop");
                return;
            }
        };

        let mut announce_interval = interval(ANNOUNCE_INTERVAL);

        // Announce immediately
        self.announce_presence(socket).await;

        let mut buf = vec![0u8; 1024];

        loop {
            tokio::select! {
                _ = announce_interval.tick() => {
                    self.announce_presence(socket).await;
                }
                Ok((len, addr)) = socket.recv_from(&mut buf) => {
                    self.handle_announcement(&buf[..len], addr).await;
                }
            }
        }
    }

    async fn announce_presence(&self, socket: &UdpSocket) {
        // Production announcement format: "SWEETMCP|build_id|port"
        let announcement = format!("SWEETMCP|{}|{}", BUILD_ID, self.local_port);
        let dest = SocketAddr::new(IpAddr::V4(MDNS_MULTICAST_ADDR), MDNS_PORT);

        match socket.send_to(announcement.as_bytes(), dest).await {
            Ok(_) => debug!("Announced presence via mDNS"),
            Err(e) => warn!("Failed to send mDNS announcement: {}", e),
        }
    }

    pub async fn handle_announcement(&self, data: &[u8], from: SocketAddr) {
        match std::str::from_utf8(data) {
            Ok(announcement) => {
                let parts: Vec<&str> = announcement.split('|').collect();
                if parts.len() == 3 && parts[0] == "SWEETMCP" {
                    let peer_build_id = parts[1];
                    let peer_port: u16 = match parts[2].parse() {
                        Ok(p) => p,
                        Err(_) => return,
                    };

                    // Check build ID compatibility
                    if peer_build_id != BUILD_ID {
                        debug!(
                            "Ignoring mDNS announcement from {} - build ID mismatch: {} != {}",
                            from, peer_build_id, BUILD_ID
                        );
                        return;
                    }

                    // Add peer with announced port
                    let peer_addr = SocketAddr::new(from.ip(), peer_port);
                    if self.registry.add_peer(peer_addr) {
                        info!(
                            "Discovered peer via mDNS: {} (build: {})",
                            peer_addr, peer_build_id
                        );
                    }
                }
            }
            Err(_) => debug!("Invalid mDNS announcement from {}", from),
        }
    }
}
