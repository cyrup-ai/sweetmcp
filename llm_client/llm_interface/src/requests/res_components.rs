// Internal imports
use crate::requests::CompletionRequest;
use crate::llms::api::{
    anthropic::completion::res::AnthropicCompletionResponse,
    openai::completion::res::OpenAiCompletionResponse,
};

// Internal feature-specific imports
#[cfg(feature = "llama_cpp_backend")]
use crate::llms::local::llama_cpp::completion::res::LlamaCppCompletionResponse;
#[cfg(feature = "mistral_rs_backend")]
use crate::llms::local::mistral_rs::completion::res::MistralCompletionResponse;
use serde::{Deserialize, Serialize}; // Added import

/// The log probability of the completion.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InferenceProbabilities {
    /// The token selected by the model.
    pub content: Option<String>,
    /// An array of length n_probs.
    pub top_probs: Vec<TopProbabilities>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TopProbabilities {
    /// The token.
    pub token: String,
    /// The log probability of this token.
    pub prob: f32,
}

/// The settings used to generate the completion.
/// NOTE: This struct holds settings *reported back* by the backend in the response,
/// which might differ from the requested settings in `requests::completion::settings::GenerationSettings`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResponseGenerationSettings {
    /// The model used
    pub model: String,
    // pub prompt: String, // Need to think how to handle tokens vs. text
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: f32,
    pub temperature: f32,
    pub top_p: Option<f32>,
    /// The number of choices to generate.
    pub n_choices: u8,
    /// The number of tokens to predict same as max_tokens.
    pub n_predict: Option<i64>,
    pub logit_bias: Option<Vec<serde_json::Value>>,
    pub grammar: Option<String>,
    pub stop_sequences: Vec<String>, // change toi vec of stop sequences
}

impl ResponseGenerationSettings {
    #[cfg(feature = "llama_cpp_backend")]
    #[allow(dead_code)]
    pub fn new_from_llama(res: &LlamaCppCompletionResponse) -> Self {
        Self {
            model: res.model.to_owned(),
            frequency_penalty: Some(res.generation_settings.frequency_penalty),
            presence_penalty: res.generation_settings.presence_penalty,
            temperature: res.generation_settings.temperature,
            top_p: Some(res.generation_settings.top_p),
            n_choices: 1,
            n_predict: Some(res.generation_settings.n_predict as i64),
            logit_bias: res.generation_settings.logit_bias.clone(),
            grammar: Some(res.generation_settings.grammar.to_owned()),
            stop_sequences: res.generation_settings.stop.clone(),
        }
    }

    #[cfg(feature = "mistral_rs_backend")]
    pub fn new_from_mistral(req: &CompletionRequest, res: &MistralCompletionResponse) -> Self {
        // Mistral response doesn't echo back all settings, use response values where available
        Self {
            model: res.model.to_string(),
            frequency_penalty: req.config.frequency_penalty,
            presence_penalty: req.config.presence_penalty.unwrap_or_default(),
            temperature: req.config.temperature,
            top_p: req.config.top_p,
            n_choices: 1,
            n_predict: req.config.actual_request_tokens.map(|x| x as i64), // Use i64 like others
            // n_ctx: req.backend.inference_ctx_size(), // Access via backend
            // Mistral response doesn't directly provide these, use request values or defaults
            logit_bias: None,       // Not in Mistral response
            grammar: None,          // Not in Mistral response
            stop_sequences: vec![], // Not directly in Mistral response, maybe infer from finish reason?
        }
    }

    pub fn new_from_openai(req: &CompletionRequest, res: &OpenAiCompletionResponse) -> Self {
        // OpenAI response doesn't echo back all settings, use response values where available
        Self {
            model: res.model.to_owned(),
            frequency_penalty: Some(req.config.frequency_penalty.unwrap_or_default()),
            presence_penalty: req.config.presence_penalty.unwrap_or_default(),
            temperature: req.config.temperature,
            top_p: req.config.top_p,
            n_choices: 1,
            n_predict: req.config.actual_request_tokens.map(|x| x as i64),

            logit_bias: None,       // Not in OpenAI response
            grammar: None,          // Not in OpenAI response
            stop_sequences: vec![], // Not directly in OpenAI response, maybe infer from finish reason?
        }
    }

    pub fn new_from_anthropic(req: &CompletionRequest, res: &AnthropicCompletionResponse) -> Self {
        // Anthropic response doesn't echo back all settings, use response values where available
        Self {
            model: res.model.to_string(),
            frequency_penalty: req.config.frequency_penalty,
            presence_penalty: req.config.presence_penalty.unwrap_or_default(),
            temperature: req.config.temperature,
            top_p: req.config.top_p,
            n_choices: 1,
            n_predict: req.config.actual_request_tokens.map(|x| x as i64),
            logit_bias: None, // Not in Anthropic response
            grammar: None,    // Not in Anthropic response
            stop_sequences: vec![res.stop_sequence.clone().unwrap_or_default()], // Use stop_sequence if present
        }
    }
}

