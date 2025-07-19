//! SSE wire format encoder
//!
//! Implements Server-Sent Events encoding according to RFC 6455 specification.
//! Handles proper field formatting, multiline data, and Unicode encoding.

use super::events::SseEvent;
use std::fmt::Write;

/// SSE encoder for converting events to wire format
///
/// Implements the Server-Sent Events protocol as specified in RFC 6455.
/// Handles proper field formatting, Unicode encoding, and multiline data.
#[derive(Debug, Default, Clone)]
pub struct SseEncoder;

impl SseEncoder {
    /// Create a new SSE encoder
    pub fn new() -> Self {
        Self
    }

    /// Encode an SSE event to wire format
    ///
    /// Produces output according to RFC 6455:
    /// - event: <event_type>
    /// - data: <data_line>
    /// - id: <event_id>
    /// - <empty_line>
    ///
    /// Multiline data is properly handled with multiple data: fields.
    /// Unicode content is preserved with proper UTF-8 encoding.
    pub fn encode(&self, event: &SseEvent) -> String {
        let mut output = String::new();

        // Add event type if present
        if let Some(ref event_type) = event.event_type {
            writeln!(output, "event: {}", event_type).expect("String write cannot fail");
        }

        // Add data field(s) - handle multiline data properly
        for line in event.data.lines() {
            writeln!(output, "data: {}", line).expect("String write cannot fail");
        }

        // Add event ID if present
        if let Some(ref id) = event.id {
            writeln!(output, "id: {}", id).expect("String write cannot fail");
        }

        // Add empty line to terminate the event
        output.push('\n');

        output
    }

    /// Encode multiple events to wire format
    pub fn encode_multiple(&self, events: &[SseEvent]) -> String {
        events.iter().map(|event| self.encode(event)).collect()
    }

    /// Create a comment line (ignored by SSE parsers)
    ///
    /// Comments start with ':' and are used for keep-alive or debugging.
    pub fn comment(text: &str) -> String {
        format!(": {}\n\n", text)
    }

    /// Create a keep-alive comment
    pub fn keep_alive() -> String {
        Self::comment("keep-alive")
    }
}

/// Helper function to escape data for SSE format
///
/// While SSE doesn't require extensive escaping like XML/HTML,
/// we ensure proper line handling and Unicode preservation.
#[allow(dead_code)]
fn escape_sse_data(data: &str) -> String {
    // SSE data doesn't need escaping except for proper line handling
    // Unicode is preserved as-is since SSE is UTF-8
    data.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::sse::events::{EventType, SseEvent};

    #[test]
    fn test_encode_simple_event() {
        let encoder = SseEncoder::new();
        let event = SseEvent::new(EventType::Message, "Hello, World!");

        let encoded = encoder.encode(&event);
        let expected = "event: message\ndata: Hello, World!\n\n";

        assert_eq!(encoded, expected);
    }

    #[test]
    fn test_encode_data_only_event() {
        let encoder = SseEncoder::new();
        let event = SseEvent::data_only("Just data");

        let encoded = encoder.encode(&event);
        let expected = "data: Just data\n\n";

        assert_eq!(encoded, expected);
    }

    #[test]
    fn test_encode_event_with_id() {
        let encoder = SseEncoder::new();
        let event = SseEvent::new(EventType::Ping, "timestamp").with_id("ping-123");

        let encoded = encoder.encode(&event);
        let expected = "event: ping\ndata: timestamp\nid: ping-123\n\n";

        assert_eq!(encoded, expected);
    }

    #[test]
    fn test_encode_multiline_data() {
        let encoder = SseEncoder::new();
        let event = SseEvent::new(EventType::Message, "Line 1\nLine 2\nLine 3");

        let encoded = encoder.encode(&event);
        let expected = "event: message\ndata: Line 1\ndata: Line 2\ndata: Line 3\n\n";

        assert_eq!(encoded, expected);
    }

    #[test]
    fn test_encode_unicode_data() {
        let encoder = SseEncoder::new();
        let event = SseEvent::new(EventType::Message, "Hello 世界! 🌍");

        let encoded = encoder.encode(&event);
        let expected = "event: message\ndata: Hello 世界! 🌍\n\n";

        assert_eq!(encoded, expected);
    }

    #[test]
    fn test_encode_multiple_events() {
        let encoder = SseEncoder::new();
        let events = vec![
            SseEvent::ping("2025-01-07T12:00:00Z"),
            SseEvent::message(r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#),
        ];

        let encoded = encoder.encode_multiple(&events);
        let expected = "event: ping\ndata: 2025-01-07T12:00:00Z\n\nevent: message\ndata: {\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\"}\n\n";

        assert_eq!(encoded, expected);
    }

    #[test]
    fn test_comment_encoding() {
        let comment = SseEncoder::comment("This is a comment");
        assert_eq!(comment, ": This is a comment\n\n");

        let keep_alive = SseEncoder::keep_alive();
        assert_eq!(keep_alive, ": keep-alive\n\n");
    }

    #[test]
    fn test_endpoint_event_encoding() {
        let encoder = SseEncoder::new();
        let event = SseEvent::endpoint("abc123", "http://localhost:8080");

        let encoded = encoder.encode(&event);
        let expected =
            "event: endpoint\ndata: http://localhost:8080/messages?session_id=abc123\n\n";

        assert_eq!(encoded, expected);
    }

    #[test]
    fn test_json_rpc_message_encoding() {
        let encoder = SseEncoder::new();
        let json_rpc = r#"{"jsonrpc":"2.0","id":1,"result":{"tools":[{"name":"echo"}]}}"#;
        let event = SseEvent::message(json_rpc);

        let encoded = encoder.encode(&event);

        assert!(encoded.contains("event: message"));
        assert!(encoded.contains(&format!("data: {}", json_rpc)));
        assert!(encoded.ends_with("\n\n"));
    }
}
