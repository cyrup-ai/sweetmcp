use thiserror::Error;

/// Error types for daemon installation operations
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum InstallerError {
    /// User cancelled the authorization prompt
    #[error("User cancelled authorization")]
    Cancelled,

    /// Permission was denied (incorrect password, policy restriction, etc.)
    #[error("Permission denied")]
    PermissionDenied,

    /// Required executable not found on system
    #[error("Executable not found: {0}")]
    MissingExecutable(String),

    /// I/O operation failed
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Platform-specific system error
    #[error("System error: {0}")]
    System(String),

    /// Other errors
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