impl std::fmt::Display for ResponseGenerationSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        writeln!(f, "    model: {:?}", self.model)?;
        writeln!(f, "    frequency_penalty: {:?}", self.frequency_penalty)?;
        writeln!(f, "    presence_penalty: {:?}", self.presence_penalty)?;
        writeln!(f, "    temperature: {:?}", self.temperature)?;
        writeln!(f, "    top_p: {:?}", self.top_p)?;
        writeln!(f, "    n_choices: {:?}", self.n_choices)?;
        writeln!(f, "    n_predict: {:?}", self.n_predict)?;
        writeln!(f, "    logit_bias: {:?}", self.logit_bias)?;
        writeln!(f, "    grammar: {:?}", self.grammar)?;
        writeln!(f, "    stop_sequences: {:?}", self.stop_sequences)
    }
}

/// Timing statistics for the completion request.
#[derive(Debug, Clone, Serialize, PartialEq)] // Keep these derives
#[derive(Deserialize)] // Custom Deserialize implementation with defaults
#[serde(default)]
pub struct TimingUsage {
    /// Timestamp of when the request was created (client-side).
    #[serde(skip)] // Instant cannot be easily serialized/deserialized across processes/network
    pub start_time: std::time::Instant,
    /// Timestamp of when the request was completed (client-side).
    #[serde(skip)] // Instant cannot be easily serialized/deserialized across processes/network
    pub end_time: std::time::Instant,
    /// Total time taken for the request from the client's perspective (in milliseconds).
    pub total_time_ms: u128,
    /// Time taken by the backend to process the prompt (in milliseconds), if reported.
    pub prompt_processing_ms: Option<u64>,
    /// Time taken by the backend to generate the completion (in milliseconds), if reported.
    pub generation_ms: Option<u64>,
    /// Number of prompt tokens processed per millisecond by the backend, if reported.
    pub prompt_tok_per_ms: Option<f32>,
    /// Number of prompt tokens processed per second by the backend, if reported.
    pub prompt_tok_per_sec: Option<f32>,
    /// Number of completion tokens generated per millisecond by the backend, if reported.
    pub generation_tok_per_ms: Option<f32>,
    /// Number of completion tokens generated per second by the backend, if reported.
    pub generation_tok_per_sec: Option<f32>,
}

impl Default for TimingUsage {
    fn default() -> Self {
        Self {
            start_time: std::time::Instant::now(),
            end_time: std::time::Instant::now(),
            total_time_ms: 0,
            prompt_processing_ms: None,
            generation_ms: None,
            prompt_tok_per_ms: None,
            generation_tok_per_ms: None,
            prompt_tok_per_sec: None,
            generation_tok_per_sec: None,
        }
    }
}

impl TimingUsage {
    // Constructor for LlamaCpp backend
    #[cfg(feature = "llama_cpp_backend")]
    pub fn new_from_llama(
        res: &LlamaCppCompletionResponse,
        start_time: std::time::Instant,
    ) -> Self {
        let end_time = std::time::Instant::now();
        Self {
            start_time,
            end_time,
            total_time_ms: end_time.duration_since(start_time).as_millis(),
            prompt_processing_ms: Some(res.timings.prompt_ms.round() as u64),
            generation_ms: Some(res.timings.predicted_ms.round() as u64),
            prompt_tok_per_ms: Some(res.timings.prompt_per_token_ms),
            prompt_tok_per_sec: Some(res.timings.prompt_per_second),
            generation_tok_per_ms: Some(res.timings.predicted_per_token_ms),
            generation_tok_per_sec: Some(res.timings.predicted_per_second),
        }
    }

    // Constructor for MistralRs backend
    #[cfg(feature = "mistral_rs_backend")]
    pub fn new_from_mistral(
        res: &MistralCompletionResponse,
        start_time: std::time::Instant,
    ) -> Self {
        let end_time = std::time::Instant::now();
        Self {
            start_time,
            end_time,
            total_time_ms: end_time.duration_since(start_time).as_millis(),
            prompt_processing_ms: Some((res.usage.total_prompt_time_sec * 1000.0) as u64),
            generation_ms: Some((res.usage.total_completion_time_sec * 1000.0) as u64),
            prompt_tok_per_ms: Some(res.usage.avg_prompt_tok_per_sec / 1000.0),
            prompt_tok_per_sec: Some(res.usage.avg_prompt_tok_per_sec),
            generation_tok_per_ms: Some(res.usage.avg_compl_tok_per_sec / 1000.0),
            generation_tok_per_sec: Some(res.usage.avg_compl_tok_per_sec),
        }
    }

    /// Constructor for backends that don't report detailed timing (e.g., OpenAI, Anthropic).
    /// Calculates only the total client-side duration.
    pub fn new_from_generic(start_time: std::time::Instant) -> Self {
        let end_time = std::time::Instant::now();
        Self {
            start_time,
            end_time,
            total_time_ms: end_time.duration_since(start_time).as_millis(),
            prompt_processing_ms: None, // Not provided by generic backends
            generation_ms: None,        // Not provided by generic backends
            prompt_tok_per_ms: None,
            prompt_tok_per_sec: None,
            generation_tok_per_ms: None,
            generation_tok_per_sec: None,
        }
    }
}

