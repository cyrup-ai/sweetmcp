// src/utils/error.rs
//! Error types for the memory system.

use thiserror::Error;

/// Result type alias using our Error type
pub type Result<T> = std::result::Result<T, Error>;

/// Custom error type for the application
#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(Box<surrealdb::Error>),

    #[error("Vector store error: {0}")]
    VectorStore(String),

    #[error("Embedding model error: {0}")]
    Embedding(String),

    #[error("LLM provider error: {0}")]
    LLM(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("API error: {0}")]
    Api(String),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Conversion error: {0}")]
    ConversionError(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("HTTP request error: {0}")]
    HttpRequest(#[from] reqwest::Error),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Already exists: {0}")]
    AlreadyExists(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Other error: {0}")]
    Other(String),
}

// Implement axum::response::IntoResponse for AppError to use it in handlers
#[cfg(feature = "api")] // Changed from "axum-api" to "api"
impl axum::response::IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        use axum::Json;
        use axum::http::StatusCode;

        let (status, error_message) = match self {
            Error::Database(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            ),
            Error::VectorStore(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Vector store error: {}", e),
            ),
            Error::Embedding(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Embedding error: {}", e),
            ),
            Error::LLM(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("LLM error: {}", e),
            ),
            Error::Config(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Configuration error: {}", e),
            ),
            Error::Api(e) => (StatusCode::BAD_REQUEST, format!("API error: {}", e)),
            Error::Migration(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Migration error: {}", e),
            ),
            Error::NotFound(e) => (StatusCode::NOT_FOUND, e),
            Error::InvalidInput(e) => (StatusCode::BAD_REQUEST, e),
            Error::ValidationError(e) => (StatusCode::BAD_REQUEST, e),
            Error::ConversionError(e) => (StatusCode::BAD_REQUEST, e),
            Error::Io(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("I/O error: {}", e),
            ),
            Error::Serialization(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Serialization error: {}", e),
            ),
            Error::HttpRequest(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("HTTP request error: {}", e),
            ),
            Error::NotImplemented(e) => (StatusCode::NOT_IMPLEMENTED, e),
            Error::AlreadyExists(e) => (StatusCode::CONFLICT, e),
            Error::Internal(e) => (StatusCode::INTERNAL_SERVER_ERROR, e),
            Error::Other(e) => (StatusCode::INTERNAL_SERVER_ERROR, e),
        };

        (status, Json(serde_json::json!({ "error": error_message }))).into_response()
    }
}

impl From<surrealdb::Error> for Error {
    fn from(err: surrealdb::Error) -> Self {
        Error::Database(Box::new(err))
    }
}
