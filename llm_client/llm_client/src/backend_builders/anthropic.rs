use crate::LlmClient;
use llm_devices::{LoggingConfig, LoggingConfigTrait};
use llm_interface::llms::{LlmBackend, api::{
    anthropic::{AnthropicBackend, AnthropicConfig},
    ApiConfig, LlmApiConfigTrait
}};
use llm_models::{
    api_models::{AnthropicModelTrait, ApiLlmModel},
    ApiLlmPreset,
};

pub struct AnthropicBackendBuilder {
    pub config: AnthropicConfig,
    pub model: ApiLlmModel,
}

impl Default for AnthropicBackendBuilder {
    fn default() -> Self {
        // Handle potential error during default model creation
        let model = ApiLlmModel::from_preset(ApiLlmPreset::CLAUDE_3_5_SONNET)
            .expect("Failed to create default Anthropic model from preset");
            
        Self {
            config: Default::default(),
            model,
        }
    }
}

impl AnthropicBackendBuilder {
    pub fn init(self) -> crate::Result<LlmClient> {
        Ok(LlmClient::new(std::sync::Arc::new(LlmBackend::Anthropic(
            AnthropicBackend::new(self.config, self.model)?,
        ))))
    }
}

impl LlmApiConfigTrait for AnthropicBackendBuilder {
    fn api_base_config_mut(&mut self) -> &mut ApiConfig {
        &mut self.config.api_config
    }

    fn api_config(&self) -> &ApiConfig {
        &self.config.api_config
    }
}

impl AnthropicModelTrait for AnthropicBackendBuilder {
    fn model(&mut self) -> &mut ApiLlmModel {
        &mut self.model
    }
}

impl LoggingConfigTrait for AnthropicBackendBuilder {
    fn logging_config_mut(&mut self) -> &mut LoggingConfig {
        &mut self.config.logging_config
    }
}