// Note: Display implementation can be added if specific formatting is needed beyond Debug.

/// Token statistics for the completion request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TokenUsage {
    /// Number of tokens from the prompt which could be re-used from a cache (e.g., Llama.cpp's n_past), if reported.
    pub tokens_cached: Option<usize>,

    /// Number of tokens in the input prompt.
    pub prompt_tokens: usize,
    /// Number of tokens in the generated completion.
    pub completion_tokens: usize,
    /// Total number of tokens used in the request (prompt + completion).
    pub total_tokens: usize,
    /// Estimated dollar cost of the request, if applicable and calculable.
    pub dollar_cost: Option<f32>,
    /// Estimated cents cost of the request, if applicable and calculable.
    pub cents_cost: Option<f32>,
}

/// Helper function to calculate API cost based on token counts and model pricing.
fn calculate_api_cost(
    prompt_tokens: usize,
    completion_tokens: usize,
    model_info: &llm_models::ApiLlmModel,
) -> (Option<f32>, Option<f32>) {
    let cost_per_m_in = model_info.cost_per_m_in_tokens as f32;
    let cost_per_m_out = model_info.cost_per_m_out_tokens as f32;

    if cost_per_m_in == 0.0 && cost_per_m_out == 0.0 {
        return (None, None); // No cost info available
    }

    let input_cost = (prompt_tokens as f32 / 1_000_000.0) * cost_per_m_in;
    let output_cost = (completion_tokens as f32 / 1_000_000.0) * cost_per_m_out;
    let total_dollar_cost = input_cost + output_cost;
    let total_cents_cost = total_dollar_cost * 100.0;

    (Some(total_dollar_cost), Some(total_cents_cost))
}


impl TokenUsage {
    // Constructor for LlamaCpp backend
    #[cfg(feature = "llama_cpp_backend")]
    pub fn new_from_llama(res: &LlamaCppCompletionResponse) -> Self {
        Self {
            tokens_cached: Some(res.tokens_cached as usize),
            prompt_tokens: res.tokens_evaluated as usize,
            completion_tokens: res.timings.predicted_n as usize,
            total_tokens: res.tokens_evaluated as usize + res.timings.predicted_n as usize,
            dollar_cost: None,
            cents_cost: None, // Local models typically don't have direct cost
        }
    }

    // Constructor for MistralRs backend
    #[cfg(feature = "mistral_rs_backend")]
    pub fn new_from_mistral(res: &MistralCompletionResponse) -> Self {
        Self {
            tokens_cached: None, // MistralRs doesn't report cached tokens directly
            prompt_tokens: res.usage.prompt_tokens as usize,
            completion_tokens: res.usage.completion_tokens as usize,
            total_tokens: res.usage.prompt_tokens as usize + res.usage.completion_tokens as usize,
            dollar_cost: None, // Local models typically don't have direct cost
            cents_cost: None,
        }
    }

    /// Constructor for generic OpenAI-compatible backends (including GenericApi).
    /// Assumes the response structure has a `usage` field with token counts.
    pub fn new_from_generic(res: &OpenAiCompletionResponse) -> Self {
        // TODO: Add cost calculation based on model pricing if available
        if let Some(usage) = &res.usage {
            Self {
                tokens_cached: None, // OpenAI doesn't report cached tokens
                prompt_tokens: usage.prompt_tokens,
                completion_tokens: usage.completion_tokens,
                total_tokens: usage.total_tokens,
                dollar_cost: None, // Placeholder for cost calculation
                cents_cost: None,
            }
        } else {
            // Fallback if usage info is missing
            Self {
                tokens_cached: None,
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
                dollar_cost: None,
                cents_cost: None,
            }
        }
    }

    // Constructor for Anthropic backend
    pub fn new_from_anthropic(
        res: &AnthropicCompletionResponse,
        model_info: &llm_models::ApiLlmModel, // Add model_info parameter
    ) -> Self {
        let prompt_tokens = res.usage.input_tokens;
        let completion_tokens = res.usage.output_tokens;
        let (dollar_cost, cents_cost) =
            calculate_api_cost(prompt_tokens, completion_tokens, model_info);

        Self {
            tokens_cached: None, // Anthropic doesn't report cached tokens
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
            dollar_cost,
            cents_cost,
        }
    }
}
// Note: Display implementation can be added if specific formatting is needed beyond Debug.

impl std::fmt::Display for TokenUsage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        writeln!(f, "    tokens_cached: {:?}", self.tokens_cached)?;
        writeln!(f, "    prompt_tokens: {:?}", self.prompt_tokens)?;
        writeln!(f, "    completion_tokens: {:?}", self.completion_tokens)?;
        writeln!(f, "    total_tokens: {:?}", self.total_tokens)?;
        writeln!(f, "    dollar_cost: {:?}", self.dollar_cost)?;
        writeln!(f, "    cents_cost: {:?}", self.cents_cost)
    }
}
