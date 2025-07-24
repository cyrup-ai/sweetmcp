// Restore constants as they are used elsewhere
pub const JSONRPC_VERSION: &str = "2.0";
pub const PROTOCOL_VERSION: &str = "2024-11-05";
pub const SERVER_NAME: &str = "sweet-mcp-server";
pub const SERVER_VERSION: &str = "0.1.0";

pub mod config; // Make module public

// Declare context as a directory module
pub mod context;

mod container_registry;
pub mod db; // Make db module public
pub mod notifications;
pub mod plugin; // Ensure plugin module is declared and public
mod prompt;
pub mod resource; // Make resource module public
pub mod router; // Ensure router is declared
pub mod sampling; // Re-enable
pub mod security; // Zero-allocation input validation framework
mod tool; // Re-enable
mod types;
pub mod ui;

pub use config::{
    // Keep only one set of imports
    Config,
    ConfigFormat,
    EnvConfig,
    PluginConfig,
    basename,
    init_logger,
    parse_config,
    parse_config_from_str,
    validate_config,
};
// Removed obsolete db exports
pub use container_registry::*;
pub use plugin::PluginManager; // Updated path
pub use resource::resource_read;
pub use sampling::{
    CompletionUsage, CreateMessageRequest, CreateMessageResult, SamplingProgressNotification,
    SamplingTokenNotification, sampling_create_message,
};
pub use security::{
    EmailValidationRule, MemoryOperation, MemoryOperationType, MemorySafetyMetrics,
    MemorySafetyResult, MemorySafetyRule, MemorySafetyValidator, MemorySafetyViolation,
    PathTraversalValidationRule, SafetyViolationSeverity, SafetyViolationType,
    SqlInjectionValidationRule, UrlValidationRule, ValidationEngine, ValidationError,
    ValidationMetrics, ValidationResult, ValidationRule, ValidationSeverity, XssValidationRule,
};
// Restore glob export for tool
// Export specific components instead of using glob imports
pub use tool::model;
// Use more specific imports instead of glob imports to avoid ambiguity
pub use db::*;
pub use types::AsyncTask;
