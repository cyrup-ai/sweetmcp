use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    /// Underlying error from reqwest library after an API call was made
    #[error("http error: {0}")]
    Reqwest(#[from] reqwest::Error),
    /// API returns error object with details of API call failure
    #[error("{:?}: {}", .0.r#type, .0.message)]
    ApiError(ApiError),
    /// Error when API returns 503 status code
    #[error("Service unavailable: {message}")]
    ServiceUnavailable { message: String },
    /// Generic error message
    #[error("Generic error: {message}")]
    GenericError { message: String },
    /// Error when a response cannot be deserialized into a Rust type
    #[error("failed to serialize api request: {0}")]
    JSONSerialize(serde_json::Error),
    /// Error when a response cannot be deserialized into a Rust type
    #[error("failed to deserialize api response: {0}")]
    JSONDeserialize(serde_json::Error),
    /// Error from client side validation
    /// or when builder fails to build request before making API call
    #[error("invalid args: {0}")]
    InvalidArgument(String),
    /// Error parsing a Server-Sent Event (SSE) stream chunk
    #[error("failed to parse stream event: {0}")]
    StreamParseError(String),
    /// I/O error during stream processing
    #[error("stream i/o error: {0}")]
    IoError(String),
}

/// Wrapper to deserialize the error object nested in "error" JSON key (Common in OpenAI/Anthropic)
#[derive(Debug, Deserialize)]
pub(crate) struct WrappedError {
    pub(crate) error: ApiError,
}

pub(crate) fn map_deserialization_error(e: serde_json::Error, bytes: &[u8]) -> ClientError {
    tracing::error!(
        "failed deserialization of: {}",
        String::from_utf8_lossy(bytes)
    );
    ClientError::JSONDeserialize(e)
}

pub(crate) fn map_serialization_error(e: serde_json::Error) -> ClientError {
    tracing::error!("failed serialization: {}", e);
    ClientError::JSONSerialize(e)
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ApiError {
    pub message: String,
    pub r#type: Option<String>,
    pub param: Option<String>,
    pub code: Option<String>,
}
