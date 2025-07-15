use crate::requests::*;
use crate::llms::local::llama_cpp::tool_use::parse_tool_calls;
use serde::{Deserialize, Serialize};


impl CompletionResponse {
    #[cfg(feature = "llama_cpp_backend")]
    pub fn new_from_llama(
        req: &CompletionRequest,
        res: LlamaCppCompletionResponse,
    ) -> Result<Self, CompletionError> {
        let finish_reason = if res.stop_type == "eos" {
            CompletionFinishReason::Eos
        } else if res.stop_type == "limit" {
            CompletionFinishReason::StopLimit
        } else if res.stop_type == "word" {
            match req.stop_sequences.parse_string_response(&res.stopping_word) {
                Some(stop_sequence) => {
                    CompletionFinishReason::MatchingStoppingSequence(stop_sequence)
                }
                None => CompletionFinishReason::NonMatchingStoppingSequence(Some(
                    res.stopping_word.clone(),
                )),
            }
        } else {
            return Err(CompletionError::StopReasonUnsupported(
                "No stop reason provided".to_owned(),
            ));
        };
        
        // Check for tool calls in the content
        let mut content = res.content.to_owned();
        let mut tool_calls = None;
        
        // Only attempt to parse tool calls if tools were specified in the request
        let has_tools = req.config.tools.as_ref().map_or(false, |t| !t.is_empty());
        
        if has_tools {
            let parsed_tool_calls = parse_tool_calls(&content);
            
            if !parsed_tool_calls.is_empty() {
                // Convert the llama specific tool calls to the common format
                let common_tool_calls = parsed_tool_calls
                    .iter()
                    .map(|tc| tc.into())
                    .collect();
                
                tool_calls = Some(common_tool_calls);
                
                // Optionally remove the tool call syntax from the content
                // This is model-specific and might be needed for some UIs
                // This regex will attempt to match and remove the tool call syntax
                if let Ok(re) = regex::Regex::new(r"<tool:.*?</tool>|\w+\s*\(\s*\{.*?\}\s*\)") {
                    content = re.replace_all(&content, "").trim().to_string();
                }
                
                // If the content is now empty after removing tool calls, add a note
                if content.trim().is_empty() {
                    content = "[Model response contains only tool calls]".to_string();
                }
            }
        }

        Ok(Self {
            id: "llama_cpp".to_owned(),
            index: None,
            content,
            finish_reason,
            completion_probabilities: None,
            truncated: res.truncated,
            generation_settings: GenerationSettings::new_from_llama_cpp(req, &res),
            timing_usage: TimingUsage::new_from_llama(&res, req.start_time),
            token_usage: TokenUsage::new_from_llama(&res),
            tool_calls,
        })
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq, Serialize)]
pub struct LlamaCppCompletionResponse {
    pub content: String,
    pub model: String,
    pub prompt: String,
    pub generation_settings: LlamaGenerationSettings,
    pub timings: LlamaTimings,
    pub stop_type: String,
    pub stopping_word: String,
    pub tokens_cached: usize,
    pub tokens_evaluated: usize,
    pub truncated: bool,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Serialize)]
pub struct LlamaGenerationSettings {
    pub frequency_penalty: f32,
    pub presence_penalty: f32,
    pub temperature: f32,
    pub top_p: f32,
    pub n_predict: isize,
    pub logit_bias: Option<Vec<serde_json::Value>>,
    pub grammar: String,
    pub stop: Vec<String>,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Serialize)]
pub struct LlamaTimings {
    pub predicted_ms: f32,
    pub prompt_per_token_ms: f32,
    pub predicted_per_token_ms: f32,
    pub prompt_ms: f32,
    pub prompt_per_second: f32,
    pub predicted_n: f32,
    pub prompt_n: f32,
    pub predicted_per_second: f32,
}
