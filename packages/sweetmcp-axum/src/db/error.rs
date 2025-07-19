use crate::db::config;
use std::fmt::{self, Display}; // Use Display trait for formatting
use thiserror::Error; // Use thiserror crate for error handling

/// Specialized error type for SurrealDB operations
#[derive(Error, Debug)]
pub enum SurrealdbError {
    /// Error from SurrealDB
    #[error("Database error: {0}")]
    Database(#[from] surrealdb::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Error from IO operations
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Error from validation
    #[error("Validation error: {0}")]
    Validation(SurrealdbErrorContext),

    /// Error from authentication
    #[error("Authentication error: {0}")]
    Authentication(SurrealdbErrorContext),

    /// Entity not found
    #[error("Entity not found: {0}")]
    NotFound(SurrealdbErrorContext),

    /// Duplicate entity
    #[error("Duplicate entity: {0}")]
    Duplicate(SurrealdbErrorContext),

    /// Transaction error
    #[error("Transaction error: {0}")]
    Transaction(SurrealdbErrorContext),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(SurrealdbErrorContext),

    /// Other errors
    #[error("Other error: {0}")]
    Other(SurrealdbErrorContext),
}

/// Context information for errors to provide more details
#[derive(Debug, Clone)]
pub struct SurrealdbErrorContext {
    /// The main error message
    message: String,
    /// Optional resource identifier (table, record ID, etc.)
    resource: Option<String>,
    /// Optional operation context (what was being attempted)
    operation: Option<String>,
}

impl Display for SurrealdbErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;

        if let Some(resource) = &self.resource {
            write!(f, " (resource: {})", resource)?;
        }

        if let Some(operation) = &self.operation {
            write!(f, " during {}", operation)?;
        }

        Ok(())
    }
}

impl SurrealdbErrorContext {
    /// Create a new error context with just a message
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            resource: None,
            operation: None,
        }
    }

    /// Add resource context to the error
    pub fn with_resource(mut self, resource: impl Into<String>) -> Self {
        self.resource = Some(resource.into());
        self
    }

    /// Add operation context to the error
    pub fn with_operation(mut self, operation: impl Into<String>) -> Self {
        self.operation = Some(operation.into());
        self
    }
}

impl From<String> for SurrealdbErrorContext {
    fn from(message: String) -> Self {
        Self::new(message)
    }
}

impl From<&str> for SurrealdbErrorContext {
    fn from(message: &str) -> Self {
        Self::new(message)
    }
}

impl SurrealdbError {
    /// Create a validation error
    pub fn validation(msg: impl Into<SurrealdbErrorContext>) -> Self {
        Self::Validation(msg.into())
    }

    /// Create an authentication error
    pub fn authentication(msg: impl Into<SurrealdbErrorContext>) -> Self {
        Self::Authentication(msg.into())
    }

    /// Create a not found error
    pub fn not_found(msg: impl Into<SurrealdbErrorContext>) -> Self {
        Self::NotFound(msg.into())
    }

    /// Create a duplicate error
    pub fn duplicate(msg: impl Into<SurrealdbErrorContext>) -> Self {
        Self::Duplicate(msg.into())
    }

    /// Create a transaction error
    pub fn transaction(msg: impl Into<SurrealdbErrorContext>) -> Self {
        Self::Transaction(msg.into())
    }

    /// Create a configuration error
    pub fn configuration(msg: impl Into<SurrealdbErrorContext>) -> Self {
        Self::Configuration(msg.into())
    }

    /// Create a general error
    pub fn other(msg: impl Into<SurrealdbErrorContext>) -> Self {
        Self::Other(msg.into())
    }

    /// Add resource context to any error variant
    pub fn with_resource(self, resource: impl Into<String>) -> Self {
        match self {
            Self::Validation(ctx) => Self::Validation(ctx.with_resource(resource)),
            Self::Authentication(ctx) => Self::Authentication(ctx.with_resource(resource)),
            Self::NotFound(ctx) => Self::NotFound(ctx.with_resource(resource)),
            Self::Duplicate(ctx) => Self::Duplicate(ctx.with_resource(resource)),
            Self::Transaction(ctx) => Self::Transaction(ctx.with_resource(resource)),
            Self::Configuration(ctx) => Self::Configuration(ctx.with_resource(resource)),
            Self::Other(ctx) => Self::Other(ctx.with_resource(resource)),
            other => other,
        }
    }

    /// Add operation context to any error variant
    pub fn with_operation(self, operation: impl Into<String>) -> Self {
        match self {
            Self::Validation(ctx) => Self::Validation(ctx.with_operation(operation)),
            Self::Authentication(ctx) => Self::Authentication(ctx.with_operation(operation)),
            Self::NotFound(ctx) => Self::NotFound(ctx.with_operation(operation)),
            Self::Duplicate(ctx) => Self::Duplicate(ctx.with_operation(operation)),
            Self::Transaction(ctx) => Self::Transaction(ctx.with_operation(operation)),
            Self::Configuration(ctx) => Self::Configuration(ctx.with_operation(operation)),
            Self::Other(ctx) => Self::Other(ctx.with_operation(operation)),
            other => other,
        }
    }

    /// Check if this is a not found error
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound(_))
    }

    /// Check if this is a duplicate error
    pub fn is_duplicate(&self) -> bool {
        matches!(self, Self::Duplicate(_))
    }

    /// Check if this is a validation error
    pub fn is_validation(&self) -> bool {
        matches!(self, Self::Validation(_))
    }

    /// Check if this is a connection error
    pub fn is_connection_error(&self) -> bool {
        match self {
            Self::Database(e) => {
                let err_str = e.to_string();
                err_str.contains("connection")
                    || err_str.contains("connect")
                    || err_str.contains("timeout")
            }
            _ => false,
        }
    }
}

/// Convert from config::Error to SurrealdbError
impl From<config::Error> for SurrealdbError {
    fn from(err: config::Error) -> Self {
        SurrealdbError::configuration(SurrealdbErrorContext::new(err.to_string()))
    }
}
