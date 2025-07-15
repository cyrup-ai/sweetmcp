use crate::requests::{common::tools::{ToolChoice, ToolDefinition}, error::CompletionError}; // Correct path for CompletionError
use serde::{Deserialize, Serialize};

/// Configuration for a completion request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompletionRequestConfig {
    /// The maximum number of tokens allowed for the generated answer.
    /// This is the requested max tokens, not the actual used tokens.
    pub max_request_tokens: usize,
    /// The actual maximum number of tokens used for the request after accounting for prompt tokens.
    /// This is calculated internally.
    pub actual_request_tokens: Option<usize>,
    /// Controls randomness: lowering results in less random completions.
    /// As the temperature approaches zero, the model will become deterministic
    /// and repetitive. Value is between 0.0 and 2.0.
    pub temperature: f32,
    /// The nucleus sampling threshold, e.g. 0.1.
    /// Value is between 0.0 and 1.0.
    pub top_p: Option<f32>,
    /// The top-k sampling threshold, e.g. 40.
    /// Value is typically a positive integer.
    pub top_k: Option<usize>,
    /// Penalize new tokens based on their existing frequency in the text so far,
    /// decreasing the model's likelihood to repeat the same line verbatim.
    /// Value is between -2.0 and 2.0.
    pub frequency_penalty: Option<f32>,
    /// Penalize new tokens based on whether they appear in the text so far,
    /// increasing the model's likelihood to talk about new topics.
    /// Value is between -2.0 and 2.0.
    pub presence_penalty: Option<f32>,
    /// The number of completion choices to generate.
    pub n_choices: Option<u32>, // Note: Some backends might only support 1

    /// Whether to stream back partial progress. If set to true, partial message deltas will be
    /// sent as data-only server-sent events as they become available, with the stream terminated
    /// by a data: [DONE] message.
    pub stream_response: bool,

    /// Definitions of tools the model may call.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,
    /// Controls which tool the model should use, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    
    /// The number of tokens requested for the response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_response_tokens: Option<usize>,

    // Fields needed by CompletionRequest::request
    pub retry_after_fail_n_times: u8,
    pub increase_limit_on_fail: bool,
    pub cache_prompt: bool, // Added cache_prompt, seems related to set_cache/clear_cache logic
}

impl Default for CompletionRequestConfig {
    fn default() -> Self {
        Self {
            max_request_tokens: 1024, // Default max tokens
            actual_request_tokens: None,
            temperature: 0.5, // Default temperature (matches Anthropic default)
            top_p: None,
            top_k: None,
            frequency_penalty: None,
            presence_penalty: None,
            n_choices: Some(1),
            stream_response: false,
            tools: None,
            tool_choice: None,
            requested_response_tokens: None,
            retry_after_fail_n_times: 3,  // Default retry count
            increase_limit_on_fail: true, // Default behavior
            cache_prompt: true,           // Default cache behavior
        }
    }
}

impl CompletionRequestConfig {
    pub fn new() -> Self {
        Default::default()
    }

    // Builder methods for convenience
    pub fn with_max_request_tokens(mut self, max_tokens: usize) -> Self {
        self.max_request_tokens = max_tokens;
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    pub fn with_top_k(mut self, top_k: usize) -> Self {
        self.top_k = Some(top_k);
        self
    }

    pub fn with_frequency_penalty(mut self, penalty: f32) -> Self {
        self.frequency_penalty = Some(penalty);
        self
    }

    pub fn with_presence_penalty(mut self, penalty: f32) -> Self {
        self.presence_penalty = Some(penalty);
        self
    }

    pub fn with_n_choices(mut self, n: u32) -> Self {
        self.n_choices = Some(n);
        self
    }

    pub fn with_stream_response(mut self, stream: bool) -> Self {
        self.stream_response = stream;
        self
    }

    pub fn with_tools(mut self, tools: Vec<ToolDefinition>) -> Self {
        self.tools = Some(tools);
        self
    }

    pub fn with_tool_choice(mut self, tool_choice: ToolChoice) -> Self {
        self.tool_choice = Some(tool_choice);
        self
    }

    // Method needed by CompletionRequest::request
    pub fn set_max_tokens_for_request(&mut self, total_prompt_tokens: usize) -> Result<(), String> {
        let available_tokens = self.max_request_tokens.saturating_sub(total_prompt_tokens);
        if available_tokens == 0 {
            Err(format!(
                "Prompt tokens ({}) exceed or equal max request tokens ({}). No space left for generation.",
                total_prompt_tokens, self.max_request_tokens
            ))
        } else {
            self.actual_request_tokens = Some(available_tokens);
            Ok(())
        }
    }

    // Method needed by CompletionRequest::request
    pub fn increase_token_limit(
        &mut self,
        total_prompt_tokens: usize,
        increase_by: Option<usize>,
    ) -> Result<(), CompletionError> {
        let increase_amount = increase_by.unwrap_or(50); // Default increase
        self.max_request_tokens += increase_amount;
        // Recalculate actual_request_tokens after increasing the limit
        self.set_max_tokens_for_request(total_prompt_tokens)
            .map_err(|err: String| {
                // Convert String to RequestTokenLimitError using GenericPromptError variant
                let token_err = llm_prompt::RequestTokenLimitError::GenericPromptError { 
                    e: err 
                };
                CompletionError::RequestTokenLimitError(token_err)
            })
    }
}
