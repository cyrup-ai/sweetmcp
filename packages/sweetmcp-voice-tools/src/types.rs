//! Type definitions for voice operations

use serde::{Deserialize, Serialize};

/// Parameters for the speak operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakParams {
    /// Text to synthesize into speech
    pub text: String,

    /// Optional voice ID (defaults to system default)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice_id: Option<String>,

    /// Optional speed modifier (0.5 to 2.0, default 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f32>,
}

/// Parameters for the listen operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListenParams {
    /// Microphone device ID (e.g., "default", "USB Microphone")
    pub microphone_id: String,

    /// Duration to listen in seconds (1-300)
    pub duration_seconds: u32,

    /// Optional wake word to listen for
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wake_word: Option<String>,
}

/// Result of a listen operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListenResult {
    /// Transcribed text
    pub text: String,

    /// Whether wake word was detected (if specified)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wake_word_detected: Option<bool>,

    /// Confidence score (0.0 to 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f32>,

    /// Detected language (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

/// Voice service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConfig {
    /// QUIC endpoint for voice service
    pub endpoint: String,

    /// Default voice ID
    pub default_voice: Option<String>,

    /// Default microphone
    pub default_microphone: Option<String>,

    /// VAD (Voice Activity Detection) sensitivity (0.0 to 1.0)
    pub vad_sensitivity: f32,
}

impl Default for VoiceConfig {
    fn default() -> Self {
        Self {
            endpoint: "localhost:33336".to_string(),
            default_voice: None,
            default_microphone: Some("default".to_string()),
            vad_sensitivity: 0.5,
        }
    }
}
