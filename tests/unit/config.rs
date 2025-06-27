use std::time::Duration;
use sweetmcp::config::{Config, parse_duration};

#[test]
fn test_parse_duration() {
    assert_eq!(parse_duration("30s").expect("Failed to parse 30s"), Duration::from_secs(30));
    assert_eq!(parse_duration("5m").expect("Failed to parse 5m"), Duration::from_secs(300));
    assert_eq!(parse_duration("2h").expect("Failed to parse 2h"), Duration::from_secs(7200));
    assert_eq!(parse_duration("1d").expect("Failed to parse 1d"), Duration::from_secs(86400));
    
    assert!(parse_duration("").is_err());
    assert!(parse_duration("30").is_err());
    assert!(parse_duration("30x").is_err());
}

#[test]
fn test_config_validation() -> Result<(), Box<dyn std::error::Error>> {
    // This test requires setting up a valid JWT secret
    std::env::set_var("SWEETMCP_JWT_SECRET", base64_url::encode(&[0u8; 32]));
    
    let config = Config::from_env()?;
    assert!(config.validate().is_ok());
    
    Ok(())
}