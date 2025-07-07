//! Error types for voice operations

use thiserror::Error;

pub type VoiceResult<T> = Result<T, VoiceError>;

#[derive(Debug, Error)]
pub enum VoiceError {
    #[error("Voice service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Invalid voice ID: {0}")]
    InvalidVoiceId(String),

    #[error("Invalid microphone: {0}")]
    InvalidMicrophone(String),

    #[error("Transcription failed: {0}")]
    TranscriptionFailed(String),

    #[error("Synthesis failed: {0}")]
    SynthesisFailed(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Invalid duration: {0} seconds (must be between 1-300)")]
    InvalidDuration(u32),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}
