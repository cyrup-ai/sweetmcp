use std::fmt;

/// Comprehensive error types for browser plugin operations
#[derive(Debug)]
#[allow(dead_code)]
pub enum BrowserError {
    /// Invalid input parameters
    InvalidInput(String),
    /// Command serialization error
    SerializationError(String),
    /// Browser operation failed
    OperationFailed(String),
    /// Timeout occurred
    Timeout(String),
    /// Element not found
    ElementNotFound(String),
    /// Navigation error
    NavigationError(String),
    /// Script execution error
    ScriptError(String),
    /// Screenshot capture error
    ScreenshotError(String),
    /// Vision analysis error
    VisionError(String),
    /// Automation task error
    AutomationError(String),
}

impl fmt::Display for BrowserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BrowserError::InvalidInput(msg) => write!(f, "Invalid input: {msg}"),
            BrowserError::SerializationError(msg) => write!(f, "Serialization error: {msg}"),
            BrowserError::OperationFailed(msg) => write!(f, "Operation failed: {msg}"),
            BrowserError::Timeout(msg) => write!(f, "Timeout: {msg}"),
            BrowserError::ElementNotFound(msg) => write!(f, "Element not found: {msg}"),
            BrowserError::NavigationError(msg) => write!(f, "Navigation error: {msg}"),
            BrowserError::ScriptError(msg) => write!(f, "Script error: {msg}"),
            BrowserError::ScreenshotError(msg) => write!(f, "Screenshot error: {msg}"),
            BrowserError::VisionError(msg) => write!(f, "Vision error: {msg}"),
            BrowserError::AutomationError(msg) => write!(f, "Automation error: {msg}"),
        }
    }
}

impl std::error::Error for BrowserError {}

/// Convert BrowserError to extism Error
/// Note: Using Into trait to avoid conflict with anyhow's blanket implementation
pub fn browser_error_to_extism(err: BrowserError) -> extism_pdk::Error {
    extism_pdk::Error::msg(err.to_string())
}

/// Helper functions for error handling
pub fn validate_url(url: &str) -> Result<(), BrowserError> {
    if url.is_empty() {
        return Err(BrowserError::InvalidInput(
            "URL cannot be empty".to_string(),
        ));
    }

    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(BrowserError::InvalidInput(
            "URL must start with http:// or https://".to_string(),
        ));
    }

    Ok(())
}

pub fn validate_selector(selector: &str) -> Result<(), BrowserError> {
    if selector.is_empty() {
        return Err(BrowserError::InvalidInput(
            "Selector cannot be empty".to_string(),
        ));
    }

    // Basic validation for common selector patterns
    if selector.contains("//") {
        // XPath selector - basic validation
        if selector.chars().filter(|&c| c == '[').count()
            != selector.chars().filter(|&c| c == ']').count()
        {
            return Err(BrowserError::InvalidInput(
                "Invalid XPath: unmatched brackets".to_string(),
            ));
        }
    }

    Ok(())
}

#[allow(dead_code)]
pub fn validate_timeout(duration: i64) -> Result<(), BrowserError> {
    if duration <= 0 {
        return Err(BrowserError::InvalidInput(
            "Timeout duration must be positive".to_string(),
        ));
    }

    if duration > 300000 {
        // 5 minutes max
        return Err(BrowserError::InvalidInput(
            "Timeout duration cannot exceed 5 minutes (300000ms)".to_string(),
        ));
    }

    Ok(())
}
