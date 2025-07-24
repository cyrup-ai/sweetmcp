//! Protocol types and context definitions
//!
//! This module provides core protocol types and context structures
//! for protocol normalization with zero allocation patterns and
//! blazing-fast performance.

use serde::{Deserialize, Serialize};

/// Supported protocol types for normalization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Proto {
    /// GraphQL protocol
    GraphQL,
    /// JSON-RPC 2.0 protocol
    JsonRpc,
    /// Cap'n Proto binary protocol
    Capnp,
    /// MCP Streamable HTTP protocol
    McpStreamableHttp,
}

impl Proto {
    /// Check if protocol is binary
    pub fn is_binary(&self) -> bool {
        matches!(self, Proto::Capnp)
    }

    /// Check if protocol supports streaming
    pub fn supports_streaming(&self) -> bool {
        matches!(self, Proto::McpStreamableHttp)
    }

    /// Get protocol name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Proto::GraphQL => "graphql",
            Proto::JsonRpc => "json-rpc",
            Proto::Capnp => "capnp",
            Proto::McpStreamableHttp => "mcp-streamable-http",
        }
    }

    /// Parse protocol from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "graphql" => Some(Proto::GraphQL),
            "json-rpc" | "jsonrpc" => Some(Proto::JsonRpc),
            "capnp" | "capnproto" => Some(Proto::Capnp),
            "mcp-streamable-http" | "mcp" => Some(Proto::McpStreamableHttp),
            _ => None,
        }
    }

    /// Get default content type for protocol
    pub fn default_content_type(&self) -> &'static str {
        match self {
            Proto::GraphQL => "application/json",
            Proto::JsonRpc => "application/json",
            Proto::Capnp => "application/octet-stream",
            Proto::McpStreamableHttp => "application/json",
        }
    }

    /// Check if protocol requires special handling
    pub fn requires_special_handling(&self) -> bool {
        matches!(self, Proto::GraphQL | Proto::Capnp)
    }
}

/// Context for tracking protocol conversion
#[derive(Debug, Clone)]
pub struct ProtocolContext {
    /// The original protocol type
    pub protocol: Proto,
    /// Original query for GraphQL response shaping
    pub original_query: Option<String>,
    /// Unique request identifier
    pub request_id: String,
    /// Additional metadata for conversion
    pub metadata: ProtocolMetadata,
}

impl ProtocolContext {
    /// Create new protocol context
    pub fn new(protocol: Proto, request_id: String) -> Self {
        Self {
            protocol,
            original_query: None,
            request_id,
            metadata: ProtocolMetadata::default(),
        }
    }

    /// Create context with original query
    pub fn with_query(protocol: Proto, request_id: String, query: String) -> Self {
        Self {
            protocol,
            original_query: Some(query),
            request_id,
            metadata: ProtocolMetadata::default(),
        }
    }

    /// Create context with metadata
    pub fn with_metadata(protocol: Proto, request_id: String, metadata: ProtocolMetadata) -> Self {
        Self {
            protocol,
            original_query: None,
            request_id,
            metadata,
        }
    }

    /// Check if context has original query
    pub fn has_original_query(&self) -> bool {
        self.original_query.is_some()
    }

    /// Get original query reference
    pub fn original_query(&self) -> Option<&str> {
        self.original_query.as_deref()
    }

    /// Set original query
    pub fn set_original_query(&mut self, query: String) {
        self.original_query = Some(query);
    }

    /// Get protocol type
    pub fn protocol(&self) -> &Proto {
        &self.protocol
    }

    /// Get request ID
    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    /// Get metadata
    pub fn metadata(&self) -> &ProtocolMetadata {
        &self.metadata
    }

    /// Update metadata
    pub fn set_metadata(&mut self, metadata: ProtocolMetadata) {
        self.metadata = metadata;
    }

    /// Check if context is valid
    pub fn is_valid(&self) -> bool {
        !self.request_id.is_empty()
    }

    /// Create error context
    pub fn create_error_context(error_msg: &str) -> Self {
        Self {
            protocol: Proto::JsonRpc,
            original_query: None,
            request_id: uuid::Uuid::new_v4().to_string(),
            metadata: ProtocolMetadata {
                error_message: Some(error_msg.to_string()),
                ..Default::default()
            },
        }
    }
}

/// Additional metadata for protocol conversion
#[derive(Debug, Clone, Default)]
pub struct ProtocolMetadata {
    /// Content type from original request
    pub content_type: Option<String>,
    /// User agent from original request
    pub user_agent: Option<String>,
    /// Custom headers that may affect conversion
    pub custom_headers: std::collections::HashMap<String, String>,
    /// Timestamp when conversion started
    pub conversion_start: Option<std::time::Instant>,
    /// Error message if conversion failed
    pub error_message: Option<String>,
    /// Additional conversion options
    pub options: ConversionOptions,
}

impl ProtocolMetadata {
    /// Create new metadata
    pub fn new() -> Self {
        Self::default()
    }

    /// Set content type
    pub fn with_content_type(mut self, content_type: String) -> Self {
        self.content_type = Some(content_type);
        self
    }

    /// Set user agent
    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }

    /// Add custom header
    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.custom_headers.insert(key, value);
        self
    }

    /// Set conversion start time
    pub fn mark_conversion_start(&mut self) {
        self.conversion_start = Some(std::time::Instant::now());
    }

    /// Get conversion duration
    pub fn conversion_duration(&self) -> Option<std::time::Duration> {
        self.conversion_start.map(|start| start.elapsed())
    }

    /// Check if metadata has error
    pub fn has_error(&self) -> bool {
        self.error_message.is_some()
    }

    /// Set error message
    pub fn set_error(&mut self, error: String) {
        self.error_message = Some(error);
    }
}

