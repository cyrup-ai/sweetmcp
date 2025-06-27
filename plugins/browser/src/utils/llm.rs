use std::fmt;

use kalosm_llama::Llama;
use tracing::{debug, info};

use crate::utils::errors::UtilsError;
use crate::utils::utils::llama;

/// Error type for LLM operations
#[derive(Debug)]
pub enum LlmError {
    ModelError(String),
}

impl fmt::Display for LlmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LlmError::ModelError(err) => write!(f, "Model error: {}", err),
        }
    }
}

impl std::error::Error for LlmError {}

/// Get the LLM model based on configuration
pub fn get_llm_model(
    _provider_override: &str,
    _model_name_override: Option<&str>,
    _temperature_override: f32,
) -> Result<Llama, UtilsError> {
    // We're now standardizing on using the llama() function from utils module
    // Parameters are ignored as we're using a consistent model from configuration
    info!("Getting Llama model using standardized configuration");
    llama()
}

/// LLM configuration for izing model behavior
#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub provider: String,
    pub model_name: String,
    pub temperature: f32,
    pub max_tokens: Option<usize>,
    pub top_p: Option<f32>,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: "phi-4".to_string(),
            model_name: "phi-4".to_string(),
            temperature: 0.7,
            max_tokens: None,
            top_p: None,
        }
    }
}

impl LlmConfig {
    /// Create a new LLM configuration with default values
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set the temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }
    
    /// Set the maximum number of tokens
    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }
    
    /// Set the top_p value
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }
    
    /// Create an LLM instance based on the configuration
    pub async fn create_llm(&self) -> Result<Llama, UtilsError> {
        // Create Llama model
        debug!("Creating Phi-4 LLM model");
        
        // Use the Llama builder with phi-4 source
        let model = Llama::builder()
            .with_source(LlamaSource::phi_4())
            .build()
            .await
            .map_err(|e| UtilsError::ModelError(e.to_string()))?;
        
        Ok(model)
    }
}
