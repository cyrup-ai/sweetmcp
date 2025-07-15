// Streaming completion response types and utilities
use crate::requests::completion::{CompletionError, CompletionFinishReason};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use futures_util::Stream;

/// Represents a single chunk in a streaming completion response
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompletionChunk {
    /// The ID of the completion
    pub id: String,
    
    /// The model used for the completion
    pub model: String,
    
    /// Timestamp when the chunk was created
    pub created: Option<u64>,
    
    /// The delta content for this chunk
    pub delta: CompletionDelta,
    
    /// The finish reason if this is the final chunk
    pub finish_reason: Option<CompletionFinishReason>,
    
    /// Usage information (only present in the final chunk)
    pub usage: Option<CompletionUsage>,
}

/// Delta content within a streaming chunk
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompletionDelta {
    /// The incremental content for this chunk
    pub content: Option<String>,
    
    /// Role information (typically only in the first chunk)
    pub role: Option<String>,
}

/// Usage statistics for the completion
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompletionUsage {
    /// Number of tokens in the prompt
    pub prompt_tokens: u32,
    
    /// Number of tokens in the completion
    pub completion_tokens: u32,
    
    /// Total tokens used
    pub total_tokens: u32,
}

/// Type alias for a completion stream
pub type CompletionStream = Pin<Box<dyn Stream<Item = Result<CompletionChunk, CompletionError>> + Send>>;

/// Server-Sent Event structure
#[derive(Debug, Clone, PartialEq)]
pub struct SseEvent {
    /// Event type (e.g., "message", "error", "done")
    pub event_type: Option<String>,
    
    /// Event data
    pub data: String,
    
    /// Event ID
    pub id: Option<String>,
    
    /// Retry interval in milliseconds
    pub retry: Option<u64>,
}

impl SseEvent {
    /// Parse a single SSE event from raw text
    pub fn parse(raw_event: &str) -> Result<Option<Self>, CompletionError> {
        let mut event_type = None;
        let mut data_lines = Vec::new();
        let mut id = None;
        let mut retry = None;
        
        for line in raw_event.lines() {
            let line = line.trim();
            
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with(':') {
                continue;
            }
            
            if let Some((field, value)) = line.split_once(':') {
                let field = field.trim();
                let value = value.trim();
                
                match field {
                    "event" => event_type = Some(value.to_string()),
                    "data" => data_lines.push(value),
                    "id" => id = Some(value.to_string()),
                    "retry" => {
                        retry = value.parse().ok();
                    }
                    _ => {
                        // Unknown field, ignore
                    }
                }
            } else if line.starts_with("data:") {
                // Handle "data:" without space
                data_lines.push(&line[5..]);
            }
        }
        
        if data_lines.is_empty() {
            return Ok(None);
        }
        
        let data = data_lines.join("\n");
        
        Ok(Some(SseEvent {
            event_type,
            data,
            id,
            retry,
        }))
    }
}

/// Parse SSE stream from bytes
pub async fn parse_sse_stream(
    stream: impl Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send + 'static,
) -> impl Stream<Item = Result<SseEvent, CompletionError>> + Send {
    use futures_util::StreamExt;
    use tokio_stream::wrappers::UnboundedReceiverStream;
    
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    
    tokio::spawn(async move {
        let mut buffer = String::new();
        let mut stream = Box::pin(stream);
        
        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    match String::from_utf8(chunk.to_vec()) {
                        Ok(text) => {
                            buffer.push_str(&text);
                            
                            // Split on double newlines to separate events
                            while let Some(pos) = buffer.find("\n\n") {
                                let event_text = buffer[..pos].to_string();
                                buffer = buffer[pos + 2..].to_string();
                                
                                match SseEvent::parse(&event_text) {
                                    Ok(Some(event)) => {
                                        if tx.send(Ok(event)).is_err() {
                                            return; // Receiver dropped
                                        }
                                    }
                                    Ok(None) => {
                                        // Empty event, continue
                                    }
                                    Err(e) => {
                                        let _ = tx.send(Err(e));
                                        return;
                                    }
                                }
                            }
                        }
                        Err(_) => {
                            let _ = tx.send(Err(CompletionError::RequestBuilderError(
                                "Invalid UTF-8 in SSE stream".to_string(),
                            )));
                            return;
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(CompletionError::ClientError(
                        crate::llms::api::error::ClientError::Reqwest(e),
                    )));
                    return;
                }
            }
        }
        
        // Process any remaining data in buffer
        if !buffer.trim().is_empty() {
            match SseEvent::parse(&buffer) {
                Ok(Some(event)) => {
                    let _ = tx.send(Ok(event));
                }
                Ok(None) => {
                    // Empty event
                }
                Err(e) => {
                    let _ = tx.send(Err(e));
                }
            }
        }
    });
    
    UnboundedReceiverStream::new(rx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sse_event_parse() {
        let raw_event = "event: message\ndata: {\"content\": \"Hello\"}\nid: 123\n";
        let event = SseEvent::parse(raw_event).unwrap().unwrap();
        
        assert_eq!(event.event_type, Some("message".to_string()));
        assert_eq!(event.data, "{\"content\": \"Hello\"}");
        assert_eq!(event.id, Some("123".to_string()));
    }

    #[test]
    fn test_sse_event_parse_multiline_data() {
        let raw_event = "data: line 1\ndata: line 2\ndata: line 3\n";
        let event = SseEvent::parse(raw_event).unwrap().unwrap();
        
        assert_eq!(event.data, "line 1\nline 2\nline 3");
    }

    #[test]
    fn test_sse_event_parse_empty() {
        let raw_event = "";
        let event = SseEvent::parse(raw_event).unwrap();
        
        assert!(event.is_none());
    }

    #[test]
    fn test_sse_event_parse_comments() {
        let raw_event = ": This is a comment\ndata: actual data\n: Another comment\n";
        let event = SseEvent::parse(raw_event).unwrap().unwrap();
        
        assert_eq!(event.data, "actual data");
    }
}