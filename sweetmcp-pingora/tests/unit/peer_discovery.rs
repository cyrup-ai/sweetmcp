use sweetmcp::peer_discovery::*;
use std::net::SocketAddr;
use std::time::Duration;

#[test]
fn test_peer_info_backoff() {
    let mut peer = PeerInfo::new("127.0.0.1:8080".parse().expect("Valid IP address"));
    
    // Initial state
    assert!(peer.healthy);
    assert_eq!(peer.failure_count, 0);
    
    // First failure - 1 second backoff
    peer.mark_failed();
    assert!(!peer.healthy);
    assert_eq!(peer.failure_count, 1);
    let backoff = peer.calculate_backoff();
    assert_eq!(backoff, Duration::from_secs(1));
    
    // Second failure - 2 seconds
    peer.mark_failed();
    assert_eq!(peer.failure_count, 2);
    assert_eq!(peer.calculate_backoff(), Duration::from_secs(2));
    
    // Third failure - 4 seconds
    peer.mark_failed();
    assert_eq!(peer.failure_count, 3);
    assert_eq!(peer.calculate_backoff(), Duration::from_secs(4));
    
    // Max backoff is 60 seconds
    peer.failure_count = 10;
    assert_eq!(peer.calculate_backoff(), Duration::from_secs(60));
    
    // Success resets everything
    peer.mark_success();
    assert!(peer.healthy);
    assert_eq!(peer.failure_count, 0);
}

#[test]
fn test_peer_registry() {
    let registry = PeerRegistry::new();
    let addr1: SocketAddr = "127.0.0.1:8080".parse().expect("Valid IP address");
    let addr2: SocketAddr = "127.0.0.1:8081".parse().expect("Valid IP address");
    
    // Add peers
    assert!(registry.add_peer(addr1));
    assert!(!registry.add_peer(addr1)); // Duplicate
    assert!(registry.add_peer(addr2));
    
    // All peers are healthy initially
    assert_eq!(registry.get_healthy_peers().len(), 2);
    assert_eq!(registry.healthy_peer_count(), 2);
    
    // Mark one as failed
    registry.mark_peer_failed(&addr1);
    assert_eq!(registry.get_healthy_peers().len(), 1);
    assert_eq!(registry.healthy_peer_count(), 1);
    
    // Failed peer should be in retry list
    assert_eq!(registry.get_peers_to_retry().len(), 0); // Not yet, backoff delay
    
    // All peers still known
    assert_eq!(registry.get_all_peers().len(), 2);
}

#[tokio::test]
async fn test_peer_discovery() {
    let registry = PeerRegistry::new();
    let discovery = PeerDiscovery::new(registry.clone());
    
    // Initially no backends
    let (backends, readiness) = discovery.discover().await.expect("Discovery should succeed");
    assert_eq!(backends.len(), 0);
    assert_eq!(readiness.len(), 0);
    
    // Add some peers
    registry.add_peer("127.0.0.1:8080".parse().expect("Valid IP address"));
    registry.add_peer("127.0.0.1:8081".parse().expect("Valid IP address"));
    
    // Now we have backends
    let (backends, readiness) = discovery.discover().await.expect("Discovery should succeed");
    assert_eq!(backends.len(), 2);
    assert_eq!(readiness.len(), 0);
    
    // Mark one as failed
    registry.mark_peer_failed(&"127.0.0.1:8080".parse().expect("Valid IP address"));
    
    // Only one healthy backend
    let (backends, _) = discovery.discover().await.expect("Discovery should succeed");
    assert_eq!(backends.len(), 1);
}