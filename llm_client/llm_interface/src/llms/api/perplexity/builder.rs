// Internal imports
use super::*;
use generic_openai::{GenericApiBackend, GenericApiConfig};
use llm_devices::{LoggingConfig, LoggingConfigTrait};
use llm_models::{ApiLlmModel, ApiLlmPreset, PerplexityModelTrait};
use crate::requests::{CompletionRequest, CompletionResponse, CompletionError};

/// Perplexity GenericApiBackend wrapper to validate tool usage
pub struct PerplexityGenericApiBackend {
    base: GenericApiBackend,
}

impl PerplexityGenericApiBackend {
    pub fn new(config: GenericApiConfig, model: ApiLlmModel) -> crate::Result<Self> {
        Ok(Self {
            base: GenericApiBackend::new(config, model)?,
        })
    }

    /// Performs validation on the request before forwarding to the underlying backend
    pub async fn completion_request(
        &self,
        request: &CompletionRequest,
    ) -> crate::Result<CompletionResponse, CompletionError> {
        // Check if the request contains tool definitions
        if let Some(tools) = &request.config.tools {
            if !tools.is_empty() {
                return Err(CompletionError::RequestBuilderError(
                    "Perplexity API does not support tool calling. Please remove tools from your request."
                    .to_string(),
                ));
            }
        }
        
        // Forward to the base backend if no tools are present
        self.base.completion_request(request).await
    }
}

// Everything here can be implemented for any struct.
pub struct PerplexityBackendBuilder {
    pub config: GenericApiConfig,
    pub model: ApiLlmModel,
}

impl Default for PerplexityBackendBuilder {
    fn default() -> Self {
        let mut config = GenericApiConfig::default();
        config.api_config.host = "api.perplexity.ai".to_string();
        config.api_config.api_key_env_var = "PERPLEXITY_API_KEY".to_string();
        config.logging_config.logger_name = "perplexity".to_string();
        Self {
            config,
            model: ApiLlmModel::from_preset(ApiLlmPreset::SONAR)
                .expect("Failed to create SONAR model preset"),
        }
    }
}

impl PerplexityBackendBuilder {
    pub fn init(self) -> crate::Result<std::sync::Arc<LlmBackend>> {
        // Create a GenericApi backend for Perplexity
        // Note: We'll validate the tool usage in the client implementation
        let backend = GenericApiBackend::new(self.config, self.model)?;
        
        // Return a standard GenericApi variant
        Ok(std::sync::Arc::new(LlmBackend::GenericApi(backend)))
    }
}



impl PerplexityModelTrait for PerplexityBackendBuilder {
    fn model(&mut self) -> &mut ApiLlmModel {
        &mut self.model
    }
}

impl LlmApiConfigTrait for PerplexityBackendBuilder {
    fn api_base_config_mut(&mut self) -> &mut ApiConfig {
        &mut self.config.api_config
    }

    fn api_config(&self) -> &ApiConfig {
        &self.config.api_config
    }
}

impl LoggingConfigTrait for PerplexityBackendBuilder {
    fn logging_config_mut(&mut self) -> &mut LoggingConfig {
        &mut self.config.logging_config
    }
}
