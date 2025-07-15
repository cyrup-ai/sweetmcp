// OpenAI-specific streaming response handling
use crate::requests::{
    CompletionChunk, CompletionDelta, CompletionError, CompletionFinishReason, CompletionUsage,
    SseEvent,
};
use serde::{Deserialize, Serialize};

/// OpenAI streaming response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiStreamingResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<OpenAiStreamingChoice>,
    pub usage: Option<OpenAiUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiStreamingChoice {
    pub index: u32,
    pub delta: OpenAiDelta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiDelta {
    pub role: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

impl OpenAiStreamingResponse {
    /// Convert OpenAI streaming response to generic CompletionChunk
    pub fn to_completion_chunk(self) -> Result<CompletionChunk, CompletionError> {
        let choice = self.choices.into_iter().next().ok_or_else(|| {
            CompletionError::RequestBuilderError("No choices in OpenAI streaming response".to_string())
        })?;

        let finish_reason = choice.finish_reason.and_then(|reason| {
            match reason.as_str() {
                "stop" => Some(CompletionFinishReason::Eos),
                "length" => Some(CompletionFinishReason::StopLimit),
                "content_filter" => Some(CompletionFinishReason::NonMatchingStoppingSequence(None)),
                _ => Some(CompletionFinishReason::Eos),
            }
        });

        let usage = self.usage.map(|u| CompletionUsage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        });

        Ok(CompletionChunk {
            id: self.id,
            model: self.model,
            created: Some(self.created),
            delta: CompletionDelta {
                content: choice.delta.content,
                role: choice.delta.role,
            },
            finish_reason,
            usage,
        })
    }
}

/// Parse OpenAI SSE event into CompletionChunk
pub fn parse_openai_sse_event(event: SseEvent) -> Result<Option<CompletionChunk>, CompletionError> {
    // Handle special events
    if event.data == "[DONE]" {
        return Ok(None); // End of stream
    }

    // Parse JSON data
    match serde_json::from_str::<OpenAiStreamingResponse>(&event.data) {
        Ok(response) => Ok(Some(response.to_completion_chunk()?)),
        Err(e) => {
            // Log the error but don't fail the entire stream for malformed chunks
            tracing::warn!("Failed to parse OpenAI SSE event: {} - Data: {}", e, event.data);
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_openai_streaming_response() {
        let json = r#"{
            "id": "chatcmpl-123",
            "object": "chat.completion.chunk",
            "created": 1677652288,
            "model": "gpt-4",
            "choices": [{
                "index": 0,
                "delta": {
                    "content": "Hello"
                },
                "finish_reason": null
            }]
        }"#;

        let response: OpenAiStreamingResponse = serde_json::from_str(json).unwrap();
        let chunk = response.to_completion_chunk().unwrap();

        assert_eq!(chunk.id, "chatcmpl-123");
        assert_eq!(chunk.model, "gpt-4");
        assert_eq!(chunk.delta.content, Some("Hello".to_string()));
        assert_eq!(chunk.finish_reason, None);
    }

    #[test]
    fn test_parse_openai_sse_done_event() {
        let event = SseEvent {
            event_type: None,
            data: "[DONE]".to_string(),
            id: None,
            retry: None,
        };

        let result = parse_openai_sse_event(event).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_openai_sse_content_event() {
        let json = r#"{
            "id": "chatcmpl-123",
            "object": "chat.completion.chunk",
            "created": 1677652288,
            "model": "gpt-4",
            "choices": [{
                "index": 0,
                "delta": {
                    "content": "Hello"
                },
                "finish_reason": null
            }]
        }"#;

        let event = SseEvent {
            event_type: Some("message".to_string()),
            data: json.to_string(),
            id: Some("123".to_string()),
            retry: None,
        };

        let chunk = parse_openai_sse_event(event).unwrap().unwrap();
        assert_eq!(chunk.delta.content, Some("Hello".to_string()));
    }
}