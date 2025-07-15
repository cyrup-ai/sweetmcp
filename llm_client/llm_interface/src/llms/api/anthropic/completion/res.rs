use crate::{
    llms::api::error::ApiError, // Keep ApiError for StreamError
    requests::{
        CompletionError, // Use public re-export
        CompletionFinishReason, CompletionRequest, CompletionResponse,
        GenerationSettings, TimingUsage, TokenUsage,
        common::tools::{ToolCall, ToolCallSummary}, // Import from common::tools
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

// --- Structures for Non-Streaming Response ---

impl CompletionResponse {
    /// Creates a generic `CompletionResponse` from an `AnthropicCompletionResponse`.
    pub fn new_from_anthropic(
        req: &CompletionRequest,
        res: AnthropicCompletionResponse,
    ) -> Result<Self, CompletionError> {
        let mut finish_reason = match res.stop_reason {
            StopReason::EndTurn => CompletionFinishReason::Eos,
            StopReason::StopSequence => {
                if let Some(stopping_string) = &res.stop_sequence {
                    if let Some(stop_sequence) =
                        req.stop_sequences.parse_string_response(stopping_string)
                    {
                        CompletionFinishReason::MatchingStoppingSequence(stop_sequence)
                    } else {
                        CompletionFinishReason::NonMatchingStoppingSequence(Some(
                            stopping_string.clone(),
                        ))
                    }
                } else {
                    CompletionFinishReason::NonMatchingStoppingSequence(None)
                }
            }
            StopReason::MaxTokens => CompletionFinishReason::StopLimit,
            StopReason::ToolUse => CompletionFinishReason::ToolUse(None), // Placeholder
        };

        // Process content blocks: accumulate text and extract tool calls
        let mut content_text = String::new();
        let mut extracted_tool_calls: Vec<ToolCall> = Vec::new(); // Explicit type

        for block in &res.content {
            match block {
                CompletionContent::Text { text } => {
                    content_text.push_str(text);
                }
                CompletionContent::ToolUse { id, name, input } => extracted_tool_calls.push(
                    ToolCall {
                        id: id.clone(),
                        name: name.clone(),
                        arguments: input.clone(), // Anthropic 'input' maps to generic 'arguments'
                    },
                ),
            }
        }

        // Update finish_reason based on extracted calls and original stop reason
        if !extracted_tool_calls.is_empty() {
            let summaries = extracted_tool_calls
                .iter()
                .map(|call| ToolCallSummary {
                    name: call.name.clone(),
                    input: call.arguments.clone(),
                })
                .collect();
            // If original reason wasn't ToolUse, warn but prioritize ToolUse because calls exist
            if !matches!(res.stop_reason, StopReason::ToolUse) {
                tracing::warn!("Detected tool_use content blocks, but original stop_reason was {:?}. Overriding finish_reason to ToolUse.", res.stop_reason);
            }
            finish_reason = CompletionFinishReason::ToolUse(Some(summaries));
        } else if matches!(res.stop_reason, StopReason::ToolUse) {
            // Original reason was ToolUse, but no calls found - this is an inconsistency
            return Err(CompletionError::GenericError(
                "Response stop_reason was ToolUse, but no tool_use content blocks found."
                    .to_string(),
            ));
        }

        // Check for empty content only if the finish reason is *not* ToolUse
        if content_text.is_empty() && !matches!(finish_reason, CompletionFinishReason::ToolUse(_)) {
            return Err(CompletionError::ReponseContentEmpty);
        }

        // Assign extracted tool calls
        let tool_calls = if extracted_tool_calls.is_empty() {
            None
        } else {
            Some(extracted_tool_calls)
        };

        Ok(Self {
            id: res.id.to_owned(),
            index: None,           // Anthropic doesn't provide an index like OpenAI
            content: content_text, // Use the accumulated text content
            finish_reason,
            completion_probabilities: None, // Anthropic doesn't provide completion probabilities
            truncated: matches!(res.stop_reason, StopReason::MaxTokens), // Set truncated based on stop_reason
            tool_calls, // Assign the extracted tool calls
            generation_settings: GenerationSettings::new_from_anthropic(req, &res), // Use the struct from settings.rs
            timing_usage: TimingUsage::new_from_generic(req.start_time),
            token_usage: TokenUsage::new_from_anthropic(&res, &req.backend.anthropic().unwrap().model), // Pass model info
        })
    }
}

/// Represents the full Anthropic Messages API response structure (non-streaming).
#[derive(Debug, Deserialize, Clone, PartialEq, Serialize)]
pub struct AnthropicCompletionResponse {
    /// Unique object identifier.
    ///
    /// The format and length of IDs may change over time.
    pub id: String,
    /// Content generated by the model.
    ///
    /// This is an array of content blocks, each of which has a type that determines its shape.
    /// Known types: "text", "tool_use".
    pub content: Vec<CompletionContent>,
    /// The model that handled the request.
    pub model: String, // Consider using an enum if models are fixed
    /// The reason that we stopped.
    ///
    /// This may be one of the following values:
    ///
    /// "end_turn": the model reached a natural stopping point
    /// "max_tokens": we exceeded the requested max_tokens or the model's maximum
    /// "stop_sequence": one of your provided custom stop_sequences was generated
    pub stop_reason: StopReason,
    /// Which custom stop sequence was generated, if any.
    ///
    /// This value will be a non-null string if one of your custom stop sequences was generated.
    pub stop_sequence: Option<String>,
    /// Billing and rate-limit usage.
    ///
    /// Anthropic's API bills and rate-limits by token counts, as tokens represent the underlying cost to our systems.
    ///
    /// Under the hood, the API transforms requests into a format suitable for the model. The model's output then goes through a parsing stage before becoming an API response. As a result, the token counts in usage will not match one-to-one with the exact visible content of an API request or response.
    ///
    /// For example, output_tokens will be non-zero, even for an empty string response from Claude.
    pub usage: CompletionUsage,
}

// --- Common Structures (Used in both Streaming and Non-Streaming) ---

/// Represents different types of content blocks in responses or stream events.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(tag = "type")] // Use the 'type' field to determine the variant
#[serde(rename_all = "snake_case")]
pub enum CompletionContent {
    Text {
        text: String,
    },
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
}

/// Usage statistics for the completion request.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct CompletionUsage {
    /// The number of input tokens which were used.
    pub input_tokens: usize,
    /// The number of output tokens which were used.
    pub output_tokens: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    /// The model reached a natural stopping point.
    EndTurn,
    /// We exceeded the requested max_tokens or the model's maximum.
    MaxTokens,
    /// One of your provided custom stop_sequences was generated.
    StopSequence,
    /// Claude wants to use an external tool.
    ToolUse,
}

// --- Structures for Streaming Response (Server-Sent Events) ---

/// Represents a single event received from the Anthropic Messages SSE stream.
#[derive(Debug, Deserialize, Clone, PartialEq, Serialize)]
#[serde(tag = "type")] // The 'type' field in the JSON data determines the variant
#[serde(rename_all = "snake_case")]
pub enum AnthropicStreamEvent {
    MessageStart {
        message: MessageStart,
    },
    ContentBlockStart {
        index: usize,
        content_block: ContentBlockStart,
    },
    Ping, // Keepalive event
    ContentBlockDelta {
        index: usize,
        delta: Delta,
    },
    ContentBlockStop {
        index: usize,
    },
    MessageDelta {
        delta: MessageDelta,
        usage: CompletionUsage,
    },
    MessageStop {
        // Contains the final StopReason, but the JSON payload itself might just be {"type": "message_stop"}
        // The actual StopReason comes from the MessageDelta event before this.
        // Let's adjust the enum variant if needed based on actual stream data.
        // For now, assume it might contain the reason directly for robustness.
        #[serde(skip_serializing_if = "Option::is_none")] // Make optional if not always present
        stop_reason: Option<StopReason>,
    },
    Error {
        error: StreamError,
    },
}

/// Data associated with the `message_start` event.
#[derive(Debug, Deserialize, Clone, PartialEq, Serialize)]
pub struct MessageStart {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String, // "message"
    pub role: String,                    // "assistant"
    pub content: Vec<CompletionContent>, // Usually empty
    pub model: String,
    pub stop_reason: Option<StopReason>, // Usually null
    pub stop_sequence: Option<String>,   // Usually null
    pub usage: CompletionUsage,          // Initial usage (input tokens)
}

/// Data associated with the `content_block_start` event.
#[derive(Debug, Deserialize, Clone, PartialEq, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum ContentBlockStart {
    Text {
        text: String,
    }, // Usually empty
    ToolUse {
        id: String,
        name: String,
        input: JsonValue,
    }, // Input might be partial? Assume complete.
}

