//! QUIC protocol definitions for voice service communication

use crate::types::{ListenParams, ListenResult, SpeakParams};
use serde::{Deserialize, Serialize};

/// Request types for voice operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "params")]
pub enum VoiceRequest {
    /// Request to speak text
    Speak(SpeakParams),

    /// Request to listen for audio
    Listen(ListenParams),

    /// Request list of available voices
    ListVoices,

    /// Request list of available microphones
    ListMicrophones,
}

/// Response types for voice operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum VoiceResponse {
    /// Speaking completed successfully
    SpeakComplete,

    /// Listen operation result
    ListenResult(ListenResult),

    /// List of available voice IDs
    VoiceList(Vec<String>),

    /// List of available microphone IDs
    MicrophoneList(Vec<String>),

    /// Error response
    Error { code: String, message: String },
}

/// Create a voice service client endpoint
pub fn voice_endpoint() -> String {
    std::env::var("VOICE_SERVICE_ENDPOINT").unwrap_or_else(|_| "localhost:33336".to_string())
}
