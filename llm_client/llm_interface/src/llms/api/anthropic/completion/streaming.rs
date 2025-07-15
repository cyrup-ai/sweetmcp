// Anthropic-specific streaming response handling
use crate::requests::{
    CompletionChunk, CompletionDelta, CompletionError, CompletionFinishReason, CompletionUsage,
    SseEvent,
};
use serde::{Deserialize, Serialize};

/// Anthropic streaming event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AnthropicStreamingEvent {
    #[serde(rename = "message_start")]
    MessageStart {
        message: AnthropicMessage,
    },
    #[serde(rename = "content_block_start")]
    ContentBlockStart {
        index: u32,
        content_block: AnthropicContentBlock,
    },
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta {
        index: u32,
        delta: AnthropicDelta,
    },
    #[serde(rename = "content_block_stop")]
    ContentBlockStop {
        index: u32,
    },
    #[serde(rename = "message_delta")]
    MessageDelta {
        delta: AnthropicMessageDelta,
        usage: Option<AnthropicUsage>,
    },
    #[serde(rename = "message_stop")]
    MessageStop,
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "error")]
    Error {
        error: AnthropicError,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicMessage {
    pub id: String,
    #[serde(rename = "type")]
    pub message_type: String,
    pub role: String,
    pub content: Vec<AnthropicContentBlock>,
    pub model: String,
    pub stop_reason: Option<String>,
    pub stop_sequence: Option<String>,
    pub usage: AnthropicUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AnthropicContentBlock {
    #[serde(rename = "text")]
    Text {
        text: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AnthropicDelta {
    #[serde(rename = "text_delta")]
    TextDelta {
        text: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicMessageDelta {
    pub stop_reason: Option<String>,
    pub stop_sequence: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicError {
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
}

/// Context for tracking streaming state
#[derive(Debug, Default)]
pub struct AnthropicStreamingContext {
    pub message_id: Option<String>,
    pub model: Option<String>,
    pub role: Option<String>,
    pub accumulated_content: String,
}

impl AnthropicStreamingEvent {
    /// Convert Anthropic streaming event to generic CompletionChunk
    pub fn to_completion_chunk(
        self,
        context: &mut AnthropicStreamingContext,
    ) -> Result<Option<CompletionChunk>, CompletionError> {
        match self {
            AnthropicStreamingEvent::MessageStart { message } => {
                context.message_id = Some(message.id.clone());
                context.model = Some(message.model.clone());
                context.role = Some(message.role.clone());
                context.accumulated_content.clear();

                Ok(Some(CompletionChunk {
                    id: message.id,
                    model: message.model,
                    created: None,
                    delta: CompletionDelta {
                        content: None,
                        role: Some(message.role),
                    },
                    finish_reason: None,
                    usage: None,
                }))
            }
            AnthropicStreamingEvent::ContentBlockDelta { delta, .. } => {
                match delta {
                    AnthropicDelta::TextDelta { text } => {
                        context.accumulated_content.push_str(&text);

                        Ok(Some(CompletionChunk {
                            id: context.message_id.clone().unwrap_or_default(),
                            model: context.model.clone().unwrap_or_default(),
                            created: None,
                            delta: CompletionDelta {
                                content: Some(text),
                                role: None,
                            },
                            finish_reason: None,
                            usage: None,
                        }))
                    }
                }
            }
            AnthropicStreamingEvent::MessageDelta { delta, usage } => {
                let finish_reason = delta.stop_reason.as_ref().map(|reason| {
                    match reason.as_str() {
                        "end_turn" => CompletionFinishReason::Eos,
                        "max_tokens" => CompletionFinishReason::StopLimit,
                        "stop_sequence" => CompletionFinishReason::MatchingStoppingSequence(
                            delta.stop_sequence.clone().unwrap_or_default(),
                        ),
                        _ => CompletionFinishReason::Eos,
                    }
                });

                let completion_usage = usage.map(|u| CompletionUsage {
                    prompt_tokens: u.input_tokens,
                    completion_tokens: u.output_tokens,
                    total_tokens: u.input_tokens + u.output_tokens,
                });

                Ok(Some(CompletionChunk {
                    id: context.message_id.clone().unwrap_or_default(),
                    model: context.model.clone().unwrap_or_default(),
                    created: None,
                    delta: CompletionDelta {
                        content: None,
                        role: None,
                    },
                    finish_reason,
                    usage: completion_usage,
                }))
            }
            AnthropicStreamingEvent::MessageStop => {
                // Final chunk indicating end of stream
                Ok(Some(CompletionChunk {
                    id: context.message_id.clone().unwrap_or_default(),
                    model: context.model.clone().unwrap_or_default(),
                    created: None,
                    delta: CompletionDelta {
                        content: None,
                        role: None,
                    },
                    finish_reason: Some(CompletionFinishReason::Eos),
                    usage: None,
                }))
            }
            AnthropicStreamingEvent::Error { error } => {
                Err(CompletionError::RequestBuilderError(format!(
                    "Anthropic streaming error: {} - {}",
                    error.error_type, error.message
                )))
            }
            AnthropicStreamingEvent::Ping
            | AnthropicStreamingEvent::ContentBlockStart { .. }
            | AnthropicStreamingEvent::ContentBlockStop { .. } => {
                // These events don't produce completion chunks
                Ok(None)
            }
        }
    }
}

/// Parse Anthropic SSE event into CompletionChunk
pub fn parse_anthropic_sse_event(
    event: SseEvent,
    context: &mut AnthropicStreamingContext,
) -> Result<Option<CompletionChunk>, CompletionError> {
    // Parse JSON data
    match serde_json::from_str::<AnthropicStreamingEvent>(&event.data) {
        Ok(streaming_event) => streaming_event.to_completion_chunk(context),
        Err(e) => {
            // Log the error but don't fail the entire stream for malformed chunks
            tracing::warn!("Failed to parse Anthropic SSE event: {} - Data: {}", e, event.data);
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_anthropic_message_start() {
        let json = r#"{
            "type": "message_start",
            "message": {
                "id": "msg_123",
                "type": "message",
                "role": "assistant",
                "content": [],
                "model": "claude-3-sonnet-20240229",
                "stop_reason": null,
                "stop_sequence": null,
                "usage": {
                    "input_tokens": 10,
                    "output_tokens": 0
                }
            }
        }"#;

        let event: AnthropicStreamingEvent = serde_json::from_str(json).unwrap();
        let mut context = AnthropicStreamingContext::default();
        let chunk = event.to_completion_chunk(&mut context).unwrap().unwrap();

        assert_eq!(chunk.id, "msg_123");
        assert_eq!(chunk.model, "claude-3-sonnet-20240229");
        assert_eq!(chunk.delta.role, Some("assistant".to_string()));
        assert_eq!(context.message_id, Some("msg_123".to_string()));
    }

    #[test]
    fn test_parse_anthropic_content_delta() {
        let json = r#"{
            "type": "content_block_delta",
            "index": 0,
            "delta": {
                "type": "text_delta",
                "text": "Hello"
            }
        }"#;

        let event: AnthropicStreamingEvent = serde_json::from_str(json).unwrap();
        let mut context = AnthropicStreamingContext {
            message_id: Some("msg_123".to_string()),
            model: Some("claude-3-sonnet-20240229".to_string()),
            role: Some("assistant".to_string()),
            accumulated_content: String::new(),
        };

        let chunk = event.to_completion_chunk(&mut context).unwrap().unwrap();

        assert_eq!(chunk.delta.content, Some("Hello".to_string()));
        assert_eq!(context.accumulated_content, "Hello");
    }

    #[test]
    fn test_parse_anthropic_message_delta_with_stop() {
        let json = r#"{
            "type": "message_delta",
            "delta": {
                "stop_reason": "end_turn",
                "stop_sequence": null
            },
            "usage": {
                "input_tokens": 10,
                "output_tokens": 25
            }
        }"#;

        let event: AnthropicStreamingEvent = serde_json::from_str(json).unwrap();
        let mut context = AnthropicStreamingContext {
            message_id: Some("msg_123".to_string()),
            model: Some("claude-3-sonnet-20240229".to_string()),
            role: Some("assistant".to_string()),
            accumulated_content: String::new(),
        };

        let chunk = event.to_completion_chunk(&mut context).unwrap().unwrap();

        assert_eq!(chunk.finish_reason, Some(CompletionFinishReason::Eos));
        assert!(chunk.usage.is_some());
        let usage = chunk.usage.unwrap();
        assert_eq!(usage.prompt_tokens, 10);
        assert_eq!(usage.completion_tokens, 25);
        assert_eq!(usage.total_tokens, 35);
    }
}