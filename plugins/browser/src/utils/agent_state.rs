use std::sync::Arc;
use tokio::sync::Mutex;

/// Manages agent execution state with thread-safe access
#[derive(Clone)]
pub struct AgentState {
    stop_requested: bool,
    last_valid_state: Option<String>,
}

impl AgentState {
    /// Create a new agent state
    pub fn new() -> Self {
        Self {
            stop_requested: false,
            last_valid_state: None,
        }
    }
    
    /// Request the agent to stop execution
    pub fn request_stop(&mut self) -> Result<(), String> {
        self.stop_requested = true;
        Ok(())
    }
    
    /// Clear the stop request flag
    pub fn clear_stop(&mut self) {
        self.stop_requested = false;
        self.last_valid_state = None;
    }
    
    /// Check if stop has been requested
    pub fn is_stop_requested(&self) -> bool {
        self.stop_requested
    }
    
    /// Set the last valid browser state for recovery
    pub fn set_last_valid_state(&mut self, state: String) {
        self.last_valid_state = Some(state);
    }
    
    /// Get the last valid browser state
    pub fn get_last_valid_state(&self) -> Option<String> {
        self.last_valid_state.clone()
    }
}

impl Default for AgentState {
    fn default() -> Self {
        Self::new()
    }
}