/// Data associated with the `content_block_delta` event.
#[derive(Debug, Deserialize, Clone, PartialEq, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Delta {
    TextDelta { text: String },
    InputJsonDelta { partial_json: String }, // For tool use input streaming
}

/// Data associated with the `message_delta` event.
#[derive(Debug, Deserialize, Clone, PartialEq, Serialize)]
pub struct MessageDelta {
    pub stop_reason: StopReason,
    pub stop_sequence: Option<String>,
    // Usage is reported here according to docs example for final output tokens
    pub usage: Option<MessageDeltaUsage>, // Make usage optional as it might only appear at the end
}

/// Usage information specific to the `message_delta` event.
#[derive(Debug, Deserialize, Clone, PartialEq, Serialize)]
pub struct MessageDeltaUsage {
    pub output_tokens: usize,
}

/// Data associated with the `error` event.
#[derive(Debug, Deserialize, Clone, PartialEq, Serialize)]
pub struct StreamError {
    #[serde(rename = "type")]
    pub error_type: String, // e.g., "error"
    pub error: ApiError, // Use imported ApiError
}

/// Parses lines belonging to a single Server-Sent Event into an `AnthropicStreamEvent`.
///
/// Handles `event:` and `data:` fields according to the SSE specification.
/// Anthropic uses the `event:` field to denote the type (e.g., `message_start`, `ping`)
/// and the `data:` field contains a JSON payload (except for `ping`).
pub fn parse_sse_event(lines: &[String]) -> Result<Option<AnthropicStreamEvent>, String> {
    let mut event_name: Option<String> = None;
    let mut data_buf = String::new();

    for line in lines {
        if let Some(name) = line.strip_prefix("event:") {
            event_name = Some(name.trim().to_string());
        } else if let Some(data) = line.strip_prefix("data:") {
            // Append data, handling potential multi-line data (though Anthropic seems single-line JSON)
            // No need to add newline here as JSON parser handles the combined string.
            data_buf.push_str(data.trim_start()); // Trim leading space often present after "data:"
        } // Ignore empty lines and comments (lines starting with ':')
    }

    let event_name =
        event_name.ok_or_else(|| format!("SSE event name missing in lines: {:?}", lines))?;

    // Handle events based on the 'event:' field name
    match event_name.as_str() {
        "ping" => Ok(Some(AnthropicStreamEvent::Ping)),
        "error" => {
            // Error event has JSON data
            if data_buf.is_empty() {
                return Err(format!("SSE data missing for event: {}", event_name));
            }
            // Deserialize the data part into the StreamError struct first
            match serde_json::from_str::<StreamError>(&data_buf) {
                Ok(stream_error) => Ok(Some(AnthropicStreamEvent::Error {
                    error: stream_error,
                })),
                Err(e) => Err(format!(
                    "Failed to parse JSON for SSE event '{}': {}. Data: '{}'",
                    event_name, e, data_buf
                )),
            }
        }
        // All other known events have JSON data where the 'type' field inside the JSON
        // matches the event name. We deserialize into the enum directly.
        "message_start"
        | "content_block_start"
        | "content_block_delta"
        | "content_block_stop"
        | "message_delta"
        | "message_stop" => {
            if data_buf.is_empty() {
                // content_block_stop and message_stop might legitimately have no data?
                // Let's check the spec again. Example shows message_stop has data: {"type": "message_stop"}
                // Example shows content_block_stop has data: {"type": "content_block_stop", "index": 0}
                // So, data should generally be present.
                return Err(format!("SSE data missing for event: {}", event_name));
            }
            // Use serde's externally tagged enum deserialization based on the 'type' field in the JSON
            serde_json::from_str::<AnthropicStreamEvent>(&data_buf)
                .map(Some)
                .map_err(|e| {
                    format!(
                        "Failed to parse JSON for SSE event '{}': {}. Data: '{}'",
                        event_name, e, data_buf
                    )
                })
        }
        _ => Err(format!("Unknown SSE event name: {}", event_name)),
    }
}

