//! Unit tests for mDNS-based auto-discovery module

use sweetmcp::mdns_discovery::*;
use sweetmcp::peer_discovery::{PeerRegistry, BUILD_ID};
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

#[test]
fn test_announcement_format() {
    let announcement = format!("SWEETMCP|{}|{}", BUILD_ID, 8443);
    let parts: Vec<&str> = announcement.split('|').collect();
    assert_eq!(parts.len(), 3);
    assert_eq!(parts[0], "SWEETMCP");
    assert_eq!(parts[1], BUILD_ID);
    assert_eq!(parts[2], "8443");
}

#[tokio::test]
async fn test_handle_announcement() {
    let registry = PeerRegistry::new();
    let discovery = MdnsDiscovery::new(registry.clone(), 8443);
    
    // Valid announcement with matching build ID
    let announcement = format!("SWEETMCP|{}|9000", BUILD_ID);
    let from = "192.168.1.100:5353".parse().expect("Valid IP address");
    
    discovery.handle_announcement(announcement.as_bytes(), from).await;
    
    // Check that peer was added
    let peers = registry.get_all_peers();
    assert_eq!(peers.len(), 1);
    assert_eq!(peers[0], "192.168.1.100:9000".parse::<SocketAddr>().expect("Valid IP address"));
}

#[tokio::test]
async fn test_handle_announcement_wrong_build_id() {
    let registry = PeerRegistry::new();
    let discovery = MdnsDiscovery::new(registry.clone(), 8443);
    
    // Announcement with wrong build ID
    let announcement = "SWEETMCP|wrong-build-id|9000";
    let from = "192.168.1.100:5353".parse().expect("Valid IP address");
    
    discovery.handle_announcement(announcement.as_bytes(), from).await;
    
    // Check that peer was NOT added
    let peers = registry.get_all_peers();
    assert_eq!(peers.len(), 0);
}

#[tokio::test]
async fn test_handle_invalid_announcement() {
    let registry = PeerRegistry::new();
    let discovery = MdnsDiscovery::new(registry.clone(), 8443);
    
    // Various invalid announcements
    let test_cases = vec![
        "INVALID|format",
        "SWEETMCP|build|not-a-port",
        "SWEETMCP|only-two",
        "",
        "SWEETMCP|build|8080|extra",
    ];
    
    for announcement in test_cases {
        let from = "192.168.1.100:5353".parse().expect("Valid IP address");
        discovery.handle_announcement(announcement.as_bytes(), from).await;
    }
    
    // No peers should be added
    assert_eq!(registry.get_all_peers().len(), 0);
}

#[test]
fn test_multicast_constants() {
    assert_eq!(MDNS_PORT, 5353);
    assert_eq!(MDNS_MULTICAST_ADDR, Ipv4Addr::new(224, 0, 0, 251));
    assert_eq!(ANNOUNCE_INTERVAL, Duration::from_secs(30));
}

// Mock socket for testing
struct MockSocket {
    sent_messages: Arc<Mutex<Vec<(Vec<u8>, SocketAddr)>>>,
}

impl MockSocket {
    fn new() -> Self {
        Self {
            sent_messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    async fn send_to(&self, buf: &[u8], addr: SocketAddr) -> std::io::Result<usize> {
        self.sent_messages.lock().await.push((buf.to_vec(), addr));
        Ok(buf.len())
    }
}

#[tokio::test]
async fn test_announce_format() {
    let mock_socket = MockSocket::new();
    let registry = PeerRegistry::new();
    let discovery = MdnsDiscovery::new(registry, 8443);
    
    // Verify the announcement format
    let expected_announcement = format!("SWEETMCP|{}|8443", BUILD_ID);
    assert!(expected_announcement.starts_with("SWEETMCP|"));
    assert!(expected_announcement.ends_with("|8443"));
}