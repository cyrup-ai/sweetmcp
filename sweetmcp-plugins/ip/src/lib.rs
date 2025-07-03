mod plugin;

use extism_pdk::*;
use plugin::types::{CallToolRequest, CallToolResult, Content, ContentType, ListToolsResult, ToolDescription};
use serde_json::json;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Called when the IP tool is invoked
pub(crate) fn call(input: CallToolRequest) -> Result<CallToolResult, Error> {
    extism_pdk::log!(
        LogLevel::Info,
        "IP plugin called with args: {:?}",
        input.params.arguments
    );
    
    let args = input.params.arguments.unwrap_or_default();
    
    match input.params.name.as_str() {
        "get_public_ip" => get_public_ip(),
        "validate_ip" => validate_ip(args),
        "ip_info" => get_ip_info(args),
        "is_private" => check_private_ip(args),
        "ip_to_binary" => ip_to_binary(args),
        "create_ipv4" => create_ipv4(args),
        "create_ipv6" => create_ipv6(args),
        "cidr_contains" => cidr_contains(args),
        _ => Err(Error::msg(format!("Unknown IP operation: {}", input.params.name)))
    }
}

/// Get public IP address
fn get_public_ip() -> Result<CallToolResult, Error> {
    // For now, return a placeholder - full HTTP requests would need more setup
    Ok(CallToolResult {
        is_error: None,
        content: vec![Content {
            annotations: None,
            text: Some("Public IP detection would require HTTP request to external service".into()),
            mime_type: Some("text/plain".into()),
            r#type: ContentType::Text,
            data: None,
        }],
    })
}

/// Validate IP address format
fn validate_ip(args: serde_json::Map<String, serde_json::Value>) -> Result<CallToolResult, Error> {
    let ip_str = match args.get("ip") {
        Some(v) => v.as_str().ok_or_else(|| Error::msg("ip must be a string"))?,
        None => return Err(Error::msg("ip is required for validate_ip")),
    };
    
    let result = match ip_str.parse::<IpAddr>() {
        Ok(ip) => {
            let ip_type = match ip {
                IpAddr::V4(_) => "IPv4",
                IpAddr::V6(_) => "IPv6",
            };
            json!({
                "valid": true,
                "type": ip_type,
                "address": ip_str
            })
        },
        Err(_) => {
            json!({
                "valid": false,
                "error": "Invalid IP address format"
            })
        }
    };
    
    Ok(CallToolResult {
        is_error: None,
        content: vec![Content {
            annotations: None,
            text: Some(result.to_string()),
            mime_type: Some("application/json".into()),
            r#type: ContentType::Text,
            data: None,
        }],
    })
}

/// Get IP address information
fn get_ip_info(args: serde_json::Map<String, serde_json::Value>) -> Result<CallToolResult, Error> {
    let ip_str = match args.get("ip") {
        Some(v) => v.as_str().ok_or_else(|| Error::msg("ip must be a string"))?,
        None => return Err(Error::msg("ip is required for ip_info")),
    };
    
    let ip = ip_str.parse::<IpAddr>()
        .map_err(|_| Error::msg("Invalid IP address format"))?;
    
    let info = match ip {
        IpAddr::V4(ipv4) => {
            json!({
                "address": ip_str,
                "type": "IPv4",
                "is_private": ipv4.is_private(),
                "is_loopback": ipv4.is_loopback(),
                "is_multicast": ipv4.is_multicast(),
                "is_broadcast": ipv4.is_broadcast(),
                "octets": ipv4.octets()
            })
        },
        IpAddr::V6(ipv6) => {
            json!({
                "address": ip_str,
                "type": "IPv6",
                "is_loopback": ipv6.is_loopback(),
                "is_multicast": ipv6.is_multicast(),
                "segments": ipv6.segments()
            })
        }
    };
    
    Ok(CallToolResult {
        is_error: None,
        content: vec![Content {
            annotations: None,
            text: Some(info.to_string()),
            mime_type: Some("application/json".into()),
            r#type: ContentType::Text,
            data: None,
        }],
    })
}

