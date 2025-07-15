use serde::{Deserialize, Serialize}; // Added Serialize

// Internal imports
use crate::llms::api::anthropic::completion::res::AnthropicCompletionResponse;
use crate::llms::api::openai::completion::res::OpenAiCompletionResponse;
use crate::llms::local::llama_cpp::completion::res::LlamaCppCompletionResponse;
use crate::requests::CompletionRequest;

/// Settings used for the generation process, often echoed back in responses or used internally.
/// This struct focuses on common generation parameters.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GenerationSettings {
    /// The temperature used for sampling. Range varies by backend (e.g., 0.0-2.0 for OpenAI, 0.0-1.0 for Anthropic).
    pub temperature: f32,
    /// The nucleus sampling probability (e.g., 0.1 means consider tokens comprising the top 10% probability mass).
    pub top_p: Option<f32>,
    /// Consider only the top K most likely tokens for sampling (e.g., 40).
    pub top_k: Option<usize>,
    // Note: Other settings like frequency_penalty, presence_penalty are part of CompletionRequestConfig
    // but might not always be echoed back by all backends in a dedicated settings structure within the response.
    // Llama.cpp *does* echo them back, so they could potentially be added here if needed for consistency,
    // but currently, they are primarily request-side configurations.
}

impl GenerationSettings {
    /// Creates new GenerationSettings from the configuration of a CompletionRequest.
    /// This is useful when a backend doesn't echo back the settings used.
    /// Creates new GenerationSettings from a CompletionRequest.
    pub fn new_from_request(req: &CompletionRequest) -> Self {
        Self {
            temperature: req.config.temperature,
            top_p: req.config.top_p,
            top_k: req.config.top_k,
        }
    }

    /// Creates new GenerationSettings using values from an Anthropic request,
    /// as the response doesn't echo these settings.
    pub fn new_from_anthropic(req: &CompletionRequest, _res: &AnthropicCompletionResponse) -> Self {
        // Anthropic response doesn't echo back these specific settings, use request values.
        // Note: Anthropic temperature is 0.0-1.0, while request config is 0.0-2.0.
        // The conversion happens during request creation. Here we store the original request value.
        Self::new_from_request(req)
    }

    /// Creates new GenerationSettings using values from an OpenAI request,
    /// as the response doesn't echo these settings.
    pub fn new_from_openai(req: &CompletionRequest, _res: &OpenAiCompletionResponse) -> Self {
        // OpenAI response doesn't echo back these specific settings, use request values.
        Self::new_from_request(req)
    }

    /// Creates new GenerationSettings from a LlamaCpp response, which echoes these settings.
    #[cfg(feature = "llama_cpp_backend")]
    pub fn new_from_llama_cpp(_req: &CompletionRequest, res: &LlamaCppCompletionResponse) -> Self {
        // LlamaCpp response includes these generation settings directly.
        Self {
            temperature: res.generation_settings.temperature,
            top_p: Some(res.generation_settings.top_p), // Llama.cpp reports top_p
            top_k: None, // Llama.cpp doesn't report top_k in its response
        }
    }
}