/// Options for protocol conversion
#[derive(Debug, Clone)]
pub struct ConversionOptions {
    /// Whether to preserve original field names
    pub preserve_field_names: bool,
    /// Whether to validate converted JSON-RPC
    pub validate_jsonrpc: bool,
    /// Maximum depth for nested objects
    pub max_depth: usize,
    /// Whether to include debug information
    pub include_debug_info: bool,
    /// Custom timeout for conversion
    pub timeout_ms: Option<u64>,
}

impl Default for ConversionOptions {
    fn default() -> Self {
        Self {
            preserve_field_names: true,
            validate_jsonrpc: true,
            max_depth: 10,
            include_debug_info: false,
            timeout_ms: Some(5000), // 5 seconds
        }
    }
}

impl ConversionOptions {
    /// Create options for development
    pub fn development() -> Self {
        Self {
            include_debug_info: true,
            timeout_ms: Some(30000), // 30 seconds for debugging
            ..Default::default()
        }
    }

    /// Create options for production
    pub fn production() -> Self {
        Self {
            include_debug_info: false,
            timeout_ms: Some(1000), // 1 second for production
            ..Default::default()
        }
    }

    /// Create options for testing
    pub fn testing() -> Self {
        Self {
            validate_jsonrpc: false, // Skip validation for faster tests
            timeout_ms: Some(10000), // 10 seconds for tests
            ..Default::default()
        }
    }
}

/// Result type for protocol conversion
pub type ConversionResult<T> = Result<T, ConversionError>;

/// Errors that can occur during protocol conversion
#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    #[error("Invalid protocol format: {0}")]
    InvalidFormat(String),
    
    #[error("Unsupported protocol: {0}")]
    UnsupportedProtocol(String),
    
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("GraphQL parsing error: {0}")]
    GraphQLError(String),
    
    #[error("Cap'n Proto error: {0}")]
    CapnProtoError(String),
    
    #[error("Conversion timeout after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl ConversionError {
    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            ConversionError::InvalidFormat(_) => false,
            ConversionError::UnsupportedProtocol(_) => false,
            ConversionError::JsonError(_) => false,
            ConversionError::GraphQLError(_) => true,
            ConversionError::CapnProtoError(_) => true,
            ConversionError::Timeout { .. } => true,
            ConversionError::ValidationError(_) => true,
            ConversionError::InternalError(_) => false,
        }
    }

    /// Get error severity
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            ConversionError::InvalidFormat(_) => ErrorSeverity::Error,
            ConversionError::UnsupportedProtocol(_) => ErrorSeverity::Warning,
            ConversionError::JsonError(_) => ErrorSeverity::Error,
            ConversionError::GraphQLError(_) => ErrorSeverity::Warning,
            ConversionError::CapnProtoError(_) => ErrorSeverity::Warning,
            ConversionError::Timeout { .. } => ErrorSeverity::Warning,
            ConversionError::ValidationError(_) => ErrorSeverity::Info,
            ConversionError::InternalError(_) => ErrorSeverity::Critical,
        }
    }

    /// Create JSON-RPC error response
    pub fn to_jsonrpc_error(&self, id: Option<serde_json::Value>) -> serde_json::Value {
        let (code, message) = match self {
            ConversionError::InvalidFormat(msg) => (-32700, format!("Parse error: {}", msg)),
            ConversionError::UnsupportedProtocol(proto) => (-32601, format!("Method not found: unsupported protocol {}", proto)),
            ConversionError::JsonError(e) => (-32700, format!("Parse error: {}", e)),
            ConversionError::GraphQLError(msg) => (-32602, format!("Invalid params: {}", msg)),
            ConversionError::CapnProtoError(msg) => (-32602, format!("Invalid params: {}", msg)),
            ConversionError::Timeout { timeout_ms } => (-32603, format!("Internal error: timeout after {}ms", timeout_ms)),
            ConversionError::ValidationError(msg) => (-32602, format!("Invalid params: {}", msg)),
            ConversionError::InternalError(msg) => (-32603, format!("Internal error: {}", msg)),
        };

        serde_json::json!({
            "jsonrpc": "2.0",
            "id": id.unwrap_or(serde_json::Value::Null),
            "error": {
                "code": code,
                "message": message
            }
        })
    }
}

/// Error severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Protocol detection result
#[derive(Debug, Clone)]
pub struct ProtocolDetection {
    /// Detected protocol
    pub protocol: Proto,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
    /// Detection method used
    pub method: DetectionMethod,
}

/// Methods used for protocol detection
#[derive(Debug, Clone, PartialEq)]
pub enum DetectionMethod {
    /// Detected by content type header
    ContentType,
    /// Detected by request structure
    Structure,
    /// Detected by user agent
    UserAgent,
    /// Detected by URL path
    UrlPath,
    /// Default fallback detection
    Fallback,
}

impl ProtocolDetection {
    /// Create new detection result
    pub fn new(protocol: Proto, confidence: f64, method: DetectionMethod) -> Self {
        Self {
            protocol,
            confidence,
            method,
        }
    }

    /// Check if detection is confident
    pub fn is_confident(&self) -> bool {
        self.confidence >= 0.8
    }

    /// Check if detection is uncertain
    pub fn is_uncertain(&self) -> bool {
        self.confidence < 0.5
    }
}