/// Check if IP is private
fn check_private_ip(args: serde_json::Map<String, serde_json::Value>) -> Result<CallToolResult, Error> {
    let ip_str = match args.get("ip") {
        Some(v) => v.as_str().ok_or_else(|| Error::msg("ip must be a string"))?,
        None => return Err(Error::msg("ip is required for is_private")),
    };
    
    let ip = ip_str.parse::<IpAddr>()
        .map_err(|_| Error::msg("Invalid IP address format"))?;
    
    let is_private = match ip {
        IpAddr::V4(ipv4) => ipv4.is_private(),
        IpAddr::V6(_) => false, // IPv6 private determination is more complex
    };
    
    let result = json!({
        "address": ip_str,
        "is_private": is_private
    });
    
    Ok(CallToolResult {
        is_error: None,
        content: vec![Content {
            annotations: None,
            text: Some(result.to_string()),
            mime_type: Some("application/json".into()),
            r#type: ContentType::Text,
            data: None,
        }],
    })
}

/// Convert IP to binary representation
fn ip_to_binary(args: serde_json::Map<String, serde_json::Value>) -> Result<CallToolResult, Error> {
    let ip_str = match args.get("ip") {
        Some(v) => v.as_str().ok_or_else(|| Error::msg("ip must be a string"))?,
        None => return Err(Error::msg("ip is required for ip_to_binary")),
    };
    
    let ip = ip_str.parse::<IpAddr>()
        .map_err(|_| Error::msg("Invalid IP address format"))?;
    
    let binary_repr = match ip {
        IpAddr::V4(ipv4) => {
            let octets = ipv4.octets();
            format!("{:08b}.{:08b}.{:08b}.{:08b}", 
                octets[0], octets[1], octets[2], octets[3])
        },
        IpAddr::V6(ipv6) => {
            let segments = ipv6.segments();
            segments.iter()
                .map(|s| format!("{:016b}", s))
                .collect::<Vec<_>>()
                .join(":")
        }
    };
    
    let result = json!({
        "address": ip_str,
        "binary": binary_repr
    });
    
    Ok(CallToolResult {
        is_error: None,
        content: vec![Content {
            annotations: None,
            text: Some(result.to_string()),
            mime_type: Some("application/json".into()),
            r#type: ContentType::Text,
            data: None,
        }],
    })
}

/// Create IPv4 address from octets
fn create_ipv4(args: serde_json::Map<String, serde_json::Value>) -> Result<CallToolResult, Error> {
    let octets = match args.get("octets") {
        Some(v) => {
            let arr = v.as_array().ok_or_else(|| Error::msg("octets must be an array"))?;
            if arr.len() != 4 {
                return Err(Error::msg("octets must contain exactly 4 values"));
            }
            let mut octets = [0u8; 4];
            for (i, val) in arr.iter().enumerate() {
                octets[i] = val.as_u64()
                    .ok_or_else(|| Error::msg("octets must be numbers"))?
                    .try_into()
                    .map_err(|_| Error::msg("octets must be valid u8 values (0-255)"))?;
            }
            octets
        },
        None => return Err(Error::msg("octets is required for create_ipv4")),
    };
    
    let ipv4 = Ipv4Addr::new(octets[0], octets[1], octets[2], octets[3]);
    let result = json!({
        "address": ipv4.to_string(),
        "octets": octets,
        "is_private": ipv4.is_private(),
        "is_loopback": ipv4.is_loopback(),
        "is_multicast": ipv4.is_multicast(),
        "is_broadcast": ipv4.is_broadcast(),
    });
    
    Ok(CallToolResult {
        is_error: None,
        content: vec![Content {
            annotations: None,
            text: Some(result.to_string()),
            mime_type: Some("application/json".into()),
            r#type: ContentType::Text,
            data: None,
        }],
    })
}

