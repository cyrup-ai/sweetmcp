use sweetmcp::dns_discovery::*;

#[test]
fn test_service_name_format() {
    // Standard SRV format
    assert!(should_use_dns_discovery().is_none());
    
    std::env::set_var("SWEETMCP_DNS_SERVICE", "_sweetmcp._tcp.example.com");
    assert_eq!(should_use_dns_discovery(), Some("_sweetmcp._tcp.example.com".to_string()));
    
    std::env::remove_var("SWEETMCP_DNS_SERVICE");
    std::env::set_var("SWEETMCP_DOMAIN", "example.com");
    assert_eq!(should_use_dns_discovery(), Some("_sweetmcp._tcp.example.com".to_string()));
    
    std::env::remove_var("SWEETMCP_DOMAIN");
}