// Reexport OpenAI's tool use implementations to be used with Generic OpenAI providers
pub use crate::llms::api::openai::tool_use::*;

// OpenAI-compatible providers may have slight differences in their tool implementations.
// This module can be extended with provider-specific customizations as needed.