/// Create IPv6 address from segments
fn create_ipv6(args: serde_json::Map<String, serde_json::Value>) -> Result<CallToolResult, Error> {
    let segments = match args.get("segments") {
        Some(v) => {
            let arr = v.as_array().ok_or_else(|| Error::msg("segments must be an array"))?;
            if arr.len() != 8 {
                return Err(Error::msg("segments must contain exactly 8 values"));
            }
            let mut segments = [0u16; 8];
            for (i, val) in arr.iter().enumerate() {
                segments[i] = val.as_u64()
                    .ok_or_else(|| Error::msg("segments must be numbers"))?
                    .try_into()
                    .map_err(|_| Error::msg("segments must be valid u16 values (0-65535)"))?;
            }
            segments
        },
        None => return Err(Error::msg("segments is required for create_ipv6")),
    };
    
    let ipv6 = Ipv6Addr::new(
        segments[0], segments[1], segments[2], segments[3],
        segments[4], segments[5], segments[6], segments[7]
    );
    
    let result = json!({
        "address": ipv6.to_string(),
        "segments": segments,
        "is_loopback": ipv6.is_loopback(),
        "is_multicast": ipv6.is_multicast(),
        "is_unspecified": ipv6.is_unspecified(),
    });
    
    Ok(CallToolResult {
        is_error: None,
        content: vec![Content {
            annotations: None,
            text: Some(result.to_string()),
            mime_type: Some("application/json".into()),
            r#type: ContentType::Text,
            data: None,
        }],
    })
}

/// Check if an IP is within a CIDR range
fn cidr_contains(args: serde_json::Map<String, serde_json::Value>) -> Result<CallToolResult, Error> {
    let ip_str = match args.get("ip") {
        Some(v) => v.as_str().ok_or_else(|| Error::msg("ip must be a string"))?,
        None => return Err(Error::msg("ip is required for cidr_contains")),
    };
    
    let cidr_str = match args.get("cidr") {
        Some(v) => v.as_str().ok_or_else(|| Error::msg("cidr must be a string"))?,
        None => return Err(Error::msg("cidr is required for cidr_contains")),
    };
    
    let ip = ip_str.parse::<IpAddr>()
        .map_err(|_| Error::msg("Invalid IP address format"))?;
    
    // Parse CIDR notation (e.g., "192.168.1.0/24")
    let parts: Vec<&str> = cidr_str.split('/').collect();
    if parts.len() != 2 {
        return Err(Error::msg("Invalid CIDR notation"));
    }
    
    let base_ip = parts[0].parse::<IpAddr>()
        .map_err(|_| Error::msg("Invalid base IP in CIDR"))?;
    
    let prefix_len: u8 = parts[1].parse()
        .map_err(|_| Error::msg("Invalid prefix length in CIDR"))?;
    
    let contains = match (base_ip, ip) {
        (IpAddr::V4(base), IpAddr::V4(test)) => {
            if prefix_len > 32 {
                return Err(Error::msg("IPv4 prefix length must be 0-32"));
            }
            let base_octets = base.octets();
            let test_octets = test.octets();
            let mask = if prefix_len == 0 { 0 } else { !((1u32 << (32 - prefix_len)) - 1) };
            
            let base_u32 = u32::from_be_bytes(base_octets);
            let test_u32 = u32::from_be_bytes(test_octets);
            
            (base_u32 & mask) == (test_u32 & mask)
        },
        (IpAddr::V6(base), IpAddr::V6(test)) => {
            if prefix_len > 128 {
                return Err(Error::msg("IPv6 prefix length must be 0-128"));
            }
            let base_segments = base.segments();
            let test_segments = test.segments();
            
            let full_segments = (prefix_len / 16) as usize;
            let partial_bits = prefix_len % 16;
            
            // Check full segments
            for i in 0..full_segments {
                if base_segments[i] != test_segments[i] {
                    return Ok(make_result(false, ip_str, cidr_str));
                }
            }
            
            // Check partial segment
            if partial_bits > 0 && full_segments < 8 {
                let mask = !((1u16 << (16 - partial_bits)) - 1);
                if (base_segments[full_segments] & mask) != (test_segments[full_segments] & mask) {
                    return Ok(make_result(false, ip_str, cidr_str));
                }
            }
            
            true
        },
        _ => false, // IPv4 vs IPv6 mismatch
    };
    
    Ok(make_result(contains, ip_str, cidr_str))
}

