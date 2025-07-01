mod controller;

pub use controller::{ActionModel, ActionResult, Controller};

use std::fmt;
use std::future::Future;
use std::pin::Pin;
use thiserror::Error;

use crate::browser::{BrowserContext, BrowserError};

/// Error type for controller operations
#[derive(Error, Debug)]
pub enum ControllerError {
    #[error("Missing parameter: {0}")]
    MissingParameter(String),
    
    #[error("Unknown action: {0}")]
    UnknownAction(String),
    
    #[error("Browser error: {0}")]
    BrowserError(String),
    
    #[error("Clipboard error: {0}")]
    ClipboardError(String),
    
    #[error("JSON parse error: {0}")]
    JsonParseError(String),
    
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}

/// Result type for controller operations
pub type ControllerResult<T> = Result<T, ControllerError>;

/// Action executor trait
pub trait ActionExecutor {
    /// Execute an action with the given parameters and browser context
    /// 
    /// This is an async function that returns a ControllerResult<ActionResult>
    /// without exposing the Future type directly.
    async fn execute(&self, action: &ActionModel, browser: &BrowserContext) -> ControllerResult<ActionResult>;
}

/// Implement ActionExecutor for any function with the correct signature
impl<F, Fut> ActionExecutor for F
where
    F: Fn(&ActionModel, &BrowserContext) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ControllerResult<ActionResult>> + Send + 'static,
{
    async fn execute(&self, action: &ActionModel, browser: &BrowserContext) -> ControllerResult<ActionResult> {
        // Call the function directly and await its future
        self(action, browser).await
    }
}

/// Implement From<BrowserError> for ControllerError
impl From<BrowserError> for ControllerError {
    fn from(err: BrowserError) -> Self {
        ControllerError::BrowserError(err.to_string())
    }
}

/// Implement From<serde_json::Error> for ControllerError
impl From<serde_json::Error> for ControllerError {
    fn from(err: serde_json::Error) -> Self {
        ControllerError::JsonParseError(err.to_string())
    }
}