#[cfg(test)]
mod tests {
    // Keep existing tests for non-streaming response parsing
    use super::*;
    use crate::llms::LlmBackend; // Assuming LlmBackend is accessible
    // Use the public re-exports rather than private modules
    use crate::requests::StoppingSequence;
    use llm_models::ApiLlmModel;
    use std::sync::Arc;

    // Helper to create a dummy Anthropic backend Arc
    fn dummy_anthropic_backend() -> Arc<LlmBackend> {
        let config = crate::llms::api::anthropic::AnthropicConfig::default();
        let model = ApiLlmModel::default();
        let backend = crate::llms::api::anthropic::AnthropicBackend {
            client: crate::llms::api::ApiClient::new(config),
            model,
        };
        Arc::new(LlmBackend::Anthropic(backend))
    }

    // Helper to create a basic CompletionRequest for testing response parsing
    fn create_base_test_request() -> CompletionRequest {
        CompletionRequest::new(dummy_anthropic_backend())
    }

    #[test]
    fn test_response_new_from_anthropic_text() {
        let req = create_base_test_request();
        let anthropic_res = AnthropicCompletionResponse {
            id: "msg_1".to_string(),
            content: vec![CompletionContent::Text {
                text: "Hello there".to_string(),
            }],
            model: "claude-3-5-sonnet-20240620".to_string(),
            stop_reason: StopReason::EndTurn,
            stop_sequence: None,
            usage: CompletionUsage {
                input_tokens: 10,
                output_tokens: 5,
            },
        };

        let generic_res = CompletionResponse::new_from_anthropic(&req, anthropic_res).unwrap();

        assert_eq!(generic_res.id, "msg_1");
        assert_eq!(generic_res.content, "Hello there");
        assert_eq!(generic_res.finish_reason, CompletionFinishReason::Eos);
        assert!(generic_res.tool_calls.is_none());
        assert_eq!(generic_res.token_usage.prompt_tokens, 10);
        assert_eq!(generic_res.token_usage.completion_tokens, 5);
    }

