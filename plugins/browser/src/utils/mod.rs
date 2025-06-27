mod agent_state;
mod deep_research;
mod errors;
mod llm;
mod utils;

pub use agent_state::AgentState;
pub use deep_research::{DeepResearch, ResearchResult, ResearchOptions};
pub use errors::UtilsError;
pub use llm::LlmConfig;
pub use utils::{llama, encode_image, get_latest_files, capture_screenshot};

/// Result type for utility functions
pub type UtilsResult<T> = Result<T, UtilsError>;
