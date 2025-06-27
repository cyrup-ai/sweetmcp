use log;

/// Console logger service supporting styled output
#[derive(Clone)]
pub struct ConsoleLogger {
    // Configuration parameters could be added here
}

impl ConsoleLogger {
    /// Create a new console logger
    pub fn new() -> Self {
        Self {}
    }

    /// Log a warning message (yellow)
    pub fn warn(&self, message: &str) {
        // In production, this would use proper terminal styling
        log::warn!("{}", message);
    }

    /// Log a success message (green)
    pub fn success(&self, message: &str) {
        // In production, this would use proper terminal styling
        log::info!("{}", message);
    }

    /// Log an error message (red)
    pub fn error(&self, message: &str) {
        // In production, this would use proper terminal styling
        log::error!("{}", message);
    }
}
