//! MCP tool definitions for voice operations

use crate::{Tool, ToolInputSchema, ToolInputSchemaProperty};
use std::collections::HashMap;

/// Create the speak tool definition
pub fn speak_tool() -> Tool {
    let mut properties = HashMap::new();

    properties.insert(
        "text".to_string(),
        ToolInputSchemaProperty {
            type_name: Some("string".to_string()),
            enum_values: None,
            description: Some("Text to convert to speech".to_string()),
        },
    );

    properties.insert(
        "voice_id".to_string(),
        ToolInputSchemaProperty {
            type_name: Some("string".to_string()),
            enum_values: None,
            description: Some("Voice ID to use (optional, defaults to system voice)".to_string()),
        },
    );

    properties.insert(
        "speed".to_string(),
        ToolInputSchemaProperty {
            type_name: Some("number".to_string()),
            enum_values: None,
            description: Some("Speech speed (0.5-2.0, default 1.0)".to_string()),
        },
    );

    Tool {
        name: "speak".to_string(),
        description: Some(
            "Convert text to speech and play it through the system audio. \
            Perfect for making the assistant speak responses out loud, \
            reading content to users, or providing audio feedback."
                .to_string(),
        ),
        input_schema: ToolInputSchema {
            type_name: "object".to_string(),
            properties,
            required: vec!["text".to_string()],
        },
    }
}

/// Create the listen tool definition
pub fn listen_tool() -> Tool {
    let mut properties = HashMap::new();

    properties.insert(
        "microphone_id".to_string(),
        ToolInputSchemaProperty {
            type_name: Some("string".to_string()),
            enum_values: None,
            description: Some(
                "Microphone device to use (e.g., 'default', 'USB Microphone')".to_string(),
            ),
        },
    );

    properties.insert(
        "duration_seconds".to_string(),
        ToolInputSchemaProperty {
            type_name: Some("integer".to_string()),
            enum_values: None,
            description: Some("How long to listen in seconds (1-300)".to_string()),
        },
    );

    properties.insert(
        "wake_word".to_string(),
        ToolInputSchemaProperty {
            type_name: Some("string".to_string()),
            enum_values: None,
            description: Some(
                "Optional wake word to listen for (e.g., 'hey assistant')".to_string(),
            ),
        },
    );

    Tool {
        name: "listen".to_string(),
        description: Some(
            "Listen to audio from the microphone and transcribe it to text. \
            Use this to hear what the user is saying, capture voice commands, \
            or enable voice-based interactions. Supports wake word detection \
            for hands-free activation."
                .to_string(),
        ),
        input_schema: ToolInputSchema {
            type_name: "object".to_string(),
            properties,
            required: vec!["microphone_id".to_string(), "duration_seconds".to_string()],
        },
    }
}
