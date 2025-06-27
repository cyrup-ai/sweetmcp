// Restore constants as they are used elsewhere
pub const JSONRPC_VERSION: &str = "2.0";
pub const PROTOCOL_VERSION: &str = "2024-11-05";
pub const SERVER_NAME: &str = "sweet-mcp-server";
pub const SERVER_VERSION: &str = "0.1.0";

pub mod config; // Make module public

// Declare context as a directory module
pub mod context;

mod daemon_integration;

pub mod db; // Make db module public
mod r#mod;
pub mod notifications;
pub mod plugin; // Ensure plugin module is declared and public
mod prompt;
mod container_registry;
pub mod resource; // Make resource module public
pub mod router; // Ensure router is declared
mod sampling; // Re-enable
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
pub use plugin::PluginManager; // Updated path
pub use container_registry::*;
pub use resource::resource_read;
pub use sampling::{

    SamplingProgressNotification,
    SamplingTokenNotification,
    CompletionUsage,
    CreateMessageRequest,
    CreateMessageResult,
    sampling_create_message,
};
// Restore glob export for tool
// Export specific components instead of using glob imports
pub use tool::model;
// Use more specific imports instead of glob imports to avoid ambiguity
pub use types::AsyncTask;
pub use db::*;
