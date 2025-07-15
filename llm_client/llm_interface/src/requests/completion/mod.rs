// Internal modules
pub mod error; // Make error module public
mod chunk; // Added chunk module
mod config;
mod request;
mod response;
mod settings;

// Public exports
// Export types defined within this module's submodules
pub use self::{
    chunk::CompletionResponseChunk, // Added export
    config::CompletionRequestConfig,
    request::CompletionRequest,
    response::{CompletionFinishReason, CompletionResponse}, 
    settings::GenerationSettings,
};

// Re-export ToolCallSummary from common::tools