    #[test]
    fn test_response_new_from_anthropic_stop_sequence() {
        let mut req = create_base_test_request();
        req.stop_sequences = StopSequences {
            sequences: vec![StoppingSequence::InferenceDone(" Human:".to_string())],
            required: false,
        };
        let anthropic_res = AnthropicCompletionResponse {
            id: "msg_2".to_string(),
            content: vec![CompletionContent::Text {
                text: "Okay.".to_string(),
            }],
            model: "claude-3-5-sonnet-20240620".to_string(),
            stop_reason: StopReason::StopSequence,
            stop_sequence: Some(" Human:".to_string()),
            usage: CompletionUsage {
                input_tokens: 8,
                output_tokens: 2,
            },
        };

        let generic_res = CompletionResponse::new_from_anthropic(&req, anthropic_res).unwrap();

        assert_eq!(generic_res.content, "Okay.");
        assert!(matches!(
            generic_res.finish_reason,
            CompletionFinishReason::MatchingStoppingSequence(_)
        ));
        if let CompletionFinishReason::MatchingStoppingSequence(seq) = generic_res.finish_reason {
            assert_eq!(seq.as_str(), " Human:");
        }
        assert!(generic_res.tool_calls.is_none());
    }

    #[test]
    fn test_response_new_from_anthropic_tool_use() {
        let req = create_base_test_request();
        let tool_input = serde_json::json!({"location": "SFO"});
        let anthropic_res = AnthropicCompletionResponse {
            id: "msg_3".to_string(),
            content: vec![
                CompletionContent::Text {
                    text: "Okay, using the tool.".to_string(),
                },
                CompletionContent::ToolUse {
                    id: "toolu_abc".to_string(),
                    name: "get_weather".to_string(),
                    input: tool_input.clone(),
                },
            ],
            model: "claude-3-5-sonnet-20240620".to_string(),
            stop_reason: StopReason::ToolUse,
            stop_sequence: None,
            usage: CompletionUsage {
                input_tokens: 15,
                output_tokens: 25,
            },
        };

        let generic_res = CompletionResponse::new_from_anthropic(&req, anthropic_res).unwrap();

        assert_eq!(generic_res.id, "msg_3");
        assert_eq!(generic_res.content, "Okay, using the tool."); // Only text part
        assert!(matches!(
            generic_res.finish_reason,
            CompletionFinishReason::ToolUse(Some(_))
        ));
        assert!(generic_res.tool_calls.is_some());
        let calls = generic_res.tool_calls.unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].id, "toolu_abc");
        assert_eq!(calls[0].name, "get_weather");
        assert_eq!(calls[0].arguments, tool_input);
    }

    #[test]
    fn test_response_new_from_anthropic_tool_use_only() {
        let req = create_base_test_request();
        let tool_input = serde_json::json!({"location": "SFO"});
        let anthropic_res = AnthropicCompletionResponse {
            id: "msg_4".to_string(),
            content: vec![CompletionContent::ToolUse {
                // No text block
                id: "toolu_abc".to_string(),
                name: "get_weather".to_string(),
                input: tool_input.clone(),
            }],
            model: "claude-3-5-sonnet-20240620".to_string(),
            stop_reason: StopReason::ToolUse,
            stop_sequence: None,
            usage: CompletionUsage {
                input_tokens: 15,
                output_tokens: 20,
            },
        };

        let generic_res = CompletionResponse::new_from_anthropic(&req, anthropic_res).unwrap();

        assert_eq!(generic_res.id, "msg_4");
        assert_eq!(generic_res.content, ""); // Content is empty
        assert!(matches!(
            generic_res.finish_reason,
            CompletionFinishReason::ToolUse(Some(_))
        ));
        assert!(generic_res.tool_calls.is_some());
        assert_eq!(generic_res.tool_calls.as_ref().unwrap().len(), 1);
        assert_eq!(generic_res.tool_calls.as_ref().unwrap()[0].id, "toolu_abc");
    }

    #[test]
    fn test_response_new_from_anthropic_tool_use_content_but_other_stop_reason() {
        // Scenario where model outputs tool use but hits max tokens simultaneously
        let req = create_base_test_request();
        let tool_input = serde_json::json!({"location": "SFO"});
        let anthropic_res = AnthropicCompletionResponse {
            id: "msg_5".to_string(),
            content: vec![CompletionContent::ToolUse {
                id: "toolu_abc".to_string(),
                name: "get_weather".to_string(),
                input: tool_input.clone(),
            }],
            model: "claude-3-5-sonnet-20240620".to_string(),
            stop_reason: StopReason::MaxTokens, // Stop reason is MaxTokens
            stop_sequence: None,
            usage: CompletionUsage {
                input_tokens: 15,
                output_tokens: 20,
            },
        };

        let generic_res = CompletionResponse::new_from_anthropic(&req, anthropic_res).unwrap();

        // We decided to prioritize ToolUse finish reason if tool calls are present
        assert!(matches!(
            generic_res.finish_reason,
            CompletionFinishReason::ToolUse(Some(_))
        ));
        assert!(generic_res.tool_calls.is_some());
        assert_eq!(generic_res.tool_calls.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_response_new_from_anthropic_empty_content_error() {
        let req = create_base_test_request();
        let anthropic_res = AnthropicCompletionResponse {
            id: "msg_err".to_string(),
            content: vec![], // Empty content
            model: "claude-3-5-sonnet-20240620".to_string(),
            stop_reason: StopReason::EndTurn, // Not ToolUse
            stop_sequence: None,
            usage: CompletionUsage {
                input_tokens: 5,
                output_tokens: 0,
            },
        };

        let result = CompletionResponse::new_from_anthropic(&req, anthropic_res);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CompletionError::ReponseContentEmpty
        ));
    }
}