fn make_result(contains: bool, ip: &str, cidr: &str) -> CallToolResult {
    let result = json!({
        "ip": ip,
        "cidr": cidr,
        "contains": contains
    });
    
    CallToolResult {
        is_error: None,
        content: vec![Content {
            annotations: None,
            text: Some(result.to_string()),
            mime_type: Some("application/json".into()),
            r#type: ContentType::Text,
            data: None,
        }],
    }
}

/// Called by MCP to understand how and why to use this IP tool
pub(crate) fn describe() -> Result<ListToolsResult, Error> {
    Ok(ListToolsResult {
        tools: vec![
            ToolDescription {
                name: "get_public_ip".into(),
                description: "Get your current public IP address. Use this tool when you need to determine your external IP address for network configuration or troubleshooting.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }).as_object().unwrap().clone(),
            },
            ToolDescription {
                name: "validate_ip".into(),
                description: "Validate an IP address format and determine if it's IPv4 or IPv6. Use this tool to check if an IP address string is properly formatted.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "ip": {
                            "type": "string",
                            "description": "The IP address to validate"
                        }
                    },
                    "required": ["ip"]
                }).as_object().unwrap().clone(),
            },
            ToolDescription {
                name: "ip_info".into(),
                description: "Get detailed information about an IP address including type, properties, and structure. Use this tool for network analysis and IP address characterization.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "ip": {
                            "type": "string",
                            "description": "The IP address to analyze"
                        }
                    },
                    "required": ["ip"]
                }).as_object().unwrap().clone(),
            },
            ToolDescription {
                name: "is_private".into(),
                description: "Check if an IP address is in a private network range (RFC 1918). Use this tool to determine if an IP is internal/private or public/routable.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "ip": {
                            "type": "string",
                            "description": "The IP address to check"
                        }
                    },
                    "required": ["ip"]
                }).as_object().unwrap().clone(),
            },
            ToolDescription {
                name: "ip_to_binary".into(),
                description: "Convert an IP address to its binary representation. Use this tool for network calculations, subnetting, or educational purposes.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "ip": {
                            "type": "string",
                            "description": "The IP address to convert to binary"
                        }
                    },
                    "required": ["ip"]
                }).as_object().unwrap().clone(),
            },
            ToolDescription {
                name: "create_ipv4".into(),
                description: "Create an IPv4 address from octets and get its properties. Use this tool to construct IPv4 addresses programmatically and analyze their characteristics.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "octets": {
                            "type": "array",
                            "items": {"type": "integer"},
                            "minItems": 4,
                            "maxItems": 4,
                            "description": "Array of 4 octets (0-255) for IPv4 address"
                        }
                    },
                    "required": ["octets"]
                }).as_object().unwrap().clone(),
            },
            ToolDescription {
                name: "create_ipv6".into(),
                description: "Create an IPv6 address from segments and get its properties. Use this tool to construct IPv6 addresses programmatically and analyze their characteristics.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "segments": {
                            "type": "array",
                            "items": {"type": "integer"},
                            "minItems": 8,
                            "maxItems": 8,
                            "description": "Array of 8 segments (0-65535) for IPv6 address"
                        }
                    },
                    "required": ["segments"]
                }).as_object().unwrap().clone(),
            },
            ToolDescription {
                name: "cidr_contains".into(),
                description: "Check if an IP address is within a CIDR range. Use this tool for network planning, ACL validation, and subnet calculations.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "ip": {
                            "type": "string",
                            "description": "The IP address to check"
                        },
                        "cidr": {
                            "type": "string",
                            "description": "The CIDR notation (e.g., '192.168.1.0/24' or '2001:db8::/32')"
                        }
                    },
                    "required": ["ip", "cidr"]
                }).as_object().unwrap().clone(),
            },
        ],
    })
}