pub mod model;
pub mod service;

// Re-export core service functions and PromptService
pub use service::{prompts_get_handler, prompts_list_handler, PromptService};
