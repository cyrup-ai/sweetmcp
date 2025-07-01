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
        ],
    })
}